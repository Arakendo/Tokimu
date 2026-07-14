use crate::{PlatformEventHandler, PlatformInputEvent, PlatformResult, WindowConfig};
use tokimu_core::FrameOutcome;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokimu_input::{KeyCode, MouseButton};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Debug)]
pub struct NativeError(String);

impl NativeError {
    fn window_creation(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for NativeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for NativeError {}

const DEFAULT_FRAME_INTERVAL: Duration = Duration::from_nanos(16_666_667);

pub fn backend_name() -> &'static str {
    "native"
}

pub fn run_window(config: WindowConfig) -> PlatformResult<()> {
    run_window_with_handler(config, |_| {})
}

pub fn run_window_with_app<H>(config: WindowConfig, app: H) -> PlatformResult<()>
where
    H: PlatformEventHandler,
{
    run_window_with_handler(config, app)
}

pub fn run_window_with_handler<F>(config: WindowConfig, event_handler: F) -> PlatformResult<()>
where
    F: PlatformEventHandler,
{
    let event_loop = EventLoop::new()?;
    let mut app = NativeWindowApp::new(config, event_handler);
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.pending_error {
        return Err(error);
    }
    Ok(())
}

struct NativeWindowApp<F>
where
    F: PlatformEventHandler,
{
    config: WindowConfig,
    event_handler: F,
    window: Option<Arc<Window>>,
    last_frame: Option<Instant>,
    next_frame_deadline: Option<Instant>,
    pending_error: Option<Box<dyn Error>>,
}

impl<F> NativeWindowApp<F>
where
    F: PlatformEventHandler,
{
    fn new(config: WindowConfig, event_handler: F) -> Self {
        Self {
            config,
            event_handler,
            window: None,
            last_frame: None,
            next_frame_deadline: None,
            pending_error: None,
        }
    }

    fn window_attributes(&self) -> WindowAttributes {
        Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
    }

    fn record_handler_error(&mut self, event_loop: &ActiveEventLoop, error: Box<dyn Error>) {
        self.pending_error = Some(error);
        event_loop.exit();
    }

    fn emit(&mut self, event_loop: &ActiveEventLoop, event: PlatformInputEvent) {
        if let Err(error) = self.event_handler.on_platform_event(event) {
            self.record_handler_error(event_loop, error);
        }
    }
}

impl<F> ApplicationHandler for NativeWindowApp<F>
where
    F: PlatformEventHandler,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + DEFAULT_FRAME_INTERVAL));

        if self.window.is_none() {
            let window = event_loop
                .create_window(self.window_attributes())
                .map_err(|error| NativeError::window_creation(error.to_string()))
                .expect("failed to create native window");
            let now = Instant::now();
            let window = Arc::new(window);
            self.window = Some(window.clone());
            self.last_frame = Some(now);
            self.next_frame_deadline = Some(now + DEFAULT_FRAME_INTERVAL);

            if let Err(error) = self.event_handler.on_native_window_created(window) {
                self.record_handler_error(event_loop, error);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let Some(deadline) = self.next_frame_deadline else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + DEFAULT_FRAME_INTERVAL));
            return;
        };

        if now < deadline {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            return;
        }

        let previous = self.last_frame.replace(now).unwrap_or(now);
        let delta_seconds = now.duration_since(previous).as_secs_f64();
        self.next_frame_deadline = Some(now + DEFAULT_FRAME_INTERVAL);
        match self.event_handler.on_frame(delta_seconds) {
            Ok(FrameOutcome::Continue) => {}
            Ok(FrameOutcome::Exit) => {
                event_loop.exit();
                return;
            }
            Err(error) => {
                self.record_handler_error(event_loop, error);
                return;
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + DEFAULT_FRAME_INTERVAL));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.emit(event_loop, PlatformInputEvent::CloseRequested);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.emit(event_loop, PlatformInputEvent::Resized {
                    width: size.width,
                    height: size.height,
                });
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = map_key_code(event.physical_key) {
                    self.emit(event_loop, PlatformInputEvent::KeyboardInput {
                        key,
                        pressed: event.state == ElementState::Pressed,
                    });
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.emit(event_loop, PlatformInputEvent::CursorMoved {
                    x: position.x as f32,
                    y: position.y as f32,
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) = map_mouse_button(button) {
                    self.emit(event_loop, PlatformInputEvent::MouseInput {
                        button,
                        pressed: state == ElementState::Pressed,
                    });
                }
            }
            _ => {}
        }
    }
}

fn map_key_code(key: PhysicalKey) -> Option<KeyCode> {
    match key {
        PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => Some(KeyCode::Escape),
        PhysicalKey::Code(winit::keyboard::KeyCode::Space) => Some(KeyCode::Space),
        PhysicalKey::Code(winit::keyboard::KeyCode::KeyA) => Some(KeyCode::KeyA),
        PhysicalKey::Code(winit::keyboard::KeyCode::KeyD) => Some(KeyCode::KeyD),
        PhysicalKey::Code(winit::keyboard::KeyCode::KeyS) => Some(KeyCode::KeyS),
        PhysicalKey::Code(winit::keyboard::KeyCode::KeyW) => Some(KeyCode::KeyW),
        PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft) => Some(KeyCode::ArrowLeft),
        PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight) => Some(KeyCode::ArrowRight),
        PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp) => Some(KeyCode::ArrowUp),
        PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown) => Some(KeyCode::ArrowDown),
        _ => None,
    }
}

fn map_mouse_button(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_key_codes() {
        assert_eq!(
            map_key_code(PhysicalKey::Code(winit::keyboard::KeyCode::KeyW)),
            Some(KeyCode::KeyW)
        );
        assert_eq!(
            map_key_code(PhysicalKey::Code(winit::keyboard::KeyCode::Escape)),
            Some(KeyCode::Escape)
        );
        assert_eq!(
            map_key_code(PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft)),
            Some(KeyCode::ArrowLeft)
        );
    }

    #[test]
    fn maps_supported_mouse_buttons() {
        assert_eq!(
            map_mouse_button(winit::event::MouseButton::Left),
            Some(MouseButton::Left)
        );
        assert_eq!(map_mouse_button(winit::event::MouseButton::Back), None);
    }
}
