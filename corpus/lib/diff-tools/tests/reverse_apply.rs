use std::collections::BTreeMap;

use diff_tools::{
    apply_exact, diff_text, DiffGenerationConfig, DiffLimits, ExactPatchConfig, TextDocument,
};

#[test]
fn reversing_an_applied_generated_diff_restores_the_original_content() {
    let limits = DiffLimits::default();
    let source = "alpha\nbeta\ngamma\n";
    let target = "alpha\ndelta\ngamma\nomega\n";
    let original = TextDocument::parse(source, limits).expect("source parses");
    let changed = TextDocument::parse(target, limits).expect("target parses");
    let document = diff_text(
        "notes.txt",
        &original,
        "notes.txt",
        &changed,
        DiffGenerationConfig::default(),
        limits,
    )
    .expect("diff generation succeeds");

    let mut files = BTreeMap::new();
    files.insert("notes.txt".to_owned(), source.to_owned());
    let forward = apply_exact(&document, &files, ExactPatchConfig::default());
    assert!(forward.committed);
    assert_eq!(forward.files.get("notes.txt"), Some(&target.to_owned()));

    let reversed = document.reversed(limits).expect("reverse diff is valid");
    let backward = apply_exact(&reversed, &forward.files, ExactPatchConfig::default());
    assert!(backward.committed);
    assert_eq!(backward.files, files);
}

#[test]
fn reversing_swaps_file_identities() {
    let limits = DiffLimits::default();
    let original = TextDocument::parse("old\n", limits).expect("source parses");
    let changed = TextDocument::parse("new\n", limits).expect("target parses");
    let document = diff_text(
        "old-name.txt",
        &original,
        "new-name.txt",
        &changed,
        DiffGenerationConfig::default(),
        limits,
    )
    .expect("diff generation succeeds");

    let reversed = document.reversed(limits).expect("reverse diff is valid");
    let file = &reversed.files()[0];
    assert_eq!(file.old_path(), "new-name.txt");
    assert_eq!(file.new_path(), "old-name.txt");
}
