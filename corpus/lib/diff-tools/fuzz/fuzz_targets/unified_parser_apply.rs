#![no_main]

use std::collections::BTreeMap;

use diff_tools::{DiffLimits, ExactPatchConfig, apply_exact, parse_unified_diff};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    let limits = DiffLimits {
        max_input_bytes: 8 * 1024,
        max_lines: 512,
        max_hunk_lines: 512,
        ..DiffLimits::default()
    };
    let source_files = BTreeMap::from([
        ("notes.txt".to_owned(), "alpha\nbeta\n".to_owned()),
        ("unicode.txt".to_owned(), "alpha\n\u{03b2}\n".to_owned()),
    ]);

    if let Ok(document) = parse_unified_diff(input, limits) {
        let _ = apply_exact(&document, &source_files, ExactPatchConfig::default());
    }
});
