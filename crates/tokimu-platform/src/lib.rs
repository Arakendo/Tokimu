pub mod clock;
pub mod input;
pub mod native;
pub mod wasm;
pub mod window;

pub type PlatformResult<T> = Result<T, Box<dyn std::error::Error>>;

pub use clock::Clock;
pub use input::{PlatformEventHandler, PlatformInputEvent};
pub use native::run_window;
pub use native::run_window_with_app;
pub use native::run_window_with_handler;
pub use native::NativeError;
pub use winit::window::Window as NativeWindow;
pub use window::WindowConfig;
