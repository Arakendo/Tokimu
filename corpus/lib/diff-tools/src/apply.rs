use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{DiffDocument, DiffFile, DiffHunk, DiffLine, DiffOperation};

/// How an exact multi-file patch reports and commits its changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExactPatchPolicy {
    /// Reject the complete patch when any file or hunk cannot be applied.
    Atomic,
    /// Retain successful files while reporting each rejected file explicitly.
    PerFile,
}

/// Configuration for an exact, in-memory unified-diff application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactPatchConfig {
    pub policy: ExactPatchPolicy,
}

impl Default for ExactPatchConfig {
    fn default() -> Self {
        Self {
            policy: ExactPatchPolicy::Atomic,
        }
    }
}

/// The result of exact patch application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactPatchResult {
    pub committed: bool,
    pub files: BTreeMap<String, String>,
    pub report: ExactPatchReport,
}

/// Structured application evidence that callers can render without reparsing text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactPatchReport {
    pub policy: ExactPatchPolicy,
    pub files: Vec<ExactFileReport>,
}

/// Application evidence for one file diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactFileReport {
    pub old_path: String,
    pub new_path: String,
    pub outcome: ExactFileOutcome,
    pub hunks: Vec<ExactHunkReport>,
}

/// Whether one file's hunk sequence was applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExactFileOutcome {
    Applied,
    Rejected { reason: ExactPatchRejection },
}

/// Application evidence for one hunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactHunkReport {
    pub hunk_index: usize,
    pub outcome: ExactHunkOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExactHunkOutcome {
    Applied,
    Rejected { reason: ExactPatchRejection },
}

/// A deterministic reason an exact hunk could not apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum ExactPatchRejection {
    #[error("source path is absent")]
    MissingSource,
    #[error("hunk starts at line {start}, outside the source document")]
    RangeOutOfBounds { start: usize },
    #[error("hunk context does not match source at line {line}")]
    ContextMismatch { line: usize },
    #[error("source final-newline state does not match the patch header")]
    SourceFinalNewlineMismatch,
    #[error("target path {path:?} would overwrite an unrelated source")]
    PathCollision { path: String },
    #[error(
        "hunk source range starting at {start} overlaps or precedes a range ending at {previous_end}"
    )]
    OverlappingOrOutOfOrderHunk { start: usize, previous_end: usize },
}

/// Applies a structured diff against an in-memory, path-keyed text map.
///
/// The initial implementation is exact only: hunk source ranges and context
/// must match verbatim. No fuzzy relocation or partial silent success occurs.
pub fn apply_exact(
    document: &DiffDocument,
    source_files: &BTreeMap<String, String>,
    config: ExactPatchConfig,
) -> ExactPatchResult {
    let mut candidate = source_files.clone();
    let mut reports = Vec::with_capacity(document.files().len());
    let mut rejected = false;

    for file in document.files() {
        if file.old_path() != file.new_path() && candidate.contains_key(file.new_path()) {
            rejected = true;
            let reason = ExactPatchRejection::PathCollision {
                path: file.new_path().to_owned(),
            };
            reports.push(ExactFileReport {
                old_path: file.old_path().to_owned(),
                new_path: file.new_path().to_owned(),
                outcome: ExactFileOutcome::Rejected {
                    reason: reason.clone(),
                },
                hunks: file
                    .hunks()
                    .iter()
                    .enumerate()
                    .map(|(hunk_index, _)| rejected_hunk(hunk_index, reason.clone()))
                    .collect(),
            });
            continue;
        }

        let Some(source) = candidate.get(file.old_path()).cloned() else {
            rejected = true;
            reports.push(ExactFileReport {
                old_path: file.old_path().to_owned(),
                new_path: file.new_path().to_owned(),
                outcome: ExactFileOutcome::Rejected {
                    reason: ExactPatchRejection::MissingSource,
                },
                hunks: file
                    .hunks()
                    .iter()
                    .enumerate()
                    .map(|(hunk_index, _)| ExactHunkReport {
                        hunk_index,
                        outcome: ExactHunkOutcome::Rejected {
                            reason: ExactPatchRejection::MissingSource,
                        },
                    })
                    .collect(),
            });
            continue;
        };

        match apply_file_hunks(&source, file) {
            Ok((result, hunks)) => {
                if file.old_path() != file.new_path() {
                    candidate.remove(file.old_path());
                }
                candidate.insert(file.new_path().to_owned(), result);
                reports.push(ExactFileReport {
                    old_path: file.old_path().to_owned(),
                    new_path: file.new_path().to_owned(),
                    outcome: ExactFileOutcome::Applied,
                    hunks,
                });
            }
            Err((reason, hunks)) => {
                rejected = true;
                reports.push(ExactFileReport {
                    old_path: file.old_path().to_owned(),
                    new_path: file.new_path().to_owned(),
                    outcome: ExactFileOutcome::Rejected { reason },
                    hunks,
                });
            }
        }
    }

    let committed = !rejected || matches!(config.policy, ExactPatchPolicy::PerFile);
    ExactPatchResult {
        committed,
        files: if committed {
            candidate
        } else {
            source_files.clone()
        },
        report: ExactPatchReport {
            policy: config.policy,
            files: reports,
        },
    }
}

