use std::collections::BTreeMap;

use super::{
    apply_exact, apply_fuzzy_hunk, compare_json, diff_text, locate_fuzzy_hunk, parse_unified_diff,
    write_unified_diff, DiffAlgorithm, DiffDiagnostic, DiffDiagnosticSeverity, DiffDocument,
    DiffDocumentError, DiffFile, DiffGenerationConfig, DiffGenerationError, DiffHunk, DiffLimits,
    DiffLine, DiffOperation, ExactFileOutcome, ExactHunkOutcome, ExactPatchConfig,
    ExactPatchPolicy, ExactPatchRejection, FuzzyHunkApplication, FuzzyHunkSearch, FuzzyMatchConfig,
    FuzzyMatchError, FuzzyPatchConfig, HunkRange, JsonComparisonConfig, JsonComparisonError,
    JsonDifferenceKind, LineEnding, NewlineComparison, TextDocument, TextDocumentError,
    TextNormalization, UnifiedDiffErrorKind, WhitespaceComparison,
};

#[test]
fn parses_lf_content_and_preserves_the_final_newline_fact() {
    let document = TextDocument::parse("alpha\nbeta\n", DiffLimits::default()).unwrap();

    assert_eq!(document.lines(), ["alpha", "beta"]);
    assert_eq!(document.line_ending(), LineEnding::Lf);
    assert!(document.ends_with_newline());
}

#[test]
fn preserves_a_missing_final_newline() {
    let document = TextDocument::parse("café\nnaïve\ndelta", DiffLimits::default()).unwrap();

    assert_eq!(document.lines(), ["café", "naïve", "delta"]);
    assert_eq!(document.line_ending(), LineEnding::Lf);
    assert!(!document.ends_with_newline());
}

#[test]
fn identifies_crlf_and_mixed_line_endings() {
    let crlf = TextDocument::parse("alpha\r\nbeta\r\n", DiffLimits::default()).unwrap();
    let mixed = TextDocument::parse("alpha\nbeta\r\n", DiffLimits::default()).unwrap();

    assert_eq!(crlf.line_ending(), LineEnding::Crlf);
    assert_eq!(mixed.line_ending(), LineEnding::Mixed);
}

#[test]
fn rejects_inputs_before_unbounded_line_growth() {
    let limits = DiffLimits {
        max_input_bytes: 32,
        max_lines: 2,
        ..DiffLimits::default()
    };

    let error = TextDocument::parse("one\ntwo\nthree", limits).unwrap_err();
    assert_eq!(
        error,
        TextDocumentError::TooManyLines {
            actual: 3,
            limit: 2,
        }
    );
}

#[test]
fn structured_diffs_preserve_operation_order_and_normalization_policy() {
    let hunk = DiffHunk::new(
        HunkRange { start: 2, count: 1 },
        HunkRange { start: 2, count: 1 },
        vec![
            DiffLine::new(DiffOperation::Context, "before"),
            DiffLine::new(DiffOperation::Remove, "old"),
            DiffLine::new(DiffOperation::Add, "new"),
        ],
        Some("rename a value".into()),
    );
    let file = DiffFile::new("before.txt", "after.txt", vec![hunk]);
    let document = DiffDocument::new(
        vec![file],
        TextNormalization::default(),
        DiffLimits::default(),
    )
    .unwrap()
    .with_diagnostics(
        vec![DiffDiagnostic::new(
            DiffDiagnosticSeverity::Information,
            "fixture",
            "constructed in-memory",
        )],
        DiffLimits::default(),
    )
    .unwrap();

    assert_eq!(document.files()[0].old_path(), "before.txt");
    assert_eq!(
        document.files()[0].hunks()[0].lines[1].operation,
        DiffOperation::Remove
    );
    assert_eq!(document.files()[0].hunks()[0].lines[2].text, "new");
    assert_eq!(document.normalization(), TextNormalization::default());
    assert_eq!(document.diagnostics()[0].code, "fixture");

    let encoded = serde_json::to_string(&document).unwrap();
    let decoded: DiffDocument = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, document);
}

