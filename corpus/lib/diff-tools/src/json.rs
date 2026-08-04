//! Structural JSON comparison for diagnostic artifacts.
//!
//! This adapter compares JSON syntax trees without assigning domain meaning to
//! the values. Callers select volatile paths explicitly; no field is ignored
//! by convention.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Bounded policy for a structural JSON comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonComparisonConfig {
    /// JSON-pointer paths that are intentionally volatile for this comparison.
    pub ignored_paths: BTreeSet<String>,
    /// Maximum retained differences before comparison fails explicitly.
    pub max_differences: usize,
}

impl Default for JsonComparisonConfig {
    fn default() -> Self {
        Self {
            ignored_paths: BTreeSet::new(),
            max_differences: 256,
        }
    }
}

/// A structural difference at a stable JSON-pointer location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonDifference {
    pub path: String,
    pub kind: JsonDifferenceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsonDifferenceKind {
    ValueChanged { expected: Value, actual: Value },
    MissingExpectedKey { key: String },
    UnexpectedActualKey { key: String },
    ArrayLengthChanged { expected: usize, actual: usize },
}

/// Machine-readable evidence from an artifact comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonComparison {
    pub equal: bool,
    pub ignored_paths: Vec<String>,
    pub differences: Vec<JsonDifference>,
}

/// One authoritative artifact comparison within an explicitly ordered pipeline.
#[derive(Clone, Debug)]
pub struct JsonArtifactStage {
    pub stage: String,
    pub expected: Value,
    pub actual: Value,
    pub config: JsonComparisonConfig,
}

/// A structural comparison for an ordered set of diagnostic artifacts.
#[derive(Clone, Debug)]
pub struct JsonArtifactComparison {
    pub stages: Vec<JsonStageComparison>,
    pub first_divergent_stage: Option<String>,
}

/// The structural result for one supplied diagnostic stage.
#[derive(Clone, Debug)]
pub struct JsonStageComparison {
    pub stage: String,
    pub comparison: JsonComparison,
}

/// Compares two JSON documents structurally with explicit volatile-field policy.
pub fn compare_json(
    expected: &Value,
    actual: &Value,
    config: &JsonComparisonConfig,
) -> Result<JsonComparison, JsonComparisonError> {
    if config.max_differences == 0 {
        return Err(JsonComparisonError::DifferenceLimitZero);
    }

    let mut differences = Vec::new();
    compare_value(expected, actual, "", config, &mut differences)?;
    Ok(JsonComparison {
        equal: differences.is_empty(),
        ignored_paths: config.ignored_paths.iter().cloned().collect(),
        differences,
    })
}

fn compare_value(
    expected: &Value,
    actual: &Value,
    path: &str,
    config: &JsonComparisonConfig,
    differences: &mut Vec<JsonDifference>,
) -> Result<(), JsonComparisonError> {
    if config.ignored_paths.contains(path) {
        return Ok(());
    }

    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            for key in expected.keys() {
                let child = join_pointer(path, key);
                match actual.get(key) {
                    Some(actual) => {
                        compare_value(&expected[key], actual, &child, config, differences)?
                    }
                    None => push_difference(
                        differences,
                        config,
                        JsonDifference {
                            path: child,
                            kind: JsonDifferenceKind::MissingExpectedKey { key: key.clone() },
                        },
                    )?,
                }
            }
            for key in actual.keys().filter(|key| !expected.contains_key(*key)) {
                push_difference(
                    differences,
                    config,
                    JsonDifference {
                        path: join_pointer(path, key),
                        kind: JsonDifferenceKind::UnexpectedActualKey { key: key.clone() },
                    },
                )?;
            }
        }
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.len() != actual.len() {
                push_difference(
                    differences,
                    config,
                    JsonDifference {
                        path: pointer_or_root(path),
                        kind: JsonDifferenceKind::ArrayLengthChanged {
                            expected: expected.len(),
                            actual: actual.len(),
                        },
                    },
                )?;
            }
            for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
                compare_value(
                    expected,
                    actual,
                    &join_pointer(path, &index.to_string()),
                    config,
                    differences,
                )?;
            }
        }
        _ if expected == actual => {}
        _ => push_difference(
            differences,
            config,
            JsonDifference {
                path: pointer_or_root(path),
                kind: JsonDifferenceKind::ValueChanged {
                    expected: expected.clone(),
                    actual: actual.clone(),
                },
            },
        )?,
    }
    Ok(())
}

