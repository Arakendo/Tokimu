use tokimu::{OutputChannel, OutputRouter as SharedOutputRouter, OutputVerbosity};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Channel {
    Lifecycle,
    Env,
    App,
}

pub type OutputRouter = SharedOutputRouter<Channel>;

impl OutputChannel for Channel {
    fn channel_tag(self) -> &'static str {
        match self {
            Channel::Lifecycle => "[lifecycle]",
            Channel::Env => "[env]",
            Channel::App => "[app]",
        }
    }

    fn is_enabled_by_default(self) -> bool {
        matches!(self, Channel::Lifecycle | Channel::Env)
    }

    fn parse_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "lifecycle" => Some(Channel::Lifecycle),
            "env" => Some(Channel::Env),
            "app" => Some(Channel::App),
            _ => None,
        }
    }

    fn all_channels() -> &'static [Self] {
        &[Channel::Lifecycle, Channel::Env, Channel::App]
    }

    fn verbosity_channels(verbosity: OutputVerbosity) -> &'static [Self] {
        match verbosity {
            OutputVerbosity::Quiet => &[Channel::Lifecycle],
            OutputVerbosity::Normal | OutputVerbosity::Verbose | OutputVerbosity::Trace => {
                &[Channel::Lifecycle, Channel::Env, Channel::App]
            }
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
    fn quiet_verbosity_keeps_only_lifecycle() {
        let mut router = OutputRouter::default();
        router.set_verbosity(OutputVerbosity::Quiet);

        assert!(router.is_channel_enabled(Channel::Lifecycle));
        assert!(!router.is_channel_enabled(Channel::Env));
    }

    #[test]
    fn one_shot_messages_follow_channel_policy() {
        let mut router = OutputRouter::default();
        router.set_verbosity(OutputVerbosity::Quiet);
        assert!(!router.is_channel_enabled(Channel::Env));
        assert!(router.is_channel_enabled(Channel::Lifecycle));
    }

    #[test]
    fn emit_one_shot_is_allowed_on_enabled_channels() {
        let mut router = OutputRouter::default();
        router.set_verbosity(OutputVerbosity::Quiet);
        router.emit_one_shot(Channel::Lifecycle, "backend selected");

        assert!(router.is_channel_enabled(Channel::Lifecycle));
    }

    #[test]
    fn normal_verbosity_enables_app_status_lines() {
        let mut router = OutputRouter::default();
        router.set_verbosity(OutputVerbosity::Normal);

        assert!(router.is_channel_enabled(Channel::App));
    }

    #[test]
    fn explicit_channel_overrides_apply_on_top_of_verbosity() {
        let mut router = OutputRouter::default();
        router.set_verbosity(OutputVerbosity::Quiet);
        router.apply_channel_overrides("+env");

        assert!(router.is_channel_enabled(Channel::Lifecycle));
        assert!(router.is_channel_enabled(Channel::Env));
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
