//! Runtime-observation adapter for the provider-neutral Diff Tools contract.
//!
//! This module owns the corpus-specific decision about which observation
//! envelope fields are provenance and which payload is compared. Diff Tools
//! receives JSON values only; it never receives a `World`, command queue, or
//! playback state.

use diff_tools::{compare_json, JsonComparison, JsonComparisonConfig, JsonComparisonError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ObservationEnvelope;

/// Explicit comparison policy for two runtime-observation snapshots.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationComparisonConfig {
    /// JSON-pointer payload fields that are intentionally volatile for this
    /// particular comparison. No runtime field is ignored by convention.
    pub ignored_payload_paths: std::collections::BTreeSet<String>,
    /// Maximum retained structural differences.
    pub max_differences: usize,
}

impl Default for ObservationComparisonConfig {
    fn default() -> Self {
        Self {
            ignored_payload_paths: std::collections::BTreeSet::new(),
            max_differences: 64,
        }
    }
}

/// Envelope metadata retained beside a structural payload comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationProvenance {
    pub version: u16,
    pub sequence: u64,
    pub tick: u64,
    pub revision: u64,
}

impl From<&ObservationEnvelope> for ObservationProvenance {
    fn from(observation: &ObservationEnvelope) -> Self {
        Self {
            version: observation.version,
            sequence: observation.sequence,
            tick: observation.tick,
            revision: observation.revision,
        }
    }
}

/// Corpus-owned report joining observation provenance to a structural payload
/// comparison. `payload` remains the only part passed to Diff Tools.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationDiffReport {
    pub schema: String,
    pub kind: String,
    pub before: ObservationProvenance,
    pub after: ObservationProvenance,
    pub payload: JsonComparison,
}

#[derive(Debug, Error)]
pub enum ObservationDiffError {
    #[error("cannot compare observation schemas `{before}` and `{after}`")]
    SchemaMismatch { before: String, after: String },
    #[error("cannot compare observation versions {before} and {after}")]
    VersionMismatch { before: u16, after: u16 },
    #[error("cannot compare observation kinds `{before}` and `{after}`")]
    KindMismatch { before: String, after: String },
    #[error("runtime observation could not be converted into a JSON artifact: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Comparison(#[from] JsonComparisonError),
}

/// Compare two compatible, owned runtime observations.
///
/// Schema, version, and kind are compatibility gates rather than volatile
/// fields. Sequence, tick, and revision remain visible in the report as the
/// consumer's before/after provenance; they are not silently hidden inside the
/// structural payload comparison.
pub fn compare_observation_snapshots(
    before: &ObservationEnvelope,
    after: &ObservationEnvelope,
    config: &ObservationComparisonConfig,
) -> Result<ObservationDiffReport, ObservationDiffError> {
    if before.schema != after.schema {
        return Err(ObservationDiffError::SchemaMismatch {
            before: before.schema.to_owned(),
            after: after.schema.to_owned(),
        });
    }
    if before.version != after.version {
        return Err(ObservationDiffError::VersionMismatch {
            before: before.version,
            after: after.version,
        });
    }
    if before.kind != after.kind {
        return Err(ObservationDiffError::KindMismatch {
            before: before.kind.to_owned(),
            after: after.kind.to_owned(),
        });
    }

    let comparison = compare_json(
        &serde_json::to_value(&before.payload)?,
        &serde_json::to_value(&after.payload)?,
        &JsonComparisonConfig {
            ignored_paths: config.ignored_payload_paths.clone(),
            max_differences: config.max_differences,
        },
    )?;

    Ok(ObservationDiffReport {
        schema: before.schema.to_owned(),
        kind: before.kind.to_owned(),
        before: before.into(),
        after: after.into(),
        payload: comparison,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_session, CommandAuthority, CommandRequest, ObservationLimits, Position,
        RuntimeCommand,
    };

    #[test]
    fn reports_payload_change_and_retains_revision_provenance() {
        let mut session = build_session(2);
        let arm = session.arm_id();
        let before = session.observe(4, Some(arm), ObservationLimits::default());
        session.enqueue(CommandRequest {
            id: 7,
            target: arm.0,
            authority: CommandAuthority::Operator,
            expected_revision: Some(session.revision()),
            command: RuntimeCommand::MoveBy {
                delta: Position {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        });
        session.apply_pending_at_tick(3);
        let after = session.observe(5, Some(arm), ObservationLimits::default());

        let report =
            compare_observation_snapshots(&before, &after, &ObservationComparisonConfig::default())
                .unwrap();

        assert!(!report.payload.equal);
        assert_eq!(report.before.revision, 0);
        assert_eq!(report.after.revision, 1);
        assert!(report
            .payload
            .differences
            .iter()
            .any(|difference| difference.path.ends_with("/x")));
    }

    #[test]
    fn rejects_incompatible_observation_versions() {
        let session = build_session(2);
        let before = session.observe(0, None, ObservationLimits::default());
        let mut after = before.clone();
        after.version = after.version.saturating_add(1);

        assert!(matches!(
            compare_observation_snapshots(&before, &after, &ObservationComparisonConfig::default(),),
            Err(ObservationDiffError::VersionMismatch { .. })
        ));
    }
}
