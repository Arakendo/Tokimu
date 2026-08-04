use std::fmt::Write;

use thiserror::Error;

use crate::{
    DiffDocument, DiffDocumentError, DiffFile, DiffHunk, DiffLimits, DiffLine, DiffOperation,
    HunkRange, LineEnding, TextFormat, TextNormalization,
};

const FINAL_NEWLINE_MARKER: &str = "\\ No newline at end of file";

/// Parses Tokimu's admitted, bounded unified-diff subset.
///
/// The initial dialect deliberately accepts only plain file headers and hunks.
/// Git metadata, rename records, and binary patches are rejected explicitly
/// rather than guessed. The standard final-newline marker is retained as a
/// source/target format fact on the parsed file.
pub fn parse_unified_diff(
    input: &str,
    limits: DiffLimits,
) -> Result<DiffDocument, UnifiedDiffError> {
    if input.len() > limits.max_input_bytes {
        return Err(UnifiedDiffError::new(
            1,
            UnifiedDiffErrorKind::InputTooLarge {
                actual: input.len(),
                limit: limits.max_input_bytes,
            },
        ));
    }

    let lines = protocol_lines(input, limits)?;
    let mut cursor = 0;
    let mut files = Vec::new();

    while cursor < lines.len() {
        let old_path = parse_file_header(lines[cursor], "--- ", cursor + 1, true)?;
        cursor += 1;

        let new_header = lines.get(cursor).ok_or_else(|| {
            UnifiedDiffError::new(cursor + 1, UnifiedDiffErrorKind::MissingNewFileHeader)
        })?;
        let new_path = parse_file_header(new_header, "+++ ", cursor + 1, false)?;
        cursor += 1;

        let mut hunks = Vec::new();
        let mut final_newlines = FinalNewlines::default();
        while cursor < lines.len() && !lines[cursor].starts_with("--- ") {
            if !lines[cursor].starts_with("@@ ") {
                return Err(UnifiedDiffError::new(
                    cursor + 1,
                    UnifiedDiffErrorKind::ExpectedHunkHeader,
                ));
            }

            let (old_range, new_range, header) = parse_hunk_header(lines[cursor], cursor + 1)?;
            cursor += 1;
            let mut hunk_lines: Vec<DiffLine> = Vec::new();

            while cursor < lines.len()
                && !lines[cursor].starts_with("@@ ")
                && !lines[cursor].starts_with("--- ")
            {
                let line = lines[cursor];
                if line == FINAL_NEWLINE_MARKER {
                    let previous = hunk_lines.last().ok_or_else(|| {
                        UnifiedDiffError::new(
                            cursor + 1,
                            UnifiedDiffErrorKind::InvalidFinalNewlineMarker,
                        )
                    })?;
                    final_newlines.record(previous.operation, cursor + 1)?;
                    cursor += 1;
                    continue;
                }

                let Some((prefix, text)) = line.get(..1).map(|prefix| (prefix, &line[1..])) else {
                    return Err(UnifiedDiffError::new(
                        cursor + 1,
                        UnifiedDiffErrorKind::UnexpectedHunkLine,
                    ));
                };
                let operation = match prefix {
                    " " => DiffOperation::Context,
                    "-" => DiffOperation::Remove,
                    "+" => DiffOperation::Add,
                    _ => {
                        return Err(UnifiedDiffError::new(
                            cursor + 1,
                            UnifiedDiffErrorKind::UnexpectedHunkLine,
                        ));
                    }
                };
                hunk_lines.push(DiffLine::new(operation, text));
                if hunk_lines.len() > limits.max_hunk_lines {
                    return Err(UnifiedDiffError::new(
                        cursor + 1,
                        UnifiedDiffErrorKind::TooManyHunkLines {
                            actual: hunk_lines.len(),
                            limit: limits.max_hunk_lines,
                        },
                    ));
                }
                cursor += 1;
            }

            validate_hunk_counts(&old_range, &new_range, &hunk_lines, cursor)?;
            hunks.push(DiffHunk::new(old_range, new_range, hunk_lines, header));
            if hunks.len() > limits.max_hunks_per_file {
                return Err(UnifiedDiffError::new(
                    cursor,
                    UnifiedDiffErrorKind::TooManyHunks {
                        actual: hunks.len(),
                        limit: limits.max_hunks_per_file,
                    },
                ));
            }
        }

        if hunks.is_empty() {
            return Err(UnifiedDiffError::new(
                cursor + 1,
                UnifiedDiffErrorKind::MissingHunk,
            ));
        }
        let file = DiffFile::new(old_path, new_path, hunks)
            .with_optional_text_formats(final_newlines.into_text_formats());
        files.push(file);
        if files.len() > limits.max_files {
            return Err(UnifiedDiffError::new(
                cursor,
                UnifiedDiffErrorKind::TooManyFiles {
                    actual: files.len(),
                    limit: limits.max_files,
                },
            ));
        }
    }

    DiffDocument::new(files, TextNormalization::default(), limits)
        .map_err(|error| UnifiedDiffError::new(1, UnifiedDiffErrorKind::Document(error)))
}

