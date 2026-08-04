use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpaceRegistry, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, ResourceSearchQuery, ResourceStoreDescriptor,
    ResourceStoreOrigin, ResourceStoreProvenance, ResourceStoreRegistryError, ResourceVisibility,
    StoreId, StoreOpenOutcome, VisibilityQuery,
};
use resource_space_assets::{
    decode_gltf_from_resource_space, load_resource_asset,
    resolve_gltf_external_images_from_resource_space,
};
use resource_space_json::{read_json_resource, store_json_resource};
use resource_space_xml::resolve_xml_external_references_from_resource_space;
use serde::{Deserialize, Serialize};
use tokimu_assets::{AssetLoader, AssetStore};

mod benchmark;
mod report;

use benchmark::run_resource_space_benchmark;
use report::{
    output_path, write_report, BridgeObservation, FixtureObservation, IdentityObservation,
    MutationWindowObservation, NavigationObservation, QualifiedKeyObservation,
    RegistryIdentityObservation, ResourceSpaceConformanceReport, StoreObservation,
};

struct ByteCountLoader;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ProjectManifest {
    title: String,
    primary_scene: String,
    revision: u32,
}

impl AssetLoader for ByteCountLoader {
    type Output = usize;

    fn load(&self, source: &[u8]) -> anyhow::Result<Self::Output> {
        Ok(source.len())
    }
}

fn name(value: &str) -> Result<ResourceName, Box<dyn std::error::Error>> {
    Ok(ResourceName::parse(value, AddressCasePolicy::Sensitive)?)
}

const TRIANGLE_GLTF: &[u8] = br#"{
  "asset":{"version":"2.0"},
  "buffers":[{"uri":"triangle.bin","byteLength":42}],
  "bufferViews":[
    {"buffer":0,"byteOffset":0,"byteLength":36},
    {"buffer":0,"byteOffset":36,"byteLength":6}
  ],
  "accessors":[
    {"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"},
    {"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}
  ],
  "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}],
  "images":[{"uri":"swatch.png","mimeType":"image/png"}],
  "textures":[{"source":0}]
}"#;

const SVG_REFERENCE_DOCUMENT: &[u8] = br#"<svg xmlns:xlink="http://www.w3.org/1999/xlink"><use xlink:href="symbols.svg#notice"/></svg>"#;

