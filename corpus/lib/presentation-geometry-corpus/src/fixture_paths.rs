//! Discovery and labeling for external corpus fixtures.

use crate::{W3cSvgCase, W3cSvgSource};
use std::path::{Path, PathBuf};

pub(crate) fn find_w3c_fixture_root() -> Option<PathBuf> {
    find_ancestor_fixture(
        "third-party/fixtures/w3c-svg-1.1-2nd-edition",
        |candidate| {
            candidate.join("provenance.json").is_file()
                && candidate.join("selected/selection-v1.toml").is_file()
        },
    )
}

pub(crate) fn w3c_svg_source_path(root: &Path, case: W3cSvgCase) -> PathBuf {
    match case.source {
        W3cSvgSource::UpstreamSvg => root.join("upstream/svg").join(case.file_name),
        W3cSvgSource::DerivedProfileFixture => root.join("selected/derived").join(case.file_name),
    }
}

pub(crate) fn w3c_source_label(case: W3cSvgCase) -> String {
    match case.source {
        W3cSvgSource::UpstreamSvg => {
            format!("W3C SVG 1.1 2nd Edition/{}", case.file_name)
        }
        W3cSvgSource::DerivedProfileFixture => format!(
            "W3C SVG 1.1 2nd Edition derived geometry/{}",
            case.file_name
        ),
    }
}

pub(crate) fn find_lucide_corpus_root() -> Option<PathBuf> {
    find_ancestor_fixture("target/lucide-corpus-100", |candidate| {
        candidate.join("provenance.json").is_file()
    })
}

fn find_ancestor_fixture(
    relative_path: &str,
    is_fixture_root: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let mut directory = std::env::current_dir().ok()?;
    loop {
        let candidate = directory.join(relative_path);
        if is_fixture_root(&candidate) {
            return Some(candidate);
        }
        if !directory.pop() {
            return None;
        }
    }
}
