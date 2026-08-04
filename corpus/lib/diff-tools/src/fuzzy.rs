use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::BTreeMap;

use crate::{
    apply_exact, DiffDocument, DiffDocumentError, DiffHunk, DiffLimits, DiffOperation,
    ExactPatchConfig, HunkRange, TextDocument, TextDocumentError, TextNormalization,
};

/// Bounded policy for locating a stale hunk without applying it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FuzzyMatchConfig {
    /// Maximum positive or negative line displacement from the declared range.
    pub max_offset_lines: usize,
    /// Maximum equally-valid candidates retained before the search rejects.
    pub max_candidates: usize,
}

impl Default for FuzzyMatchConfig {
    fn default() -> Self {
        Self {
            max_offset_lines: 128,
            max_candidates: 16,
        }
    }
}

/// A source location whose context and removed lines exactly match a stale hunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FuzzyHunkCandidate {
    /// One-based source line where the source-facing hunk sequence begins.
    pub source_start: usize,
    /// Signed displacement from the hunk's declared source range.
    pub offset_lines: isize,
    /// The number of context lines that contributed to the exact candidate match.
    pub matched_context_lines: usize,
}

/// A bounded fuzzy search result. This type does not imply application approval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuzzyHunkSearch {
    NoMatch,
    Unique(FuzzyHunkCandidate),
    Ambiguous(Vec<FuzzyHunkCandidate>),
}

/// Configuration for applying one uniquely located stale hunk.
///
/// Fuzzy application is intentionally single-hunk and in-memory. Multi-hunk
/// ordering, file transactions, and reverse application remain separate work.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FuzzyPatchConfig {
    pub matching: FuzzyMatchConfig,
    pub limits: DiffLimits,
}

/// The explicit outcome of trying to apply a stale hunk.
///
/// An applied result remains visibly fuzzy: callers receive the candidate that
/// justified relocation rather than an exact-application report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuzzyHunkApplication {
    Applied {
        content: String,
        candidate: FuzzyHunkCandidate,
    },
    NoMatch,
    Ambiguous {
        candidates: Vec<FuzzyHunkCandidate>,
    },
}

/// Applies a hunk only when bounded context matching yields one candidate.
///
/// The relocated hunk is delegated to the exact in-memory patch engine. This
/// keeps source replacement semantics in one implementation and makes an
/// unexpected exact rejection an explicit diagnostic failure.
pub fn apply_fuzzy_hunk(
    source: &str,
    hunk: &DiffHunk,
    config: FuzzyPatchConfig,
) -> Result<FuzzyHunkApplication, FuzzyPatchApplyError> {
    let source_document = TextDocument::parse(source, config.limits)?;
    let search = locate_fuzzy_hunk(&source_document, hunk, config.matching)?;
    let FuzzyHunkSearch::Unique(candidate) = search else {
        return Ok(match search {
            FuzzyHunkSearch::NoMatch => FuzzyHunkApplication::NoMatch,
            FuzzyHunkSearch::Ambiguous(candidates) => {
                FuzzyHunkApplication::Ambiguous { candidates }
            }
            FuzzyHunkSearch::Unique(_) => unreachable!("unique matches return above"),
        });
    };

    let relocated = relocated_hunk(hunk, &candidate)?;
    let path = "__diff_tools_fuzzy_input__";
    let document = DiffDocument::new(
        vec![crate::DiffFile::new(path, path, vec![relocated])],
        TextNormalization::default(),
        config.limits,
    )?;
    let source_files = BTreeMap::from([(path.to_owned(), source.to_owned())]);
    let applied = apply_exact(&document, &source_files, ExactPatchConfig::default());
    if !applied.committed {
        return Err(FuzzyPatchApplyError::UnexpectedExactRejection);
    }

    let content = applied
        .files
        .get(path)
        .cloned()
        .ok_or(FuzzyPatchApplyError::MissingAppliedContent)?;
    Ok(FuzzyHunkApplication::Applied { content, candidate })
}

