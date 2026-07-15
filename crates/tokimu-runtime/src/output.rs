use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::config::{startup_output_channels, startup_output_verbosity, OutputVerbosity};

#[derive(Debug)]
pub struct RepeatCoalescer<Channel> {
    last_message: HashMap<Channel, String>,
    repeat_count: HashMap<Channel, usize>,
}

impl<Channel> Default for RepeatCoalescer<Channel> {
    fn default() -> Self {
        Self {
            last_message: HashMap::new(),
            repeat_count: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepeatCoalescerUpdate {
    FirstMessage,
    Repeated,
    Replaced {
        previous_message: String,
        repeat_count: usize,
    },
}

impl<Channel> RepeatCoalescer<Channel>
where
    Channel: Copy + Eq + Hash,
{
    pub fn record(
        &mut self,
        channel: Channel,
        message: impl Into<String>,
    ) -> RepeatCoalescerUpdate {
        let message = message.into();

        match self.last_message.get(&channel) {
            Some(previous_message) if previous_message == &message => {
                *self.repeat_count.entry(channel).or_insert(0) += 1;
                RepeatCoalescerUpdate::Repeated
            }
            Some(previous_message) => {
                let previous_message = previous_message.clone();
                let repeat_count = self.repeat_count.remove(&channel).unwrap_or(0);
                self.last_message.insert(channel, message);
                self.repeat_count.insert(channel, 0);
                RepeatCoalescerUpdate::Replaced {
                    previous_message,
                    repeat_count,
                }
            }
            None => {
                self.last_message.insert(channel, message);
                self.repeat_count.insert(channel, 0);
                RepeatCoalescerUpdate::FirstMessage
            }
        }
    }

    pub fn flush(&mut self, channel: Channel) -> Option<(String, usize)> {
        let message = self.last_message.get(&channel)?.clone();
        let repeat_count = self.repeat_count.get(&channel).copied().unwrap_or(0);
        self.repeat_count.insert(channel, 0);
        Some((message, repeat_count))
    }

    pub fn drain(&mut self, channel: Channel) -> Option<(String, usize)> {
        let message = self.last_message.remove(&channel)?;
        let repeat_count = self.repeat_count.remove(&channel).unwrap_or(0);
        Some((message, repeat_count))
    }
}

pub trait OutputChannel: Copy + Eq + Hash {
    fn channel_tag(self) -> &'static str;

    fn is_enabled_by_default(self) -> bool;

    fn parse_name(name: &str) -> Option<Self>
    where
        Self: Sized;

    fn all_channels() -> &'static [Self]
    where
        Self: Sized;

    fn verbosity_channels(verbosity: OutputVerbosity) -> &'static [Self]
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct OutputRouter<Channel> {
    last_sampled_emit_seconds: HashMap<Channel, f64>,
    last_sampled_summary_seconds: HashMap<Channel, f64>,
    one_shot_emitted: HashSet<(Channel, String)>,
    channel_enabled: HashMap<Channel, bool>,
    coalescer: RepeatCoalescer<Channel>,
}

impl<Channel> Default for OutputRouter<Channel> {
    fn default() -> Self {
        Self {
            last_sampled_emit_seconds: HashMap::new(),
            last_sampled_summary_seconds: HashMap::new(),
            one_shot_emitted: HashSet::new(),
            channel_enabled: HashMap::new(),
            coalescer: RepeatCoalescer::default(),
        }
    }
}

impl<Channel> OutputRouter<Channel>
where
    Channel: OutputChannel + 'static,
{
    pub fn with_startup_policy() -> Self {
        let mut router = Self::default();
        router.set_verbosity(startup_output_verbosity().unwrap_or(OutputVerbosity::Normal));
        if let Some(spec) = startup_output_channels() {
            router.apply_channel_overrides(&spec);
        }
        router
    }

    pub fn emit_one_shot(&mut self, channel: Channel, message: impl Into<String>) {
        let message = message.into();
        if !self.is_channel_enabled(channel) {
            return;
        }

        if self.one_shot_emitted.insert((channel, message.clone())) {
            println!("{} {}", channel.channel_tag(), message);
        }
    }

    pub fn emit_on_change(&mut self, channel: Channel, value: impl Into<String>) {
        let value = value.into();
        if !self.is_channel_enabled(channel) {
            return;
        }

        self.emit_coalesced(channel, value);
    }

    pub fn emit_sampled(
        &mut self,
        channel: Channel,
        elapsed_seconds: f64,
        message: impl Into<String>,
    ) {
        let message = message.into();
        if !self.is_channel_enabled(channel) {
            return;
        }

        let should_emit = self
            .last_sampled_emit_seconds
            .get(&channel)
            .map(|last_emit| elapsed_seconds - *last_emit >= 1.0)
            .unwrap_or(true);

        if should_emit {
            self.last_sampled_emit_seconds
                .insert(channel, elapsed_seconds);
            if self
                .last_sampled_summary_seconds
                .get(&channel)
                .map(|last_emit| elapsed_seconds - *last_emit >= 1.0)
                .unwrap_or(true)
            {
                self.flush_coalesced_channel(channel);
                self.last_sampled_summary_seconds
                    .insert(channel, elapsed_seconds);
            }
            self.emit_coalesced(channel, message);
        }
    }

    pub fn emit_event(&mut self, channel: Channel, message: impl Into<String>) {
        let message = message.into();
        if !self.is_channel_enabled(channel) {
            return;
        }

        self.emit_coalesced(channel, message);
    }

    pub fn set_channel_enabled(&mut self, channel: Channel, enabled: bool) {
        self.channel_enabled.insert(channel, enabled);
    }

    pub fn set_verbosity(&mut self, verbosity: OutputVerbosity) {
        let enabled_channels = Channel::verbosity_channels(verbosity);

        self.channel_enabled.clear();
        for channel in Channel::all_channels() {
            self.channel_enabled.insert(*channel, false);
        }

        for channel in enabled_channels {
            self.set_channel_enabled(*channel, true);
        }
    }

    pub fn apply_channel_overrides(&mut self, spec: &str) {
        for (channel, enabled) in parse_channel_overrides::<Channel>(spec) {
            self.set_channel_enabled(channel, enabled);
        }
    }

    pub fn is_channel_enabled(&self, channel: Channel) -> bool {
        self.channel_enabled
            .get(&channel)
            .copied()
            .unwrap_or_else(|| channel.is_enabled_by_default())
    }

    pub fn flush(&mut self) {
        self.flush_coalesced_messages();
    }

    fn emit_coalesced(&mut self, channel: Channel, message: String) {
        match self.coalescer.record(channel, message) {
            RepeatCoalescerUpdate::FirstMessage => {
                if let Some((last_message, repeat_count)) = self.coalescer.flush(channel) {
                    debug_assert_eq!(repeat_count, 0);
                    println!("{} {}", channel.channel_tag(), last_message);
                }
            }
            RepeatCoalescerUpdate::Repeated => {}
            RepeatCoalescerUpdate::Replaced {
                previous_message,
                repeat_count,
            } => {
                if repeat_count > 0 {
                    println!(
                        "{} {} (repeated {}x)",
                        channel.channel_tag(),
                        previous_message,
                        repeat_count
                    );
                }

                if let Some((last_message, repeat_count)) = self.coalescer.flush(channel) {
                    debug_assert_eq!(repeat_count, 0);
                    println!("{} {}", channel.channel_tag(), last_message);
                }
            }
        }
    }

    fn flush_coalesced_messages(&mut self) {
        for channel in Channel::all_channels() {
            self.flush_coalesced_channel(*channel);
        }
    }

    fn flush_coalesced_channel(&mut self, channel: Channel) {
        let Some((message, repeat_count)) = self.coalescer.drain(channel) else {
            return;
        };

        if repeat_count > 0 {
            println!(
                "{} {} (repeated {}x)",
                channel.channel_tag(),
                message,
                repeat_count
            );
        }
    }
}

fn parse_channel_overrides<Channel>(spec: &str) -> Vec<(Channel, bool)>
where
    Channel: OutputChannel,
{
    spec.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }

            let (enabled, name) = match entry.as_bytes().first().copied() {
                Some(b'+') => (true, &entry[1..]),
                Some(b'-') => (false, &entry[1..]),
                _ => return None,
            };

            Channel::parse_name(name).map(|channel| (channel, enabled))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::config::OutputVerbosity;

    use super::{OutputChannel, OutputRouter, RepeatCoalescer, RepeatCoalescerUpdate};

    #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
    enum TestChannel {
        Lifecycle,
        Env,
        App,
    }

    impl OutputChannel for TestChannel {
        fn channel_tag(self) -> &'static str {
            match self {
                TestChannel::Lifecycle => "[lifecycle]",
                TestChannel::Env => "[env]",
                TestChannel::App => "[app]",
            }
        }

        fn is_enabled_by_default(self) -> bool {
            matches!(self, TestChannel::Lifecycle | TestChannel::Env)
        }

        fn parse_name(name: &str) -> Option<Self> {
            match name.to_ascii_lowercase().as_str() {
                "lifecycle" => Some(TestChannel::Lifecycle),
                "env" => Some(TestChannel::Env),
                "app" => Some(TestChannel::App),
                _ => None,
            }
        }

        fn all_channels() -> &'static [Self] {
            &[TestChannel::Lifecycle, TestChannel::Env, TestChannel::App]
        }

        fn verbosity_channels(verbosity: OutputVerbosity) -> &'static [Self] {
            match verbosity {
                OutputVerbosity::Quiet => &[TestChannel::Lifecycle],
                OutputVerbosity::Normal | OutputVerbosity::Verbose | OutputVerbosity::Trace => {
                    &[TestChannel::Lifecycle, TestChannel::Env, TestChannel::App]
                }
            }
        }
    }

