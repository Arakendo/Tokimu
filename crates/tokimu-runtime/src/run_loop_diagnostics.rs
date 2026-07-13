use crate::RunLoopSummary;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunLoopDiagnostics {
    frame_count: u64,
    total_fixed_updates: u64,
    fixed_step_cap_hits: u64,
    last_frame_delta_seconds: f64,
    max_frame_delta_seconds: f64,
    last_summary: Option<RunLoopSummary>,
}

impl RunLoopDiagnostics {
    pub fn record_frame(&mut self, summary: RunLoopSummary) {
        self.frame_count += 1;
        self.total_fixed_updates += u64::from(summary.fixed_updates);
        if summary.hit_fixed_step_cap {
            self.fixed_step_cap_hits += 1;
        }

        self.last_frame_delta_seconds = summary.frame_delta_seconds;
        self.max_frame_delta_seconds = self
            .max_frame_delta_seconds
            .max(summary.frame_delta_seconds);
        self.last_summary = Some(summary);
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn total_fixed_updates(&self) -> u64 {
        self.total_fixed_updates
    }

    pub fn fixed_step_cap_hits(&self) -> u64 {
        self.fixed_step_cap_hits
    }

    pub fn last_frame_delta_seconds(&self) -> f64 {
        self.last_frame_delta_seconds
    }

    pub fn max_frame_delta_seconds(&self) -> f64 {
        self.max_frame_delta_seconds
    }

    pub fn last_summary(&self) -> Option<RunLoopSummary> {
        self.last_summary
    }
}
