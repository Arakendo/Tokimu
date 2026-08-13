mod fixture;

use std::{fs, path::PathBuf};

use archive_provider::{
    ArchiveCompression, ArchiveEntryObservation, ArchiveError, ArchiveFormat, ArchiveProvider,
    ArchiveReadLimits, ArchiveWriteEntry, ArchiveWriteLimits, ArchiveWriter,
    SevenZipArchiveProvider, TarArchiveProvider, ZipArchiveProvider,
};
use compression_provider::{
    CompressionCodec, CompressionGoal, CompressionProvider, DecodeLimits, DecodeRequest,
    EncodeRequest, FlateCompressionProvider,
};
use fixture::zip_fixture;
use resource_space::{
    AddressCasePolicy, ContentFingerprint, FolderId, InMemoryResourceSpace, ResourceMetadata,
    ResourceName, ResourceRootDescriptor, ResourceRootId, StoreId, VisibilityQuery,
};
use resource_space_archive::{
    copy_archive_entry, export_resource_subtree, inspect_archive_resource,
    ArchiveResourceCollisionPolicy, CopyArchiveEntryRequest, ExportResourceSubtreeRequest,
    InspectArchiveResourceRequest,
};
use serde::Serialize;

const EXPECTED_TEXT: &[u8] = b"Tokimu archive evidence";

#[derive(Serialize)]
struct Report {
    schema: u32,
    artifact: ArtifactProvenance,
    claim: &'static str,
    archive_bytes: u64,
    total_uncompressed_bytes: u64,
    entries: Vec<ArchiveEntryObservation>,
    selected_entry: SelectedEntryObservation,
    bounded_inspection: BoundedInspectionObservation,
    resource_space: ResourceSpaceObservation,
    resource_subtree_export: ResourceSubtreeExportObservation,
    deterministic_write: DeterministicWriteObservation,
    tar_gzip_composition: TarGzipCompositionObservation,
    zip_tar_conformance: ZipTarConformanceObservation,
    seven_zip_compatibility: SevenZipCompatibilityObservation,
}

#[derive(Serialize)]
struct ArtifactProvenance {
    generator: &'static str,
    selection: &'static str,
    fixture_fingerprint: String,
    zip_provider: &'static str,
    tar_provider: &'static str,
    seven_zip_provider: &'static str,
    gzip_provider: &'static str,
    read_limits: ArchiveReadLimits,
    write_limits: ArchiveWriteLimits,
}

#[derive(Serialize)]
struct SelectedEntryObservation {
    normalized_name: String,
    bytes: u64,
    byte_identical: bool,
}

#[derive(Serialize)]
struct BoundedInspectionObservation {
    category: &'static str,
    rejected: bool,
}

#[derive(Serialize)]
struct ResourceSpaceObservation {
    source_retained: bool,
    copied_entry_retained: bool,
    copied_bytes_identical: bool,
    resources: usize,
}

#[derive(Serialize)]
struct ResourceSubtreeExportObservation {
    folders: u32,
    resources: u32,
    archive_bytes: u64,
    deterministic_metadata: bool,
    manifest_entries: usize,
}

#[derive(Serialize)]
struct DeterministicWriteObservation {
    archive_bytes: u64,
    entries: u32,
    byte_identical_rebuild: bool,
    read_after_write_identical: bool,
    deterministic_metadata: bool,
}

#[derive(Serialize)]
struct TarGzipCompositionObservation {
    tar_bytes: u64,
    gzip_bytes: u64,
    decoded_tar_bytes: u64,
    entry_count: usize,
    selected_bytes_identical: bool,
}

#[derive(Serialize)]
struct ZipTarConformanceObservation {
    logical_manifest_identical: bool,
    readme_bytes_identical: bool,
    data_bytes_identical: bool,
}

