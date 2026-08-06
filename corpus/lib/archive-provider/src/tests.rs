use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::seeds::{UNSAFE_ENTRY_NAME_SEEDS, ZIP_INPUT_SEEDS};
use super::*;

fn limits() -> ArchiveReadLimits {
    ArchiveReadLimits::new(64 * 1024, 32, 16 * 1024, 32 * 1024, 128)
}

fn write_limits() -> ArchiveWriteLimits {
    ArchiveWriteLimits::new(64 * 1024, 32, 16 * 1024, 32 * 1024, 128)
}

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, bytes) in entries {
        if name.ends_with('/') {
            writer
                .add_directory(*name, options)
                .expect("directory fixture should start");
        } else {
            writer
                .start_file(*name, options)
                .expect("file fixture should start");
            writer.write_all(bytes).expect("fixture bytes should write");
        }
    }
    writer.finish().expect("fixture should finish").into_inner()
}

fn symlink_zip_bytes() -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    writer
        .add_symlink(
            "link.txt",
            "target.txt",
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )
        .expect("symlink fixture should write");
    writer.finish().expect("fixture should finish").into_inner()
}

#[test]
fn manifest_and_selected_read_preserve_order_names_and_bytes() {
    let bytes = zip_bytes(&[
        ("docs/", b""),
        ("docs/readme.txt", b"Tokimu archive evidence"),
        ("data.bin", &[0, 1, 2, 3, 255]),
    ]);
    let provider = ZipArchiveProvider;

    let manifest = provider
        .inspect(ArchiveFormat::Zip, &bytes, limits())
        .expect("valid archive should inspect");
    assert_eq!(manifest.entries.len(), 3);
    assert_eq!(manifest.entries[0].kind, ArchiveEntryKind::Directory);
    assert_eq!(manifest.entries[1].normalized_name, "docs/readme.txt");
    assert_eq!(manifest.total_uncompressed_bytes, 28);

    let result = provider
        .read_entry(ArchiveFormat::Zip, &bytes, "docs/readme.txt", limits())
        .expect("selected regular file should read");
    assert_eq!(result.bytes, b"Tokimu archive evidence");
}

#[test]
fn unsafe_and_overlong_names_are_rejected_before_exposure() {
    let provider = ZipArchiveProvider;
    for name in [
        "../escape.txt",
        "/absolute.txt",
        "C:/drive.txt",
        "a\\..\\escape",
    ] {
        let bytes = zip_bytes(&[(name, b"unsafe")]);
        assert!(matches!(
            provider.inspect(ArchiveFormat::Zip, &bytes, limits()),
            Err(ArchiveError::UnsafeEntryName { .. })
        ));
    }

    let bytes = zip_bytes(&[("long-name.txt", b"long")]);
    let constrained = ArchiveReadLimits::new(4096, 4, 64, 64, 4);
    assert!(matches!(
        provider.inspect(ArchiveFormat::Zip, &bytes, constrained),
        Err(ArchiveError::UnsafeEntryName { .. })
    ));
}

#[test]
fn named_adversarial_seeds_stay_at_archive_and_entry_name_boundaries() {
    let provider = ZipArchiveProvider;
    for seed in ZIP_INPUT_SEEDS {
        let error = match provider.inspect(ArchiveFormat::Zip, seed.bytes, limits()) {
            Ok(_) => panic!("archive seed `{}` inspected successfully", seed.id),
            Err(error) => error,
        };
        assert!(
            matches!(
                error,
                ArchiveError::MalformedArchive { .. } | ArchiveError::TruncatedArchive { .. }
            ),
            "archive seed `{}` escaped the structural failure boundary: {error:?}",
            seed.id
        );
    }

    for name in UNSAFE_ENTRY_NAME_SEEDS {
        let bytes = zip_bytes(&[(name, b"unsafe")]);
        assert!(
            matches!(
                provider.inspect(ArchiveFormat::Zip, &bytes, limits()),
                Err(ArchiveError::UnsafeEntryName { .. })
            ),
            "entry-name seed `{name}` was not rejected"
        );
    }
}

