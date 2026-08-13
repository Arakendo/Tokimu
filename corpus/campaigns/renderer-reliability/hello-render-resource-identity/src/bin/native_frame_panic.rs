//! Corpus-only fatal-path marker.
//!
//! This intentionally panics from a native frame handler. It exists to retain
//! the negative claim required by Slice 4: Tokimu does not catch an arbitrary
//! panic and continue through potentially corrupted application state.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokimu_core::FrameOutcome;
#[cfg(not(target_arch = "wasm32"))]
use tokimu_platform::{
    run_window_with_app, NativeWindow, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    WindowConfig,
};

#[cfg(not(target_arch = "wasm32"))]
struct IntentionalFatalFrame;

#[cfg(not(target_arch = "wasm32"))]
impl PlatformEventHandler for IntentionalFatalFrame {
    fn on_platform_event(&mut self, _event: PlatformInputEvent) -> PlatformResult<()> {
        Ok(())
    }

    fn on_native_window_created(&mut self, _window: Arc<NativeWindow>) -> PlatformResult<()> {
        println!("AR-0024/0027 native fatal fixture: window created");
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        eprintln!(
            "AR-0024/0027 native fatal fixture: intentionally panicking; no continuation is claimed"
        );
        panic!("intentional corpus fatal frame panic");
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Failure Boundary | intentional fatal marker".to_owned(),
            width: 480,
            height: 180,
        },
        IntentionalFatalFrame,
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    println!("native_frame_panic is intentionally native-only corpus evidence");
}
