#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeConfig {
    pub fixed_time_step_seconds: f64,
    pub max_fixed_steps_per_frame: u32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            fixed_time_step_seconds: 1.0 / 60.0,
            max_fixed_steps_per_frame: 8,
        }
    }
}
