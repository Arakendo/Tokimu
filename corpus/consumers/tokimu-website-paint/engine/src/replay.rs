use crate::{
    PaintCommand, PaintSession, PaintSessionConfig, Rgba8, SessionError, SessionObservation,
};
use serde::{Deserialize, Serialize};

/// The provider-neutral initial state admitted by the first replay format.
/// Imported image replay needs an explicit source identity and remains deferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlankReplayDocument {
    pub width: u32,
    pub height: u32,
    pub color: Rgba8,
}

/// A portable, application-local edit script for deterministic corpus evidence.
/// It records commands and semantic outcomes; it never records Canvas state,
/// browser events, decoder internals, or renderer resources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaintReplay {
    pub schema: u32,
    pub config: PaintSessionConfig,
    pub initial_document: BlankReplayDocument,
    pub commands: Vec<PaintCommand>,
}

impl PaintReplay {
    pub fn blank(
        width: u32,
        height: u32,
        color: Rgba8,
        config: PaintSessionConfig,
        commands: Vec<PaintCommand>,
    ) -> Self {
        Self {
            schema: 1,
            config,
            initial_document: BlankReplayDocument {
                width,
                height,
                color,
            },
            commands,
        }
    }

    pub fn execute(&self) -> Result<PaintReplayObservation, SessionError> {
        let mut session = PaintSession::new_blank(
            self.initial_document.width,
            self.initial_document.height,
            self.initial_document.color,
            self.config,
        )?;
        replay_commands(&mut session, &self.commands)
    }
}

/// Deterministic, application-local evidence for one ordered edit sequence.
/// It records semantic outcomes rather than presentation pixels or browser state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaintReplayObservation {
    pub schema: u32,
    pub command_count: usize,
    pub initial: SessionObservation,
    pub terminal: SessionObservation,
}

pub fn replay_commands(
    session: &mut PaintSession,
    commands: &[PaintCommand],
) -> Result<PaintReplayObservation, SessionError> {
    let initial = session.observation();
    for command in commands {
        session.apply(command)?;
    }

    Ok(PaintReplayObservation {
        schema: 1,
        command_count: commands.len(),
        initial,
        terminal: session.observation(),
    })
}

#[cfg(test)]
mod tests {
    use super::{replay_commands, PaintReplay};
    use crate::{PaintCommand, PaintSession, PaintSessionConfig, PixelPoint, Rgba8};

    const BLACK: Rgba8 = Rgba8 {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    const WHITE: Rgba8 = Rgba8 {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    };

    #[test]
    fn equivalent_command_replays_produce_the_same_terminal_fingerprint() {
        let commands = [PaintCommand::PencilStroke {
            points: vec![PixelPoint { x: 0, y: 0 }, PixelPoint { x: 1, y: 0 }],
            color: WHITE,
        }];
        let mut first =
            PaintSession::new_blank(2, 1, BLACK, PaintSessionConfig::default()).unwrap();
        let mut second =
            PaintSession::new_blank(2, 1, BLACK, PaintSessionConfig::default()).unwrap();

        let first_replay = replay_commands(&mut first, &commands).unwrap();
        let second_replay = replay_commands(&mut second, &commands).unwrap();

        assert_eq!(first_replay.command_count, 1);
        assert_eq!(
            first_replay.terminal.document.unwrap().pixel_fingerprint,
            second_replay.terminal.document.unwrap().pixel_fingerprint
        );
    }

    #[test]
    fn serialized_blank_replay_preserves_its_terminal_document_and_history() {
        let replay = PaintReplay::blank(
            2,
            1,
            BLACK,
            PaintSessionConfig::default(),
            vec![PaintCommand::PencilStroke {
                points: vec![PixelPoint { x: 1, y: 0 }],
                color: WHITE,
            }],
        );
        let serialized = serde_json::to_string_pretty(&replay).unwrap();
        let restored: PaintReplay = serde_json::from_str(&serialized).unwrap();

        let observation = restored.execute().unwrap();
        assert_eq!(observation.command_count, 1);
        assert!(observation.terminal.document.unwrap().dirty);
        assert_eq!(observation.terminal.history.unwrap().undo_depth, 1);
    }
}