#[test]
fn duplicate_normalized_names_are_rejected() {
    let bytes = zip_bytes(&[
        ("folder/./same.txt", b"first"),
        ("folder/same.txt", b"second"),
    ]);
    let error = ZipArchiveProvider
        .inspect(ArchiveFormat::Zip, &bytes, limits())
        .expect_err("normalized duplicate must fail");
    assert_eq!(
        error,
        ArchiveError::DuplicateEntryName {
            normalized_name: "folder/same.txt".to_owned(),
        }
    );
}

#[test]
fn archive_entry_count_entry_size_and_total_output_limits_are_distinct() {
    let bytes = zip_bytes(&[("a", b"1234"), ("b", b"5678")]);
    let provider = ZipArchiveProvider;

    assert!(matches!(
        provider.inspect(
            ArchiveFormat::Zip,
            &bytes,
            ArchiveReadLimits::new(4096, 1, 64, 128, 64),
        ),
        Err(ArchiveError::EntryCountLimitExceeded { .. })
    ));
    assert!(matches!(
        provider.inspect(
            ArchiveFormat::Zip,
            &bytes,
            ArchiveReadLimits::new(4096, 4, 3, 128, 64),
        ),
        Err(ArchiveError::EntryLimitExceeded { .. })
    ));
    assert!(matches!(
        provider.inspect(
            ArchiveFormat::Zip,
            &bytes,
            ArchiveReadLimits::new(4096, 4, 64, 7, 64),
        ),
        Err(ArchiveError::TotalOutputLimitExceeded { .. })
    ));
    assert!(matches!(
        provider.inspect(
            ArchiveFormat::Zip,
            &bytes,
            ArchiveReadLimits::new(8, 4, 64, 128, 64),
        ),
        Err(ArchiveError::ArchiveLimitExceeded { .. })
    ));
}

#[test]
fn missing_entries_and_directory_reads_are_explicit() {
    let bytes = zip_bytes(&[("folder/", b""), ("folder/value", b"value")]);
    let provider = ZipArchiveProvider;

    assert!(matches!(
        provider.read_entry(ArchiveFormat::Zip, &bytes, "missing", limits()),
        Err(ArchiveError::EntryNotFound { .. })
    ));
    assert!(matches!(
        provider.read_entry(ArchiveFormat::Zip, &bytes, "folder/", limits()),
        Err(ArchiveError::UnsupportedEntryKind { .. })
    ));
}

#[test]
fn symlink_entries_are_rejected_during_inspection() {
    let bytes = symlink_zip_bytes();
    let error = ZipArchiveProvider
        .inspect(ArchiveFormat::Zip, &bytes, limits())
        .expect_err("symlink entries must not cross the portable boundary");
    assert!(matches!(
        error,
        ArchiveError::UnsupportedEntryKind { ref kind, .. } if kind == "symlink"
    ));
}

#[test]
fn truncated_central_directory_is_classified() {
    let mut bytes = zip_bytes(&[("value.txt", b"value")]);
    bytes.truncate(bytes.len() - 10);
    let error = ZipArchiveProvider
        .inspect(ArchiveFormat::Zip, &bytes, limits())
        .expect_err("truncated archive should fail");
    assert!(
        matches!(
            error,
            ArchiveError::TruncatedArchive { .. } | ArchiveError::MalformedArchive { .. }
        ),
        "unexpected category: {error:?}"
    );
}

