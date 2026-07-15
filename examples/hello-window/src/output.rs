use tokimu::{OutputChannel, OutputRouter as SharedOutputRouter, OutputVerbosity};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Channel {
    Lifecycle,
    Env,
    Frame,
    Input,
    App,
    Warn,
    Error,
    Trace,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cadence {
    OneShot,
    OnChange,
    OnEvent,
    Sampled,
    PerFrame,
}

pub type OutputRouter = SharedOutputRouter<Channel>;

impl OutputChannel for Channel {
    fn channel_tag(self) -> &'static str {
        match self {
            Channel::Lifecycle => "[lifecycle]",
            Channel::Env => "[env]",
            Channel::Frame => "[frame]",
            Channel::Input => "[input]",
            Channel::App => "[app]",
            Channel::Warn => "[warn]",
            Channel::Error => "[error]",
            Channel::Trace => "[trace]",
        }
    }

    fn is_enabled_by_default(self) -> bool {
        !matches!(self, Channel::Frame | Channel::Trace)
    }

    fn parse_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "lifecycle" => Some(Channel::Lifecycle),
            "env" => Some(Channel::Env),
            "frame" => Some(Channel::Frame),
            "input" => Some(Channel::Input),
            "app" => Some(Channel::App),
            "warn" => Some(Channel::Warn),
            "error" => Some(Channel::Error),
            "trace" => Some(Channel::Trace),
            _ => None,
        }
    }

    fn all_channels() -> &'static [Self] {
        &[
            Channel::Lifecycle,
            Channel::Env,
            Channel::Frame,
            Channel::Input,
            Channel::App,
            Channel::Warn,
            Channel::Error,
            Channel::Trace,
        ]
    }

    fn verbosity_channels(verbosity: OutputVerbosity) -> &'static [Self] {
        match verbosity {
            OutputVerbosity::Quiet => &[Channel::Lifecycle, Channel::Warn, Channel::Error],
            OutputVerbosity::Normal => &[
                Channel::Lifecycle,
                Channel::Env,
                Channel::Input,
                Channel::App,
                Channel::Warn,
                Channel::Error,
            ],
            OutputVerbosity::Verbose => &[
                Channel::Lifecycle,
                Channel::Env,
                Channel::Frame,
                Channel::Input,
                Channel::App,
                Channel::Warn,
                Channel::Error,
            ],
            OutputVerbosity::Trace => &[
                Channel::Lifecycle,
                Channel::Env,
                Channel::Frame,
                Channel::Input,
                Channel::App,
                Channel::Warn,
                Channel::Error,
                Channel::Trace,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_and_env_are_enabled_by_default() {
        let router = OutputRouter::default();

        assert!(router.is_channel_enabled(Channel::Lifecycle));
        assert!(router.is_channel_enabled(Channel::Env));
    }

    #[test]
    fn quiet_verbosity_keeps_only_lifecycle_warn_and_error() {
        let mut router = OutputRouter::default();
        router.set_verbosity(OutputVerbosity::Quiet);

        assert!(router.is_channel_enabled(Channel::Lifecycle));
        assert!(router.is_channel_enabled(Channel::Warn));
        assert!(router.is_channel_enabled(Channel::Error));
        assert!(!router.is_channel_enabled(Channel::Env));
        assert!(!router.is_channel_enabled(Channel::Frame));
        assert!(!router.is_channel_enabled(Channel::Input));
        assert!(!router.is_channel_enabled(Channel::App));
        assert!(!router.is_channel_enabled(Channel::Trace));
    }

    #[test]
    fn frame_channel_can_be_enabled_explicitly() {
        let mut router = OutputRouter::default();
        router.set_channel_enabled(Channel::Frame, true);

        assert!(router.is_channel_enabled(Channel::Frame));
    }

    #[test]
    fn verbose_verbosity_enables_frame_but_not_trace() {
        let mut router = OutputRouter::default();
        router.set_verbosity(OutputVerbosity::Verbose);

        assert!(router.is_channel_enabled(Channel::Frame));
        assert!(!router.is_channel_enabled(Channel::Trace));
    }

    #[test]
    fn explicit_channel_overrides_apply_on_top_of_verbosity() {
        let mut router = OutputRouter::default();
        router.set_verbosity(OutputVerbosity::Quiet);
        router.apply_channel_overrides("+frame,-warn,+trace");

        assert!(router.is_channel_enabled(Channel::Frame));
        assert!(!router.is_channel_enabled(Channel::Warn));
        assert!(router.is_channel_enabled(Channel::Trace));
    }

    #[test]
    fn parses_verbosity_values_case_insensitively() {
        assert_eq!(
            OutputVerbosity::parse("quiet"),
            Some(OutputVerbosity::Quiet)
        );
        assert_eq!(
            OutputVerbosity::parse("VERBOSE"),
            Some(OutputVerbosity::Verbose)
        );
        assert_eq!(OutputVerbosity::parse("unknown"), None);
    }
}