/// Writes a structured document using one deterministic unified-diff form.
pub fn write_unified_diff(document: &DiffDocument) -> Result<String, UnifiedDiffWriteError> {
    let mut output = String::new();
    for file in document.files() {
        if let (Some(old), Some(new)) = (file.old_format(), file.new_format()) {
            if old.line_ending != new.line_ending {
                return Err(UnifiedDiffWriteError::UnrepresentableLineEndingChange {
                    path: file.new_path().to_owned(),
                });
            }
        }

        writeln!(output, "--- {}", file.old_path()).expect("writing to string cannot fail");
        writeln!(output, "+++ {}", file.new_path()).expect("writing to string cannot fail");
        let final_newlines = FinalNewlines::from_file(file);
        for (hunk_index, hunk) in file.hunks().iter().enumerate() {
            validate_hunk_counts(&hunk.old_range, &hunk.new_range, &hunk.lines, 0).map_err(
                |error| UnifiedDiffWriteError::InvalidHunk {
                    path: file.new_path().to_owned(),
                    reason: error.kind.to_string(),
                },
            )?;
            write!(
                output,
                "@@ -{},{} +{},{} @@",
                hunk.old_range.start,
                hunk.old_range.count,
                hunk.new_range.start,
                hunk.new_range.count
            )
            .expect("writing to string cannot fail");
            if let Some(header) = &hunk.header {
                write!(output, " {header}").expect("writing to string cannot fail");
            }
            output.push('\n');
            for (line_index, line) in hunk.lines.iter().enumerate() {
                let prefix = match line.operation {
                    DiffOperation::Context => ' ',
                    DiffOperation::Remove => '-',
                    DiffOperation::Add => '+',
                };
                writeln!(output, "{prefix}{}", line.text).expect("writing to string cannot fail");
                if final_newlines.applies_to(hunk_index, line_index, line.operation) {
                    writeln!(output, "{FINAL_NEWLINE_MARKER}")
                        .expect("writing to string cannot fail");
                }
            }
        }
    }
    Ok(output)
}

#[derive(Debug, Clone, Copy, Default)]
struct FinalNewlines {
    old_missing: bool,
    new_missing: bool,
    old_marker_line: Option<(usize, usize)>,
    new_marker_line: Option<(usize, usize)>,
}

impl FinalNewlines {
    fn record(&mut self, operation: DiffOperation, line: usize) -> Result<(), UnifiedDiffError> {
        let marks_old = !matches!(operation, DiffOperation::Add);
        let marks_new = !matches!(operation, DiffOperation::Remove);
        if (marks_old && self.old_missing) || (marks_new && self.new_missing) {
            return Err(UnifiedDiffError::new(
                line,
                UnifiedDiffErrorKind::InvalidFinalNewlineMarker,
            ));
        }
        self.old_missing |= marks_old;
        self.new_missing |= marks_new;
        Ok(())
    }

