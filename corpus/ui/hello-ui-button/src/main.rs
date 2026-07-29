use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{
    layout_bitmap_text, window_to_world, UiButton, UiButtonId, UiButtonSpec, UiControlRole,
    UiDrawer, UiInteractionState, UiRect, UiSurfaceCommand, UiSurfaceRole, UiTheme,
};

const BOX_MESH: MeshHandle = MeshHandle(1);
const GLYPH_MESH: MeshHandle = MeshHandle(2);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const BUTTON_MATERIAL: MaterialHandle = MaterialHandle(2);
const HOVER_MATERIAL: MaterialHandle = MaterialHandle(3);
const ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(4);
const BORDER_MATERIAL: MaterialHandle = MaterialHandle(5);
const TEXT_MATERIAL: MaterialHandle = MaterialHandle(6);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(7);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Button".into(),
            width: 900,
            height: 620,
        },
        HelloUiButtonApp::new(),
    )
}

struct HelloUiButtonApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    cursor_position: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    hovered: bool,
    pressed: bool,
}

impl Default for HelloUiButtonApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            cursor_position: [0.0, 0.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
            hovered: false,
            pressed: false,
        }
    }
}

impl HelloUiButtonApp {
    fn new() -> Self {
        Self::default()
    }

    fn button(&self) -> UiButton {
        let spec = UiButtonSpec::new(UiButtonId(0), "PRESS ME");
        UiButton::from_intrinsic(spec.id, spec.label, [0.0, 0.0], &self.theme)
    }

    fn state(&self) -> UiInteractionState {
        if self.pressed {
            UiInteractionState::Selected
        } else if self.hovered {
            UiInteractionState::Hovered
        } else {
            UiInteractionState::Idle
        }
    }

    fn surface_material(role: UiSurfaceRole, state: UiInteractionState) -> MaterialHandle {
        if matches!(role, UiSurfaceRole::Accent | UiSurfaceRole::Selected)
            || matches!(state, UiInteractionState::Selected)
        {
            ACTIVE_MATERIAL
        } else if matches!(state, UiInteractionState::Hovered) {
            HOVER_MATERIAL
        } else {
            BUTTON_MATERIAL
        }
    }

    fn draw_surface(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        command: &UiSurfaceCommand,
        state: UiInteractionState,
    ) {
        let rect = command.rect;
        if matches!(
            command.style.elevation,
            ui_tools::UiElevation::Raised | ui_tools::UiElevation::Floating
        ) {
            let shadow = UiRect::new([rect.center[0] + 0.012, rect.center[1] - 0.012], rect.size);
            renderer.submit(&[Self::mesh_command(shadow, MUTED_MATERIAL, pipeline)]);
        }

        if let Some(border_role) = command.style.border_role {
            let border = command.style.border_width;
            let border_rect = UiRect::new(
                rect.center,
                [rect.size[0] + border * 2.0, rect.size[1] + border * 2.0],
            );
            renderer.submit(&[Self::mesh_command(
                border_rect,
                if matches!(border_role, UiSurfaceRole::Accent) {
                    ACTIVE_MATERIAL
                } else {
                    BORDER_MATERIAL
                },
                pipeline,
            )]);
        }

        renderer.submit(&[Self::mesh_command(
            rect,
            Self::surface_material(command.style.role, state),
            pipeline,
        )]);
    }

    fn mesh_command(
        rect: UiRect,
        material: MaterialHandle,
        pipeline: PipelineHandle,
    ) -> RenderCommand {
        RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: BOX_MESH,
            material,
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })
    }

    fn draw_text(
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

impl PlatformEventHandler for HelloUiButtonApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;

        renderer.upload_mesh(BOX_MESH, &Mesh::quad());
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        for (handle, name, color) in [
            (
                BACKDROP_MATERIAL,
                "button-backdrop",
                Color::rgb(0.05, 0.06, 0.09),
            ),
            (BUTTON_MATERIAL, "button-idle", Color::rgb(0.20, 0.24, 0.30)),
            (HOVER_MATERIAL, "button-hover", Color::rgb(0.29, 0.38, 0.50)),
            (
                ACTIVE_MATERIAL,
                "button-active",
                Color::rgb(0.38, 0.62, 0.86),
            ),
            (
                BORDER_MATERIAL,
                "button-border",
                Color::rgb(0.12, 0.15, 0.20),
            ),
            (TEXT_MATERIAL, "button-text", Color::rgb(0.94, 0.96, 1.0)),
            (
                MUTED_MATERIAL,
                "button-shadow",
                Color::rgb(0.10, 0.12, 0.16),
            ),
        ] {
            renderer.upload_material(handle, &Material::new(name, color))?;
        }
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-button-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::CursorMoved { x, y } => {
                self.cursor_position = [x, y];
                self.hovered = self
                    .button()
                    .contains(window_to_world(self.window_size, self.cursor_position));
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed,
            } => {
                self.pressed = pressed && self.hovered;
            }
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let button = self.button();
        let state = self.state();
        let pipeline = self.pipeline;
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

        let mut surfaces = Vec::new();
        let mut text = Vec::new();
        {
            let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &self.theme);
            drawer.button(&button, state, UiControlRole::Primary);
        }
        for command in surfaces {
            Self::draw_surface(renderer, pipeline, &command, state);
        }
        for command in text {
            Self::draw_text(renderer, pipeline, &command);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
