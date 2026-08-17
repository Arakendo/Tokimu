//! Native GPU evidence for AR-0030's unstable G2 renderer intake.
//!
//! The source-derived depth declarations are rebuilt for every frame and have
//! no persistent mesh identity. Persistent background, far-wall, and near
//! controls are presented beside them to prove both the depth relationship and
//! that the experimental intake does not change the existing persistent-
//! resource path.

use std::sync::Arc;

use hello_doom_visibility_conformance::{
    observe_authoritative_sky_regions, prepare_authoritative_sky_depth_declarations,
    prepare_authoritative_sky_submission_local_geometry, terminal_sky_ordered_fixture,
    SubmissionIdentity, SubmissionLocalGeometryLimits,
};
use tokimu::{
    experimental_submission_local_geometry::{
        ExperimentalLocalGeometryDraw, ExperimentalSubmissionIdentity,
        ExperimentalSubmissionLocalGeometry, ExperimentalSubmissionLocalGeometryBuilder,
    },
    run_window_with_app, BlendMode, Camera, CameraHandle, ClearCommand, Color, ColorWriteMask,
    CullMode, DepthTest, DrawMeshCommand, FrameOutcome, Instance2d, Material, MaterialHandle, Mesh,
    MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineKind, PipelineRenderState,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer, WgpuBackend,
    WindowConfig,
};

const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const FAR_WALL_MESH: MeshHandle = MeshHandle(2);
const NEAR_OBJECT_MESH: MeshHandle = MeshHandle(3);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const FAR_WALL_MATERIAL: MaterialHandle = MaterialHandle(2);
const SKY_DEPTH_MATERIAL: MaterialHandle = MaterialHandle(3);
const NEAR_OBJECT_MATERIAL: MaterialHandle = MaterialHandle(4);
const MISSING_MATERIAL: MaterialHandle = MaterialHandle(999);
const CAMERA: CameraHandle = CameraHandle(1);

fn main() -> PlatformResult<()> {
    let exit_after_evidence = std::env::args().any(|argument| argument == "--exit-after-evidence");
    run_window_with_app(
        WindowConfig {
            title: "Tokimu AR-0030 G2 submission-local geometry | loading".into(),
            width: 960,
            height: 600,
        },
        SubmissionLocalGeometryPresentation {
            exit_after_evidence,
            ..SubmissionLocalGeometryPresentation::default()
        },
    )
}

#[derive(Default)]
struct SubmissionLocalGeometryPresentation {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [u32; 2],
    background_pipeline: PipelineHandle,
    local_depth_pipeline: PipelineHandle,
    far_wall_pipeline: PipelineHandle,
    frame: u64,
    persistent_uploads: u64,
    persistent_replacements: u64,
    baseline_geometry_fingerprint: Option<String>,
    exit_after_evidence: bool,
}