#[test]
fn malformed_central_directory_is_not_a_generic_provider_failure() {
    let mut bytes = zip_bytes(&[("value.txt", b"value")]);
    let offset = bytes
        .windows(4)
        .position(|window| window == [0x50, 0x4b, 0x01, 0x02])
        .expect("fixture should contain a central-directory record");
    bytes[offset + 2] = 0xff;

    let error = ZipArchiveProvider
        .inspect(ArchiveFormat::Zip, &bytes, limits())
        .expect_err("damaged central directory should fail");
    assert!(
        matches!(
            error,
            ArchiveError::MalformedArchive { .. } | ArchiveError::TruncatedArchive { .. }
        ),
        "unexpected category: {error:?}"
    );
}

#[test]
fn crc_corruption_is_an_integrity_failure() {
    let payload = b"distinct payload bytes";
    let mut bytes = zip_bytes(&[("value.txt", payload)]);
    let offset = bytes
        .windows(payload.len())
        .position(|window| window == payload)
        .expect("stored payload should be visible");
    bytes[offset] ^= 0xff;

    let error = ZipArchiveProvider
        .read_entry(ArchiveFormat::Zip, &bytes, "value.txt", limits())
        .expect_err("CRC corruption should fail");
    assert!(
        matches!(error, ArchiveError::IntegrityFailure { .. }),
        "unexpected category: {error:?}"
    );
}

#[test]
fn encrypted_flag_is_rejected_without_requesting_a_password() {
    let mut bytes = zip_bytes(&[("secret.txt", b"secret")]);
    set_encrypted_flags(&mut bytes);

    let error = ZipArchiveProvider
        .inspect(ArchiveFormat::Zip, &bytes, limits())
        .expect_err("encrypted entry should fail");
    assert!(
        matches!(error, ArchiveError::EncryptedEntry { .. }),
        "unexpected category: {error:?}"
    );
}

fn set_encrypted_flags(bytes: &mut [u8]) {
    for signature in [[0x50, 0x4b, 0x03, 0x04], [0x50, 0x4b, 0x01, 0x02]] {
        let offset = bytes
            .windows(4)
            .position(|window| window == signature)
            .expect("fixture should contain ZIP header");
        let flag_offset = if signature[2] == 0x03 {
            offset + 6
        } else {
            offset + 8
        };
        let flags = u16::from_le_bytes([bytes[flag_offset], bytes[flag_offset + 1]]) | 1;
        bytes[flag_offset..flag_offset + 2].copy_from_slice(&flags.to_le_bytes());
    }
}

#[test]
fn deterministic_writer_produces_identical_bytes_and_round_trips() {
    let entries = vec![
        ArchiveWriteEntry::directory("docs"),
        ArchiveWriteEntry::file(
            "docs/readme.txt",
            b"Tokimu deterministic ZIP".to_vec(),
            ArchiveCompression::Deflate,
        ),
        ArchiveWriteEntry::file(
            "data.bin",
            vec![0, 1, 2, 3, 255],
            ArchiveCompression::Stored,
        ),
    ];
    let provider = ZipArchiveProvider;

    let first = provider
        .write_archive(ArchiveFormat::Zip, &entries, write_limits())
        .expect("bounded ZIP should write");
    let second = provider
        .write_archive(ArchiveFormat::Zip, &entries, write_limits())
        .expect("equivalent bounded ZIP should write");
    assert_eq!(first.bytes, second.bytes);
    assert!(first.observation.deterministic_metadata);
    assert_eq!(first.observation.entry_count, 3);
    assert_eq!(first.observation.total_input_bytes, 29);

    let manifest = provider
        .inspect(ArchiveFormat::Zip, &first.bytes, limits())
        .expect("written ZIP should inspect");
    assert_eq!(manifest.entries[0].normalized_name, "docs/");
    assert_eq!(manifest.entries[1].compression, ArchiveCompression::Deflate);
    let read = provider
        .read_entry(
            ArchiveFormat::Zip,
            &first.bytes,
            "docs/readme.txt",
            limits(),
        )
        .expect("written file should read");
    assert_eq!(read.bytes, b"Tokimu deterministic ZIP");
}