    fn into_text_formats(self) -> Option<(TextFormat, TextFormat)> {
        (self.old_missing || self.new_missing).then_some((
            TextFormat {
                line_ending: LineEnding::Lf,
                ends_with_newline: !self.old_missing,
            },
            TextFormat {
                line_ending: LineEnding::Lf,
                ends_with_newline: !self.new_missing,
            },
        ))
    }

    fn from_file(file: &DiffFile) -> Self {
        let Some((old, new)) = file.old_format().zip(file.new_format()) else {
            return Self::default();
        };
        let mut result = Self {
            old_missing: !old.ends_with_newline,
            new_missing: !new.ends_with_newline,
            ..Self::default()
        };
        if result.old_missing {
            result.old_marker_line = find_terminal_line(file, true);
        }
        if result.new_missing {
            result.new_marker_line = find_terminal_line(file, false);
        }
        result
    }

    fn applies_to(&self, hunk: usize, line: usize, operation: DiffOperation) -> bool {
        let location = (hunk, line);
        (self.old_missing
            && self.old_marker_line == Some(location)
            && !matches!(operation, DiffOperation::Add))
            || (self.new_missing
                && self.new_marker_line == Some(location)
                && !matches!(operation, DiffOperation::Remove))
    }
}

fn find_terminal_line(file: &DiffFile, old: bool) -> Option<(usize, usize)> {
    file.hunks()
        .iter()
        .enumerate()
        .rev()
        .find_map(|(hunk_index, hunk)| {
            hunk.lines
                .iter()
                .enumerate()
                .rev()
                .find_map(|(line_index, line)| {
                    let participates = if old {
                        !matches!(line.operation, DiffOperation::Add)
                    } else {
                        !matches!(line.operation, DiffOperation::Remove)
                    };
                    participates.then_some((hunk_index, line_index))
                })
        })
}

fn protocol_lines(input: &str, limits: DiffLimits) -> Result<Vec<&str>, UnifiedDiffError> {
    let mut lines: Vec<_> = input.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    for line in &mut lines {
        *line = line.strip_suffix('\r').unwrap_or(line);
    }
    if lines.len() > limits.max_lines {
        return Err(UnifiedDiffError::new(
            lines.len(),
            UnifiedDiffErrorKind::TooManyLines {
                actual: lines.len(),
                limit: limits.max_lines,
            },
        ));
    }
    Ok(lines)
}

fn parse_file_header(
    line: &str,
    prefix: &str,
    number: usize,
    old: bool,
) -> Result<String, UnifiedDiffError> {
    let Some(path) = line.strip_prefix(prefix) else {
        return Err(UnifiedDiffError::new(
            number,
            if old {
                UnifiedDiffErrorKind::MissingOldFileHeader
            } else {
                UnifiedDiffErrorKind::MissingNewFileHeader
            },
        ));
    };
    if path.is_empty() {
        return Err(UnifiedDiffError::new(
            number,
            UnifiedDiffErrorKind::EmptyPath,
        ));
    }
    Ok(path.to_owned())
}

fn parse_hunk_header(
    line: &str,
    number: usize,
) -> Result<(HunkRange, HunkRange, Option<String>), UnifiedDiffError> {
    let Some(body) = line.strip_prefix("@@ -") else {
        return Err(UnifiedDiffError::new(
            number,
            UnifiedDiffErrorKind::InvalidHunkHeader,
        ));
    };
    let Some((ranges, suffix)) = body.split_once(" @@") else {
        return Err(UnifiedDiffError::new(
            number,
            UnifiedDiffErrorKind::InvalidHunkHeader,
        ));
    };
    let Some((old_range, new_range)) = ranges.split_once(" +") else {
        return Err(UnifiedDiffError::new(
            number,
            UnifiedDiffErrorKind::InvalidHunkHeader,
        ));
    };
    let old_range = parse_range(old_range, number)?;
    let new_range = parse_range(new_range, number)?;
    let header = suffix.strip_prefix(' ').map(str::to_owned);
    Ok((old_range, new_range, header))
}

