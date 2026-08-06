use archive_provider::{
    ArchiveCompression, ArchiveFormat, ArchiveProvider, ArchiveReadLimits, ArchiveWriteEntry,
    ArchiveWriteLimits, ArchiveWriter, SevenZipArchiveProvider, TarArchiveProvider,
    ZipArchiveProvider,
};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};

use super::*;

const ZIP_FIXTURE_HEX: &str = "504b0304140000000000c46c045d00000000000000000000000005000000646f63732f504b0304140000000000c46c045dc15f4bbf17000000170000000f000000646f63732f726561646d652e747874546f6b696d7520617263686976652065766964656e6365504b0304140000000000c46c045d58f8357b050000000500000008000000646174612e62696e00010203ff504b01021400140000000000c46c045d000000000000000000000000050000000000000000000000000000000000646f63732f504b01021400140000000000c46c045dc15f4bbf17000000170000000f0000000000000000000000000023000000646f63732f726561646d652e747874504b01021400140000000000c46c045d58f8357b0500000005000000080000000000000000000000000067000000646174612e62696e504b05060000000003000300a6000000920000000000";

fn fixture_bytes() -> Vec<u8> {
    ZIP_FIXTURE_HEX
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn name(value: &str) -> ResourceName {
    ResourceName::parse(value, AddressCasePolicy::Sensitive).expect("valid fixture name")
}

fn fixture_space() -> (InMemoryResourceSpace, FolderId) {
    let mut space = InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(2);
    space
        .create_root(
            ResourceRootDescriptor::new(ResourceRootId::from_u128(3), "fixtures"),
            folder,
            ResourceMetadata::default(),
        )
        .unwrap();
    space
        .insert_resource(
            folder,
            name("fixture.zip"),
            fixture_bytes(),
            ResourceMetadata::default(),
        )
        .unwrap();
    (space, folder)
}

fn limits() -> ArchiveReadLimits {
    ArchiveReadLimits::new(4096, 16, 1024, 4096, 128)
}

fn tar_fixture_bytes() -> Vec<u8> {
    TarArchiveProvider
        .write_archive(
            ArchiveFormat::Tar,
            &[
                ArchiveWriteEntry::directory("docs/"),
                ArchiveWriteEntry::file(
                    "docs/readme.txt",
                    b"Tokimu archive evidence".as_slice(),
                    ArchiveCompression::Stored,
                ),
                ArchiveWriteEntry::file("data.bin", [0, 1, 2, 3, 255], ArchiveCompression::Stored),
            ],
            ArchiveWriteLimits::new(4096, 16, 1024, 4096, 128),
        )
        .expect("TAR fixture should write")
        .bytes
}

#[test]
fn inspection_preserves_source_identity_without_mutation() {
    let (space, folder) = fixture_space();
    let source = space
        .resource(folder, &name("fixture.zip"))
        .unwrap()
        .unwrap();
    let inspection = inspect_archive_resource(
        &space,
        InspectArchiveResourceRequest {
            source_folder: folder,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            limits: limits(),
        },
        &ZipArchiveProvider,
    )
    .unwrap();

    assert_eq!(inspection.source(), source.key());
    assert_eq!(
        inspection.source_fingerprint(),
        &source.content_fingerprint()
    );
    assert_eq!(inspection.manifest().entries.len(), 3);
    assert_eq!(space.summary().resources(), 1);
}

#[test]
fn derived_view_is_read_only_and_preserves_source_provenance() {
    let (space, folder) = fixture_space();
    let source = space
        .resource(folder, &name("fixture.zip"))
        .unwrap()
        .unwrap();
    let before = space.summary();

    let view = open_archive_derived_view(
        &space,
        InspectArchiveResourceRequest {
            source_folder: folder,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            limits: limits(),
        },
        &ZipArchiveProvider,
    )
    .expect("fixture should open as a derived archive view");

    assert!(view.is_read_only());
    assert_eq!(view.source(), source.key());
    assert_eq!(view.source_fingerprint(), &source.content_fingerprint());
    assert_eq!(view.format(), ArchiveFormat::Zip);
    assert_eq!(view.entries().len(), 3);
    assert_eq!(view.entries()[1].normalized_name(), "docs/readme.txt");
    let read = read_archive_derived_entry(
        &space,
        &view,
        "docs/readme.txt",
        limits(),
        &ZipArchiveProvider,
    )
    .expect("derived view should read an admitted entry without materializing it");
    assert_eq!(read.bytes, b"Tokimu archive evidence");
    assert_eq!(space.summary(), before);
}

#[test]
fn reopening_a_derived_view_after_source_replacement_changes_its_fingerprint() {
    let (mut space, folder) = fixture_space();
    let request = InspectArchiveResourceRequest {
        source_folder: folder,
        source_name: name("fixture.zip"),
        format: ArchiveFormat::Zip,
        limits: limits(),
    };
    let first = open_archive_derived_view(&space, request.clone(), &ZipArchiveProvider)
        .expect("initial view should open");

    let replacement = ZipArchiveProvider
        .write_archive(
            ArchiveFormat::Zip,
            &[ArchiveWriteEntry::file(
                "changed.txt",
                b"replacement".as_slice(),
                ArchiveCompression::Stored,
            )],
            ArchiveWriteLimits::new(4096, 16, 1024, 4096, 128),
        )
        .expect("replacement archive should write");
    space
        .replace_resource(
            folder,
            &name("fixture.zip"),
            replacement.bytes,
            ResourceMetadata::default(),
        )
        .expect("source archive should replace");

    let second = open_archive_derived_view(&space, request, &ZipArchiveProvider)
        .expect("replacement view should open");

    assert_ne!(first.source_fingerprint(), second.source_fingerprint());
    assert_eq!(second.entries().len(), 1);
    assert_eq!(second.entries()[0].normalized_name(), "changed.txt");
    assert!(matches!(
        read_archive_derived_entry(
            &space,
            &first,
            "docs/readme.txt",
            limits(),
            &ZipArchiveProvider,
        ),
        Err(ResourceArchiveBridgeError::DerivedViewStale)
    ));
}

#[test]
fn derived_view_and_eager_import_keep_distinct_resource_space_semantics() {
    let (mut space, root) = fixture_space();
    let view = open_archive_derived_view(
        &space,
        InspectArchiveResourceRequest {
            source_folder: root,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            limits: limits(),
        },
        &ZipArchiveProvider,
    )
    .expect("fixture should open as a derived view");

    let summary_before_import = space.summary();
    let import = import_archive_subtree(
        &mut space,
        ImportArchiveSubtreeRequest {
            source_folder: root,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            limits: limits(),
            destination_parent: root,
            destination_root_name: name("materialized"),
            first_folder_id: FolderId::from_u128(10),
            metadata: ResourceMetadata::default(),
        },
        &ZipArchiveProvider,
    )
    .expect("fixture should materialize only through an explicit import");

    assert!(view.is_read_only());
    assert_eq!(view.entries().len(), 3);
    assert_eq!(summary_before_import.resources(), 1);
    assert_eq!(import.resources(), 2);
    assert_eq!(space.summary().resources(), 3);
}

#[test]
fn selected_entry_copies_to_explicit_destination() {
    let (mut space, folder) = fixture_space();
    let result = copy_archive_entry(
        &mut space,
        CopyArchiveEntryRequest {
            source_folder: folder,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            entry_name: "docs/readme.txt".to_owned(),
            limits: limits(),
            destination_folder: folder,
            destination_name: name("copied.txt"),
            collision: ArchiveResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata::default(),
        },
        &ZipArchiveProvider,
    )
    .unwrap();

    assert_eq!(result.entry().bytes().as_ref(), b"Tokimu archive evidence");
    assert_eq!(
        result.observation().entry().normalized_name,
        "docs/readme.txt"
    );
    assert_eq!(
        result.observation().mutation(),
        ArchiveResourceMutation::Inserted
    );
}

#[test]
fn failed_entry_read_does_not_create_destination() {
    let (mut space, folder) = fixture_space();
    let destination = name("missing.txt");
    let error = copy_archive_entry(
        &mut space,
        CopyArchiveEntryRequest {
            source_folder: folder,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            entry_name: "not-present.txt".to_owned(),
            limits: limits(),
            destination_folder: folder,
            destination_name: destination.clone(),
            collision: ArchiveResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata::default(),
        },
        &ZipArchiveProvider,
    )
    .expect_err("missing entry must fail");

    assert!(matches!(error, ResourceArchiveBridgeError::Archive(_)));
    assert!(space.resource(folder, &destination).unwrap().is_none());
}

#[test]
fn rejected_collision_preserves_destination_bytes() {
    let (mut space, folder) = fixture_space();
    let destination = name("existing.txt");
    space
        .insert_resource(
            folder,
            destination.clone(),
            b"existing".as_slice(),
            ResourceMetadata::default(),
        )
        .unwrap();

    let error = copy_archive_entry(
        &mut space,
        CopyArchiveEntryRequest {
            source_folder: folder,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            entry_name: "docs/readme.txt".to_owned(),
            limits: limits(),
            destination_folder: folder,
            destination_name: destination.clone(),
            collision: ArchiveResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata::default(),
        },
        &ZipArchiveProvider,
    )
    .expect_err("collision must fail");

    assert!(matches!(
        error,
        ResourceArchiveBridgeError::DestinationExists { .. }
    ));
    assert_eq!(
        space
            .resource(folder, &destination)
            .unwrap()
            .unwrap()
            .bytes()
            .as_ref(),
        b"existing"
    );
}

#[test]
fn explicit_replace_is_observed() {
    let (mut space, folder) = fixture_space();
    let destination = name("existing.txt");
    space
        .insert_resource(
            folder,
            destination.clone(),
            b"existing".as_slice(),
            ResourceMetadata::default(),
        )
        .unwrap();

    let result = copy_archive_entry(
        &mut space,
        CopyArchiveEntryRequest {
            source_folder: folder,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            entry_name: "docs/readme.txt".to_owned(),
            limits: limits(),
            destination_folder: folder,
            destination_name: destination,
            collision: ArchiveResourceCollisionPolicy::Replace,
            metadata: ResourceMetadata::default(),
        },
        &ZipArchiveProvider,
    )
    .unwrap();

    assert_eq!(
        result.observation().mutation(),
        ArchiveResourceMutation::Replaced
    );
    assert_eq!(result.entry().bytes().as_ref(), b"Tokimu archive evidence");
}

#[test]
fn subtree_export_preserves_deterministic_logical_hierarchy() {
    let (mut space, root) = fixture_space();
    let docs = FolderId::from_u128(4);
    space
        .create_folder(docs, root, name("docs"), ResourceMetadata::default())
        .unwrap();
    space
        .insert_resource(
            root,
            name("top.txt"),
            b"top".as_slice(),
            ResourceMetadata::default(),
        )
        .unwrap();
    space
        .insert_resource(
            docs,
            name("readme.txt"),
            b"nested".as_slice(),
            ResourceMetadata::default(),
        )
        .unwrap();

    let request = ExportResourceSubtreeRequest {
        source_folder: root,
        format: ArchiveFormat::Zip,
        limits: ArchiveWriteLimits::new(4096, 16, 1024, 4096, 128),
        file_compression: ArchiveCompression::Stored,
        visibility: resource_space::VisibilityQuery::All,
    };
    let first = export_resource_subtree(&space, request, &ZipArchiveProvider).unwrap();
    let second = export_resource_subtree(&space, request, &ZipArchiveProvider).unwrap();

    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.folders(), 1);
    assert_eq!(first.resources(), 3);
    assert!(first.archive().deterministic_metadata);

    let manifest = ZipArchiveProvider
        .inspect(ArchiveFormat::Zip, first.bytes(), limits())
        .unwrap();
    assert_eq!(
        manifest
            .entries
            .iter()
            .map(|entry| entry.normalized_name.as_str())
            .collect::<Vec<_>>(),
        vec!["fixture.zip", "top.txt", "docs/", "docs/readme.txt"]
    );
}

