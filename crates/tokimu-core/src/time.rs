#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedTimeStep {
    pub step_seconds: f64,
}

impl Default for FixedTimeStep {
    fn default() -> Self {
        Self {
            step_seconds: 1.0 / 60.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeStepPolicy {
    Fixed(FixedTimeStep),
    Variable,
}

impl Default for TimeStepPolicy {
    fn default() -> Self {
        Self::Fixed(FixedTimeStep::default())
    }
}
