use crate::PlatformResult;
use std::sync::Arc;
use tokimu_core::FrameOutcome;
use tokimu_input::{InputEvent, KeyCode, MouseButton};
use winit::window::Window;

pub trait PlatformEventHandler {
    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()>;

    fn on_native_window_created(&mut self, _window: Arc<Window>) -> PlatformResult<()> {
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        Ok(FrameOutcome::Continue)
    }
}

impl<F> PlatformEventHandler for F
where
    F: FnMut(PlatformInputEvent),
{
    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        self(event);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlatformInputEvent {
    CloseRequested,
    Resized { width: u32, height: u32 },
    KeyboardInput { key: KeyCode, pressed: bool },
    TextInput(String),
    CursorMoved { x: f32, y: f32 },
    MouseMotion { delta_x: f32, delta_y: f32 },
    MouseInput { button: MouseButton, pressed: bool },
}

impl PlatformInputEvent {
    pub fn as_input_event(&self) -> Option<InputEvent> {
        match self {
            Self::KeyboardInput { key, pressed } => Some(InputEvent::KeyboardInput {
                key: *key,
                pressed: *pressed,
            }),
            Self::TextInput(text) => Some(InputEvent::TextInput(text.clone())),
            Self::CursorMoved { x, y } => Some(InputEvent::CursorMoved { x: *x, y: *y }),
            Self::MouseMotion { .. } => None,
            Self::MouseInput { button, pressed } => Some(InputEvent::MouseInput {
                button: *button,
                pressed: *pressed,
            }),
            Self::CloseRequested | Self::Resized { .. } => None,
        }
    }
}
