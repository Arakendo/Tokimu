use crate::{
    apply_command, command::CommandError, document::DocumentStateSnapshot, EditObservation,
    EditableRasterDocument, PaintCommand,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub max_transactions: usize,
    pub max_retained_bytes: usize,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_transactions: 64,
            max_retained_bytes: 32 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryObservation {
    pub schema: u32,
    pub undo_depth: usize,
    pub redo_depth: usize,
    pub retained_bytes: usize,
    pub max_retained_bytes: usize,
    pub max_transactions: usize,
    pub evictions: u64,
}

#[derive(Clone, Debug)]
struct HistoryEntry {
    before: DocumentStateSnapshot,
    after: DocumentStateSnapshot,
}

impl HistoryEntry {
    fn retained_bytes(&self) -> usize {
        self.before.pixels.len() + self.after.pixels.len()
    }
}

#[derive(Clone, Debug)]
pub struct PaintWorkspace {
    document: EditableRasterDocument,
    config: HistoryConfig,
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    retained_bytes: usize,
    evictions: u64,
}

impl PaintWorkspace {
    pub fn new(document: EditableRasterDocument, config: HistoryConfig) -> Self {
        Self {
            document,
            config,
            undo: Vec::new(),
            redo: Vec::new(),
            retained_bytes: 0,
            evictions: 0,
        }
    }

    pub fn document(&self) -> &EditableRasterDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut EditableRasterDocument {
        &mut self.document
    }

    pub fn history_observation(&self) -> HistoryObservation {
        HistoryObservation {
            schema: 1,
            undo_depth: self.undo.len(),
            redo_depth: self.redo.len(),
            retained_bytes: self.retained_bytes,
            max_retained_bytes: self.config.max_retained_bytes,
            max_transactions: self.config.max_transactions,
            evictions: self.evictions,
        }
    }

    pub fn apply(&mut self, command: &PaintCommand) -> Result<EditObservation, HistoryError> {
        if self.config.max_transactions == 0 {
            return Err(HistoryError::TransactionsDisabled);
        }

        let snapshot_bytes = self
            .document
            .pixels()
            .len()
            .checked_mul(2)
            .ok_or(HistoryError::TransactionSizeOverflow)?;
        if snapshot_bytes > self.config.max_retained_bytes {
            return Err(HistoryError::TransactionExceedsBudget {
                required: snapshot_bytes,
                limit: self.config.max_retained_bytes,
            });
        }

        let before = self.document.state_snapshot();
        let edit = apply_command(&mut self.document, command)?;
        if edit.no_op {
            return Ok(edit);
        }
        let after = self.document.state_snapshot();
        self.discard_redo();
        self.make_room(snapshot_bytes);
        self.retained_bytes += snapshot_bytes;
        self.undo.push(HistoryEntry { before, after });
        Ok(edit)
    }

    pub fn undo(&mut self) -> bool {
        let Some(entry) = self.undo.pop() else {
            return false;
        };
        self.document.restore_state(entry.before.clone());
        self.redo.push(entry);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(entry) = self.redo.pop() else {
            return false;
        };
        self.document.restore_state(entry.after.clone());
        self.undo.push(entry);
        true
    }

    pub fn reset_history(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.retained_bytes = 0;
        self.evictions = 0;
    }

    fn discard_redo(&mut self) {
        for entry in self.redo.drain(..) {
            self.retained_bytes -= entry.retained_bytes();
        }
    }

    fn make_room(&mut self, next_entry_bytes: usize) {
        while !self.undo.is_empty()
            && (self.undo.len() >= self.config.max_transactions
                || self.retained_bytes + next_entry_bytes > self.config.max_retained_bytes)
        {
            let entry = self.undo.remove(0);
            self.retained_bytes -= entry.retained_bytes();
            self.evictions += 1;
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum HistoryError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error("history is disabled because its transaction limit is zero")]
    TransactionsDisabled,
    #[error("history transaction byte arithmetic overflowed")]
    TransactionSizeOverflow,
    #[error(
        "history requires {required} bytes for one transaction, exceeding its {limit}-byte budget"
    )]
    TransactionExceedsBudget { required: usize, limit: usize },
}

#[cfg(test)]
mod tests {
    use super::{HistoryConfig, HistoryError, PaintWorkspace};
    use crate::{
        CommandError, DocumentConfig, EditableRasterDocument, PaintCommand, PixelPoint, Rgba8,
    };

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

