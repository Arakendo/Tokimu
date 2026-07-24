use std::{collections::HashMap, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, ViewportRect, WgpuBackend, WindowConfig,
};
use ui_tools::{
    layout_bitmap_text, lower_surface_to_vector, tessellate_convex_fill, UiLabel, UiLabelAnchor,
    UiRadius, UiRect, UiRegion, UiSurfaceCommand, UiSurfaceRole, UiSurfaceVectorLayerKind,
    UiTextRole, UiTheme,
};

const GLYPH_MESH: MeshHandle = MeshHandle(2);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);
const SURFACE_MESH_BASE: u64 = 10;

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const FRAME_MATERIAL: MaterialHandle = MaterialHandle(2);
const INNER_MATERIAL: MaterialHandle = MaterialHandle(3);
const ACCENT_MATERIAL: MaterialHandle = MaterialHandle(4);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(5);
const TEXT_MATERIAL: MaterialHandle = MaterialHandle(6);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Box".into(),
            width: 1000,
            height: 680,
        },
        HelloUiBoxApp::new(),
    )
}

struct HelloUiBoxApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    stats_reported: bool,
    frame_count: u32,
    surface_meshes: HashMap<MeshHandle, Mesh>,
}

impl Default for HelloUiBoxApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
            stats_reported: false,
            frame_count: 0,
            surface_meshes: HashMap::new(),
        }
    }
}

impl HelloUiBoxApp {
    fn new() -> Self {
        Self::default()
    }

    fn material_for_role(role: UiSurfaceRole) -> MaterialHandle {
        match role {
            UiSurfaceRole::Background => BACKDROP_MATERIAL,
            UiSurfaceRole::Region => FRAME_MATERIAL,
            UiSurfaceRole::Panel => INNER_MATERIAL,
            UiSurfaceRole::Card => INNER_MATERIAL,
            UiSurfaceRole::Toolbar => FRAME_MATERIAL,
            UiSurfaceRole::Raised => ACCENT_MATERIAL,
            UiSurfaceRole::Selected => ACCENT_MATERIAL,
            UiSurfaceRole::Accent => ACCENT_MATERIAL,
            UiSurfaceRole::Overlay => MUTED_MATERIAL,
        }
    }

    fn draw_surface_commands(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        commands: &[UiSurfaceCommand],
        cached_meshes: &mut HashMap<MeshHandle, Mesh>,
        window_size: [f32; 2],
    ) {
        let mut batches: Vec<(
            UiSurfaceVectorLayerKind,
            UiSurfaceRole,
            Option<ui_tools::UiPixelRect>,
            Vec<[f32; 3]>,
        )> = Vec::new();
        for command in commands {
            for layer in lower_surface_to_vector(command) {
                let viewport = match layer.clip {
                    Some(clip) => clip.to_pixel_rect(window_size),
                    None => None,
                };
                if layer.clip.is_some() && viewport.is_none() {
                    continue;
                }
                let Ok(positions) = tessellate_convex_fill(&layer.path) else {
                    continue;
                };
                let batch = batches.iter_mut().find(|(kind, role, batch_viewport, _)| {
                    *kind == layer.kind && *role == layer.role && *batch_viewport == viewport
                });
                if let Some((_, _, _, batch_positions)) = batch {
                    batch_positions.extend(positions.into_iter().map(|[x, y]| [x, y, 0.0]));
                } else {
                    batches.push((
                        layer.kind,
                        layer.role,
                        viewport,
                        positions.into_iter().map(|[x, y]| [x, y, 0.0]).collect(),
                    ));
                }
            }
        }

        batches.sort_by_key(|(kind, _, _, _)| match kind {
            UiSurfaceVectorLayerKind::Shadow => 0,
            UiSurfaceVectorLayerKind::Border => 1,
            UiSurfaceVectorLayerKind::Fill => 2,
        });
        for (index, (kind, role, viewport, positions)) in batches.into_iter().enumerate() {
            let mesh_handle = MeshHandle(SURFACE_MESH_BASE + index as u64);
            let mesh = Mesh::uniform_normal(positions, [0.0, 0.0, 1.0]);
            if cached_meshes.get(&mesh_handle) != Some(&mesh) {
                renderer.upload_mesh(mesh_handle, &mesh);
                cached_meshes.insert(mesh_handle, mesh);
            }
            let material = match kind {
                UiSurfaceVectorLayerKind::Shadow => MUTED_MATERIAL,
                UiSurfaceVectorLayerKind::Border | UiSurfaceVectorLayerKind::Fill => {
                    Self::material_for_role(role)
                }
            };
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: mesh_handle,
                material,
                pipeline,
                instance: Instance2d::new([0.0, 0.0], [1.0, 1.0], 0.0),
                camera: Some(CAMERA_HANDLE),
                viewport: viewport.map(|rect| ViewportRect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                }),
            })]);
        }
    }

    fn draw_text_command(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        command: &ui_tools::UiTextCommand,
    ) {
        let commands = layout_bitmap_text(&command.spec, command.style.height)
            .into_iter()
            .map(|quad| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: GLYPH_MESH,
                    material: TEXT_MATERIAL,
                    pipeline,
                    instance: Instance2d::new(quad.center, quad.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                })
            })
            .collect::<Vec<_>>();
        renderer.submit(&commands);
    }
}

