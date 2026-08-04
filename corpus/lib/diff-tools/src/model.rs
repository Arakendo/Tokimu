use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Limits applied before a text document becomes diff input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLimits {
    pub max_input_bytes: usize,
    pub max_lines: usize,
    pub max_files: usize,
    pub max_hunks_per_file: usize,
    pub max_hunk_lines: usize,
    pub max_diagnostics: usize,
    pub max_edit_matrix_cells: usize,
}

impl Default for DiffLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 8 * 1024 * 1024,
            max_lines: 250_000,
            max_files: 1_024,
            max_hunks_per_file: 16_384,
            max_hunk_lines: 65_536,
            max_diagnostics: 1_024,
            max_edit_matrix_cells: 4_000_000,
        }
    }
}

/// The newline convention observed in an input text document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEnding {
    Lf,
    Crlf,
    Cr,
    Mixed,
    None,
}

/// In-memory text with source facts required for an honest later diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextDocument {
    lines: Vec<String>,
    line_ending: LineEnding,
    ends_with_newline: bool,
}

/// Source formatting facts that can change even when all text lines match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFormat {
    pub line_ending: LineEnding,
    pub ends_with_newline: bool,
}

impl TextDocument {
    pub fn parse(source: &str, limits: DiffLimits) -> Result<Self, TextDocumentError> {
        if source.len() > limits.max_input_bytes {
            return Err(TextDocumentError::InputTooLarge {
                actual: source.len(),
                limit: limits.max_input_bytes,
            });
        }

        let ends_with_newline = source.ends_with(['\n', '\r']);
        let mut line_endings = Vec::new();
        let mut lines = Vec::new();
        let mut current = String::new();
        let mut chars = source.chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                        line_endings.push(LineEnding::Crlf);
                    } else {
                        line_endings.push(LineEnding::Cr);
                    }
                    lines.push(std::mem::take(&mut current));
                }
                '\n' => {
                    line_endings.push(LineEnding::Lf);
                    lines.push(std::mem::take(&mut current));
                }
                _ => current.push(character),
            }
        }

        if !source.is_empty() && (!ends_with_newline || line_endings.is_empty()) {
            lines.push(current);
        }

        if lines.len() > limits.max_lines {
            return Err(TextDocumentError::TooManyLines {
                actual: lines.len(),
                limit: limits.max_lines,
            });
        }

        Ok(Self {
            lines,
            line_ending: observed_line_ending(&line_endings),
            ends_with_newline,
        })
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn ends_with_newline(&self) -> bool {
        self.ends_with_newline
    }

    pub fn format(&self) -> TextFormat {
        TextFormat {
            line_ending: self.line_ending,
            ends_with_newline: self.ends_with_newline,
        }
    }
}

