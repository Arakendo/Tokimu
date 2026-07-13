use crate::RuntimeConfig;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RunLoopSummary {
    pub frame_delta_seconds: f64,
    pub fixed_updates: u32,
    pub requested_fixed_updates: u32,
    pub hit_fixed_step_cap: bool,
    pub accumulator_seconds: f64,
}

pub fn tick_fixed_updates(
    config: RuntimeConfig,
    mut accumulator_seconds: f64,
    frame_delta_seconds: f64,
) -> RunLoopSummary {
    accumulator_seconds += frame_delta_seconds;

    let requested_fixed_updates =
        (accumulator_seconds / config.fixed_time_step_seconds).floor() as u32;

    let mut fixed_updates = 0;
    while accumulator_seconds >= config.fixed_time_step_seconds
        && fixed_updates < config.max_fixed_steps_per_frame
    {
        accumulator_seconds -= config.fixed_time_step_seconds;
        fixed_updates += 1;
    }

    RunLoopSummary {
        frame_delta_seconds,
        fixed_updates,
        requested_fixed_updates,
        hit_fixed_step_cap: requested_fixed_updates > fixed_updates,
        accumulator_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_respects_max_fixed_steps() {
        let config = RuntimeConfig {
            fixed_time_step_seconds: 0.25,
            max_fixed_steps_per_frame: 2,
        };

        let result = tick_fixed_updates(config, 0.0, 1.0);
        assert_eq!(result.frame_delta_seconds, 1.0);
        assert_eq!(result.fixed_updates, 2);
        assert_eq!(result.requested_fixed_updates, 4);
        assert!(result.hit_fixed_step_cap);
        assert_eq!(result.accumulator_seconds, 0.5);
    }

    #[test]
    fn tick_reports_uncapped_frames() {
        let config = RuntimeConfig {
            fixed_time_step_seconds: 0.25,
            max_fixed_steps_per_frame: 8,
        };

        let result = tick_fixed_updates(config, 0.0, 0.5);
        assert_eq!(result.fixed_updates, 2);
        assert_eq!(result.requested_fixed_updates, 2);
        assert!(!result.hit_fixed_step_cap);
        assert_eq!(result.accumulator_seconds, 0.0);
    }
}