impl PlatformEventHandler for HelloUiBoxApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-box-backdrop", Color::rgb(0.05, 0.06, 0.09)),
        )?;
        renderer.upload_material(
            FRAME_MATERIAL,
            &Material::new("ui-box-frame", Color::rgb(0.20, 0.22, 0.28)),
        )?;
        renderer.upload_material(
            INNER_MATERIAL,
            &Material::new("ui-box-inner", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            ACCENT_MATERIAL,
            &Material::new("ui-box-accent", Color::rgb(0.32, 0.54, 0.82)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-box-muted", Color::rgb(0.10, 0.12, 0.15)),
        )?;
        renderer.upload_material(
            TEXT_MATERIAL,
            &Material::new("ui-box-text", Color::rgb(0.86, 0.89, 0.95)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-box-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_camera(
            CAMERA_HANDLE,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.06, 0.09),
        })]);

        let outer = UiRegion::new(
            ui_tools::UiRegionKind::Panel,
            UiSurfaceRole::Panel,
            UiRect::new([0.0, 0.02], [0.88, 0.60]),
        );
        let inner = UiRegion::new(
            ui_tools::UiRegionKind::Panel,
            UiSurfaceRole::Panel,
            UiRect::new([0.0, -0.02], [0.66, 0.36]),
        );
        let strip = UiRegion::new(
            ui_tools::UiRegionKind::Panel,
            UiSurfaceRole::Raised,
            UiRect::new([0.0, 0.26], [0.66, 0.08]),
        );
        let label = UiLabel::new("BOX", UiLabelAnchor::Center, [0.0, 0.26]);
        let subtitle = UiLabel::new("BOUNDS NESTING SCALE", UiLabelAnchor::Center, [0.0, -0.245]);

        let mut surfaces = Vec::new();
        let mut text = Vec::new();
        {
            let mut drawer = ui_tools::UiDrawer::new(&mut surfaces, &mut text, &self.theme);
            drawer.surface(&outer);
            drawer.surface(&inner);
            drawer.surface(&strip);
            drawer.label(&label, UiTextRole::Title);
            drawer.label(&subtitle, UiTextRole::Caption);
        }

        // The outer frame is deliberately square; the nested frame remains
        // rounded so the corpus compares both geometry variants directly.
        if let Some(outer_command) = surfaces.first_mut() {
            outer_command.style.radius = UiRadius::None;
        }

        // Exercise semantic clip metadata and renderer scissor lowering with
        // one intentionally partial nested surface.
        if let Some(inner_command) = surfaces.get_mut(1) {
            inner_command.clip = Some(UiRect::new([0.0, -0.02], [0.48, 0.26]));
        }

        Self::draw_surface_commands(
            renderer,
            self.pipeline,
            &surfaces,
            &mut self.surface_meshes,
            self.window_size,
        );
        for command in text {
            Self::draw_text_command(renderer, self.pipeline, &command);
        }

        let stats = renderer.present()?;
        self.frame_count = self.frame_count.saturating_add(1);
        if !self.stats_reported {
            println!(
                "hello-ui-box first-frame stats: draw_calls={}, cumulative_mesh_uploads={}, mesh_replacements={}",
                stats.draw_calls, stats.mesh_uploads, stats.mesh_replacements
            );
            self.stats_reported = true;
        } else if self.frame_count == 2 {
            println!(
                "hello-ui-box second-frame stats: draw_calls={}, cumulative_mesh_uploads={}, mesh_replacements={}",
                stats.draw_calls, stats.mesh_uploads, stats.mesh_replacements
            );
        }
        Ok(FrameOutcome::Continue)
    }
}
