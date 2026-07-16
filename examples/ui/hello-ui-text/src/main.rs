use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{UiRect, UiTextAlign, UiTextRole, UiTextSpec, UiTheme};

const GLYPH_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const TITLE_MATERIAL: MaterialHandle = MaterialHandle(2);
const BODY_MATERIAL: MaterialHandle = MaterialHandle(3);
const CAPTION_MATERIAL: MaterialHandle = MaterialHandle(4);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(5);

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
        text: &str,
        position: [f32; 2],
        anchor: UiTextAlign,
        height: f32,
        material: MaterialHandle,
        pipeline: PipelineHandle,
    ) -> Vec<RenderCommand> {
        let cell = (height / 9.0).max(0.0025);
        let width = Self::measure_text_width(text, cell);
        let start_x = match anchor {
            UiTextAlign::Start => position[0],
            UiTextAlign::Center => position[0] - width * 0.5,
            UiTextAlign::End => position[0] - width,
        };
        let top_y = position[1] + height * 0.5 - cell * 0.9;
        let mut x_cursor = start_x;
        let mut commands = Vec::new();

        for ch in text.chars() {
            if ch == ' ' {
                x_cursor += cell * 3.2;
                continue;
            }

            for (row_index, row_bits) in Self::glyph_rows(ch).into_iter().enumerate() {
                for column in 0..5 {
                    let mask = 1 << (4 - column);
                    if row_bits & mask == 0 {
                        continue;
                    }

                    let center_x = x_cursor + column as f32 * cell;
                    let center_y = top_y - row_index as f32 * cell;
                    commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: GLYPH_MESH,
                        material,
                        pipeline,
                        instance: Instance2d::new(
                            [center_x, center_y],
                            [cell * 0.74, cell * 0.74],
                            0.0,
                        ),
                        camera: Some(CAMERA_HANDLE),
                        viewport: None,
                    }));
                }
            }

            x_cursor += cell * 5.2;
        }

        commands
    }

    fn measure_text_width(text: &str, cell: f32) -> f32 {
        text.chars().fold(0.0, |width, ch| {
            width + if ch == ' ' { cell * 3.2 } else { cell * 5.2 }
        })
    }

    fn glyph_rows(ch: char) -> [u8; 7] {
        match ch.to_ascii_uppercase() {
            'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
            'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
            'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
            'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
            'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
            'G' => [0b01111, 0b10000, 0b10000, 0b10011, 0b10001, 0b10001, 0b01111],
            'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'I' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111],
            'J' => [0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110],
            'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
            'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
            'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
            'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
            'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
            'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
            'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
            'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
            'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100],
            'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
            'X' => [0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001],
            'Y' => [0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'Z' => [0b11111, 0b00010, 0b00100, 0b00100, 0b01000, 0b10000, 0b11111],
            '+' => [0b00100, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00100],
            '?' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100],
            _ => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100],
        }
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

        let title = UiTextSpec::new("TEXT", UiRect::new([0.0, 0.36], [0.92, 0.12]), UiTextRole::Title)
            .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let body = UiTextSpec::new("ALIGNMENT / SCALE / ROLE", UiRect::new([0.0, 0.08], [0.92, 0.10]), UiTextRole::Body)
            .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let left = UiTextSpec::new("START", UiRect::new([-0.45, -0.20], [0.36, 0.08]), UiTextRole::Caption)
            .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        let right = UiTextSpec::new("END", UiRect::new([0.45, -0.20], [0.36, 0.08]), UiTextRole::Caption)
            .with_alignment(UiTextAlign::End, UiTextAlign::Center);

        for spec in [title, body, left, right] {
            let style = self.theme.text(spec.role);
            let material = Self::material_for_role(spec.role);
            let commands = Self::build_text_commands(
                spec.text.as_str(),
                spec.rect.center,
                spec.align_x,
                style.height,
                material,
                self.pipeline,
            );
            renderer.submit(&commands);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
