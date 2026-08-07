use std::{collections::BTreeMap, io, path::PathBuf, sync::Arc};

use gltf_corpus::{decode_glb_file, DecodedAnimation, DecodedModel, DecodedPrimitive};
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};
use tokimu_assets::AssetStore;
use tokimu_core::math::{Mat4, Vec3, Vec4};

const CAMERA_HANDLE: CameraHandle = CameraHandle(1);
const MODEL_MATERIAL: MaterialHandle = MaterialHandle(1);
const FLOOR_MESH: MeshHandle = MeshHandle(2_000);
const FLOOR_MATERIAL: MaterialHandle = MaterialHandle(2);
const HOLE_PUNCH_SOURCE: &str = "corpus/assets/CheckLicense/hole_punch1.glb";

fn main() -> PlatformResult<()> {
    if std::env::args()
        .skip(1)
        .any(|argument| argument == "--verify-assets")
    {
        let path = hole_punch_path();
        let model = decode_glb_file(&path)?;
        println!(
            "verified hole-punch asset: {} (meshes={}, animations={})",
            path.display(),
            model.summary.meshes,
            model.summary.animations
        );
        return Ok(());
    }

    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Hole Punch".into(),
            width: 1280,
            height: 720,
        },
        HelloHolePunchApp::default(),
    )
}

#[derive(Default)]
struct HelloHolePunchApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    pipeline: PipelineHandle,
    assets: AssetStore,
    model: Option<DecodedModel>,
    meshes: Vec<(MeshHandle, Mesh)>,
    scene_presentation_transform: Mat4,
    animation_count: usize,
    active_animation: String,
}

impl HelloHolePunchApp {
    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello Hole Punch | meshes={} | animations={} | clip={} | elapsed={:.1}s",
                self.meshes.len(),
                self.animation_count,
                self.active_animation,
                self.elapsed_seconds
            ));
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let seconds = self.elapsed_seconds as f32;
        self.refresh_animated_meshes()?;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        for (handle, mesh) in &self.meshes {
            renderer.upload_mesh(*handle, mesh);
        }

        let mut camera = Camera::perspective_3d(self.window_size[0], self.window_size[1]);
        let orbit = seconds * 0.22;
        let eye = Vec3::new(orbit.cos() * 4.8, 2.7, orbit.sin() * 4.8);
        camera.view = Mat4::look_at_rh(eye, Vec3::new(0.0, 0.25, 0.0), Vec3::Y);
        renderer.upload_camera(CAMERA_HANDLE, camera);

        let mut commands = vec![
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.045, 0.06, 0.09),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: FLOOR_MESH,
                material: FLOOR_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
        ];
        commands.extend(self.meshes.iter().map(|(handle, _)| {
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: *handle,
                material: MODEL_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            })
        }));

        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        self.update_window_title();
        Ok(FrameOutcome::Continue)
    }

    fn refresh_animated_meshes(&mut self) -> PlatformResult<()> {
        let Some(model) = self.model.as_ref() else {
            return Ok(());
        };
        let (translations, active_animation) =
            sample_animation_cycle(&model.animations, self.elapsed_seconds as f32);
        let mut next_meshes = scene_meshes(model, &translations)?;
        apply_scene_transform(&mut next_meshes, self.scene_presentation_transform);
        if next_meshes.len() != self.meshes.len() {
            return Err(io::Error::other("animated scene changed its primitive count").into());
        }
        for ((_, target), source) in self.meshes.iter_mut().zip(next_meshes) {
            *target = source;
        }
        self.active_animation = active_animation;
        Ok(())
    }
}

impl PlatformEventHandler for HelloHolePunchApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let model = decode_glb_file(hole_punch_path())?;
        self.animation_count = model.summary.animations;
        let mut source_meshes = scene_meshes(&model, &BTreeMap::new())?;
        // This asset is authored upright while the example presents the tool
        // resting on its back. Keep that inspection adjustment outside glTF
        // node semantics, then freeze normalization from the bind pose.
        let presentation_orientation = presentation_orientation();
        apply_scene_transform(&mut source_meshes, presentation_orientation);
        let normalization = normalization_transform(&source_meshes);
        self.scene_presentation_transform = normalization * presentation_orientation;
        apply_scene_transform(&mut source_meshes, normalization);
        self.meshes = source_meshes
            .into_iter()
            .enumerate()
            .map(|(index, mesh)| (MeshHandle((index as u64) + 1), mesh))
            .collect();
        if self.meshes.is_empty() {
            return Err(
                io::Error::other("hole_punch2.glb has no renderable scene primitives").into(),
            );
        }
        self.assets
            .allocate_with_source::<Mesh, _>(HOLE_PUNCH_SOURCE);
        self.model = Some(model);

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        for (handle, mesh) in &self.meshes {
            renderer.upload_mesh(*handle, mesh);
        }
        renderer.upload_mesh(FLOOR_MESH, &floor_mesh());
        renderer.upload_material(
            MODEL_MATERIAL,
            &Material::new("hole-punch", Color::rgb(0.76, 0.69, 0.52)),
        )?;
        renderer.upload_material(
            FLOOR_MATERIAL,
            &Material::new("floor", Color::rgb(0.075, 0.095, 0.13)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hole-punch-pipeline",
            PipelineKind::LitColor3d,
        ))?;
        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::CloseRequested => Ok(()),
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.elapsed_seconds += delta_seconds;
        self.render_scene()
    }
}

