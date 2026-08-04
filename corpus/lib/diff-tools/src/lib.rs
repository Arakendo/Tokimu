//! Incubating, provider-neutral structured diff contracts.
//!
//! This crate accepts in-memory text only. Filesystem, repository, UI, and
//! domain-document semantics remain with adapters and consumers.

mod apply;
mod fuzzy;
mod generate;
mod json;
mod model;
mod unified;

pub use apply::{
    apply_exact, ExactFileOutcome, ExactFileReport, ExactHunkOutcome, ExactHunkReport,
    ExactPatchConfig, ExactPatchPolicy, ExactPatchRejection, ExactPatchReport, ExactPatchResult,
};
pub use fuzzy::{
    apply_fuzzy_hunk, locate_fuzzy_hunk, FuzzyHunkApplication, FuzzyHunkCandidate, FuzzyHunkSearch,
    FuzzyMatchConfig, FuzzyMatchError, FuzzyPatchApplyError, FuzzyPatchConfig,
};
pub use generate::{diff_text, DiffGenerationConfig, DiffGenerationError};
pub use json::{
    compare_json, compare_json_artifact_stages, format_json_artifact_summary,
    json_artifact_summary, JsonArtifactComparison, JsonArtifactComparisonError, JsonArtifactStage,
    JsonComparison, JsonComparisonConfig, JsonComparisonError, JsonDifference, JsonDifferenceKind,
    JsonStageComparison,
};
pub use model::{
    DiffAlgorithm, DiffDiagnostic, DiffDiagnosticSeverity, DiffDocument, DiffDocumentError,
    DiffFile, DiffHunk, DiffLimits, DiffLine, DiffOperation, HunkRange, LineEnding,
    NewlineComparison, TextDocument, TextDocumentError, TextFormat, TextNormalization,
    WhitespaceComparison,
};
pub use unified::{
    parse_unified_diff, write_unified_diff, UnifiedDiffError, UnifiedDiffErrorKind,
    UnifiedDiffWriteError,
};

#[cfg(test)]
mod tests;
