//! Corpus-private terminal-outcome classification for ADR-0017.
//!
//! The observer deliberately lives outside the browser page whose survival it
//! measures. It classifies retained evidence; it does not diagnose a crash
//! cause or provide engine recovery policy.

use std::process::ExitStatus;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubjectTerminalEvent {
    Completed { operation: String },
    StructuredFailure { operation: String, detail: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubjectProcessState {
    Running,
    Exited { code: Option<i32> },
}

impl From<ExitStatus> for SubjectProcessState {
    fn from(status: ExitStatus) -> Self {
        Self::Exited {
            code: status.code(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalOutcome {
    Completed { operation: String },
    StructuredFailure { operation: String, detail: String },
    ExternallyTerminated { exit_code: Option<i32> },
    UnresolvedDisappearance { reason: &'static str },
    Running,
}

/// Classifies only evidence supplied by the page, the owned browser process,
/// and the bounded liveness deadline. It never assigns an unobserved cause.
pub fn classify_terminal_outcome(
    terminal_event: Option<SubjectTerminalEvent>,
    process_state: SubjectProcessState,
    heartbeat_expired: bool,
    subject_started: bool,
) -> TerminalOutcome {
    if let Some(event) = terminal_event {
        return match event {
            SubjectTerminalEvent::Completed { operation } => {
                TerminalOutcome::Completed { operation }
            }
            SubjectTerminalEvent::StructuredFailure { operation, detail } => {
                TerminalOutcome::StructuredFailure { operation, detail }
            }
        };
    }

    if let SubjectProcessState::Exited { code } = process_state {
        return TerminalOutcome::ExternallyTerminated { exit_code: code };
    }

    if heartbeat_expired {
        return TerminalOutcome::UnresolvedDisappearance {
            reason: if subject_started {
                "page-heartbeat-expired-while-browser-process-remained-live"
            } else {
                "page-never-acknowledged-observer-before-start-deadline"
            },
        };
    }

    TerminalOutcome::Running
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_page_event_wins_before_supervisor_shutdown() {
        assert_eq!(
            classify_terminal_outcome(
                Some(SubjectTerminalEvent::Completed {
                    operation: "rotation".into(),
                }),
                SubjectProcessState::Running,
                false,
                true,
            ),
            TerminalOutcome::Completed {
                operation: "rotation".into(),
            }
        );
    }

    #[test]
    fn structured_failure_remains_distinct_from_process_loss() {
        assert_eq!(
            classify_terminal_outcome(
                Some(SubjectTerminalEvent::StructuredFailure {
                    operation: "rotation".into(),
                    detail: "WASM returned an error".into(),
                }),
                SubjectProcessState::Running,
                false,
                true,
            ),
            TerminalOutcome::StructuredFailure {
                operation: "rotation".into(),
                detail: "WASM returned an error".into(),
            }
        );
    }

    #[test]
    fn browser_exit_is_external_termination_without_a_guessed_cause() {
        assert_eq!(
            classify_terminal_outcome(
                None,
                SubjectProcessState::Exited { code: Some(9) },
                false,
                true,
            ),
            TerminalOutcome::ExternallyTerminated { exit_code: Some(9) }
        );
    }

    #[test]
    fn lost_page_heartbeat_is_an_unresolved_disappearance() {
        assert_eq!(
            classify_terminal_outcome(None, SubjectProcessState::Running, true, true),
            TerminalOutcome::UnresolvedDisappearance {
                reason: "page-heartbeat-expired-while-browser-process-remained-live",
            }
        );
    }

    #[test]
    fn missing_initial_acknowledgement_is_not_reported_as_skipped() {
        assert_eq!(
            classify_terminal_outcome(None, SubjectProcessState::Running, true, false),
            TerminalOutcome::UnresolvedDisappearance {
                reason: "page-never-acknowledged-observer-before-start-deadline",
            }
        );
    }
}