#[test]
fn structured_diffs_reject_excessive_hunks_before_consumers_observe_them() {
    let file = DiffFile::new(
        "before.txt",
        "after.txt",
        vec![
            DiffHunk::new(
                HunkRange { start: 1, count: 0 },
                HunkRange { start: 1, count: 1 },
                vec![DiffLine::new(DiffOperation::Add, "value")],
                None,
            ),
            DiffHunk::new(
                HunkRange { start: 2, count: 0 },
                HunkRange { start: 3, count: 1 },
                vec![DiffLine::new(DiffOperation::Add, "another")],
                None,
            ),
        ],
    );
    let limits = DiffLimits {
        max_hunks_per_file: 1,
        ..DiffLimits::default()
    };

    let error = DiffDocument::new(vec![file], TextNormalization::default(), limits).unwrap_err();
    assert_eq!(
        error,
        DiffDocumentError::TooManyHunks {
            path: "after.txt".into(),
            actual: 2,
            limit: 1,
        }
    );
}

#[test]
fn keeps_an_empty_document_distinct_from_one_empty_line() {
    let empty = TextDocument::parse("", DiffLimits::default()).unwrap();
    let line = TextDocument::parse("\n", DiffLimits::default()).unwrap();

    assert!(empty.lines().is_empty());
    assert_eq!(line.lines(), [""]);
    assert!(line.ends_with_newline());
}

#[test]
fn rejects_excessive_diagnostics_before_consumers_observe_them() {
    let document = DiffDocument::new(
        Vec::new(),
        TextNormalization::default(),
        DiffLimits::default(),
    )
    .unwrap();
    let limits = DiffLimits {
        max_diagnostics: 1,
        ..DiffLimits::default()
    };

    let error = document
        .with_diagnostics(
            vec![
                DiffDiagnostic::new(DiffDiagnosticSeverity::Warning, "one", "first"),
                DiffDiagnostic::new(DiffDiagnosticSeverity::Warning, "two", "second"),
            ],
            limits,
        )
        .unwrap_err();

    assert_eq!(
        error,
        DiffDocumentError::TooManyDiagnostics {
            actual: 2,
            limit: 1,
        }
    );
}

#[test]
fn identical_inputs_generate_an_empty_document_with_an_observable_algorithm() {
    let source = TextDocument::parse("one\ntwo\n", DiffLimits::default()).unwrap();
    let document = diff_text(
        "same.txt",
        &source,
        "same.txt",
        &source,
        DiffGenerationConfig::default(),
        DiffLimits::default(),
    )
    .unwrap();

    assert!(document.files().is_empty());
    assert_eq!(document.algorithm(), Some(DiffAlgorithm::LcsV1));
}

#[test]
fn generation_is_deterministic_and_preserves_context_ranges() {
    let old = TextDocument::parse("one\ntwo\nthree\nfour\n", DiffLimits::default()).unwrap();
    let new = TextDocument::parse("one\ntokimu\nthree\nfour\n", DiffLimits::default()).unwrap();
    let config = DiffGenerationConfig {
        context_lines: 1,
        ..DiffGenerationConfig::default()
    };
    let first = diff_text(
        "old.txt",
        &old,
        "new.txt",
        &new,
        config,
        DiffLimits::default(),
    )
    .unwrap();
    let second = diff_text(
        "old.txt",
        &old,
        "new.txt",
        &new,
        config,
        DiffLimits::default(),
    )
    .unwrap();

    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    let hunk = &first.files()[0].hunks()[0];
    assert_eq!(hunk.old_range, HunkRange { start: 1, count: 3 });
    assert_eq!(hunk.new_range, HunkRange { start: 1, count: 3 });
    assert_eq!(hunk.lines.len(), 4);
    assert_eq!(hunk.lines[1].operation, DiffOperation::Remove);
    assert_eq!(hunk.lines[2].operation, DiffOperation::Add);
}