#[test]
fn writer_rejects_unsafe_duplicate_and_invalid_entries() {
    let provider = ZipArchiveProvider;
    let unsafe_entry = [ArchiveWriteEntry::file(
        "../escape.txt",
        b"no".to_vec(),
        ArchiveCompression::Stored,
    )];
    assert!(matches!(
        provider.write_archive(ArchiveFormat::Zip, &unsafe_entry, write_limits()),
        Err(ArchiveError::UnsafeEntryName { .. })
    ));

    let duplicates = [
        ArchiveWriteEntry::file("docs/a.txt", vec![], ArchiveCompression::Stored),
        ArchiveWriteEntry::file("docs\\a.txt", vec![], ArchiveCompression::Stored),
    ];
    assert!(matches!(
        provider.write_archive(ArchiveFormat::Zip, &duplicates, write_limits()),
        Err(ArchiveError::DuplicateEntryName { .. })
    ));

    let invalid_directory = [ArchiveWriteEntry {
        name: "docs/".to_owned(),
        kind: ArchiveEntryKind::Directory,
        compression: ArchiveCompression::Stored,
        bytes: vec![1],
    }];
    assert!(matches!(
        provider.write_archive(ArchiveFormat::Zip, &invalid_directory, write_limits()),
        Err(ArchiveError::InvalidWriteEntry { .. })
    ));
}

#[test]
fn writer_enforces_entry_input_and_archive_output_limits() {
    let provider = ZipArchiveProvider;
    let entry = [ArchiveWriteEntry::file(
        "value.txt",
        vec![7; 32],
        ArchiveCompression::Stored,
    )];

    let entry_error = provider
        .write_archive(
            ArchiveFormat::Zip,
            &entry,
            ArchiveWriteLimits::new(1024, 2, 16, 64, 64),
        )
        .expect_err("entry limit should fail before writing");
    assert!(matches!(
        entry_error,
        ArchiveError::EntryLimitExceeded { .. }
    ));

    let output_error = provider
        .write_archive(
            ArchiveFormat::Zip,
            &entry,
            ArchiveWriteLimits::new(16, 2, 64, 64, 64),
        )
        .expect_err("archive output limit should remain bounded");
    assert_eq!(
        output_error,
        ArchiveError::OutputLimitExceeded { limit_bytes: 16 }
    );
}

#[test]
fn tar_writer_produces_deterministic_portable_archives() {
    let entries = [
        ArchiveWriteEntry::directory("docs"),
        ArchiveWriteEntry::file(
            "docs/readme.txt",
            b"TAR corpus".to_vec(),
            ArchiveCompression::Stored,
        ),
        ArchiveWriteEntry::file("data.bin", vec![1, 2, 3], ArchiveCompression::Stored),
    ];
    let provider = TarArchiveProvider;
    let first = provider
        .write_archive(ArchiveFormat::Tar, &entries, write_limits())
        .expect("bounded TAR should write");
    let second = provider
        .write_archive(ArchiveFormat::Tar, &entries, write_limits())
        .expect("equivalent TAR should write");
    assert_eq!(first.bytes, second.bytes);
    assert!(first.observation.deterministic_metadata);

    let manifest = provider
        .inspect(ArchiveFormat::Tar, &first.bytes, limits())
        .expect("written TAR should inspect");
    assert_eq!(manifest.entries.len(), 3);
    assert_eq!(manifest.entries[1].normalized_name, "docs/readme.txt");
    assert_eq!(manifest.entries[1].compression, ArchiveCompression::Stored);
    let read = provider
        .read_entry(
            ArchiveFormat::Tar,
            &first.bytes,
            "docs/readme.txt",
            limits(),
        )
        .expect("written TAR file should read");
    assert_eq!(read.bytes, b"TAR corpus");
}