fn parse_range(value: &str, number: usize) -> Result<HunkRange, UnifiedDiffError> {
    let (start, count) = match value.split_once(',') {
        Some((start, count)) => (start, count),
        None => (value, "1"),
    };
    let start = start
        .parse()
        .map_err(|_| UnifiedDiffError::new(number, UnifiedDiffErrorKind::InvalidHunkHeader))?;
    let count = count
        .parse()
        .map_err(|_| UnifiedDiffError::new(number, UnifiedDiffErrorKind::InvalidHunkHeader))?;
    Ok(HunkRange { start, count })
}

fn validate_hunk_counts(
    old_range: &HunkRange,
    new_range: &HunkRange,
    lines: &[DiffLine],
    number: usize,
) -> Result<(), UnifiedDiffError> {
    let old_count = lines
        .iter()
        .filter(|line| !matches!(line.operation, DiffOperation::Add))
        .count();
    let new_count = lines
        .iter()
        .filter(|line| !matches!(line.operation, DiffOperation::Remove))
        .count();
    if old_count != old_range.count || new_count != new_range.count {
        return Err(UnifiedDiffError::new(
            number,
            UnifiedDiffErrorKind::HunkCountMismatch {
                declared_old: old_range.count,
                actual_old: old_count,
                declared_new: new_range.count,
                actual_new: new_count,
            },
        ));
    }
    Ok(())
}

/// A location-aware rejected unified-diff input.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unified diff error at line {line}: {kind}")]
pub struct UnifiedDiffError {
    pub line: usize,
    pub kind: UnifiedDiffErrorKind,
}

impl UnifiedDiffError {
    fn new(line: usize, kind: UnifiedDiffErrorKind) -> Self {
        Self { line, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UnifiedDiffErrorKind {
    #[error("input exceeds the {limit}-byte limit ({actual} bytes)")]
    InputTooLarge { actual: usize, limit: usize },
    #[error("input exceeds the {limit}-line limit ({actual} lines)")]
    TooManyLines { actual: usize, limit: usize },
    #[error("expected an old-file header")]
    MissingOldFileHeader,
    #[error("expected a new-file header")]
    MissingNewFileHeader,
    #[error("file path is empty")]
    EmptyPath,
    #[error("expected a hunk header")]
    ExpectedHunkHeader,
    #[error("hunk header is malformed")]
    InvalidHunkHeader,
    #[error("file contains no hunks")]
    MissingHunk,
    #[error("hunk line does not begin with context, removal, or addition prefix")]
    UnexpectedHunkLine,
    #[error("final-newline marker does not follow a hunk line or marks the same side twice")]
    InvalidFinalNewlineMarker,
    #[error("unsupported unified-diff extension: {extension}")]
    UnsupportedExtension { extension: String },
    #[error("hunk has {actual} lines, exceeding the {limit}-line limit")]
    TooManyHunkLines { actual: usize, limit: usize },
    #[error("file has {actual} hunks, exceeding the {limit}-hunk limit")]
    TooManyHunks { actual: usize, limit: usize },
    #[error("document has {actual} files, exceeding the {limit}-file limit")]
    TooManyFiles { actual: usize, limit: usize },
    #[error(
        "hunk counts do not match: old declared {declared_old}, actual {actual_old}; new declared {declared_new}, actual {actual_new}"
    )]
    HunkCountMismatch {
        declared_old: usize,
        actual_old: usize,
        declared_new: usize,
        actual_new: usize,
    },
    #[error("structured document is invalid: {0}")]
    Document(DiffDocumentError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UnifiedDiffWriteError {
    #[error("file {path:?} changes line-ending convention, which unified diff cannot represent")]
    UnrepresentableLineEndingChange { path: String },
    #[error("file {path:?} contains an invalid hunk: {reason}")]
    InvalidHunk { path: String, reason: String },
}
