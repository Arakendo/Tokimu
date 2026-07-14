pub mod clock;
pub mod input;
#[cfg(not(target_arch = "wasm32"))]
pub mod openxr;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod wasm;
pub mod window;

pub type PlatformResult<T> = Result<T, Box<dyn std::error::Error>>;

pub use clock::Clock;
pub use input::{PlatformEventHandler, PlatformInputEvent};
#[cfg(not(target_arch = "wasm32"))]
pub use native::run_window;
#[cfg(not(target_arch = "wasm32"))]
pub use native::run_window_with_app;
#[cfg(not(target_arch = "wasm32"))]
pub use native::run_window_with_handler;
#[cfg(not(target_arch = "wasm32"))]
pub use native::NativeError;
pub use winit::window::Window as NativeWindow;
pub use window::WindowConfig;