impl SubmissionLocalGeometryPresentation {
    fn source_submission(
        submission: u64,
        source_x_jitter: i16,
        material: MaterialHandle,
        pipeline: PipelineHandle,
    ) -> PlatformResult<(ExperimentalSubmissionLocalGeometry, String)> {
        let mut fixture = terminal_sky_ordered_fixture()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        fixture.viewer.position[0] = fixture.viewer.position[0].saturating_add(source_x_jitter);
        let regions = observe_authoritative_sky_regions(&fixture, 41, "static-source-fixture")
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let depth = prepare_authoritative_sky_depth_declarations(&regions, 0.25, "doom-sky:SKY1");
        let source = prepare_authoritative_sky_submission_local_geometry(
            &depth,
            SubmissionIdentity(submission),
            SubmissionLocalGeometryLimits::default(),
        )
        .map_err(|error| std::io::Error::other(error.to_string()))?;
        let geometry_fingerprint = blake3::hash(
            format!(
                "source-x-jitter={source_x_jitter};positions={:?}",
                source
                    .payloads
                    .iter()
                    .map(|payload| &payload.positions)
                    .collect::<Vec<_>>()
            )
            .as_bytes(),
        )
        .to_hex()
        .to_string();

        let mut builder = ExperimentalSubmissionLocalGeometryBuilder::new(
            ExperimentalSubmissionIdentity(submission),
        );
        for payload in &source.payloads {
            let geometry = builder
                .add_geometry(Mesh::uniform_normal(
                    payload.positions.clone(),
                    [0.0, 0.0, -1.0],
                ))
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            builder
                .add_draw(ExperimentalLocalGeometryDraw {
                    geometry,
                    material,
                    pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA),
                    viewport: None,
                    material_override: None,
                })
                .map_err(|error| std::io::Error::other(error.to_string()))?;
        }
        let submission = builder
            .finish()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        Ok((submission, geometry_fingerprint))
    }

    fn submit_background_control(&mut self) {
        let renderer = self.renderer.as_mut().expect("renderer initialized");
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.015, 0.02, 0.03),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: BACKGROUND_MESH,
                material: BACKGROUND_MATERIAL,
                pipeline: self.background_pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            }),
        ]);
    }

    fn submit_far_wall_control(&mut self) {
        let renderer = self.renderer.as_mut().expect("renderer initialized");
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: FAR_WALL_MESH,
            material: FAR_WALL_MATERIAL,
            pipeline: self.far_wall_pipeline,
            instance: Instance2d::identity(),
            camera: Some(CAMERA),
            viewport: None,
        })]);
    }

    fn submit_near_object_control(&mut self) {
        let renderer = self.renderer.as_mut().expect("renderer initialized");
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: NEAR_OBJECT_MESH,
            material: NEAR_OBJECT_MATERIAL,
            pipeline: self.far_wall_pipeline,
            instance: Instance2d::identity(),
            camera: Some(CAMERA),
            viewport: None,
        })]);
    }
}