fn relocated_hunk(
    hunk: &DiffHunk,
    candidate: &FuzzyHunkCandidate,
) -> Result<DiffHunk, FuzzyPatchApplyError> {
    let mut relocated = hunk.clone();
    relocated.old_range.start = candidate.source_start;
    relocated.new_range.start = shifted_start(hunk.new_range, candidate.offset_lines)?;
    Ok(relocated)
}

fn shifted_start(range: HunkRange, offset: isize) -> Result<usize, FuzzyPatchApplyError> {
    range
        .start
        .checked_add_signed(offset)
        .filter(|start| *start > 0)
        .ok_or(FuzzyPatchApplyError::RelocationOutOfRange { offset })
}

/// Locates exact source context near a hunk's declared range.
///
/// The first fuzzy slice deliberately accepts only source-facing hunk content
/// (context and removals). A pure insertion has no evidence for relocation and
/// therefore returns an explicit `InsufficientSourceContext` error.
pub fn locate_fuzzy_hunk(
    source: &TextDocument,
    hunk: &DiffHunk,
    config: FuzzyMatchConfig,
) -> Result<FuzzyHunkSearch, FuzzyMatchError> {
    let expected = hunk
        .lines
        .iter()
        .filter(|line| !matches!(line.operation, DiffOperation::Add))
        .collect::<Vec<_>>();
    if expected.is_empty() {
        return Err(FuzzyMatchError::InsufficientSourceContext);
    }
    if config.max_candidates == 0 {
        return Err(FuzzyMatchError::CandidateLimitZero);
    }

    let declared_index = hunk.old_range.start.saturating_sub(1);
    let start = declared_index.saturating_sub(config.max_offset_lines);
    let end = declared_index
        .saturating_add(config.max_offset_lines)
        .min(source.lines().len().saturating_sub(1));
    if source.lines().len() < expected.len() || start > end {
        return Ok(FuzzyHunkSearch::NoMatch);
    }

    let mut candidates = Vec::new();
    for candidate_index in start..=end {
        let Some(candidate_lines) = source
            .lines()
            .get(candidate_index..candidate_index.saturating_add(expected.len()))
        else {
            continue;
        };
        if !expected
            .iter()
            .zip(candidate_lines)
            .all(|(expected, actual)| expected.text == *actual)
        {
            continue;
        }

        candidates.push(FuzzyHunkCandidate {
            source_start: candidate_index + 1,
            offset_lines: candidate_index as isize - declared_index as isize,
            matched_context_lines: expected
                .iter()
                .filter(|line| matches!(line.operation, DiffOperation::Context))
                .count(),
        });
        if candidates.len() > config.max_candidates {
            return Err(FuzzyMatchError::CandidateLimitExceeded {
                limit: config.max_candidates,
            });
        }
    }

    Ok(match candidates.len() {
        0 => FuzzyHunkSearch::NoMatch,
        1 => FuzzyHunkSearch::Unique(candidates.remove(0)),
        _ => FuzzyHunkSearch::Ambiguous(candidates),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FuzzyMatchError {
    #[error("a fuzzy hunk needs at least one context or removal line")]
    InsufficientSourceContext,
    #[error("fuzzy candidate limit must be greater than zero")]
    CandidateLimitZero,
    #[error("fuzzy search found more than the {limit}-candidate limit")]
    CandidateLimitExceeded { limit: usize },
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum FuzzyPatchApplyError {
    #[error(transparent)]
    Source(#[from] TextDocumentError),
    #[error(transparent)]
    Search(#[from] FuzzyMatchError),
    #[error(transparent)]
    Document(#[from] DiffDocumentError),
    #[error("relocating the hunk by {offset} lines would leave its new range invalid")]
    RelocationOutOfRange { offset: isize },
    #[error("the exact patch engine rejected a uniquely located fuzzy hunk")]
    UnexpectedExactRejection,
    #[error("the exact patch engine did not retain the fuzzy input content")]
    MissingAppliedContent,
}