#[test]
fn subtree_export_uses_the_create_only_seven_zip_compatibility_writer() {
    let (mut space, root) = fixture_space();
    let docs = FolderId::from_u128(4);
    space
        .create_folder(docs, root, name("docs"), ResourceMetadata::default())
        .unwrap();
    space
        .insert_resource(
            docs,
            name("readme.txt"),
            b"nested".as_slice(),
            ResourceMetadata::default(),
        )
        .unwrap();

    let request = ExportResourceSubtreeRequest {
        source_folder: root,
        format: ArchiveFormat::SevenZip,
        limits: ArchiveWriteLimits::new(4096, 16, 1024, 4096, 128),
        // 7z selects compression itself. The bridge forwards the portable
        // request without reinterpreting it as ZIP-style compression.
        file_compression: ArchiveCompression::Other,
        visibility: resource_space::VisibilityQuery::All,
    };
    let first = export_resource_subtree(&space, request, &SevenZipArchiveProvider).unwrap();
    let second = export_resource_subtree(&space, request, &SevenZipArchiveProvider).unwrap();

    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.folders(), 1);
    assert_eq!(first.resources(), 2);
    let manifest = SevenZipArchiveProvider
        .inspect(ArchiveFormat::SevenZip, first.bytes(), limits())
        .unwrap();
    assert_eq!(
        manifest
            .entries
            .iter()
            .map(|entry| entry.normalized_name.trim_end_matches('/'))
            .collect::<Vec<_>>(),
        vec!["fixture.zip", "docs", "docs/readme.txt"]
    );

    let error = export_resource_subtree(
        &space,
        ExportResourceSubtreeRequest {
            file_compression: ArchiveCompression::Deflate,
            ..request
        },
        &SevenZipArchiveProvider,
    )
    .expect_err("7z must reject a ZIP-specific compression request");
    assert!(matches!(
        error,
        ResourceArchiveBridgeError::Archive(
            archive_provider::ArchiveError::UnsupportedWriteCompression {
                compression: ArchiveCompression::Deflate,
                ..
            }
        )
    ));
}

