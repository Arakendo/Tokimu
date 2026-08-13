use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// A deterministic, provider-neutral observation fixture for a future
/// persistent Resource Space provider. It records only public semantic facts.
#[derive(Debug, Serialize)]
pub struct ResourceSpaceConformanceReport {
    schema: u32,
    contract: &'static str,
    provider: &'static str,
    fixture: FixtureObservation,
    expectations: Expectations,
}

#[derive(Debug, Serialize)]
pub struct FixtureObservation {
    pub fixture_store: StoreObservation,
    pub imported_store: StoreObservation,
    pub identity: IdentityObservation,
    pub registry_identity: RegistryIdentityObservation,
    pub navigation: NavigationObservation,
    pub mutation_window: MutationWindowObservation,
    pub bridges: BridgeObservation,
}

#[derive(Debug, Serialize)]
pub struct StoreObservation {
    pub id: u128,
    pub display_name: String,
    pub origin: String,
    pub label: Option<String>,
    pub roots: usize,
    pub folders: usize,
    pub resources: usize,
    pub retained_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct IdentityObservation {
    pub equal_display_names: bool,
    pub visible_address: String,
    pub imported_address: String,
    pub visible_key: QualifiedKeyObservation,
    pub imported_key: QualifiedKeyObservation,
    pub content_equal: bool,
    pub hidden_address: String,
    pub hidden_is_enumerated_as_visible: bool,
}

/// Public create-or-open semantics for a stable logical store identity.
///
/// This records the behavior a future provider must preserve without
/// prescribing how it stores descriptors or resource bytes.
#[derive(Debug, Serialize)]
pub struct RegistryIdentityObservation {
    pub same_id_reopened_existing: bool,
    pub existing_descriptor_preserved: bool,
    pub case_policy_mismatch_rejected: bool,
}

#[derive(Debug, Serialize)]
pub struct QualifiedKeyObservation {
    pub store: u128,
    pub root: u128,
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct NavigationObservation {
    pub root_documents: Vec<String>,
    pub root_folders: Vec<String>,
    pub retained_empty_folder: String,
    pub utility_address: String,
}

#[derive(Debug, Serialize)]
pub struct MutationWindowObservation {
    pub enabled_for_fixture_store: bool,
    pub enabled_for_imported_store: bool,
    pub capacity: usize,
    pub retained_count: usize,
    pub first_sequence: u64,
    pub last_sequence: u64,
}

#[derive(Debug, Serialize)]
pub struct BridgeObservation {
    pub prepared_asset_bytes: usize,
    pub gltf_primitives: usize,
    pub gltf_indices: usize,
    pub gltf_image_address: String,
    pub xml_reference_address: String,
    pub json_primary_scene: String,
}

#[derive(Debug, Serialize)]
struct Expectations {
    provider_boundary: &'static str,
    persistence_boundary: &'static str,
    identity_boundary: &'static str,
    mutation_boundary: &'static str,
}

impl ResourceSpaceConformanceReport {
    pub fn new(fixture: FixtureObservation) -> Self {
        Self {
            schema: 1,
            contract: "resource-space-provider-conformance-v1",
            provider: "in-memory",
            fixture,
            expectations: Expectations {
                provider_boundary: "A provider must expose equivalent public semantic observations without exposing backing collections, filesystem paths, browser handles, or database records.",
                persistence_boundary: "This artifact does not claim durability, cross-process identity, transaction semantics, synchronization, or provider implementation equivalence.",
                identity_boundary: "Equal display names or content do not imply equal logical stores or qualified resource keys.",
                mutation_boundary: "Mutation observations are a bounded local diagnostic window, not a durable revision log or synchronization protocol.",
            },
        }
    }
}

pub fn output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .join("target/resource-space-conformance/hello-resource-space/conformance-v1.json")
}

pub fn write_report(
    output: &Path,
    report: &ResourceSpaceConformanceReport,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, serde_json::to_vec_pretty(report)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use diff_tools::{compare_json, JsonComparisonConfig};

    #[test]
    fn serializes_a_stable_provider_neutral_contract_marker() {
        let report = ResourceSpaceConformanceReport::new(FixtureObservation {
            fixture_store: StoreObservation {
                id: 1,
                display_name: "Project resources".to_owned(),
                origin: "Fixture".to_owned(),
                label: Some("fixture".to_owned()),
                roots: 1,
                folders: 1,
                resources: 1,
                retained_bytes: 4,
            },
            imported_store: StoreObservation {
                id: 2,
                display_name: "Project resources".to_owned(),
                origin: "Imported".to_owned(),
                label: Some("drop".to_owned()),
                roots: 1,
                folders: 1,
                resources: 1,
                retained_bytes: 4,
            },
            identity: IdentityObservation {
                equal_display_names: true,
                visible_address: "images/reference.png".to_owned(),
                imported_address: "reference.png".to_owned(),
                visible_key: QualifiedKeyObservation {
                    store: 1,
                    root: 10,
                    address: "images/reference.png".to_owned(),
                },
                imported_key: QualifiedKeyObservation {
                    store: 2,
                    root: 20,
                    address: "reference.png".to_owned(),
                },
                content_equal: true,
                hidden_address: "images/drafts/reference-dark.png".to_owned(),
                hidden_is_enumerated_as_visible: false,
            },
            registry_identity: RegistryIdentityObservation {
                same_id_reopened_existing: true,
                existing_descriptor_preserved: true,
                case_policy_mismatch_rejected: true,
            },
            navigation: NavigationObservation {
                root_documents: vec!["data.xml".to_owned()],
                root_folders: vec!["document-drafts".to_owned()],
                retained_empty_folder: "document-drafts".to_owned(),
                utility_address: "common/utilities.xsl".to_owned(),
            },
            mutation_window: MutationWindowObservation {
                enabled_for_fixture_store: true,
                enabled_for_imported_store: false,
                capacity: 16,
                retained_count: 16,
                first_sequence: 4,
                last_sequence: 19,
            },
            bridges: BridgeObservation {
                prepared_asset_bytes: 4,
                gltf_primitives: 1,
                gltf_indices: 3,
                gltf_image_address: "models/swatch.png".to_owned(),
                xml_reference_address: "models/symbols.svg".to_owned(),
                json_primary_scene: "triangle.gltf".to_owned(),
            },
        });

        let serialized = serde_json::to_string(&report).expect("report must serialize");
        assert!(serialized.contains("resource-space-provider-conformance-v1"));
        assert!(serialized.contains("This artifact does not claim durability"));
        assert!(serialized.contains("filesystem paths"));
        assert!(!serialized.contains("backing_collection"));
    }

    #[test]
    fn conformance_artifact_uses_the_public_structural_diff_contract() {
        let report = ResourceSpaceConformanceReport::new(FixtureObservation {
            fixture_store: StoreObservation {
                id: 1,
                display_name: "Project resources".to_owned(),
                origin: "Fixture".to_owned(),
                label: Some("fixture".to_owned()),
                roots: 1,
                folders: 1,
                resources: 1,
                retained_bytes: 4,
            },
            imported_store: StoreObservation {
                id: 2,
                display_name: "Project resources".to_owned(),
                origin: "Imported".to_owned(),
                label: Some("drop".to_owned()),
                roots: 1,
                folders: 1,
                resources: 1,
                retained_bytes: 4,
            },
            identity: IdentityObservation {
                equal_display_names: true,
                visible_address: "images/reference.png".to_owned(),
                imported_address: "reference.png".to_owned(),
                visible_key: QualifiedKeyObservation {
                    store: 1,
                    root: 10,
                    address: "images/reference.png".to_owned(),
                },
                imported_key: QualifiedKeyObservation {
                    store: 2,
                    root: 20,
                    address: "reference.png".to_owned(),
                },
                content_equal: true,
                hidden_address: "images/drafts/reference-dark.png".to_owned(),
                hidden_is_enumerated_as_visible: false,
            },
            registry_identity: RegistryIdentityObservation {
                same_id_reopened_existing: true,
                existing_descriptor_preserved: true,
                case_policy_mismatch_rejected: true,
            },
            navigation: NavigationObservation {
                root_documents: vec!["data.xml".to_owned()],
                root_folders: vec!["document-drafts".to_owned()],
                retained_empty_folder: "document-drafts".to_owned(),
                utility_address: "common/utilities.xsl".to_owned(),
            },
            mutation_window: MutationWindowObservation {
                enabled_for_fixture_store: true,
                enabled_for_imported_store: false,
                capacity: 16,
                retained_count: 16,
                first_sequence: 4,
                last_sequence: 19,
            },
            bridges: BridgeObservation {
                prepared_asset_bytes: 4,
                gltf_primitives: 1,
                gltf_indices: 3,
                gltf_image_address: "models/swatch.png".to_owned(),
                xml_reference_address: "models/symbols.svg".to_owned(),
                json_primary_scene: "triangle.gltf".to_owned(),
            },
        });

        let expected = serde_json::to_value(&report).expect("report must serialize");
        let equal = compare_json(&expected, &expected, &JsonComparisonConfig::default())
            .expect("public structural comparison must succeed");
        assert!(equal.equal);

        let mut changed = expected.clone();
        changed["fixture"]["identity"]["content_equal"] = serde_json::Value::Bool(false);
        let comparison = compare_json(&expected, &changed, &JsonComparisonConfig::default())
            .expect("public structural comparison must retain differences");
        assert!(!comparison.equal);
        assert_eq!(comparison.differences.len(), 1);
        assert_eq!(
            comparison.differences[0].path,
            "/fixture/identity/content_equal"
        );
    }
}