fn observed_line_ending(line_endings: &[LineEnding]) -> LineEnding {
    let Some(first) = line_endings.first().copied() else {
        return LineEnding::None;
    };

    if line_endings.iter().all(|ending| *ending == first) {
        first
    } else {
        LineEnding::Mixed
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TextDocumentError {
    #[error("diff input is {actual} bytes, exceeding the {limit}-byte limit")]
    InputTooLarge { actual: usize, limit: usize },
    #[error("diff input has {actual} lines, exceeding the {limit}-line limit")]
    TooManyLines { actual: usize, limit: usize },
}

/// A comparison decision that is intentionally separate from stored text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextNormalization {
    pub whitespace: WhitespaceComparison,
    pub newline: NewlineComparison,
}

impl Default for TextNormalization {
    fn default() -> Self {
        Self {
            whitespace: WhitespaceComparison::Exact,
            newline: NewlineComparison::Exact,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhitespaceComparison {
    Exact,
    IgnoreTrailing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewlineComparison {
    Exact,
    Normalize,
}

/// An ordered multi-file comparison result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffDocument {
    files: Vec<DiffFile>,
    normalization: TextNormalization,
    diagnostics: Vec<DiffDiagnostic>,
    algorithm: Option<DiffAlgorithm>,
}

impl DiffDocument {
    pub fn new(
        files: Vec<DiffFile>,
        normalization: TextNormalization,
        limits: DiffLimits,
    ) -> Result<Self, DiffDocumentError> {
        if files.len() > limits.max_files {
            return Err(DiffDocumentError::TooManyFiles {
                actual: files.len(),
                limit: limits.max_files,
            });
        }

        for file in &files {
            file.validate(limits)?;
        }

        Ok(Self {
            files,
            normalization,
            diagnostics: Vec::new(),
            algorithm: None,
        })
    }

    pub fn files(&self) -> &[DiffFile] {
        &self.files
    }

    pub fn normalization(&self) -> TextNormalization {
        self.normalization
    }

    pub fn diagnostics(&self) -> &[DiffDiagnostic] {
        &self.diagnostics
    }

    pub fn algorithm(&self) -> Option<DiffAlgorithm> {
        self.algorithm
    }

    /// Produces the structural inverse of this diff document.
    ///
    /// Reversal swaps file identities and hunk ranges, then exchanges additions
    /// and removals while preserving operation order. Construction diagnostics
    /// and the source-generation algorithm are intentionally not copied: they
    /// describe the original direction rather than the inverse document.
    pub fn reversed(&self, limits: DiffLimits) -> Result<Self, DiffDocumentError> {
        let files = self.files.iter().map(reverse_file).collect();
        Self::new(files, self.normalization, limits)
    }

    pub fn with_diagnostics(
        mut self,
        diagnostics: Vec<DiffDiagnostic>,
        limits: DiffLimits,
    ) -> Result<Self, DiffDocumentError> {
        if diagnostics.len() > limits.max_diagnostics {
            return Err(DiffDocumentError::TooManyDiagnostics {
                actual: diagnostics.len(),
                limit: limits.max_diagnostics,
            });
        }

        self.diagnostics = diagnostics;
        Ok(self)
    }

    pub(crate) fn with_algorithm(mut self, algorithm: DiffAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }
}

fn reverse_file(file: &DiffFile) -> DiffFile {
    DiffFile {
        old_path: file.new_path.clone(),
        new_path: file.old_path.clone(),
        hunks: file.hunks.iter().map(reverse_hunk).collect(),
        old_format: file.new_format,
        new_format: file.old_format,
    }
}

fn reverse_hunk(hunk: &DiffHunk) -> DiffHunk {
    DiffHunk {
        old_range: hunk.new_range,
        new_range: hunk.old_range,
        lines: hunk
            .lines
            .iter()
            .map(|line| DiffLine {
                operation: match line.operation {
                    DiffOperation::Context => DiffOperation::Context,
                    DiffOperation::Remove => DiffOperation::Add,
                    DiffOperation::Add => DiffOperation::Remove,
                },
                text: line.text.clone(),
            })
            .collect(),
        header: hunk.header.clone(),
    }
}

/// The deterministic edit-script implementation that produced a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffAlgorithm {
    /// Dynamic-programming longest common subsequence, version one.
    LcsV1,
}

/// A machine-readable observation emitted while constructing or consuming a diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffDiagnostic {
    pub severity: DiffDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

impl DiffDiagnostic {
    pub fn new(
        severity: DiffDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffDiagnosticSeverity {
    Information,
    Warning,
    Error,
}

/// The comparison result for one logical file identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffFile {
    old_path: String,
    new_path: String,
    hunks: Vec<DiffHunk>,
    old_format: Option<TextFormat>,
    new_format: Option<TextFormat>,
}

impl DiffFile {
    pub fn new(
        old_path: impl Into<String>,
        new_path: impl Into<String>,
        hunks: Vec<DiffHunk>,
    ) -> Self {
        Self {
            old_path: old_path.into(),
            new_path: new_path.into(),
            hunks,
            old_format: None,
            new_format: None,
        }
    }

    pub fn old_path(&self) -> &str {
        &self.old_path
    }

    pub fn new_path(&self) -> &str {
        &self.new_path
    }

    pub fn hunks(&self) -> &[DiffHunk] {
        &self.hunks
    }

    pub fn old_format(&self) -> Option<TextFormat> {
        self.old_format
    }

    pub fn new_format(&self) -> Option<TextFormat> {
        self.new_format
    }

    pub(crate) fn with_text_formats(mut self, old: TextFormat, new: TextFormat) -> Self {
        self.old_format = Some(old);
        self.new_format = Some(new);
        self
    }

    pub(crate) fn with_optional_text_formats(
        mut self,
        formats: Option<(TextFormat, TextFormat)>,
    ) -> Self {
        if let Some((old, new)) = formats {
            self = self.with_text_formats(old, new);
        }
        self
    }

    fn validate(&self, limits: DiffLimits) -> Result<(), DiffDocumentError> {
        if self.hunks.len() > limits.max_hunks_per_file {
            return Err(DiffDocumentError::TooManyHunks {
                path: self.new_path.clone(),
                actual: self.hunks.len(),
                limit: limits.max_hunks_per_file,
            });
        }

        for hunk in &self.hunks {
            if hunk.lines.len() > limits.max_hunk_lines {
                return Err(DiffDocumentError::TooManyHunkLines {
                    path: self.new_path.clone(),
                    actual: hunk.lines.len(),
                    limit: limits.max_hunk_lines,
                });
            }
        }

        Ok(())
    }
}

/// One source or target range in a hunk, using unified-diff's one-based line numbering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HunkRange {
    pub start: usize,
    pub count: usize,
}

/// A contiguous ordered sequence of line operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_range: HunkRange,
    pub new_range: HunkRange,
    pub lines: Vec<DiffLine>,
    pub header: Option<String>,
}

impl DiffHunk {
    pub fn new(
        old_range: HunkRange,
        new_range: HunkRange,
        lines: Vec<DiffLine>,
        header: Option<String>,
    ) -> Self {
        Self {
            old_range,
            new_range,
            lines,
            header,
        }
    }
}

/// One ordered line operation in a hunk. Text contains no diff display prefix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLine {
    pub operation: DiffOperation,
    pub text: String,
}

impl DiffLine {
    pub fn new(operation: DiffOperation, text: impl Into<String>) -> Self {
        Self {
            operation,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffOperation {
    Context,
    Remove,
    Add,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum DiffDocumentError {
    #[error("diff contains {actual} files, exceeding the {limit}-file limit")]
    TooManyFiles { actual: usize, limit: usize },
    #[error("diff contains {actual} diagnostics, exceeding the {limit}-diagnostic limit")]
    TooManyDiagnostics { actual: usize, limit: usize },
    #[error("diff file {path:?} contains {actual} hunks, exceeding the {limit}-hunk limit")]
    TooManyHunks {
        path: String,
        actual: usize,
        limit: usize,
    },
    #[error(
        "diff file {path:?} contains a hunk with {actual} lines, exceeding the {limit}-line limit"
    )]
    TooManyHunkLines {
        path: String,
        actual: usize,
        limit: usize,
    },
}
