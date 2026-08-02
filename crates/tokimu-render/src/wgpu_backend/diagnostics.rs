use std::sync::{Arc, Mutex};

use super::WgpuBackend;

pub(super) fn install_backend_diagnostic_sink(device: &wgpu::Device) -> Arc<Mutex<Vec<String>>> {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&messages);
    device.on_uncaptured_error(Box::new(move |error| {
        let mut messages = match sink.lock() {
            Ok(messages) => messages,
            Err(poisoned) => poisoned.into_inner(),
        };
        messages.push(format!("WebGPU backend validation failed: {error}"));
    }));
    messages
}

pub(super) fn drain_backend_diagnostic_messages(
    messages: &Mutex<Vec<String>>,
) -> Vec<tokimu_core::DiagnosticRecord> {
    let mut messages = match messages.lock() {
        Ok(messages) => messages,
        Err(poisoned) => poisoned.into_inner(),
    };
    std::mem::take(&mut *messages)
        .into_iter()
        .map(|message| {
            tokimu_core::DiagnosticRecord::error(
                tokimu_core::DiagnosticKind::BackendError,
                "tokimu-render.wgpu",
                message,
            )
        })
        .collect()
}

impl WgpuBackend {
    /// Drains renderer-adapter diagnostics without exposing backend-native error types.
    ///
    /// WebGPU shader and pipeline validation can be reported after a synchronous
    /// pipeline creation call returns. The backend records those messages in its
    /// error callback and presents them here as Tokimu diagnostics for callers to
    /// route alongside their own application diagnostics.
    pub fn drain_diagnostics(&self) -> Vec<tokimu_core::DiagnosticRecord> {
        drain_backend_diagnostic_messages(&self.backend_diagnostic_messages)
    }

    /// Flushes backend work and callbacks before diagnostics are inspected.
    ///
    /// Native WebGPU validation may be reported asynchronously after resource
    /// creation returns. Presentation diagnostics use this bounded adapter hook
    /// instead of exposing `wgpu::Device` to callers.
    pub fn poll_diagnostics(&self) {
        let _ = self._device.poll(wgpu::Maintain::Wait);
    }

    pub(super) fn record_backend_diagnostic(&self, message: impl Into<String>) {
        let mut messages = match self.backend_diagnostic_messages.lock() {
            Ok(messages) => messages,
            Err(poisoned) => poisoned.into_inner(),
        };
        messages.push(message.into());
    }
}