#[test]
fn tar_rejects_links_and_non_stored_write_requests() {
    let mut builder = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_ustar();
    header.set_size(0);
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_cksum();
    builder
        .append_link(&mut header, "link", "target")
        .expect("link fixture should write");
    let linked = builder.into_inner().expect("fixture should finish");
    assert!(matches!(
        TarArchiveProvider.inspect(ArchiveFormat::Tar, &linked, limits()),
        Err(ArchiveError::UnsupportedEntryKind { ref kind, .. }) if kind == "symlink"
    ));

    let deflated = [ArchiveWriteEntry::file(
        "value",
        vec![1],
        ArchiveCompression::Deflate,
    )];
    assert!(matches!(
        TarArchiveProvider.write_archive(ArchiveFormat::Tar, &deflated, write_limits()),
        Err(ArchiveError::UnsupportedWriteCompression { .. })
    ));
}

#[test]
fn tar_rejects_pax_extensions_before_they_become_portable_metadata() {
    let mut builder = tar::Builder::new(Vec::new());
    builder
        .append_pax_extensions([("SCHILY.xattr.user.note", b"provider-only".as_slice())])
        .expect("PAX fixture should write");
    let mut header = tar::Header::new_ustar();
    header.set_size(5);
    header.set_mode(0o644);
    header.set_entry_type(tar::EntryType::Regular);
    header.set_cksum();
    builder
        .append_data(&mut header, "value.txt", Cursor::new(b"value"))
        .expect("PAX fixture should describe one regular file");
    let bytes = builder.into_inner().expect("fixture should finish");

    let error = TarArchiveProvider
        .inspect(ArchiveFormat::Tar, &bytes, limits())
        .expect_err("PAX extensions must remain explicit unsupported metadata");
    assert!(
        matches!(
            error,
            ArchiveError::UnsupportedEntryKind { ref kind, .. } if kind == "pax-extended"
        ),
        "unexpected PAX diagnostic: {error:?}"
    );
}

#[test]
fn zip_and_tar_expose_equivalent_logical_content() {
    let entries = [
        ArchiveWriteEntry::directory("docs"),
        ArchiveWriteEntry::file(
            "docs/readme.txt",
            b"same bytes".to_vec(),
            ArchiveCompression::Stored,
        ),
        ArchiveWriteEntry::file("data.bin", vec![3, 2, 1], ArchiveCompression::Stored),
    ];
    let zip = ZipArchiveProvider
        .write_archive(ArchiveFormat::Zip, &entries, write_limits())
        .expect("ZIP fixture should write");
    let tar = TarArchiveProvider
        .write_archive(ArchiveFormat::Tar, &entries, write_limits())
        .expect("TAR fixture should write");

    let zip_manifest = ZipArchiveProvider
        .inspect(ArchiveFormat::Zip, &zip.bytes, limits())
        .expect("ZIP fixture should inspect");
    let tar_manifest = TarArchiveProvider
        .inspect(ArchiveFormat::Tar, &tar.bytes, limits())
        .expect("TAR fixture should inspect");
    let zip_entries: Vec<_> = zip_manifest
        .entries
        .iter()
        .map(|entry| (&entry.normalized_name, entry.kind, entry.uncompressed_bytes))
        .collect();
    let tar_entries: Vec<_> = tar_manifest
        .entries
        .iter()
        .map(|entry| (&entry.normalized_name, entry.kind, entry.uncompressed_bytes))
        .collect();
    assert_eq!(zip_entries, tar_entries);
    assert!(zip_manifest.entries[1].crc32.is_some());
    assert_eq!(tar_manifest.entries[1].crc32, None);

    for name in ["docs/readme.txt", "data.bin"] {
        let zip_read = ZipArchiveProvider
            .read_entry(ArchiveFormat::Zip, &zip.bytes, name, limits())
            .expect("ZIP entry should read");
        let tar_read = TarArchiveProvider
            .read_entry(ArchiveFormat::Tar, &tar.bytes, name, limits())
            .expect("TAR entry should read");
        assert_eq!(zip_read.bytes, tar_read.bytes, "entry `{name}` diverged");
    }
}