fn apply_file_hunks(
    source: &str,
    file: &DiffFile,
) -> Result<(String, Vec<ExactHunkReport>), (ExactPatchRejection, Vec<ExactHunkReport>)> {
    let hunks = file.hunks();
    let mut previous_end = 0;
    for (hunk_index, hunk) in hunks.iter().enumerate() {
        if hunk.old_range.start < previous_end {
            let reason = ExactPatchRejection::OverlappingOrOutOfOrderHunk {
                start: hunk.old_range.start,
                previous_end,
            };
            return Err((reason.clone(), vec![rejected_hunk(hunk_index, reason)]));
        }
        previous_end = hunk.old_range.start.saturating_add(hunk.old_range.count);
    }

    let mut lines = split_content_lines(source);
    let ends_with_newline = source.ends_with(['\n', '\r']);
    if let Some(old_format) = file.old_format() {
        if old_format.ends_with_newline != ends_with_newline {
            return Err((ExactPatchRejection::SourceFinalNewlineMismatch, Vec::new()));
        }
    }
    let mut shift: isize = 0;
    let mut reports = Vec::with_capacity(hunks.len());

    for (hunk_index, hunk) in hunks.iter().enumerate() {
        let base = hunk.old_range.start.saturating_sub(1);
        let Some(start) = base.checked_add_signed(shift) else {
            let reason = ExactPatchRejection::RangeOutOfBounds {
                start: hunk.old_range.start,
            };
            reports.push(rejected_hunk(hunk_index, reason.clone()));
            return Err((reason, reports));
        };

        match apply_hunk(&mut lines, start, hunk) {
            Ok(change) => {
                shift += change;
                reports.push(ExactHunkReport {
                    hunk_index,
                    outcome: ExactHunkOutcome::Applied,
                });
            }
            Err(reason) => {
                reports.push(rejected_hunk(hunk_index, reason.clone()));
                return Err((reason, reports));
            }
        }
    }

    let mut output = lines.join("\n");
    let target_ends_with_newline = file
        .new_format()
        .map(|format| format.ends_with_newline)
        .unwrap_or(ends_with_newline);
    if target_ends_with_newline && !output.is_empty() {
        output.push('\n');
    }
    Ok((output, reports))
}

fn apply_hunk(
    source: &mut Vec<String>,
    start: usize,
    hunk: &DiffHunk,
) -> Result<isize, ExactPatchRejection> {
    if start > source.len() {
        return Err(ExactPatchRejection::RangeOutOfBounds {
            start: hunk.old_range.start,
        });
    }

    let mut read = start;
    let mut replacement = Vec::new();
    for line in &hunk.lines {
        match line.operation {
            DiffOperation::Context => {
                require_source_line(source, read, line, hunk)?;
                replacement.push(line.text.clone());
                read += 1;
            }
            DiffOperation::Remove => {
                require_source_line(source, read, line, hunk)?;
                read += 1;
            }
            DiffOperation::Add => replacement.push(line.text.clone()),
        }
    }

    source.splice(start..read, replacement.iter().cloned());
    Ok(replacement.len() as isize - (read - start) as isize)
}

fn require_source_line(
    source: &[String],
    index: usize,
    expected: &DiffLine,
    hunk: &DiffHunk,
) -> Result<(), ExactPatchRejection> {
    if source
        .get(index)
        .is_some_and(|actual| actual == &expected.text)
    {
        Ok(())
    } else if index >= source.len() {
        Err(ExactPatchRejection::RangeOutOfBounds {
            start: hunk.old_range.start,
        })
    } else {
        Err(ExactPatchRejection::ContextMismatch { line: index + 1 })
    }
}

fn rejected_hunk(hunk_index: usize, reason: ExactPatchRejection) -> ExactHunkReport {
    ExactHunkReport {
        hunk_index,
        outcome: ExactHunkOutcome::Rejected { reason },
    }
}

fn split_content_lines(source: &str) -> Vec<String> {
    if source.is_empty() {
        Vec::new()
    } else {
        source
            .strip_suffix('\n')
            .unwrap_or(source)
            .split('\n')
            .map(str::to_owned)
            .collect()
    }
}
