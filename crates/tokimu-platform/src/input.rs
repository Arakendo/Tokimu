use crate::PlatformResult;
use std::sync::Arc;
use tokimu_input::{InputEvent, KeyCode, MouseButton};
use winit::window::Window;

pub trait PlatformEventHandler {
    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()>;

    fn on_native_window_created(&mut self, _window: Arc<Window>) -> PlatformResult<()> {
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<bool> {
        Ok(true)
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlatformInputEvent {
    CloseRequested,
    Resized { width: u32, height: u32 },
    KeyboardInput { key: KeyCode, pressed: bool },
    CursorMoved { x: f32, y: f32 },
    MouseInput { button: MouseButton, pressed: bool },
}

impl PlatformInputEvent {
    pub fn as_input_event(self) -> Option<InputEvent> {
        match self {
            Self::KeyboardInput { key, pressed } => Some(InputEvent::KeyboardInput { key, pressed }),
            Self::CursorMoved { x, y } => Some(InputEvent::CursorMoved { x, y }),
            Self::MouseInput { button, pressed } => Some(InputEvent::MouseInput { button, pressed }),
            Self::CloseRequested | Self::Resized { .. } => None,
        }
    }
}
