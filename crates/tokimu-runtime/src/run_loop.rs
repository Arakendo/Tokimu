use crate::RuntimeConfig;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RunLoopSummary {
    pub fixed_updates: u32,
    pub accumulator_seconds: f64,
}

pub fn tick_fixed_updates(
    config: RuntimeConfig,
    mut accumulator_seconds: f64,
    frame_delta_seconds: f64,
) -> RunLoopSummary {
    accumulator_seconds += frame_delta_seconds;

    let mut fixed_updates = 0;
    while accumulator_seconds >= config.fixed_time_step_seconds
        && fixed_updates < config.max_fixed_steps_per_frame
    {
        accumulator_seconds -= config.fixed_time_step_seconds;
        fixed_updates += 1;
    }

    RunLoopSummary {
        fixed_updates,
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
        assert_eq!(result.fixed_updates, 2);
    }
}