#[derive(Serialize)]
struct SevenZipCompatibilityObservation {
    archive_bytes: u64,
    entries: usize,
    deterministic_rebuild: bool,
    logical_manifest_identical: bool,
    readme_bytes_identical: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ZipArchiveProvider;
    let fixture = zip_fixture();
    let limits = ArchiveReadLimits::new(4096, 16, 1024, 4096, 128);
    let manifest = provider.inspect(ArchiveFormat::Zip, &fixture, limits)?;
    let selected = provider.read_entry(ArchiveFormat::Zip, &fixture, "docs/readme.txt", limits)?;
    let byte_identical = selected.bytes == EXPECTED_TEXT;
    if manifest.entries.len() != 3 || !byte_identical {
        return Err("archive corpus did not preserve its manifest or selected bytes".into());
    }

    let bounded_error = provider
        .inspect(
            ArchiveFormat::Zip,
            &fixture,
            ArchiveReadLimits::new(64, 16, 1024, 4096, 128),
        )
        .expect_err("the archive input budget must reject the fixture");
    let category = match bounded_error {
        ArchiveError::ArchiveLimitExceeded { .. } => "archive-limit-exceeded",
        other => return Err(format!("unexpected bounded inspection failure: {other}").into()),
    };

    let write_entries = [
        ArchiveWriteEntry::directory("docs"),
        ArchiveWriteEntry::file(
            "docs/readme.txt",
            EXPECTED_TEXT.to_vec(),
            ArchiveCompression::Deflate,
        ),
        ArchiveWriteEntry::file(
            "data.bin",
            vec![0, 1, 2, 3, 255],
            ArchiveCompression::Stored,
        ),
    ];
    let write_limits = ArchiveWriteLimits::new(4096, 16, 1024, 4096, 128);
    let written = provider.write_archive(ArchiveFormat::Zip, &write_entries, write_limits)?;
    let rebuilt = provider.write_archive(ArchiveFormat::Zip, &write_entries, write_limits)?;
    let byte_identical_rebuild = written.bytes == rebuilt.bytes;
    let written_read = provider.read_entry(
        ArchiveFormat::Zip,
        &written.bytes,
        "docs/readme.txt",
        limits,
    )?;
    let read_after_write_identical = written_read.bytes == EXPECTED_TEXT;
    if !byte_identical_rebuild || !read_after_write_identical {
        return Err("deterministic ZIP write corpus did not round-trip".into());
    }

    // TAR and GZip deliberately meet only through bytes. Neither provider
    // exposes the other format as a hidden combined container.
    let tar_provider = TarArchiveProvider;
    let tar_entries = [
        ArchiveWriteEntry::directory("docs"),
        ArchiveWriteEntry::file(
            "docs/readme.txt",
            EXPECTED_TEXT.to_vec(),
            ArchiveCompression::Stored,
        ),
        ArchiveWriteEntry::file(
            "data.bin",
            vec![0, 1, 2, 3, 255],
            ArchiveCompression::Stored,
        ),
    ];
    let tar = tar_provider.write_archive(ArchiveFormat::Tar, &tar_entries, write_limits)?;
    let codec = FlateCompressionProvider;
    let gzip = codec.encode(
        EncodeRequest::new(CompressionCodec::Gzip, &tar.bytes).with_goal(CompressionGoal::Balanced),
    )?;
    let decoded = codec.decode(DecodeRequest::new(
        CompressionCodec::Gzip,
        &gzip.bytes,
        // Small deterministic TAR fixtures compress unusually well. Keep the
        // corpus bounded by bytes while allowing this declared 32x expansion.
        DecodeLimits::new(4096, 4096).with_expansion_ratio(32),
    ))?;
    let tar_manifest = tar_provider.inspect(ArchiveFormat::Tar, &decoded.bytes, limits)?;
    let tar_read = tar_provider.read_entry(
        ArchiveFormat::Tar,
        &decoded.bytes,
        "docs/readme.txt",
        limits,
    )?;
    let tar_gzip_selected_bytes_identical = tar_read.bytes == EXPECTED_TEXT;
    if tar_manifest.entries.len() != 3 || !tar_gzip_selected_bytes_identical {
        return Err("TAR plus GZip composition did not preserve logical archive content".into());
    }
    let zip_written_manifest = provider.inspect(ArchiveFormat::Zip, &written.bytes, limits)?;
    let zip_readme = provider.read_entry(
        ArchiveFormat::Zip,
        &written.bytes,
        "docs/readme.txt",
        limits,
    )?;
    let zip_data = provider.read_entry(ArchiveFormat::Zip, &written.bytes, "data.bin", limits)?;
    let tar_data =
        tar_provider.read_entry(ArchiveFormat::Tar, &decoded.bytes, "data.bin", limits)?;
    let logical_manifest_identical = logical_manifest_matches(&zip_written_manifest, &tar_manifest);
    let zip_tar_readme_bytes_identical = zip_readme.bytes == tar_read.bytes;
    let zip_tar_data_bytes_identical = zip_data.bytes == tar_data.bytes;
    if !logical_manifest_identical
        || !zip_tar_readme_bytes_identical
        || !zip_tar_data_bytes_identical
    {
        return Err("ZIP and TAR providers disagreed on admitted logical content".into());
    }