impl PlatformEventHandler for SubmissionLocalGeometryPresentation {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window.clone(), self.size[0], self.size[1])?;
        self.background_pipeline = renderer.register_pipeline(
            &Pipeline::new("g2-background", PipelineKind::SolidColor2d).with_render_state(
                PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                },
            )?,
        )?;
        self.local_depth_pipeline = renderer.register_pipeline(
            &Pipeline::new("g2-local-depth", PipelineKind::SolidColor2d).with_render_state(
                PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::NONE,
                },
            )?,
        )?;
        self.far_wall_pipeline = renderer.register_pipeline(
            &Pipeline::new("g2-far-wall", PipelineKind::SolidColor2d).with_render_state(
                PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                },
            )?,
        )?;
        renderer.upload_mesh(BACKGROUND_MESH, &clip_quad(0.90, -0.95, 0.95, -0.80, 0.80));
        renderer.upload_mesh(FAR_WALL_MESH, &clip_quad(0.50, -0.45, 0.45, -0.65, 0.45));
        renderer.upload_mesh(NEAR_OBJECT_MESH, &clip_quad(0.10, -0.16, 0.16, -0.20, 0.20));
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("g2-background", Color::rgb(0.08, 0.24, 0.48)),
        )?;
        renderer.upload_material(
            FAR_WALL_MATERIAL,
            &Material::new("g2-far-wall", Color::rgb(0.94, 0.35, 0.18)),
        )?;
        renderer.upload_material(
            SKY_DEPTH_MATERIAL,
            &Material::new("g2-source-depth", Color::rgb(1.0, 0.0, 1.0)),
        )?;
        renderer.upload_material(
            NEAR_OBJECT_MATERIAL,
            &Material::new("g2-near-object", Color::rgb(0.15, 0.85, 0.40)),
        )?;
        renderer.upload_camera(CAMERA, Camera::default());
        window.set_title(&format!(
            "Tokimu AR-0030 G2 submission-local geometry | adapter={}",
            renderer.adapter_name()
        ));
        self.renderer = Some(renderer);
        self.window = Some(window);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1), height.max(1)];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(self.size[0], self.size[1]);
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let identity = 41 + self.frame;
        let source_x_jitter = if self.frame == 1 { 8 } else { 0 };
        let (valid, geometry_fingerprint) = Self::source_submission(
            identity,
            source_x_jitter,
            SKY_DEPTH_MATERIAL,
            self.local_depth_pipeline,
        )?;
        match self.frame {
            0 => self.baseline_geometry_fingerprint = Some(geometry_fingerprint.clone()),
            1 if self.baseline_geometry_fingerprint.as_deref()
                == Some(geometry_fingerprint.as_str()) =>
            {
                return Err(std::io::Error::other(
                    "source-view jitter did not change ephemeral G2 geometry",
                )
                .into());
            }
            2 if self.baseline_geometry_fingerprint.as_deref()
                != Some(geometry_fingerprint.as_str()) =>
            {
                return Err(std::io::Error::other(
                    "returning to the baseline view did not restore G2 geometry",
                )
                .into());
            }
            _ => {}
        }
        let renderer = self.renderer.as_mut().expect("renderer initialized");
        renderer.begin_frame();

        if self.frame == 2 {
            let (invalid, _) =
                Self::source_submission(900, 0, MISSING_MATERIAL, self.local_depth_pipeline)?;
            let rejection = renderer
                .submit_experimental_submission_local_geometry(&invalid)
                .expect_err("missing durable material must reject the entire local batch");
            eprintln!(
                "g2 invalid control: submission=900; rejected={rejection}; recovery=valid-submission-follows"
            );
        }

        self.submit_background_control();
        let renderer = self.renderer.as_mut().expect("renderer initialized");
        let observation = renderer
            .submit_experimental_submission_local_geometry(&valid)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        self.submit_far_wall_control();
        self.submit_near_object_control();
        let renderer = self.renderer.as_mut().expect("renderer initialized");
        if observation.persistent_mesh_identities_created != 0
            || observation.persistent_mesh_replacements != 0
        {
            return Err(std::io::Error::other("G2 intake created persistent mesh identity").into());
        }
        let stats = renderer.present()?;
        if self.frame == 0 {
            self.persistent_uploads = stats.lifetime.mesh_uploads;
            self.persistent_replacements = stats.lifetime.mesh_replacements;
        } else if stats.lifetime.mesh_uploads != self.persistent_uploads
            || stats.lifetime.mesh_replacements != self.persistent_replacements
        {
            return Err(std::io::Error::other(format!(
                "G2 frame mutated persistent mesh resources: baseline={}/{} current={}/{}",
                self.persistent_uploads,
                self.persistent_replacements,
                stats.lifetime.mesh_uploads,
                stats.lifetime.mesh_replacements,
            ))
            .into());
        }
        let diagnostics = renderer.drain_diagnostics();
        if let Some(record) = diagnostics.first() {
            return Err(std::io::Error::other(format!(
                "G2 provider diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ))
            .into());
        }
        let recovery = if self.frame == 2 {
            "after-bounded-rejection"
        } else {
            "ordinary"
        };
        eprintln!(
            "g2 frame: submission={identity}; source_x_jitter={source_x_jitter}; geometry_fingerprint={geometry_fingerprint}; payloads={}; local_draws={}; vertices={}; persistent_mesh_identities=0; persistent_mesh_replacements=0; total_draws={}; lifetime_persistent_uploads={}; lifetime_persistent_replacements={}; recovery={recovery}; diagnostic=none",
            observation.payloads,
            observation.draws,
            observation.vertices,
            stats.frame.draw_calls,
            stats.lifetime.mesh_uploads,
            stats.lifetime.mesh_replacements,
        );
        if let Some(window) = &self.window {
            window.set_title(&format!(
                "Tokimu AR-0030 G2 | submission={identity} local={}/{} persistent=3 diagnostic=none",
                observation.payloads, observation.draws
            ));
        }
        self.frame = self.frame.saturating_add(1);
        if self.exit_after_evidence && self.frame >= 3 {
            Ok(FrameOutcome::Exit)
        } else {
            Ok(FrameOutcome::Continue)
        }
    }
}

fn clip_quad(depth: f32, left: f32, right: f32, bottom: f32, top: f32) -> Mesh {
    Mesh::uniform_normal(
        vec![
            [left, bottom, depth],
            [right, bottom, depth],
            [right, top, depth],
            [left, bottom, depth],
            [right, top, depth],
            [left, top, depth],
        ],
        [0.0, 0.0, -1.0],
    )
}