#[test]
fn normalization_policy_changes_the_observed_edit_script() {
    let old = TextDocument::parse("value  \n", DiffLimits::default()).unwrap();
    let new = TextDocument::parse("value\n", DiffLimits::default()).unwrap();
    let exact = diff_text(
        "old.txt",
        &old,
        "new.txt",
        &new,
        DiffGenerationConfig::default(),
        DiffLimits::default(),
    )
    .unwrap();
    let normalized = diff_text(
        "old.txt",
        &old,
        "new.txt",
        &new,
        DiffGenerationConfig {
            normalization: TextNormalization {
                whitespace: WhitespaceComparison::IgnoreTrailing,
                newline: NewlineComparison::Normalize,
            },
            ..DiffGenerationConfig::default()
        },
        DiffLimits::default(),
    )
    .unwrap();

    assert_eq!(exact.files()[0].hunks()[0].lines.len(), 2);
    assert!(normalized.files().is_empty());
}

#[test]
fn exact_newline_policy_keeps_format_only_changes_visible() {
    let old = TextDocument::parse("value\n", DiffLimits::default()).unwrap();
    let new = TextDocument::parse("value\r\n", DiffLimits::default()).unwrap();
    let document = diff_text(
        "old.txt",
        &old,
        "new.txt",
        &new,
        DiffGenerationConfig::default(),
        DiffLimits::default(),
    )
    .unwrap();

    assert!(document.files()[0].hunks().is_empty());
    assert_eq!(
        document.files()[0].old_format().unwrap().line_ending,
        LineEnding::Lf
    );
    assert_eq!(
        document.files()[0].new_format().unwrap().line_ending,
        LineEnding::Crlf
    );
}

