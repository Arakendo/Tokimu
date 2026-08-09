use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

/// Provider-local monotonic timer for optional CPU-call diagnostics.
pub(super) struct CpuTimer {
    #[cfg(not(target_arch = "wasm32"))]
    started_at: Instant,
    #[cfg(target_arch = "wasm32")]
    started_at_milliseconds: f64,
}

impl CpuTimer {
    /// Starts a timer when the provider exposes a monotonic clock.
    pub(super) fn start() -> Option<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Some(Self {
                started_at: Instant::now(),
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|window| window.performance())
                .map(|performance| Self {
                    started_at_milliseconds: performance.now(),
                })
        }
    }

    pub(super) fn elapsed(self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.started_at.elapsed()
        }

        #[cfg(target_arch = "wasm32")]
        {
            let elapsed_milliseconds = web_sys::window()
                .and_then(|window| window.performance())
                .map(|performance| performance.now() - self.started_at_milliseconds)
                .unwrap_or(0.0)
                .max(0.0);
            Duration::from_secs_f64(elapsed_milliseconds / 1_000.0)
        }
    }
}