fn push_difference(
    differences: &mut Vec<JsonDifference>,
    config: &JsonComparisonConfig,
    difference: JsonDifference,
) -> Result<(), JsonComparisonError> {
    if differences.len() >= config.max_differences {
        return Err(JsonComparisonError::DifferenceLimitExceeded {
            limit: config.max_differences,
        });
    }
    differences.push(difference);
    Ok(())
}

fn join_pointer(parent: &str, segment: &str) -> String {
    format!("{parent}/{}", segment.replace('~', "~0").replace('/', "~1"))
}

fn pointer_or_root(path: &str) -> String {
    if path.is_empty() {
        "/".into()
    } else {
        path.into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum JsonComparisonError {
    #[error("JSON comparison difference limit must be greater than zero")]
    DifferenceLimitZero,
    #[error("JSON comparison exceeded its {limit}-difference limit")]
    DifferenceLimitExceeded { limit: usize },
}

/// Failures while comparing an ordered diagnostic artifact pipeline.
#[derive(Debug, Error)]
pub enum JsonArtifactComparisonError {
    #[error("artifact comparison requires at least one stage")]
    EmptyStages,
    #[error("artifact stage names must not be empty")]
    EmptyStageName,
    #[error("artifact stage `{stage}` was supplied more than once")]
    DuplicateStageName { stage: String },
    #[error(transparent)]
    Comparison(#[from] JsonComparisonError),
}

/// Compares ordered artifacts and reports the earliest supplied divergent stage.
///
/// Stage order is evidence supplied by the caller. Diff Tools deliberately does
/// not infer ownership or execution ordering from file names or implementation
/// details.
pub fn compare_json_artifact_stages(
    stages: impl IntoIterator<Item = JsonArtifactStage>,
) -> Result<JsonArtifactComparison, JsonArtifactComparisonError> {
    let mut names = BTreeSet::new();
    let mut results = Vec::new();
    let mut first_divergent_stage = None;

    for stage in stages {
        if stage.stage.trim().is_empty() {
            return Err(JsonArtifactComparisonError::EmptyStageName);
        }
        if !names.insert(stage.stage.clone()) {
            return Err(JsonArtifactComparisonError::DuplicateStageName { stage: stage.stage });
        }

        let comparison = compare_json(&stage.expected, &stage.actual, &stage.config)?;
        if !comparison.equal && first_divergent_stage.is_none() {
            first_divergent_stage = Some(stage.stage.clone());
        }
        results.push(JsonStageComparison {
            stage: stage.stage,
            comparison,
        });
    }

    if results.is_empty() {
        return Err(JsonArtifactComparisonError::EmptyStages);
    }

    Ok(JsonArtifactComparison {
        stages: results,
        first_divergent_stage,
    })
}

/// Produces a stable, compact machine-readable summary of ordered artifact evidence.
///
/// Detailed differences remain on each stage result. This summary is intentionally
/// small enough for a corpus manifest or CI annotation.
pub fn json_artifact_summary(comparison: &JsonArtifactComparison) -> Value {
    Value::Object(
        [
            (
                "first_divergent_stage".to_owned(),
                comparison
                    .first_divergent_stage
                    .as_ref()
                    .map_or(Value::Null, |stage| Value::String(stage.clone())),
            ),
            (
                "stages".to_owned(),
                Value::Array(
                    comparison
                        .stages
                        .iter()
                        .map(|stage| {
                            Value::Object(
                                [
                                    ("stage".to_owned(), Value::String(stage.stage.clone())),
                                    ("equal".to_owned(), Value::Bool(stage.comparison.equal)),
                                    (
                                        "difference_count".to_owned(),
                                        Value::from(stage.comparison.differences.len()),
                                    ),
                                ]
                                .into_iter()
                                .collect(),
                            )
                        })
                        .collect(),
                ),
            ),
        ]
        .into_iter()
        .collect(),
    )
}

/// Produces a concise human-readable summary of ordered artifact evidence.
pub fn format_json_artifact_summary(comparison: &JsonArtifactComparison) -> String {
    let first = comparison
        .first_divergent_stage
        .as_deref()
        .unwrap_or("none");
    let stage_summaries = comparison
        .stages
        .iter()
        .map(|stage| {
            format!(
                "{}: {} ({} differences)",
                stage.stage,
                if stage.comparison.equal {
                    "equal"
                } else {
                    "different"
                },
                stage.comparison.differences.len()
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("first divergent stage: {first}; {stage_summaries}")
}
