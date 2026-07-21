use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{
    layout_bitmap_text, UiRect, UiTextAlign, UiTextRole, UiTextSpec, UiTheme, TEXT_CORPUS,
};

const GLYPH_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const TITLE_MATERIAL: MaterialHandle = MaterialHandle(2);
const BODY_MATERIAL: MaterialHandle = MaterialHandle(3);
const CAPTION_MATERIAL: MaterialHandle = MaterialHandle(4);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(5);

fn corpus_text(id: &str) -> &'static str {
    TEXT_CORPUS
        .iter()
        .find(|sample| sample.id == id)
        .unwrap_or_else(|| panic!("missing text corpus sample: {id}"))
        .text
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Text".into(),
            width: 1260,
            height: 760,
        },
        HelloUiTextApp::new(),
    )
}

struct HelloUiTextApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
}

impl Default for HelloUiTextApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
        }
    }
}

impl HelloUiTextApp {
    fn new() -> Self {
        Self::default()
    }

    fn material_for_role(role: UiTextRole) -> MaterialHandle {
        match role {
            UiTextRole::Title => TITLE_MATERIAL,
            UiTextRole::Heading => BODY_MATERIAL,
            UiTextRole::Body => BODY_MATERIAL,
            UiTextRole::Caption => CAPTION_MATERIAL,
            UiTextRole::Button => TITLE_MATERIAL,
            UiTextRole::Chip => CAPTION_MATERIAL,
            UiTextRole::Status => MUTED_MATERIAL,
        }
    }

    fn build_text_commands(
        spec: &UiTextSpec,
        height: f32,
        material: MaterialHandle,
        pipeline: PipelineHandle,
    ) -> Vec<RenderCommand> {
        layout_bitmap_text(spec, height)
            .into_iter()
            .map(|quad| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: GLYPH_MESH,
                    material,
                    pipeline,
                    instance: Instance2d::new(quad.center, quad.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                })
            })
            .collect()
    }
}

impl PlatformEventHandler for HelloUiTextApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            TITLE_MATERIAL,
            &Material::new("ui-title", Color::rgb(0.92, 0.94, 0.98)),
        )?;
        renderer.upload_material(
            BODY_MATERIAL,
            &Material::new("ui-body", Color::rgb(0.74, 0.79, 0.87)),
        )?;
        renderer.upload_material(
            CAPTION_MATERIAL,
            &Material::new("ui-caption", Color::rgb(0.58, 0.64, 0.72)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-muted", Color::rgb(0.42, 0.46, 0.54)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-text-pipeline",
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
            color: Color::rgb(0.05, 0.06, 0.08),
        })]);

        let title = UiTextSpec::new(
            corpus_text("title"),
            UiRect::new([0.0, 0.39], [0.92, 0.10]),
            UiTextRole::Title,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let subtitle = UiTextSpec::new(
            corpus_text("subtitle"),
            UiRect::new([0.0, 0.29], [0.92, 0.07]),
            UiTextRole::Caption,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let heading = UiTextSpec::new(
            "ROLE SCALE",
            UiRect::new([-0.45, 0.19], [0.36, 0.07]),
            UiTextRole::Heading,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        let body = UiTextSpec::new(
            corpus_text("body"),
            UiRect::new([-0.45, 0.10], [0.36, 0.07]),
            UiTextRole::Body,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        let caption = UiTextSpec::new(
            corpus_text("caption"),
            UiRect::new([-0.45, 0.01], [0.36, 0.07]),
            UiTextRole::Caption,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        let status = UiTextSpec::new(
            corpus_text("status"),
            UiRect::new([-0.45, -0.08], [0.36, 0.07]),
            UiTextRole::Status,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        let alignment = UiTextSpec::new(
            "ALIGNMENT",
            UiRect::new([0.45, 0.19], [0.36, 0.07]),
            UiTextRole::Heading,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let left = UiTextSpec::new(
            corpus_text("start"),
            UiRect::new([0.45, 0.10], [0.36, 0.07]),
            UiTextRole::Caption,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        let center = UiTextSpec::new(
            corpus_text("center"),
            UiRect::new([0.45, 0.01], [0.36, 0.07]),
            UiTextRole::Caption,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let right = UiTextSpec::new(
            corpus_text("end"),
            UiRect::new([0.45, -0.08], [0.36, 0.07]),
            UiTextRole::Caption,
        )
        .with_alignment(UiTextAlign::End, UiTextAlign::Center);
        let clipped = UiTextSpec::new(
            corpus_text("clipped"),
            UiRect::new([0.0, -0.29], [0.46, 0.07]),
            UiTextRole::Status,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);

        for spec in [
            title, subtitle, heading, body, caption, status, alignment, left, center, right,
            clipped,
        ] {
            let style = self.theme.text(spec.role);
            let material = Self::material_for_role(spec.role);
            let commands = Self::build_text_commands(&spec, style.height, material, self.pipeline);
            renderer.submit(&commands);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