#[test]
fn subtree_export_rejects_logical_names_that_would_change_archive_hierarchy() {
    let (mut space, root) = fixture_space();
    space
        .insert_resource(
            root,
            name("not/a-single-component"),
            b"unsafe".as_slice(),
            ResourceMetadata::default(),
        )
        .unwrap();

    let error = export_resource_subtree(
        &space,
        ExportResourceSubtreeRequest {
            source_folder: root,
            format: ArchiveFormat::Zip,
            limits: ArchiveWriteLimits::new(4096, 16, 1024, 4096, 128),
            file_compression: ArchiveCompression::Stored,
            visibility: resource_space::VisibilityQuery::All,
        },
        &ZipArchiveProvider,
    )
    .expect_err("logical separators must not become archive hierarchy");

    assert!(matches!(
        error,
        ResourceArchiveBridgeError::UnsafeArchivePathComponent {
            kind: "resource",
            ..
        }
    ));
}

#[test]
fn archive_subtree_import_materializes_explicit_nested_folders_and_files() {
    let (mut space, root) = fixture_space();
    let observation = import_archive_subtree(
        &mut space,
        ImportArchiveSubtreeRequest {
            source_folder: root,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            limits: limits(),
            destination_parent: root,
            destination_root_name: name("imported"),
            first_folder_id: FolderId::from_u128(10),
            metadata: ResourceMetadata::default(),
        },
        &ZipArchiveProvider,
    )
    .expect("fixture archive should materialize");

    assert_eq!(observation.destination_root(), FolderId::from_u128(10));
    assert_eq!(observation.folders(), 2);
    assert_eq!(observation.resources(), 2);
    assert_eq!(observation.retained_bytes(), 28);

    let imported_root = space.folder(FolderId::from_u128(10)).unwrap();
    assert_eq!(imported_root.name().unwrap().as_str(), "imported");
    let docs = space
        .list_folders(
            FolderId::from_u128(10),
            resource_space::VisibilityQuery::All,
        )
        .unwrap()
        .into_iter()
        .next()
        .expect("docs folder was materialized");
    assert_eq!(docs.name().unwrap().as_str(), "docs");
    assert_eq!(
        space
            .resource(docs.id(), &name("readme.txt"))
            .unwrap()
            .unwrap()
            .bytes()
            .as_ref(),
        b"Tokimu archive evidence"
    );
    assert_eq!(
        space
            .resource(FolderId::from_u128(10), &name("data.bin"))
            .unwrap()
            .unwrap()
            .bytes()
            .as_ref(),
        &[0, 1, 2, 3, 255]
    );
}

