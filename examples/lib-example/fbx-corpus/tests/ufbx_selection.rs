use std::path::{Path, PathBuf};

use fbx_corpus::{decode_binary_fbx_file, source_records_json, FbxLimits, FbxProperty, FbxRecord};

#[test]
fn decodes_selected_legacy_binary_cube_deterministically() {
    let path = fixture("maya_cube_6100_binary.fbx");
    let first = decode_binary_fbx_file(&path, FbxLimits::default()).unwrap();
    let second = decode_binary_fbx_file(&path, FbxLimits::default()).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.version, 6100);
    assert!(!first.records.is_empty());
    assert!(first.records.iter().any(|record| record.name == "Objects"));
    assert_eq!(
        source_records_json(&first).unwrap(),
        source_records_json(&second).unwrap()
    );
}

#[test]
fn decodes_selected_modern_binary_cube() {
    let document =
        decode_binary_fbx_file(fixture("maya_cube_7500_binary.fbx"), FbxLimits::default()).unwrap();

    assert_eq!(document.version, 7500);
    assert!(document
        .records
        .iter()
        .any(|record| record.name == "Objects"));
    assert!(all_records(&document.records).any(|record| {
        record.properties.iter().any(|property| {
            matches!(
                property,
                FbxProperty::F32Array(_)
                    | FbxProperty::F64Array(_)
                    | FbxProperty::I32Array(_)
                    | FbxProperty::I64Array(_)
            )
        })
    }));
    assert!(document.footer_offset < document.source_bytes);
}

fn fixture(name: &str) -> PathBuf {
    workspace_root()
        .join("third-party/fixtures/fbx-corpus/upstream/data")
        .join(name)
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("fbx-corpus is nested three levels below the workspace root")
}

fn all_records(records: &[FbxRecord]) -> Box<dyn Iterator<Item = &FbxRecord> + '_> {
    Box::new(
        records
            .iter()
            .flat_map(|record| std::iter::once(record).chain(all_records(&record.children))),
    )
}
