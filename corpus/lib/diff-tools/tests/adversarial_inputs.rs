use std::{collections::BTreeMap, panic::AssertUnwindSafe};

use diff_tools::{apply_exact, parse_unified_diff, DiffLimits, ExactPatchConfig};

const VALID_PATCH: &str = "--- notes.txt\n+++ notes.txt\n@@ -1,2 +1,2 @@\n alpha\n-beta\n+gamma\n";
const FUZZ_VALID_SINGLE_EDIT: &str =
    include_str!("../fuzz/corpus/unified-parser-apply/valid-single-edit.diff");
const FUZZ_UNICODE_EDIT: &str =
    include_str!("../fuzz/corpus/unified-parser-apply/unicode-edit.diff");
const FUZZ_MISSING_FINAL_NEWLINE: &str =
    include_str!("../fuzz/corpus/unified-parser-apply/missing-final-newline.diff");
const FUZZ_MALFORMED_COUNT: &str =
    include_str!("../fuzz/corpus/unified-parser-apply/malformed-count.diff");

#[test]
fn deterministic_mutation_corpus_never_panics_at_parse_or_exact_apply_boundaries() {
    let limits = DiffLimits {
        max_input_bytes: 4 * 1024,
        max_lines: 256,
        max_hunk_lines: 256,
        ..DiffLimits::default()
    };
    let source_files = BTreeMap::from([("notes.txt".to_owned(), "alpha\nbeta\n".to_owned())]);

    for case in mutation_cases(VALID_PATCH.as_bytes(), 384) {
        let input = String::from_utf8(case).expect("the mutation corpus remains ASCII");
        let boundary_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            if let Ok(document) = parse_unified_diff(&input, limits) {
                let _ = apply_exact(&document, &source_files, ExactPatchConfig::default());
            }
        }));

        assert!(boundary_result.is_ok(), "input must not panic: {input:?}");
    }
}

#[test]
fn parser_rejects_an_oversized_adversarial_input_before_structural_work() {
    let input = format!("--- old\n+++ new\n{}", "@".repeat(128));
    let result = parse_unified_diff(
        &input,
        DiffLimits {
            max_input_bytes: 32,
            ..DiffLimits::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn fuzz_seed_corpus_retains_its_admitted_valid_and_malformed_cases() {
    let limits = DiffLimits::default();

    assert!(parse_unified_diff(FUZZ_VALID_SINGLE_EDIT, limits).is_ok());
    assert!(parse_unified_diff(FUZZ_UNICODE_EDIT, limits).is_ok());
    assert!(parse_unified_diff(FUZZ_MISSING_FINAL_NEWLINE, limits).is_ok());
    assert!(parse_unified_diff(FUZZ_MALFORMED_COUNT, limits).is_err());
}

fn mutation_cases(seed: &[u8], count: usize) -> Vec<Vec<u8>> {
    let mut cases = vec![Vec::new(), b"--- old\n".to_vec(), b"@@ -x +y @@\n".to_vec()];
    let mut state = 0xC0DE_CAFE_u64;

    for _ in 0..count {
        state = next(state);
        let mut case = seed.to_vec();
        match state % 4 {
            0 => {
                if !case.is_empty() {
                    let index = (state as usize) % case.len();
                    case.remove(index);
                }
            }
            1 => {
                let index = (state as usize) % (case.len() + 1);
                case.insert(index, b'!');
            }
            2 => {
                if !case.is_empty() {
                    let index = (state as usize) % case.len();
                    case[index] = b'@';
                }
            }
            _ => case.extend_from_slice(b"\n\\ No newline at end of file\n"),
        }
        cases.push(case);
    }

    cases
}

fn next(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
