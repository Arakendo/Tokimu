use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use tokimu_input::{InputState, KeyCode};
use ui_tools::{
    layout_bitmap_text, measure_bitmap_text_width, UiRect, UiTextAlign, UiTextInputOperation,
    UiTextInputState, UiTextRole, UiTextSpec,
};

const QUAD_MESH: MeshHandle = MeshHandle(1);
const GLYPH_MESH: MeshHandle = MeshHandle(2);
const CAMERA: CameraHandle = CameraHandle(1);
const BACKDROP: MaterialHandle = MaterialHandle(1);
const PANEL: MaterialHandle = MaterialHandle(2);
const ACTIVE: MaterialHandle = MaterialHandle(3);
const TEXT: MaterialHandle = MaterialHandle(4);
const MUTED: MaterialHandle = MaterialHandle(5);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Text Input".into(),
            width: 1100,
            height: 700,
        },
        HelloUiTextInput::default(),
    )
}

struct HelloUiTextInput {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    input_events: InputState,
    input: UiTextInputState,
    focused: bool,
    submitted: String,
}

impl Default for HelloUiTextInput {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            input_events: InputState::default(),
            input: UiTextInputState::new(""),
            focused: false,
            submitted: "NOT SUBMITTED".into(),
        }
    }
}

impl HelloUiTextInput {
    fn field(&self) -> UiRect {
        UiRect::new([0.0, 0.02], [1.30, 0.22])
    }

    fn cursor_world(&self) -> [f32; 2] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_width = width / height;
        [
            self.input_mouse_x() / width * half_width * 2.0 - half_width,
            1.0 - self.input_mouse_y() / height * 2.0,
        ]
    }

    fn input_mouse_x(&self) -> f32 {
        self.input_events.mouse.x
    }

    fn input_mouse_y(&self) -> f32 {
        self.input_events.mouse.y
    }

    fn text_commands(&self) -> Vec<RenderCommand> {
        let field = self.field();
        let value = if self.input.value().is_empty() {
            "TYPE INTO THE FIELD WITH THE CURRENT TEXT INPUT API"
        } else {
            self.input.value()
        };
        let value_spec = UiTextSpec::new(value, field.inset(0.08), UiTextRole::Body)
            .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        let title = UiTextSpec::new("TEXT INPUT", UiRect::new([0.0, 0.52], [1.4, 0.1]), UiTextRole::Title)
            .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let status = UiTextSpec::new(
            if self.focused { "FOCUSED | ARROWS MOVE | SPACE INSERTS" } else { "CLICK FIELD TO FOCUS" },
            UiRect::new([0.0, -0.36], [1.6, 0.08]), UiTextRole::Caption,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let submitted = UiTextSpec::new(
            &self.submitted,
            UiRect::new([0.0, -0.52], [1.6, 0.08]),
            UiTextRole::Status,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);

        let mut commands: Vec<RenderCommand> = [
            (title, TEXT),
            (value_spec, TEXT),
            (status, MUTED),
            (submitted, MUTED),
        ]
        .into_iter()
        .flat_map(|(spec, material)| {
            layout_bitmap_text(&spec, match spec.role {
                UiTextRole::Title => 0.07,
                UiTextRole::Body => 0.045,
                _ => 0.03,
            })
            .into_iter()
            .map(move |quad| RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: GLYPH_MESH,
                material,
                pipeline: self.pipeline,
                instance: Instance2d::new(quad.center, quad.size, 0.0),
                camera: Some(CAMERA),
                viewport: None,
            }))
        })
        .collect();

        if self.focused {
            let prefix: String = self.input.value().chars().take(self.input.caret()).collect();
            let caret_x = field.center[0] - field.size[0] * 0.5
                + 0.08
                + measure_bitmap_text_width(&prefix, 0.045)
                + 0.01;
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD_MESH,
                material: ACTIVE,
                pipeline: self.pipeline,
                instance: Instance2d::new([caret_x, field.center[1]], [0.008, 0.12], 0.0),
                camera: Some(CAMERA),
                viewport: None,
            }));
        }
        commands
    }
}

impl PlatformEventHandler for HelloUiTextInput {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        window.set_ime_allowed(true);
        self.window_size = [size.width as f32, size.height as f32];
        self.window = Some(window.clone());
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD_MESH, &Mesh::quad());
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        renderer.upload_material(BACKDROP, &Material::new("text-input-backdrop", Color::rgb(0.05, 0.06, 0.08)))?;
        renderer.upload_material(PANEL, &Material::new("text-input-panel", Color::rgb(0.20, 0.23, 0.29)))?;
        renderer.upload_material(ACTIVE, &Material::new("text-input-active", Color::rgb(0.35, 0.58, 0.86)))?;
        renderer.upload_material(TEXT, &Material::new("text-input-text", Color::rgb(0.92, 0.94, 0.98)))?;
        renderer.upload_material(MUTED, &Material::new("text-input-muted", Color::rgb(0.58, 0.64, 0.72)))?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new("hello-ui-textinput", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let Some(input_event) = event.as_input_event() {
            self.input_events.apply_event(input_event);
        }
        match event {
            PlatformInputEvent::TextInput(text) if self.focused => {
                for character in text.chars() {
                    self.input.apply(UiTextInputOperation::Insert(character));
                }
            }
            PlatformInputEvent::CursorMoved { .. } => {}
            PlatformInputEvent::MouseInput { button: MouseButton::Left, pressed: true } => {
                self.focused = self.field().contains(self.cursor_world());
            }
            PlatformInputEvent::KeyboardInput { key, pressed: true } if self.focused => {
                match key {
                    KeyCode::ArrowLeft => self.input.apply(UiTextInputOperation::MoveLeft),
                    KeyCode::ArrowRight => self.input.apply(UiTextInputOperation::MoveRight),
                    KeyCode::Backspace => self.input.apply(UiTextInputOperation::DeleteBackward),
                    KeyCode::Delete => self.input.apply(UiTextInputOperation::DeleteForward),
                    KeyCode::KeyQ => self.input.apply(UiTextInputOperation::SelectAll),
                    KeyCode::Enter | KeyCode::KeyE => self.submitted = self.input.value().to_owned(),
                    _ => {}
                }
            }
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width as f32, height as f32];
                if let Some(renderer) = self.renderer.as_mut() { renderer.resize_surface(width, height); }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let text_commands = self.text_commands();
        let field = self.field();
        let Some(renderer) = self.renderer.as_mut() else { return Ok(FrameOutcome::Continue); };
        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.window_size[0], self.window_size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: Color::rgb(0.05, 0.06, 0.08) })]);
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: QUAD_MESH,
            material: if self.focused { ACTIVE } else { PANEL },
            pipeline: self.pipeline,
            instance: Instance2d::new(field.center, field.size, 0.0),
            camera: Some(CAMERA), viewport: None,
        })]);
        renderer.submit(&text_commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