#[test]
fn repeated_and_dissimilar_lines_keep_the_same_tie_breaking_and_hunk_order() {
    let old =
        TextDocument::parse("same\nleft\nsame\nright\nsame\n", DiffLimits::default()).unwrap();
    let new =
        TextDocument::parse("same\nright\nsame\nleft\nsame\n", DiffLimits::default()).unwrap();
    let document = diff_text(
        "old.txt",
        &old,
        "new.txt",
        &new,
        DiffGenerationConfig {
            context_lines: 0,
            ..DiffGenerationConfig::default()
        },
        DiffLimits::default(),
    )
    .unwrap();

    let operations = document.files()[0]
        .hunks()
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .map(|line| (line.operation, line.text.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        operations,
        vec![
            (DiffOperation::Remove, "left"),
            (DiffOperation::Remove, "same"),
            (DiffOperation::Add, "left"),
            (DiffOperation::Add, "same"),
        ]
    );
}

#[test]
fn generation_rejects_an_edit_matrix_before_unbounded_allocation() {
    let old = TextDocument::parse("one\ntwo\n", DiffLimits::default()).unwrap();
    let new = TextDocument::parse("three\nfour\n", DiffLimits::default()).unwrap();
    let limits = DiffLimits {
        max_edit_matrix_cells: 8,
        ..DiffLimits::default()
    };

    let error = diff_text(
        "old.txt",
        &old,
        "new.txt",
        &new,
        DiffGenerationConfig::default(),
        limits,
    )
    .unwrap_err();

    assert_eq!(
        error,
        DiffGenerationError::MatrixTooLarge {
            actual: 9,
            limit: 8,
        }
    );
}

#[test]
fn unified_parser_and_writer_round_trip_the_admitted_fixture() {
    let fixture = include_str!("../fixtures/exact-addition.diff");
    let parsed = parse_unified_diff(fixture, DiffLimits::default()).unwrap();

    assert_eq!(parsed.files().len(), 1);
    assert_eq!(parsed.files()[0].old_path(), "greeting.txt");
    assert_eq!(parsed.files()[0].new_path(), "greeting.txt");
    assert_eq!(
        parsed.files()[0].hunks()[0].old_range,
        HunkRange { start: 1, count: 2 }
    );
    assert_eq!(
        parsed.files()[0].hunks()[0].new_range,
        HunkRange { start: 1, count: 3 }
    );

    let canonical = write_unified_diff(&parsed).unwrap();
    assert_eq!(canonical, fixture);
    assert_eq!(
        parse_unified_diff(&canonical, DiffLimits::default()).unwrap(),
        parsed
    );
}

#[test]
fn unified_parser_rejects_count_mismatches_with_a_location_aware_error() {
    let fixture = include_str!("../fixtures/malformed-count-mismatch.diff");
    let error = parse_unified_diff(fixture, DiffLimits::default()).unwrap_err();

    assert_eq!(error.line, 6);
    assert_eq!(
        error.kind,
        UnifiedDiffErrorKind::HunkCountMismatch {
            declared_old: 3,
            actual_old: 2,
            declared_new: 3,
            actual_new: 2,
        }
    );
}

#[test]
fn unified_parser_preserves_final_newline_markers_as_source_and_target_facts() {
    let input = "--- old.txt\n+++ new.txt\n@@ -1 +1 @@\n-old\n+new\n\\ No newline at end of file\n";
    let parsed = parse_unified_diff(input, DiffLimits::default()).unwrap();
    let file = &parsed.files()[0];
    assert!(file.old_format().unwrap().ends_with_newline);
    assert!(!file.new_format().unwrap().ends_with_newline);
    assert_eq!(
        write_unified_diff(&parsed).unwrap(),
        "--- old.txt\n+++ new.txt\n@@ -1,1 +1,1 @@\n-old\n+new\n\\ No newline at end of file\n"
    );
}

#[test]
fn unified_parser_rejects_a_final_newline_marker_without_a_preceding_hunk_line() {
    let input = "--- old.txt\n+++ new.txt\n@@ -1 +1 @@\n\\ No newline at end of file\n-old\n+new\n";
    let error = parse_unified_diff(input, DiffLimits::default()).unwrap_err();

    assert_eq!(error.line, 4);
    assert_eq!(error.kind, UnifiedDiffErrorKind::InvalidFinalNewlineMarker);
}

#[test]
fn exact_application_preserves_a_target_missing_final_newline_from_unified_input() {
    let patch = parse_unified_diff(
        "--- old.txt\n+++ new.txt\n@@ -1,1 +1,1 @@\n-old\n+new\n\\ No newline at end of file\n",
        DiffLimits::default(),
    )
    .unwrap();
    let source = BTreeMap::from([("old.txt".to_owned(), "old\n".to_owned())]);

    let applied = apply_exact(&patch, &source, ExactPatchConfig::default());

    assert!(applied.committed);
    assert_eq!(applied.files["new.txt"], "new");
}

#[test]
fn unified_writer_uses_explicit_counts_for_the_canonical_form() {
    let input = "--- old.txt\n+++ new.txt\n@@ -2 +2 @@ section\n-old\n+new\n";
    let parsed = parse_unified_diff(input, DiffLimits::default()).unwrap();

    assert_eq!(
        write_unified_diff(&parsed).unwrap(),
        "--- old.txt\n+++ new.txt\n@@ -2,1 +2,1 @@ section\n-old\n+new\n"
    );
}

#[test]
fn exact_application_updates_in_memory_content_and_reports_each_hunk() {
    let patch = parse_unified_diff(
        include_str!("../fixtures/exact-addition.diff"),
        DiffLimits::default(),
    )
    .unwrap();
    let source = BTreeMap::from([("greeting.txt".to_owned(), "hello\nworld\n".to_owned())]);

    let applied = apply_exact(&patch, &source, ExactPatchConfig::default());

    assert!(applied.committed);
    assert_eq!(applied.files["greeting.txt"], "hello\ntokimu\nworld\n");
    assert_eq!(applied.report.files[0].outcome, ExactFileOutcome::Applied);
    assert_eq!(
        applied.report.files[0].hunks[0].outcome,
        ExactHunkOutcome::Applied
    );
}

#[test]
fn atomic_application_preserves_the_original_map_when_one_file_rejects() {
    let first = DiffFile::new(
        "one.txt",
        "one.txt",
        vec![DiffHunk::new(
            HunkRange { start: 1, count: 1 },
            HunkRange { start: 1, count: 1 },
            vec![
                DiffLine::new(DiffOperation::Remove, "one"),
                DiffLine::new(DiffOperation::Add, "changed"),
            ],
            None,
        )],
    );
    let second = DiffFile::new(
        "missing.txt",
        "missing.txt",
        vec![DiffHunk::new(
            HunkRange { start: 1, count: 0 },
            HunkRange { start: 1, count: 1 },
            vec![DiffLine::new(DiffOperation::Add, "new")],
            None,
        )],
    );
    let patch = DiffDocument::new(
        vec![first, second],
        TextNormalization::default(),
        DiffLimits::default(),
    )
    .unwrap();
    let source = BTreeMap::from([("one.txt".to_owned(), "one\n".to_owned())]);

    let applied = apply_exact(&patch, &source, ExactPatchConfig::default());

    assert!(!applied.committed);
    assert_eq!(applied.files, source);
    assert_eq!(
        applied.report.files[1].outcome,
        ExactFileOutcome::Rejected {
            reason: ExactPatchRejection::MissingSource,
        }
    );
}

#[test]
fn per_file_application_retains_successes_and_reports_context_rejections() {
    let patch = parse_unified_diff(
        "--- one.txt\n+++ one.txt\n@@ -1,1 +1,1 @@\n-before\n+after\n",
        DiffLimits::default(),
    )
    .unwrap();
    let source = BTreeMap::from([("one.txt".to_owned(), "different\n".to_owned())]);

    let applied = apply_exact(
        &patch,
        &source,
        ExactPatchConfig {
            policy: ExactPatchPolicy::PerFile,
        },
    );

    assert!(applied.committed);
    assert_eq!(applied.files, source);
    assert_eq!(
        applied.report.files[0].hunks[0].outcome,
        ExactHunkOutcome::Rejected {
            reason: ExactPatchRejection::ContextMismatch { line: 1 },
        }
    );
}

#[test]
fn exact_application_rejects_overlapping_hunks_without_mutating_the_file() {
    let file = DiffFile::new(
        "one.txt",
        "one.txt",
        vec![
            DiffHunk::new(
                HunkRange { start: 1, count: 1 },
                HunkRange { start: 1, count: 1 },
                vec![
                    DiffLine::new(DiffOperation::Remove, "one"),
                    DiffLine::new(DiffOperation::Add, "first"),
                ],
                None,
            ),
            DiffHunk::new(
                HunkRange { start: 1, count: 1 },
                HunkRange { start: 1, count: 1 },
                vec![
                    DiffLine::new(DiffOperation::Remove, "one"),
                    DiffLine::new(DiffOperation::Add, "second"),
                ],
                None,
            ),
        ],
    );
    let patch = DiffDocument::new(
        vec![file],
        TextNormalization::default(),
        DiffLimits::default(),
    )
    .unwrap();
    let source = BTreeMap::from([("one.txt".to_owned(), "one\n".to_owned())]);

    let applied = apply_exact(&patch, &source, ExactPatchConfig::default());

    assert!(!applied.committed);
    assert_eq!(applied.files, source);
    assert_eq!(
        applied.report.files[0].outcome,
        ExactFileOutcome::Rejected {
            reason: ExactPatchRejection::OverlappingOrOutOfOrderHunk {
                start: 1,
                previous_end: 2,
            },
        }
    );
}

#[test]
fn exact_application_rejects_renames_that_overwrite_an_existing_path() {
    let file = DiffFile::new(
        "old.txt",
        "occupied.txt",
        vec![DiffHunk::new(
            HunkRange { start: 1, count: 1 },
            HunkRange { start: 1, count: 1 },
            vec![DiffLine::new(DiffOperation::Context, "old")],
            None,
        )],
    );
    let patch = DiffDocument::new(
        vec![file],
        TextNormalization::default(),
        DiffLimits::default(),
    )
    .unwrap();
    let source = BTreeMap::from([
        ("old.txt".to_owned(), "old\n".to_owned()),
        ("occupied.txt".to_owned(), "occupied\n".to_owned()),
    ]);

    let applied = apply_exact(&patch, &source, ExactPatchConfig::default());

    assert!(!applied.committed);
    assert_eq!(applied.files, source);
    assert_eq!(
        applied.report.files[0].outcome,
        ExactFileOutcome::Rejected {
            reason: ExactPatchRejection::PathCollision {
                path: "occupied.txt".into(),
            },
        }
    );
}

#[test]
fn fuzzy_locator_reports_a_unique_bounded_stale_hunk_candidate() {
    let patch = parse_unified_diff(
        include_str!("../fixtures/fuzzy-offset.diff"),
        DiffLimits::default(),
    )
    .unwrap();
    let source =
        TextDocument::parse("intro\nmore intro\nhello\nworld\n", DiffLimits::default()).unwrap();

    let result = locate_fuzzy_hunk(
        &source,
        &patch.files()[0].hunks()[0],
        FuzzyMatchConfig {
            max_offset_lines: 4,
            ..FuzzyMatchConfig::default()
        },
    )
    .unwrap();

    assert_eq!(
        result,
        FuzzyHunkSearch::Unique(super::FuzzyHunkCandidate {
            source_start: 3,
            offset_lines: -4,
            matched_context_lines: 2,
        })
    );
}

#[test]
fn fuzzy_locator_reports_ambiguity_instead_of_selecting_a_repeated_context() {
    let patch = parse_unified_diff(
        "--- one.txt\n+++ one.txt\n@@ -2,1 +2,1 @@\n-value\n+changed\n",
        DiffLimits::default(),
    )
    .unwrap();
    let source = TextDocument::parse(
        include_str!("../fixtures/repeated-context-source.txt"),
        DiffLimits::default(),
    )
    .unwrap();

    let result = locate_fuzzy_hunk(
        &source,
        &patch.files()[0].hunks()[0],
        FuzzyMatchConfig {
            max_offset_lines: 2,
            ..FuzzyMatchConfig::default()
        },
    )
    .unwrap();

    assert!(matches!(result, FuzzyHunkSearch::Ambiguous(candidates) if candidates.len() == 3));
}

#[test]
fn fuzzy_locator_rejects_unbounded_candidate_collection() {
    let patch = parse_unified_diff(
        "--- one.txt\n+++ one.txt\n@@ -2,1 +2,1 @@\n-value\n+changed\n",
        DiffLimits::default(),
    )
    .unwrap();
    let source = TextDocument::parse(
        include_str!("../fixtures/repeated-context-source.txt"),
        DiffLimits::default(),
    )
    .unwrap();

    let error = locate_fuzzy_hunk(
        &source,
        &patch.files()[0].hunks()[0],
        FuzzyMatchConfig {
            max_offset_lines: 2,
            max_candidates: 2,
        },
    )
    .unwrap_err();

    assert_eq!(error, FuzzyMatchError::CandidateLimitExceeded { limit: 2 });
}

#[test]
fn fuzzy_application_applies_only_a_unique_stale_hunk_and_retains_its_evidence() {
    let patch = parse_unified_diff(
        include_str!("../fixtures/fuzzy-offset.diff"),
        DiffLimits::default(),
    )
    .unwrap();

    let applied = apply_fuzzy_hunk(
        "intro\nmore intro\nhello\nworld\n",
        &patch.files()[0].hunks()[0],
        FuzzyPatchConfig {
            matching: FuzzyMatchConfig {
                max_offset_lines: 4,
                ..FuzzyMatchConfig::default()
            },
            ..FuzzyPatchConfig::default()
        },
    )
    .unwrap();

    assert_eq!(
        applied,
        FuzzyHunkApplication::Applied {
            content: "intro\nmore intro\nhello\ntokimu\nworld\n".into(),
            candidate: super::FuzzyHunkCandidate {
                source_start: 3,
                offset_lines: -4,
                matched_context_lines: 2,
            },
        }
    );
}

#[test]
fn fuzzy_application_preserves_ambiguous_candidates_without_mutating_content() {
    let patch = parse_unified_diff(
        "--- one.txt\n+++ one.txt\n@@ -2,1 +2,1 @@\n-value\n+changed\n",
        DiffLimits::default(),
    )
    .unwrap();

    let result = apply_fuzzy_hunk(
        include_str!("../fixtures/repeated-context-source.txt"),
        &patch.files()[0].hunks()[0],
        FuzzyPatchConfig {
            matching: FuzzyMatchConfig {
                max_offset_lines: 2,
                ..FuzzyMatchConfig::default()
            },
            ..FuzzyPatchConfig::default()
        },
    )
    .unwrap();

    assert!(
        matches!(result, FuzzyHunkApplication::Ambiguous { candidates } if candidates.len() == 3)
    );
}

#[test]
fn json_comparison_reports_structural_differences_with_stable_pointer_paths() {
    let expected = serde_json::json!({
        "stage": "mesh",
        "stats": { "triangles": 12, "elapsed_ms": 1.0 },
        "paths": ["a", "b"]
    });
    let actual = serde_json::json!({
        "stage": "vector",
        "stats": { "triangles": 16, "elapsed_ms": 2.0 },
        "paths": ["a"],
        "diagnostic": true
    });

    let comparison = compare_json(&expected, &actual, &JsonComparisonConfig::default()).unwrap();

    assert!(!comparison.equal);
    assert!(comparison.differences.iter().any(|difference| {
        difference.path == "/stage"
            && matches!(difference.kind, JsonDifferenceKind::ValueChanged { .. })
    }));
    assert!(comparison.differences.iter().any(|difference| {
        difference.path == "/paths"
            && matches!(
                difference.kind,
                JsonDifferenceKind::ArrayLengthChanged { .. }
            )
    }));
    assert!(comparison.differences.iter().any(|difference| {
        difference.path == "/diagnostic"
            && matches!(
                difference.kind,
                JsonDifferenceKind::UnexpectedActualKey { .. }
            )
    }));
}

#[test]
fn json_comparison_ignores_only_explicitly_selected_volatile_paths() {
    let expected = serde_json::json!({ "artifact": "mesh", "elapsed_ms": 1.0 });
    let actual = serde_json::json!({ "artifact": "mesh", "elapsed_ms": 2.0 });
    let config = JsonComparisonConfig {
        ignored_paths: ["/elapsed_ms".into()].into_iter().collect(),
        ..JsonComparisonConfig::default()
    };

    let comparison = compare_json(&expected, &actual, &config).unwrap();

    assert!(comparison.equal);
    assert_eq!(comparison.ignored_paths, ["/elapsed_ms"]);
}

#[test]
fn json_comparison_refuses_an_unbounded_difference_collection() {
    let expected = serde_json::json!({ "a": 1 });
    let actual = serde_json::json!({ "a": 2 });

    let error = compare_json(
        &expected,
        &actual,
        &JsonComparisonConfig {
            max_differences: 0,
            ..JsonComparisonConfig::default()
        },
    )
    .unwrap_err();

    assert_eq!(error, JsonComparisonError::DifferenceLimitZero);
}
