//! Corpus-only proof of native terminal error delivery.
//!
//! This intentionally returns one application frame error. `tokimu-platform`
//! closes the active native composition, then returns the error to this caller.
//! The fixture does not claim that the platform presented an in-window error.

#[cfg(not(target_arch = "wasm32"))]
use std::{error::Error, io};
#[cfg(not(target_arch = "wasm32"))]
use tokimu_core::FrameOutcome;
#[cfg(not(target_arch = "wasm32"))]
use tokimu_platform::{
    run_window_with_app, PlatformEventHandler, PlatformInputEvent, PlatformResult, WindowConfig,
};

#[cfg(not(target_arch = "wasm32"))]
struct IntentionalFrameFailure {
    frame_attempted: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl PlatformEventHandler for IntentionalFrameFailure {
    fn on_platform_event(&mut self, _event: PlatformInputEvent) -> PlatformResult<()> {
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        if !self.frame_attempted {
            self.frame_attempted = true;
            println!(
                "AR-0024/0027 native terminal fixture: application-frame-handler returning intentional error"
            );
            return Err(io::Error::other("intentional corpus application-frame failure").into());
        }

        Ok(FrameOutcome::Continue)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn Error>> {
    let result = run_window_with_app(
        WindowConfig {
            title: "Tokimu Failure Boundary | intentional terminal delivery".to_owned(),
            width: 480,
            height: 180,
        },
        IntentionalFrameFailure {
            frame_attempted: false,
        },
    );

    match result {
        Ok(()) => Err("intentional frame failure unexpectedly completed successfully".into()),
        Err(error) => {
            println!(
                "AR-0024/0027 native terminal fixture: terminal caller retained error after active composition ended: {error}"
            );
            Ok(())
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    println!("native_terminal_error is intentionally native-only corpus evidence");
}