    // 7z is a compatibility provider with a deliberately narrower writer
    // contract. It creates a fresh archive from the full requested entry set;
    // callers cannot append to or edit an existing archive.
    let seven_zip_provider = SevenZipArchiveProvider;
    let seven_zip_entries = [
        ArchiveWriteEntry::directory("docs"),
        ArchiveWriteEntry::file(
            "docs/readme.txt",
            EXPECTED_TEXT.to_vec(),
            ArchiveCompression::Other,
        ),
        ArchiveWriteEntry::file("data.bin", vec![0, 1, 2, 3, 255], ArchiveCompression::Other),
    ];
    let seven_zip = seven_zip_provider.write_archive(
        ArchiveFormat::SevenZip,
        &seven_zip_entries,
        write_limits,
    )?;
    let rebuilt_seven_zip = seven_zip_provider.write_archive(
        ArchiveFormat::SevenZip,
        &seven_zip_entries,
        write_limits,
    )?;
    let seven_zip_manifest =
        seven_zip_provider.inspect(ArchiveFormat::SevenZip, &seven_zip.bytes, limits)?;
    let seven_zip_readme = seven_zip_provider.read_entry(
        ArchiveFormat::SevenZip,
        &seven_zip.bytes,
        "docs/readme.txt",
        limits,
    )?;
    let seven_zip_deterministic_rebuild = seven_zip.bytes == rebuilt_seven_zip.bytes;
    let seven_zip_logical_manifest_identical =
        logical_content_matches(&zip_written_manifest, &seven_zip_manifest);
    let seven_zip_readme_bytes_identical = seven_zip_readme.bytes == EXPECTED_TEXT;
    if !seven_zip_deterministic_rebuild
        || !seven_zip_logical_manifest_identical
        || !seven_zip_readme_bytes_identical
    {
        return Err(format!(
            "7z compatibility writer did not preserve admitted logical content: deterministic_rebuild={seven_zip_deterministic_rebuild}, logical_manifest_identical={seven_zip_logical_manifest_identical}, readme_bytes_identical={seven_zip_readme_bytes_identical}"
        )
        .into());
    }

    let (mut space, folder) = resource_space_fixture(fixture.clone())?;
    let source_name = resource_name("fixture.zip")?;
    let destination_name = resource_name("readme-copy.txt")?;
    let inspection = inspect_archive_resource(
        &space,
        InspectArchiveResourceRequest {
            source_folder: folder,
            source_name: source_name.clone(),
            format: ArchiveFormat::Zip,
            limits,
        },
        &provider,
    )?;
    let copied = copy_archive_entry(
        &mut space,
        CopyArchiveEntryRequest {
            source_folder: folder,
            source_name: source_name.clone(),
            format: ArchiveFormat::Zip,
            entry_name: "docs/readme.txt".to_owned(),
            limits,
            destination_folder: folder,
            destination_name: destination_name.clone(),
            collision: ArchiveResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata::default(),
        },
        &provider,
    )?;
    let source_retained = space.resource(folder, &source_name)?.is_some();
    let copied_entry_retained = space.resource(folder, &destination_name)?.is_some();
    let copied_bytes_identical = copied.entry().bytes().as_ref() == EXPECTED_TEXT;
    if inspection.manifest().entries.len() != 3
        || !source_retained
        || !copied_entry_retained
        || !copied_bytes_identical
    {
        return Err(
            "archive Resource Space composition did not preserve explicit identities".into(),
        );
    }

