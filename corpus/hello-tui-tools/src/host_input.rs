//! Corpus-local host input mapping evidence.
//!
//! This is intentionally not a `tui-tools` API. Native and browser hosts own
//! their concrete event types; this small fixture proves they can both reduce
//! those events to the same normalized `TuiAction` vocabulary before view
//! state applies it.

use tui_tools::TuiAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "the corpus models the complete host-key vocabulary before native and browser adapters exercise every variant"
)]
pub enum HostKey {
    ArrowUp,
    ArrowDown,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Escape,
    Backspace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "the corpus models host input categories that are validated by unit tests but not all emitted by this one executable fixture"
)]
pub enum HostInput {
    Key(HostKey),
    Text(String),
    /// Positive values move toward newer content, matching normal wheel
    /// direction after each host has normalized its raw delta convention.
    WheelLines(i16),
    PointerActivate {
        region: &'static str,
    },
}

pub fn map_host_input(input: HostInput) -> Option<TuiAction> {
    match input {
        HostInput::Key(HostKey::ArrowUp) => Some(TuiAction::MovePrevious),
        HostInput::Key(HostKey::ArrowDown) => Some(TuiAction::MoveNext),
        HostInput::Key(HostKey::PageUp) => Some(TuiAction::PagePrevious),
        HostInput::Key(HostKey::PageDown) => Some(TuiAction::PageNext),
        HostInput::Key(HostKey::Home) => Some(TuiAction::Home),
        HostInput::Key(HostKey::End) => Some(TuiAction::End),
        HostInput::Key(HostKey::Enter) => Some(TuiAction::Activate),
        HostInput::Key(HostKey::Escape) => Some(TuiAction::Cancel),
        HostInput::Key(HostKey::Backspace) => Some(TuiAction::Backspace),
        HostInput::Text(text) if !text.is_empty() => Some(TuiAction::InsertText(text)),
        HostInput::Text(_) => None,
        HostInput::WheelLines(lines) if lines < 0 => Some(TuiAction::MovePrevious),
        HostInput::WheelLines(lines) if lines > 0 => Some(TuiAction::MoveNext),
        HostInput::WheelLines(_) => None,
        // Hit testing belongs to the host. Once a host establishes that a
        // named interactive region was activated, the surface receives only
        // the semantic request.
        HostInput::PointerActivate {
            region: "transcript",
        } => Some(TuiAction::Activate),
        HostInput::PointerActivate { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_wheel_and_pointer_events_share_one_action_vocabulary() {
        assert_eq!(
            map_host_input(HostInput::Key(HostKey::ArrowUp)),
            Some(TuiAction::MovePrevious)
        );
        assert_eq!(
            map_host_input(HostInput::WheelLines(1)),
            Some(TuiAction::MoveNext)
        );
        assert_eq!(
            map_host_input(HostInput::PointerActivate {
                region: "transcript"
            }),
            Some(TuiAction::Activate)
        );
        assert_eq!(
            map_host_input(HostInput::PointerActivate { region: "toolbar" }),
            None
        );
    }

    #[test]
    fn text_and_empty_wheel_inputs_do_not_leak_host_mechanics() {
        assert_eq!(
            map_host_input(HostInput::Text("status".to_owned())),
            Some(TuiAction::InsertText("status".to_owned()))
        );
        assert_eq!(map_host_input(HostInput::Text(String::new())), None);
        assert_eq!(map_host_input(HostInput::WheelLines(0)), None);
    }
}
