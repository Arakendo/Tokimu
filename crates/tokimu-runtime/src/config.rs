use std::env;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeConfig {
    pub fixed_time_step_seconds: f64,
    pub max_fixed_steps_per_frame: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputVerbosity {
    Quiet,
    Normal,
    Verbose,
    Trace,
}

impl OutputVerbosity {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "quiet" => Some(Self::Quiet),
            "normal" => Some(Self::Normal),
            "verbose" => Some(Self::Verbose),
            "trace" => Some(Self::Trace),
            _ => None,
        }
    }
}

pub fn startup_output_verbosity() -> Option<OutputVerbosity> {
    if let Some(value) = startup_arg_value("--output-verbosity") {
        return OutputVerbosity::parse(&value);
    }

    let value = env::var("TOKIMU_OUTPUT_VERBOSITY").ok()?;
    OutputVerbosity::parse(&value)
}

pub fn startup_output_channels() -> Option<String> {
    startup_arg_value("--output-channels").or_else(|| env::var("TOKIMU_OUTPUT_CHANNELS").ok())
}

fn startup_arg_value(flag: &str) -> Option<String> {
    env::args().skip(1).find_map(|argument| {
        argument
            .strip_prefix(flag)
            .and_then(|value| value.strip_prefix('='))
            .map(|value| value.to_string())
    })
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            fixed_time_step_seconds: 1.0 / 60.0,
            max_fixed_steps_per_frame: 8,
        }
    }
}