    // Resource Space owns hierarchy and logical names; the archive provider
    // owns encoding that hierarchy into deterministic archive entries.
    let export_folder = FolderId::from_u128(4);
    let nested_export_folder = FolderId::from_u128(5);
    space.create_folder(
        export_folder,
        folder,
        resource_name("export")?,
        ResourceMetadata::default(),
    )?;
    space.create_folder(
        nested_export_folder,
        export_folder,
        resource_name("docs")?,
        ResourceMetadata::default(),
    )?;
    space.insert_resource(
        export_folder,
        resource_name("top.txt")?,
        b"top-level resource".to_vec(),
        ResourceMetadata::default(),
    )?;
    space.insert_resource(
        nested_export_folder,
        resource_name("notes.txt")?,
        b"nested resource".to_vec(),
        ResourceMetadata::default(),
    )?;
    let subtree_export = export_resource_subtree(
        &space,
        ExportResourceSubtreeRequest {
            source_folder: export_folder,
            format: ArchiveFormat::Zip,
            limits: write_limits,
            file_compression: ArchiveCompression::Stored,
            visibility: VisibilityQuery::All,
        },
        &provider,
    )?;
    let subtree_manifest = provider.inspect(ArchiveFormat::Zip, subtree_export.bytes(), limits)?;
    if subtree_export.folders() != 1
        || subtree_export.resources() != 2
        || subtree_manifest.entries.len() != 3
    {
        return Err("Resource Space subtree export did not preserve logical hierarchy".into());
    }
    let summary = space.summary();

    let report = Report {
        schema: 2,
        artifact: ArtifactProvenance {
            generator: "hello-archive",
            selection: "first-party-archive-fixture-v1",
            fixture_fingerprint: fingerprint(&fixture),
            zip_provider: "zip-8.6.0",
            tar_provider: "tar-0.4.46",
            seven_zip_provider: "sevenz-rust2-0.21.3",
            gzip_provider: "flate2-1.1.9",
            read_limits: limits,
            write_limits,
        },
        claim:
            "bounded provider-neutral archive inspection, writing, TAR-plus-GZip composition, and Resource Space subtree export",
        archive_bytes: manifest.archive_bytes,
        total_uncompressed_bytes: manifest.total_uncompressed_bytes,
        entries: manifest.entries,
        selected_entry: SelectedEntryObservation {
            normalized_name: selected.entry.normalized_name,
            bytes: selected.bytes.len() as u64,
            byte_identical,
        },
        bounded_inspection: BoundedInspectionObservation {
            category,
            rejected: true,
        },
        resource_space: ResourceSpaceObservation {
            source_retained,
            copied_entry_retained,
            copied_bytes_identical,
            resources: summary.resources(),
        },
        resource_subtree_export: ResourceSubtreeExportObservation {
            folders: subtree_export.folders(),
            resources: subtree_export.resources(),
            archive_bytes: subtree_export.archive().archive_bytes,
            deterministic_metadata: subtree_export.archive().deterministic_metadata,
            manifest_entries: subtree_manifest.entries.len(),
        },
        deterministic_write: DeterministicWriteObservation {
            archive_bytes: written.observation.archive_bytes,
            entries: written.observation.entry_count,
            byte_identical_rebuild,
            read_after_write_identical,
            deterministic_metadata: written.observation.deterministic_metadata,
        },
        tar_gzip_composition: TarGzipCompositionObservation {
            tar_bytes: tar.observation.archive_bytes,
            gzip_bytes: gzip.observation.output_bytes,
            decoded_tar_bytes: decoded.observation.output_bytes,
            entry_count: tar_manifest.entries.len(),
            selected_bytes_identical: tar_gzip_selected_bytes_identical,
        },
        zip_tar_conformance: ZipTarConformanceObservation {
            logical_manifest_identical,
            readme_bytes_identical: zip_tar_readme_bytes_identical,
            data_bytes_identical: zip_tar_data_bytes_identical,
        },
        seven_zip_compatibility: SevenZipCompatibilityObservation {
            archive_bytes: seven_zip.observation.archive_bytes,
            entries: seven_zip_manifest.entries.len(),
            deterministic_rebuild: seven_zip_deterministic_rebuild,
            logical_manifest_identical: seven_zip_logical_manifest_identical,
            readme_bytes_identical: seven_zip_readme_bytes_identical,
        },
    };
    let path = write_report(&report)?;
    println!(
        "hello-archive: entries={}, selected={}, resource_copy={}, subtree_export={}, deterministic_write={}, tar_gzip={}, zip_tar_conformance={}, seven_zip_compatibility={}, bounded_inspection={}, artifact={}",
        report.entries.len(),
        report.selected_entry.byte_identical,
        report.resource_space.copied_bytes_identical,
        report.resource_subtree_export.manifest_entries,
        report.deterministic_write.byte_identical_rebuild,
        report.tar_gzip_composition.selected_bytes_identical,
        report.zip_tar_conformance.logical_manifest_identical,
        report.seven_zip_compatibility.logical_manifest_identical,
        report.bounded_inspection.category,
        path.display()
    );
    Ok(())
}

