use thiserror::Error;

use crate::{
    DiffAlgorithm, DiffDocument, DiffDocumentError, DiffFile, DiffHunk, DiffLimits, DiffLine,
    DiffOperation, HunkRange, NewlineComparison, TextDocument, TextNormalization,
    WhitespaceComparison,
};

/// Explicit generation choices. They are retained in the returned document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffGenerationConfig {
    pub normalization: TextNormalization,
    pub context_lines: usize,
}

impl Default for DiffGenerationConfig {
    fn default() -> Self {
        Self {
            normalization: TextNormalization::default(),
            context_lines: 3,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiffGenerationError {
    #[error("the {actual}-cell LCS matrix exceeds the {limit}-cell limit")]
    MatrixTooLarge { actual: usize, limit: usize },
    #[error(transparent)]
    Document(#[from] DiffDocumentError),
}

/// Generates one deterministic, in-memory file diff.
///
/// The initial implementation deliberately selects a bounded LCS algorithm so
/// its cost is explicit while corpus evidence establishes real workloads.
pub fn diff_text(
    old_path: impl Into<String>,
    old: &TextDocument,
    new_path: impl Into<String>,
    new: &TextDocument,
    config: DiffGenerationConfig,
    limits: DiffLimits,
) -> Result<DiffDocument, DiffGenerationError> {
    let old_path = old_path.into();
    let new_path = new_path.into();
    let operations = lcs_operations(old, new, config.normalization, limits)?;
    let line_change = operations
        .iter()
        .any(|operation| operation.operation != DiffOperation::Context);
    let newline_change = config.normalization.newline == NewlineComparison::Exact
        && (old.line_ending() != new.line_ending()
            || old.ends_with_newline() != new.ends_with_newline());

    if !line_change && !newline_change {
        return Ok(DiffDocument::new(Vec::new(), config.normalization, limits)?
            .with_algorithm(DiffAlgorithm::LcsV1));
    }

    let hunks = lower_hunks(&operations, config.context_lines);
    let file =
        DiffFile::new(old_path, new_path, hunks).with_text_formats(old.format(), new.format());
    Ok(DiffDocument::new(vec![file], config.normalization, limits)?
        .with_algorithm(DiffAlgorithm::LcsV1))
}

#[derive(Debug, Clone)]
struct IndexedOperation {
    operation: DiffOperation,
    text: String,
    old_consumed: usize,
    new_consumed: usize,
}

fn lcs_operations(
    old: &TextDocument,
    new: &TextDocument,
    normalization: TextNormalization,
    limits: DiffLimits,
) -> Result<Vec<IndexedOperation>, DiffGenerationError> {
    let rows = old.lines().len().saturating_add(1);
    let columns = new.lines().len().saturating_add(1);
    let cells = rows.saturating_mul(columns);
    if cells > limits.max_edit_matrix_cells {
        return Err(DiffGenerationError::MatrixTooLarge {
            actual: cells,
            limit: limits.max_edit_matrix_cells,
        });
    }

    let mut matrix = vec![0usize; cells];
    for old_index in (0..old.lines().len()).rev() {
        for new_index in (0..new.lines().len()).rev() {
            let value = if equal_lines(
                &old.lines()[old_index],
                &new.lines()[new_index],
                normalization.whitespace,
            ) {
                matrix[index(old_index + 1, new_index + 1, columns)] + 1
            } else {
                matrix[index(old_index + 1, new_index, columns)]
                    .max(matrix[index(old_index, new_index + 1, columns)])
            };
            matrix[index(old_index, new_index, columns)] = value;
        }
    }

    let mut old_index = 0;
    let mut new_index = 0;
    let mut operations = Vec::new();
    while old_index < old.lines().len() || new_index < new.lines().len() {
        if old_index < old.lines().len()
            && new_index < new.lines().len()
            && equal_lines(
                &old.lines()[old_index],
                &new.lines()[new_index],
                normalization.whitespace,
            )
        {
            operations.push(IndexedOperation {
                operation: DiffOperation::Context,
                text: old.lines()[old_index].clone(),
                old_consumed: 1,
                new_consumed: 1,
            });
            old_index += 1;
            new_index += 1;
            continue;
        }

        let remove_score = if old_index < old.lines().len() {
            matrix[index(old_index + 1, new_index, columns)]
        } else {
            0
        };
        let add_score = if new_index < new.lines().len() {
            matrix[index(old_index, new_index + 1, columns)]
        } else {
            0
        };

        // Prefer removal on ties. This makes the edit script stable.
        if old_index < old.lines().len()
            && (new_index == new.lines().len() || remove_score >= add_score)
        {
            operations.push(IndexedOperation {
                operation: DiffOperation::Remove,
                text: old.lines()[old_index].clone(),
                old_consumed: 1,
                new_consumed: 0,
            });
            old_index += 1;
        } else {
            operations.push(IndexedOperation {
                operation: DiffOperation::Add,
                text: new.lines()[new_index].clone(),
                old_consumed: 0,
                new_consumed: 1,
            });
            new_index += 1;
        }
    }

    Ok(operations)
}

fn equal_lines(left: &str, right: &str, whitespace: WhitespaceComparison) -> bool {
    match whitespace {
        WhitespaceComparison::Exact => left == right,
        WhitespaceComparison::IgnoreTrailing => left.trim_end() == right.trim_end(),
    }
}

fn index(row: usize, column: usize, columns: usize) -> usize {
    row * columns + column
}

fn lower_hunks(operations: &[IndexedOperation], context_lines: usize) -> Vec<DiffHunk> {
    let changes = operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            (operation.operation != DiffOperation::Context).then_some(index)
        })
        .collect::<Vec<_>>();
    if changes.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::<(usize, usize)>::new();
    for change in changes {
        let start = change.saturating_sub(context_lines);
        let end = (change + context_lines + 1).min(operations.len());
        match ranges.last_mut() {
            Some((_, previous_end)) if start <= *previous_end => {
                *previous_end = (*previous_end).max(end)
            }
            _ => ranges.push((start, end)),
        }
    }

    ranges
        .into_iter()
        .map(|(start, end)| {
            let (old_before, new_before) = consumed_before(&operations[..start]);
            let (old_count, new_count) = consumed_before(&operations[start..end]);
            let old_start = if old_count == 0 {
                old_before
            } else {
                old_before + 1
            };
            let new_start = if new_count == 0 {
                new_before
            } else {
                new_before + 1
            };
            DiffHunk::new(
                HunkRange {
                    start: old_start,
                    count: old_count,
                },
                HunkRange {
                    start: new_start,
                    count: new_count,
                },
                operations[start..end]
                    .iter()
                    .map(|operation| DiffLine::new(operation.operation, operation.text.clone()))
                    .collect(),
                None,
            )
        })
        .collect()
}

fn consumed_before(operations: &[IndexedOperation]) -> (usize, usize) {
    operations.iter().fold((0, 0), |(old, new), operation| {
        (old + operation.old_consumed, new + operation.new_consumed)
    })
}