#[test]
fn archive_subtree_import_accepts_the_same_logical_tree_from_tar() {
    let (mut space, root) = fixture_space();
    space
        .insert_resource(
            root,
            name("fixture.tar"),
            tar_fixture_bytes(),
            ResourceMetadata::default(),
        )
        .expect("TAR fixture resource should be retained");

    let observation = import_archive_subtree(
        &mut space,
        ImportArchiveSubtreeRequest {
            source_folder: root,
            source_name: name("fixture.tar"),
            format: ArchiveFormat::Tar,
            limits: limits(),
            destination_parent: root,
            destination_root_name: name("tar-imported"),
            first_folder_id: FolderId::from_u128(20),
            metadata: ResourceMetadata::default(),
        },
        &TarArchiveProvider,
    )
    .expect("TAR fixture archive should materialize");

    assert_eq!(observation.destination_root(), FolderId::from_u128(20));
    assert_eq!(observation.folders(), 2);
    assert_eq!(observation.resources(), 2);
    assert_eq!(observation.retained_bytes(), 28);

    let docs = space
        .list_folders(
            FolderId::from_u128(20),
            resource_space::VisibilityQuery::All,
        )
        .unwrap()
        .into_iter()
        .next()
        .expect("TAR docs folder was materialized");
    assert_eq!(docs.name().unwrap().as_str(), "docs");
    assert_eq!(
        space
            .resource(docs.id(), &name("readme.txt"))
            .unwrap()
            .unwrap()
            .bytes()
            .as_ref(),
        b"Tokimu archive evidence"
    );
    assert_eq!(
        space
            .resource(FolderId::from_u128(20), &name("data.bin"))
            .unwrap()
            .unwrap()
            .bytes()
            .as_ref(),
        &[0, 1, 2, 3, 255]
    );
}

#[test]
fn archive_subtree_import_rejects_destination_root_collision_before_mutation() {
    let (mut space, root) = fixture_space();
    space
        .create_folder(
            FolderId::from_u128(10),
            root,
            name("imported"),
            ResourceMetadata::default(),
        )
        .unwrap();
    let before = space.summary();

    let error = import_archive_subtree(
        &mut space,
        ImportArchiveSubtreeRequest {
            source_folder: root,
            source_name: name("fixture.zip"),
            format: ArchiveFormat::Zip,
            limits: limits(),
            destination_parent: root,
            destination_root_name: name("imported"),
            first_folder_id: FolderId::from_u128(20),
            metadata: ResourceMetadata::default(),
        },
        &ZipArchiveProvider,
    )
    .expect_err("existing destination folder must reject import");

    assert!(matches!(
        error,
        ResourceArchiveBridgeError::DestinationFolderExists { .. }
    ));
    assert_eq!(space.summary(), before);
}