fn resource_space_fixture(
    archive_bytes: Vec<u8>,
) -> Result<(InMemoryResourceSpace, FolderId), Box<dyn std::error::Error>> {
    let mut space = InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(2);
    space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(3), "archive corpus"),
        folder,
        ResourceMetadata::default(),
    )?;
    space.insert_resource(
        folder,
        resource_name("fixture.zip")?,
        archive_bytes,
        ResourceMetadata::default(),
    )?;
    Ok((space, folder))
}

fn resource_name(value: &str) -> Result<ResourceName, Box<dyn std::error::Error>> {
    Ok(ResourceName::parse(value, AddressCasePolicy::Sensitive)?)
}

fn fingerprint(bytes: &[u8]) -> String {
    let fingerprint = ContentFingerprint::blake3(bytes);
    let digest = fingerprint
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{:?}:{digest}", fingerprint.algorithm()).to_ascii_lowercase()
}

fn logical_manifest_matches(
    left: &archive_provider::ArchiveManifest,
    right: &archive_provider::ArchiveManifest,
) -> bool {
    left.entries.len() == right.entries.len()
        && left
            .entries
            .iter()
            .zip(&right.entries)
            .all(|(left, right)| {
                left.normalized_name == right.normalized_name
                    && left.kind == right.kind
                    && left.uncompressed_bytes == right.uncompressed_bytes
            })
}

/// Compares the portable logical tree rather than container metadata.
///
/// ZIP stores directory entry names with a trailing slash while the admitted
/// 7z provider reports its corresponding directory without one. Compression
/// method, compressed size, CRC availability, and that spelling difference
/// remain provider observations rather than Resource Space meaning.
fn logical_content_matches(
    left: &archive_provider::ArchiveManifest,
    right: &archive_provider::ArchiveManifest,
) -> bool {
    left.entries.len() == right.entries.len()
        && left
            .entries
            .iter()
            .zip(&right.entries)
            .all(|(left, right)| {
                left.kind == right.kind
                    && left.uncompressed_bytes == right.uncompressed_bytes
                    && logical_entry_name(left) == logical_entry_name(right)
            })
}

fn logical_entry_name(entry: &ArchiveEntryObservation) -> &str {
    match entry.kind {
        archive_provider::ArchiveEntryKind::Directory => entry
            .normalized_name
            .strip_suffix('/')
            .unwrap_or(&entry.normalized_name),
        archive_provider::ArchiveEntryKind::RegularFile => &entry.normalized_name,
    }
}

fn write_report(report: &Report) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let directory = workspace_root().join("target/hello-archive");
    fs::create_dir_all(&directory)?;
    let path = directory.join("report.json");
    fs::write(&path, serde_json::to_vec_pretty(report)?)?;
    Ok(path)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("hello-archive must remain beneath corpus/")
        .to_path_buf()
}