fn hole_punch_path() -> PathBuf {
    let staged_asset = std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(PathBuf::from))
        .map(|directory| directory.join("assets/CheckLicense/hole_punch1.glb"));
    if let Some(path) = staged_asset.filter(|path| path.is_file()) {
        return path;
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(HOLE_PUNCH_SOURCE)
}

fn presentation_orientation() -> Mat4 {
    Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
}

fn scene_meshes(
    model: &DecodedModel,
    translations: &BTreeMap<usize, Vec3>,
) -> PlatformResult<Vec<Mesh>> {
    let scene = model
        .scenes
        .first()
        .ok_or_else(|| io::Error::other("GLB has no scene"))?;
    let parents = parent_indices(model);
    let mut meshes = Vec::new();
    for scene_node in &scene.traversal {
        let node = model.nodes.get(scene_node.node).ok_or_else(|| {
            io::Error::other(format!("scene references missing node {}", scene_node.node))
        })?;
        let Some(mesh_index) = node.mesh else {
            continue;
        };
        for primitive in model
            .primitives
            .iter()
            .filter(|primitive| primitive.location.mesh == mesh_index)
        {
            let transform =
                resolved_world_transform(scene_node.node, model, &parents, translations);
            meshes.push(primitive_to_mesh(primitive, transform)?);
        }
    }

    Ok(meshes)
}

fn parent_indices(model: &DecodedModel) -> Vec<Option<usize>> {
    let mut parents = vec![None; model.nodes.len()];
    for node in &model.nodes {
        for child in &node.children {
            parents[*child] = Some(node.index);
        }
    }
    parents
}

fn resolved_world_transform(
    index: usize,
    model: &DecodedModel,
    parents: &[Option<usize>],
    translations: &BTreeMap<usize, Vec3>,
) -> Mat4 {
    let node = &model.nodes[index];
    let mut local = Mat4::from_cols_array(&node.local_transform);
    if let Some(translation) = translations.get(&index) {
        local.w_axis = Vec4::new(translation.x, translation.y, translation.z, 1.0);
    }
    parents[index].map_or(local, |parent| {
        resolved_world_transform(parent, model, parents, translations) * local
    })
}

fn primitive_to_mesh(primitive: &DecodedPrimitive, transform: Mat4) -> PlatformResult<Mesh> {
    if primitive.normals.len() != primitive.positions.len() {
        return Err(io::Error::other(format!(
            "mesh {} primitive {} has {} positions but {} normals",
            primitive.location.mesh,
            primitive.location.primitive,
            primitive.positions.len(),
            primitive.normals.len()
        ))
        .into());
    }
    let normal_transform = transform.inverse().transpose();
    let mut positions = Vec::with_capacity(primitive.indices.len());
    let mut normals = Vec::with_capacity(primitive.indices.len());
    for index in &primitive.indices {
        let index = *index as usize;
        let position = *primitive.positions.get(index).ok_or_else(|| {
            io::Error::other(format!(
                "primitive index {index} is outside decoded positions"
            ))
        })?;
        let normal = *primitive.normals.get(index).ok_or_else(|| {
            io::Error::other(format!(
                "primitive index {index} is outside decoded normals"
            ))
        })?;
        positions.push(
            transform
                .transform_point3(Vec3::from_array(position))
                .to_array(),
        );
        normals.push(
            normal_transform
                .transform_vector3(Vec3::from_array(normal))
                .normalize_or_zero()
                .to_array(),
        );
    }
    Ok(Mesh::new(positions, normals))
}

fn normalization_transform(meshes: &[Mesh]) -> Mat4 {
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);
    for mesh in meshes {
        for position in &mesh.positions {
            minimum = minimum.min(Vec3::from_array(*position));
            maximum = maximum.max(Vec3::from_array(*position));
        }
    }
    let center = (minimum + maximum) * 0.5;
    let extent = (maximum - minimum).max_element().max(1.0);
    let scale = 2.7 / extent;
    Mat4::from_scale(Vec3::splat(scale)) * Mat4::from_translation(-center)
}