    fn workspace(config: HistoryConfig) -> PaintWorkspace {
        PaintWorkspace::new(
            EditableRasterDocument::blank(2, 2, BLACK, DocumentConfig::default()).unwrap(),
            config,
        )
    }

    fn point(x: u32, y: u32) -> PaintCommand {
        PaintCommand::PencilStroke {
            points: vec![PixelPoint { x, y }],
            color: WHITE,
        }
    }

    #[test]
    fn undo_and_redo_restore_exact_document_hashes() {
        let mut workspace = workspace(HistoryConfig::default());
        let initial = workspace.document().observation().pixel_fingerprint;
        workspace.apply(&point(0, 0)).unwrap();
        let committed = workspace.document().observation().pixel_fingerprint;

        assert!(workspace.undo());
        assert_eq!(
            workspace.document().observation().pixel_fingerprint,
            initial
        );
        assert!(workspace.redo());
        assert_eq!(
            workspace.document().observation().pixel_fingerprint,
            committed
        );
    }

    #[test]
    fn edit_after_undo_discards_obsolete_redo_branch() {
        let mut workspace = workspace(HistoryConfig::default());
        workspace.apply(&point(0, 0)).unwrap();
        workspace.apply(&point(1, 0)).unwrap();
        assert!(workspace.undo());
        workspace.apply(&point(0, 1)).unwrap();

        assert!(!workspace.redo());
        assert_eq!(workspace.history_observation().redo_depth, 0);
    }

    #[test]
    fn history_evicts_oldest_transaction_within_configured_budget() {
        let mut workspace = workspace(HistoryConfig {
            max_transactions: 1,
            max_retained_bytes: 32,
        });
        workspace.apply(&point(0, 0)).unwrap();
        workspace.apply(&point(1, 0)).unwrap();
        let observation = workspace.history_observation();

        assert_eq!(observation.undo_depth, 1);
        assert_eq!(observation.evictions, 1);
        assert_eq!(observation.retained_bytes, 32);
    }

    #[test]
    fn impossible_history_budget_rejects_before_document_mutation() {
        let mut workspace = workspace(HistoryConfig {
            max_transactions: 1,
            max_retained_bytes: 31,
        });
        let before = workspace.document().observation().pixel_fingerprint;

        assert_eq!(
            workspace.apply(&point(0, 0)),
            Err(HistoryError::TransactionExceedsBudget {
                required: 32,
                limit: 31
            })
        );
        assert_eq!(workspace.document().observation().pixel_fingerprint, before);
    }

    #[test]
    fn disabled_history_rejects_before_document_mutation() {
        let mut workspace = workspace(HistoryConfig {
            max_transactions: 0,
            max_retained_bytes: 32,
        });
        let before = workspace.document().observation().pixel_fingerprint;

        assert_eq!(
            workspace.apply(&point(0, 0)),
            Err(HistoryError::TransactionsDisabled)
        );
        assert_eq!(workspace.document().observation().pixel_fingerprint, before);
    }

    #[test]
    fn reset_history_preserves_document_and_releases_snapshots() {
        let mut workspace = workspace(HistoryConfig::default());
        workspace.apply(&point(0, 0)).unwrap();
        let committed = workspace.document().observation().pixel_fingerprint;

        workspace.reset_history();
        let observation = workspace.history_observation();

        assert_eq!(
            workspace.document().observation().pixel_fingerprint,
            committed
        );
        assert_eq!(observation.undo_depth, 0);
        assert_eq!(observation.redo_depth, 0);
        assert_eq!(observation.retained_bytes, 0);
        assert_eq!(observation.evictions, 0);
    }

    #[test]
    fn malformed_command_preserves_document_and_existing_history() {
        let mut workspace = workspace(HistoryConfig::default());
        workspace.apply(&point(0, 0)).unwrap();
        let before_document = workspace.document().observation().pixel_fingerprint;
        let before_history = workspace.history_observation();

        assert_eq!(
            workspace.apply(&PaintCommand::PencilStroke {
                points: vec![PixelPoint { x: 1, y: 1 }, PixelPoint { x: 2, y: 1 }],
                color: WHITE,
            }),
            Err(HistoryError::Command(CommandError::PointOutOfBounds {
                x: 2,
                y: 1,
                width: 2,
                height: 2,
            }))
        );

        assert_eq!(
            workspace.document().observation().pixel_fingerprint,
            before_document
        );
        assert_eq!(workspace.history_observation(), before_history);
    }
}