    #[test]
    fn records_first_repeat_and_replacement_counts() {
        let mut coalescer = RepeatCoalescer::default();

        assert_eq!(
            coalescer.record("input", "move intent = (0.0, 0.0)"),
            RepeatCoalescerUpdate::FirstMessage
        );
        assert_eq!(
            coalescer.record("input", "move intent = (0.0, 0.0)"),
            RepeatCoalescerUpdate::Repeated
        );
        assert_eq!(
            coalescer.record("input", "move intent = (0.0, 0.0)"),
            RepeatCoalescerUpdate::Repeated
        );

        match coalescer.record("input", "move intent = (1.0, 0.0)") {
            RepeatCoalescerUpdate::Replaced {
                previous_message,
                repeat_count,
            } => {
                assert_eq!(previous_message, "move intent = (0.0, 0.0)");
                assert_eq!(repeat_count, 2);
            }
            other => panic!("unexpected coalescer state: {other:?}"),
        }
    }

    #[test]
    fn flush_returns_last_message_and_repeat_count() {
        let mut coalescer = RepeatCoalescer::default();

        coalescer.record("frame", "frame dt=0.0167s");
        coalescer.record("frame", "frame dt=0.0167s");

        assert_eq!(
            coalescer.flush("frame"),
            Some(("frame dt=0.0167s".to_string(), 1))
        );
        assert_eq!(
            coalescer.flush("frame"),
            Some(("frame dt=0.0167s".to_string(), 0))
        );
    }

    #[test]
    fn drain_clears_channel_state() {
        let mut coalescer = RepeatCoalescer::default();

        coalescer.record("frame", "frame dt=0.0167s");
        coalescer.record("frame", "frame dt=0.0167s");

        assert_eq!(
            coalescer.drain("frame"),
            Some(("frame dt=0.0167s".to_string(), 1))
        );
        assert_eq!(coalescer.drain("frame"), None);
    }

    #[test]
    fn generic_output_router_applies_verbosity_and_overrides() {
        let mut router = OutputRouter::<TestChannel>::default();
        router.set_verbosity(OutputVerbosity::Quiet);
        router.apply_channel_overrides("+app");

        assert!(router.is_channel_enabled(TestChannel::Lifecycle));
        assert!(!router.is_channel_enabled(TestChannel::Env));
        assert!(router.is_channel_enabled(TestChannel::App));
    }
}