fn triangle_buffer() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(42);
    for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture_store = StoreId::from_u128(1);
    let imported_store = StoreId::from_u128(2);
    let mut registry = InMemoryResourceSpaceRegistry::new();

    registry.create_new(
        ResourceStoreDescriptor::new(fixture_store, "Project resources").with_provenance(
            ResourceStoreProvenance::with_label(
                ResourceStoreOrigin::Fixture,
                "resource-space-consumer-v1",
            ),
        ),
        AddressCasePolicy::Sensitive,
    )?;
    registry.create_new(
        ResourceStoreDescriptor::new(imported_store, "Project resources").with_provenance(
            ResourceStoreProvenance::with_label(ResourceStoreOrigin::Imported, "drop-zone-1"),
        ),
        AddressCasePolicy::Sensitive,
    )?;

    let same_id_reopen = registry.create_or_open(
        ResourceStoreDescriptor::new(fixture_store, "Ignored replacement descriptor")
            .with_provenance(ResourceStoreProvenance::with_label(
                ResourceStoreOrigin::Generated,
                "ignored-on-open",
            )),
        AddressCasePolicy::Sensitive,
    )?;
    assert_eq!(
        same_id_reopen,
        StoreOpenOutcome::OpenedExisting {
            store: fixture_store
        }
    );
    let original_descriptor_preserved = registry.descriptor(fixture_store)?.display_name()
        == "Project resources"
        && registry.descriptor(fixture_store)?.provenance().label()
            == Some("resource-space-consumer-v1");
    assert!(original_descriptor_preserved);
    let case_policy_mismatch = registry
        .create_or_open(
            ResourceStoreDescriptor::new(fixture_store, "Project resources"),
            AddressCasePolicy::Insensitive,
        )
        .expect_err("opening an existing store with a different case policy must fail");
    assert!(matches!(
        case_policy_mismatch,
        ResourceStoreRegistryError::StorePolicyMismatch { store, .. } if store == fixture_store
    ));

    let fixture_root = ResourceRootId::from_u128(10);
    let fixture_root_folder = FolderId::from_u128(100);
    let images = FolderId::from_u128(101);
    let drafts = FolderId::from_u128(102);
    let models = FolderId::from_u128(103);
    let common = FolderId::from_u128(104);
    let document_assets = FolderId::from_u128(105);
    let document_drafts = FolderId::from_u128(106);
    let shared_bytes: [u8; 4] = [137, 80, 78, 71];
    {
        let space = registry.space_mut(fixture_store)?;
        space.enable_mutation_observations(16)?;
        space.create_root(
            ResourceRootDescriptor::new(fixture_root, "Corpus assets"),
            fixture_root_folder,
            ResourceMetadata::default(),
        )?;
        space.create_folder(
            images,
            fixture_root_folder,
            name("images")?,
            ResourceMetadata::default(),
        )?;
        space.create_folder(drafts, images, name("drafts")?, ResourceMetadata::default())?;
        space.create_folder(
            models,
            fixture_root_folder,
            name("models")?,
            ResourceMetadata::default(),
        )?;
        space.create_folder(
            common,
            fixture_root_folder,
            name("common")?,
            ResourceMetadata::default(),
        )?;
        space.create_folder(
            document_assets,
            fixture_root_folder,
            name("assets")?,
            ResourceMetadata::default(),
        )?;
        // This is intentionally empty: navigation does not infer folders from bytes.
        space.create_folder(
            document_drafts,
            fixture_root_folder,
            name("document-drafts")?,
            ResourceMetadata::default(),
        )?;
        space.insert_resource(
            fixture_root_folder,
            name("data.xml")?,
            b"<data/>".as_slice(),
            ResourceMetadata {
                media_type: Some("application/xml".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            fixture_root_folder,
            name("transform.xsl")?,
            b"<xsl:stylesheet/>".as_slice(),
            ResourceMetadata {
                media_type: Some("application/xslt+xml".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            common,
            name("utilities.xsl")?,
            b"<xsl:stylesheet/>".as_slice(),
            ResourceMetadata {
                media_type: Some("application/xslt+xml".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            document_assets,
            name("logo.png")?,
            shared_bytes,
            ResourceMetadata {
                media_type: Some("image/png".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            images,
            name("reference.png")?,
            shared_bytes,
            ResourceMetadata {
                media_type: Some("image/png".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            drafts,
            name("reference-dark.png")?,
            shared_bytes,
            ResourceMetadata {
                visibility: ResourceVisibility::Hidden,
                media_type: Some("image/png".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            models,
            name("triangle.gltf")?,
            TRIANGLE_GLTF,
            ResourceMetadata {
                media_type: Some("model/gltf+json".to_owned()),
                ..Default::default()
            },
        )?;
        store_json_resource(
            space,
            models,
            name("project.json")?,
            &ProjectManifest {
                title: "Resource-space corpus".to_owned(),
                primary_scene: "triangle.gltf".to_owned(),
                revision: 1,
            },
            ResourceMetadata::default(),
        )?;
        space.insert_resource(
            models,
            name("scene.svg")?,
            SVG_REFERENCE_DOCUMENT,
            ResourceMetadata {
                media_type: Some("image/svg+xml".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            models,
            name("symbols.svg")?,
            b"<svg viewBox=\"0 0 1 1\"><path id=\"notice\" d=\"M0 0h1v1H0z\"/></svg>".as_slice(),
            ResourceMetadata {
                media_type: Some("image/svg+xml".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            models,
            name("triangle.bin")?,
            triangle_buffer(),
            ResourceMetadata {
                media_type: Some("application/octet-stream".to_owned()),
                ..Default::default()
            },
        )?;
        space.insert_resource(
            models,
            name("swatch.png")?,
            shared_bytes,
            ResourceMetadata {
                media_type: Some("image/png".to_owned()),
                ..Default::default()
            },
        )?;
    }

    let imported_root = ResourceRootId::from_u128(20);
    let imported_root_folder = FolderId::from_u128(200);
    {
        let space = registry.space_mut(imported_store)?;
        space.create_root(
            ResourceRootDescriptor::new(imported_root, "Incoming resources"),
            imported_root_folder,
            ResourceMetadata::default(),
        )?;
        space.insert_resource(
            imported_root_folder,
            name("reference.png")?,
            shared_bytes,
            ResourceMetadata {
                media_type: Some("image/png".to_owned()),
                ..Default::default()
            },
        )?;
    }

    let fixture = registry.space(fixture_store)?;
    let document_root_children =
        fixture.list_children(fixture_root_folder, VisibilityQuery::VisibleOnly)?;
    let document_root_resources =
        fixture.list_resources(fixture_root_folder, VisibilityQuery::VisibleOnly)?;
    let visible = fixture.list_resources(images, VisibilityQuery::VisibleOnly)?;
    let hidden = fixture.search_resources(
        fixture_root_folder,
        &ResourceSearchQuery::new(4)
            .visibility(VisibilityQuery::HiddenOnly)
            .with_name_suffix(".png"),
    )?;
    let visible_entry = visible.first().ok_or("fixture entry missing")?;
    let hidden_entry = hidden.first().ok_or("hidden entry missing")?;
    let imported = registry
        .space(imported_store)?
        .resource(imported_root_folder, &name("reference.png")?)?
        .ok_or("imported entry missing")?;
    let triangle_document = fixture
        .resource(models, &name("triangle.gltf")?)?
        .ok_or("triangle document missing")?;
    let svg_document = fixture
        .resource(models, &name("scene.svg")?)?
        .ok_or("SVG document missing")?;
    let project_document = fixture
        .resource(models, &name("project.json")?)?
        .ok_or("project manifest missing")?;

    assert_eq!(visible.len(), 1);
    assert_eq!(hidden.len(), 1);
    assert_ne!(visible_entry.key(), imported.key());
    assert!(visible_entry.has_same_content_as(&imported));
    assert!(document_drafts != images);
    assert_eq!(
        document_root_children
            .iter()
            .map(|entry| entry.name().ok_or("unnamed folder"))
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .map(|entry| entry.as_str())
            .collect::<Vec<_>>(),
        ["assets", "common", "document-drafts", "images", "models"]
    );
    assert_eq!(
        document_root_resources
            .iter()
            .map(|entry| entry.name().as_str())
            .collect::<Vec<_>>(),
        ["data.xml", "transform.xsl"]
    );
    assert_eq!(fixture.summary().resources(), 12);
    assert_eq!(
        fixture
            .resource(common, &name("utilities.xsl")?)?
            .ok_or("document utility missing")?
            .bytes()
            .as_ref(),
        b"<xsl:stylesheet/>"
    );
    let mutation_observations = fixture.mutation_observations().collect::<Vec<_>>();
    assert_eq!(fixture.mutation_observation_capacity(), Some(16));
    assert_eq!(mutation_observations.len(), 16);
    assert_eq!(mutation_observations[0].sequence(), 4);
    assert_eq!(mutation_observations[15].sequence(), 19);
    assert!(!registry
        .space(imported_store)?
        .mutation_observations_enabled());

    let mut assets = AssetStore::default();
    let loaded = load_resource_asset(&mut assets, visible_entry, &ByteCountLoader)?;
    assert_eq!(*loaded.value(), shared_bytes.len());
    assert_eq!(assets.inventory().entries.len(), 1);
    let triangle = decode_gltf_from_resource_space(fixture, models, &triangle_document)?;
    assert_eq!(triangle.primitives.len(), 1);
    assert_eq!(triangle.primitives[0].indices, vec![0, 1, 2]);
    let gltf_images =
        resolve_gltf_external_images_from_resource_space(fixture, models, &triangle_document)?;
    assert_eq!(gltf_images.len(), 1);
    assert_eq!(gltf_images[0].source().name().as_str(), "swatch.png");
    let svg_references =
        resolve_xml_external_references_from_resource_space(fixture, models, &svg_document)?;
    assert_eq!(svg_references.len(), 1);
    assert_eq!(svg_references[0].source().name().as_str(), "symbols.svg");
    let project_manifest: ProjectManifest = read_json_resource(&project_document)?;
    assert_eq!(project_manifest.primary_scene, "triangle.gltf");

    for store in [fixture_store, imported_store] {
        let descriptor = registry.descriptor(store)?;
        println!(
            "store={:?} name={:?} origin={:?} label={:?}",
            descriptor.id(),
            descriptor.display_name(),
            descriptor.provenance().origin(),
            descriptor.provenance().label()
        );
    }
    println!(
        "fixture visible={} hidden={} resources={} retained_bytes={}",
        visible.len(),
        hidden.len(),
        fixture.summary().resources(),
        fixture.summary().retained_bytes()
    );
    println!(
        "identity=separate content=fingerprint-and-byte-equal visible_key={} hidden_key={}",
        visible_entry.key().address(),
        hidden_entry.key().address()
    );
    println!(
        "mutation-observation=bounded count={} first_sequence={} last_sequence={}",
        mutation_observations.len(),
        mutation_observations[0].sequence(),
        mutation_observations[15].sequence()
    );
    println!(
        "document-bundle=root_documents={} folders={} utility={} empty_folder=document-drafts",
        document_root_resources.len(),
        document_root_children.len(),
        fixture
            .resource(common, &name("utilities.xsl")?)?
            .ok_or("document utility missing")?
            .key()
            .address(),
    );
    println!(
        "asset-bridge=prepared handle={} source={} decoded_bytes={}",
        loaded.handle().id().0,
        loaded.source().address(),
        loaded.value()
    );
    println!(
        "gltf-bridge=decoded source={} primitives={} indices={}",
        triangle_document.key().address(),
        triangle.primitives.len(),
        triangle.primitives[0].indices.len()
    );
    println!(
        "gltf-image-bridge=resolved image_index={} source={} bytes={}",
        gltf_images[0].image_index(),
        gltf_images[0].source().key().address(),
        gltf_images[0].source().byte_len()
    );
    println!(
        "xml-bridge=resolved reference={} fragment={:?} source={}",
        svg_references[0].reference(),
        svg_references[0].fragment(),
        svg_references[0].source().key().address()
    );
    println!(
        "json-bridge=decoded title={:?} primary_scene={} revision={}",
        project_manifest.title, project_manifest.primary_scene, project_manifest.revision
    );
    let fixture_descriptor = registry.descriptor(fixture_store)?;
    let imported_descriptor = registry.descriptor(imported_store)?;
    let conformance_path = output_path();
    let report = ResourceSpaceConformanceReport::new(FixtureObservation {
        fixture_store: StoreObservation {
            id: fixture_descriptor.id().as_u128(),
            display_name: fixture_descriptor.display_name().to_owned(),
            origin: format!("{:?}", fixture_descriptor.provenance().origin()),
            label: fixture_descriptor
                .provenance()
                .label()
                .map(ToOwned::to_owned),
            roots: fixture.summary().roots(),
            folders: fixture.summary().folders(),
            resources: fixture.summary().resources(),
            retained_bytes: fixture.summary().retained_bytes(),
        },
        imported_store: StoreObservation {
            id: imported_descriptor.id().as_u128(),
            display_name: imported_descriptor.display_name().to_owned(),
            origin: format!("{:?}", imported_descriptor.provenance().origin()),
            label: imported_descriptor
                .provenance()
                .label()
                .map(ToOwned::to_owned),
            roots: registry.space(imported_store)?.summary().roots(),
            folders: registry.space(imported_store)?.summary().folders(),
            resources: registry.space(imported_store)?.summary().resources(),
            retained_bytes: registry.space(imported_store)?.summary().retained_bytes(),
        },
        identity: IdentityObservation {
            equal_display_names: fixture_descriptor.display_name()
                == imported_descriptor.display_name(),
            visible_address: visible_entry.key().address().to_string(),
            imported_address: imported.key().address().to_string(),
            visible_key: QualifiedKeyObservation {
                store: visible_entry.key().store().as_u128(),
                root: visible_entry.key().root().as_u128(),
                address: visible_entry.key().address().to_string(),
            },
            imported_key: QualifiedKeyObservation {
                store: imported.key().store().as_u128(),
                root: imported.key().root().as_u128(),
                address: imported.key().address().to_string(),
            },
            content_equal: visible_entry.has_same_content_as(&imported),
            hidden_address: hidden_entry.key().address().to_string(),
            hidden_is_enumerated_as_visible: visible
                .iter()
                .any(|entry| entry.key() == hidden_entry.key()),
        },
        registry_identity: RegistryIdentityObservation {
            same_id_reopened_existing: matches!(
                same_id_reopen,
                StoreOpenOutcome::OpenedExisting { store } if store == fixture_store
            ),
            existing_descriptor_preserved: original_descriptor_preserved,
            case_policy_mismatch_rejected: matches!(
                case_policy_mismatch,
                ResourceStoreRegistryError::StorePolicyMismatch { store, .. } if store == fixture_store
            ),
        },
        navigation: NavigationObservation {
            root_documents: document_root_resources
                .iter()
                .map(|entry| entry.name().as_str().to_owned())
                .collect(),
            root_folders: document_root_children
                .iter()
                .map(|entry| {
                    entry
                        .name()
                        .expect("ordinary fixture folders have names")
                        .as_str()
                        .to_owned()
                })
                .collect(),
            retained_empty_folder: "document-drafts".to_owned(),
            utility_address: fixture
                .resource(common, &name("utilities.xsl")?)?
                .ok_or("document utility missing")?
                .key()
                .address()
                .to_string(),
        },
        mutation_window: MutationWindowObservation {
            enabled_for_fixture_store: fixture.mutation_observations_enabled(),
            enabled_for_imported_store: registry
                .space(imported_store)?
                .mutation_observations_enabled(),
            capacity: fixture
                .mutation_observation_capacity()
                .expect("fixture mutation observations are enabled"),
            retained_count: mutation_observations.len(),
            first_sequence: mutation_observations[0].sequence(),
            last_sequence: mutation_observations[mutation_observations.len() - 1].sequence(),
        },
        bridges: BridgeObservation {
            prepared_asset_bytes: *loaded.value(),
            gltf_primitives: triangle.primitives.len(),
            gltf_indices: triangle.primitives[0].indices.len(),
            gltf_image_address: gltf_images[0].source().key().address().to_string(),
            xml_reference_address: svg_references[0].source().key().address().to_string(),
            json_primary_scene: project_manifest.primary_scene.clone(),
        },
    });
    write_report(&conformance_path, &report)?;
    println!(
        "provider-conformance-artifact={}",
        conformance_path.display()
    );
    let benchmark = run_resource_space_benchmark()?;
    println!(
        "resource-space-workload=entries={} repeated_reads={} copies={} listing_us={} reads_us={} copies_us={} shared_bytes={} retained_bytes={}",
        benchmark.entries(),
        benchmark.repeated_reads(),
        benchmark.copies(),
        benchmark.listing_elapsed().as_micros(),
        benchmark.read_elapsed().as_micros(),
        benchmark.copy_elapsed().as_micros(),
        benchmark.copied_entry_shares_bytes(),
        benchmark.retained_bytes(),
    );
    Ok(())
}
