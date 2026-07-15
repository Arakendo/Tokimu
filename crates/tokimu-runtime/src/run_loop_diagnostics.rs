use crate::RunLoopSummary;
use std::fmt;

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

impl fmt::Display for RunLoopDiagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "run loop timing")?;
        writeln!(f, "  frame count: {}", self.frame_count)?;
        writeln!(f, "  total fixed updates: {}", self.total_fixed_updates)?;
        writeln!(f, "  fixed step cap hits: {}", self.fixed_step_cap_hits)?;
        writeln!(
            f,
            "  last frame delta seconds: {:.4}",
            self.last_frame_delta_seconds
        )?;
        writeln!(
            f,
            "  max frame delta seconds: {:.4}",
            self.max_frame_delta_seconds
        )?;
        if let Some(summary) = self.last_summary {
            writeln!(f, "  last summary: {:?}", summary)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_timing_panel_text() {
        let mut diagnostics = RunLoopDiagnostics::default();
        diagnostics.record_frame(RunLoopSummary {
            frame_delta_seconds: 0.5,
            fixed_updates: 2,
            requested_fixed_updates: 2,
            hit_fixed_step_cap: false,
            accumulator_seconds: 0.0,
        });

        assert_eq!(
            format!("{diagnostics}"),
            concat!(
                "run loop timing\n",
                "  frame count: 1\n",
                "  total fixed updates: 2\n",
                "  fixed step cap hits: 0\n",
                "  last frame delta seconds: 0.5000\n",
                "  max frame delta seconds: 0.5000\n",
                "  last summary: RunLoopSummary { frame_delta_seconds: 0.5, fixed_updates: 2, requested_fixed_updates: 2, hit_fixed_step_cap: false, accumulator_seconds: 0.0 }\n"
            )
        );
    }
}