fn apply_scene_transform(meshes: &mut [Mesh], transform: Mat4) {
    for mesh in meshes {
        for position in &mut mesh.positions {
            *position = transform
                .transform_point3(Vec3::from_array(*position))
                .to_array();
        }
    }
}

fn sample_animation_cycle(
    animations: &[DecodedAnimation],
    elapsed_seconds: f32,
) -> (BTreeMap<usize, Vec3>, String) {
    if animations.is_empty() {
        return (BTreeMap::new(), "none".into());
    }
    let durations = animations
        .iter()
        .map(animation_duration)
        .map(|duration| duration.max(0.001))
        .collect::<Vec<_>>();
    let cycle_duration = durations.iter().sum::<f32>();
    let mut cursor = elapsed_seconds.rem_euclid(cycle_duration);
    let mut index = 0;
    for (candidate, duration) in durations.iter().enumerate() {
        if cursor < *duration {
            index = candidate;
            break;
        }
        cursor -= duration;
    }
    let mut result = BTreeMap::new();
    // Source clips are authored as assembly steps. Hold each completed step
    // at its final translation while interpolating the current one.
    for completed in animations.iter().take(index) {
        for channel in &completed.channels {
            result.insert(
                channel.node,
                Vec3::from_array(
                    *channel
                        .translations
                        .last()
                        .expect("animation channels have keys"),
                ),
            );
        }
    }

    let animation = &animations[index];
    for channel in &animation.channels {
        result.insert(
            channel.node,
            sample_translation(
                channel.times.as_slice(),
                channel.translations.as_slice(),
                cursor,
            ),
        );
    }
    (
        result,
        animation
            .name
            .clone()
            .map(|name| format!("{name} | held={index}"))
            .unwrap_or_else(|| format!("clip-{index} | held={index}")),
    )
}

fn animation_duration(animation: &DecodedAnimation) -> f32 {
    animation
        .channels
        .iter()
        .filter_map(|channel| channel.times.last().copied())
        .fold(0.0, f32::max)
}

fn sample_translation(times: &[f32], translations: &[[f32; 3]], time: f32) -> Vec3 {
    if time <= times[0] {
        return Vec3::from_array(translations[0]);
    }
    for index in 1..times.len() {
        if time <= times[index] {
            let start = times[index - 1];
            let amount = ((time - start) / (times[index] - start)).clamp(0.0, 1.0);
            return Vec3::from_array(translations[index - 1])
                .lerp(Vec3::from_array(translations[index]), amount);
        }
    }
    Vec3::from_array(*translations.last().expect("animation channels have keys"))
}

fn floor_mesh() -> Mesh {
    let transform = Mat4::from_translation(Vec3::new(0.0, -1.05, 0.0))
        * Mat4::from_scale(Vec3::new(5.5, 0.04, 5.5));
    let normal_transform = transform.inverse().transpose();
    let base = Mesh::cube();
    Mesh::new(
        base.positions
            .into_iter()
            .map(|position| {
                transform
                    .transform_point3(Vec3::from_array(position))
                    .to_array()
            })
            .collect(),
        base.normals
            .into_iter()
            .map(|normal| {
                normal_transform
                    .transform_vector3(Vec3::from_array(normal))
                    .normalize_or_zero()
                    .to_array()
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hole_punch_asset_exposes_the_five_translation_steps() {
        let model = decode_glb_file(hole_punch_path()).expect("hole punch asset should decode");
        let names = model
            .animations
            .iter()
            .filter_map(|animation| animation.name.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(names, ["step1", "step2", "step3", "step4", "step5"]);
        assert!(model
            .animations
            .iter()
            .flat_map(|animation| &animation.channels)
            .all(|channel| !channel.times.is_empty()
                && channel.times.len() == channel.translations.len()));
    }

    #[test]
    fn animation_cycle_selects_a_named_translation_clip() {
        let model = decode_glb_file(hole_punch_path()).expect("hole punch asset should decode");
        let (translations, active) = sample_animation_cycle(&model.animations, 0.1);
        assert_eq!(active, "step1 | held=0");
        assert!(!translations.is_empty());
    }

    #[test]
    fn animation_cycle_holds_completed_steps_while_the_next_step_runs() {
        let model = decode_glb_file(hole_punch_path()).expect("hole punch asset should decode");
        let step_one_duration = animation_duration(&model.animations[0]);
        let (translations, active) =
            sample_animation_cycle(&model.animations, step_one_duration + 0.1);

        assert_eq!(active, "step2 | held=1");
        assert!(
            translations.contains_key(&25),
            "step1 translation should persist"
        );
        assert!(
            translations.contains_key(&21),
            "step2 should animate its first node"
        );
        assert!(
            translations.contains_key(&23),
            "step2 should animate its second node"
        );
    }
}
