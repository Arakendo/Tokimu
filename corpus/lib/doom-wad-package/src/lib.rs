//! Corpus-local composition of archive provenance and WAD container inspection.
//!
//! The archive source remains the retained Resource Space resource. The WAD
//! member is read through a bounded derived view and is not materialized as a
//! second retained resource.

use archive_provider::{ArchiveEntryObservation, ArchiveProvider};
use doom_wad_provider::{inspect_wad, WadError, WadManifest, WadReadLimits};
use resource_space::{ContentFingerprint, InMemoryResourceSpace, ResourceKey};
use resource_space_archive::{
    open_archive_derived_view, read_archive_derived_entry, InspectArchiveResourceRequest,
    ResourceArchiveBridgeError,
};
use thiserror::Error;

mod map_catalog;

pub use map_catalog::{
    select_doom_episode_map, DoomMapLumpObservation, DoomMapSelection, DoomMapSelectionError,
    RequiredDoomMapLump,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectWadPackageRequest {
    pub archive: InspectArchiveResourceRequest,
    /// Normalized archive entry name selected by the caller.
    pub member_name: String,
    /// Provenance label retained by the WAD observation. It is not a path.
    pub wad_source_label: String,
    pub wad_limits: WadReadLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WadPackageObservation {
    pub archive_source: ResourceKey,
    pub archive_fingerprint: ContentFingerprint,
    pub member: ArchiveEntryObservation,
    pub wad: WadManifest,
}

/// Transient selected-member bytes plus the provenance-bearing observation.
///
/// Callers may pass these bytes straight to another bounded provider. They are
/// not inserted into Resource Space and this type makes no persistence claim.
#[derive(Debug, Eq, PartialEq)]
pub struct WadPackageRead {
    pub observation: WadPackageObservation,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum WadPackageError {
    #[error("archive/Resource Space operation failed: {0}")]
    Archive(#[from] ResourceArchiveBridgeError),
    #[error("WAD member inspection failed: {0}")]
    Wad(#[from] WadError),
}

/// Inspects a selected WAD member while retaining package and member evidence.
///
/// The caller decides the archive resource, member name, and all byte limits.
/// This bridge performs no permanent extraction and adds no Doom terms to the
/// archive or Resource Space contracts.
pub fn inspect_wad_package_member<P: ArchiveProvider>(
    space: &InMemoryResourceSpace,
    request: InspectWadPackageRequest,
    provider: &P,
) -> Result<WadPackageObservation, WadPackageError> {
    Ok(read_wad_package_member(space, request, provider)?.observation)
}

/// Reads and validates a selected WAD member without materializing it as a
/// Resource Space entry. The returned bytes remain caller-owned and transient.
pub fn read_wad_package_member<P: ArchiveProvider>(
    space: &InMemoryResourceSpace,
    request: InspectWadPackageRequest,
    provider: &P,
) -> Result<WadPackageRead, WadPackageError> {
    let archive_limits = request.archive.limits;
    let view = open_archive_derived_view(space, request.archive, provider)?;
    let archive_source = view.source().clone();
    let archive_fingerprint = view.source_fingerprint().clone();
    let read =
        read_archive_derived_entry(space, &view, &request.member_name, archive_limits, provider)?;
    let wad = inspect_wad(request.wad_source_label, &read.bytes, request.wad_limits)?;
    Ok(WadPackageRead {
        observation: WadPackageObservation {
            archive_source,
            archive_fingerprint,
            member: read.entry,
            wad,
        },
        bytes: read.bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use archive_provider::{
        ArchiveCompression, ArchiveFormat, ArchiveReadLimits, ArchiveWriteEntry,
        ArchiveWriteLimits, ArchiveWriter, ZipArchiveProvider,
    };
    use resource_space::{
        AddressCasePolicy, FolderId, ResourceMetadata, ResourceName, ResourceRootDescriptor,
        ResourceRootId, StoreId,
    };

    fn wad_bytes() -> Vec<u8> {
        let directory_offset = 13_u32;
        let mut wad = Vec::new();
        wad.extend_from_slice(b"IWAD");
        wad.extend_from_slice(&1_u32.to_le_bytes());
        wad.extend_from_slice(&directory_offset.to_le_bytes());
        wad.push(7);
        wad.extend_from_slice(&12_u32.to_le_bytes());
        wad.extend_from_slice(&1_u32.to_le_bytes());
        wad.extend_from_slice(b"LUMP\0\0\0\0");
        wad
    }

    fn map_manifest(names: &[&str]) -> doom_wad_provider::WadManifest {
        doom_wad_provider::WadManifest {
            source: doom_wad_provider::WadSourceIdentity {
                label: "synthetic/map-catalog.wad".to_owned(),
                byte_len: 0,
                blake3: "synthetic".to_owned(),
            },
            kind: doom_wad_provider::WadKind::Iwad,
            directory_offset: 0,
            directory_bytes: 0,
            total_lump_bytes: 0,
            lumps: names
                .iter()
                .enumerate()
                .map(|(index, name)| doom_wad_provider::WadLumpObservation {
                    index: index as u32,
                    offset: 0,
                    size: 0,
                    name: (*name).to_owned(),
                })
                .collect(),
            namespaces: Vec::new(),
        }
    }

    fn space_with_archive(bytes: Vec<u8>) -> (InMemoryResourceSpace, FolderId, ResourceName) {
        let mut space =
            InMemoryResourceSpace::new(StoreId::from_u128(700), AddressCasePolicy::Sensitive);
        let folder = FolderId::from_u128(701);
        space
            .create_root(
                ResourceRootDescriptor::new(ResourceRootId::from_u128(702), "doom package test"),
                folder,
                ResourceMetadata::default(),
            )
            .expect("synthetic root should be valid");
        let name = ResourceName::parse("doom-package.zip", AddressCasePolicy::Sensitive)
            .expect("synthetic archive name should be valid");
        space
            .insert_resource(folder, name.clone(), bytes, ResourceMetadata::default())
            .expect("synthetic archive should insert");
        (space, folder, name)
    }

    #[test]
    fn selected_wad_member_is_inspected_without_resource_materialization() {
        let archive = ZipArchiveProvider
            .write_archive(
                ArchiveFormat::Zip,
                &[
                    ArchiveWriteEntry::file(
                        "README.TXT",
                        b"package context",
                        ArchiveCompression::Stored,
                    ),
                    ArchiveWriteEntry::file("DOOM1.WAD", wad_bytes(), ArchiveCompression::Stored),
                ],
                ArchiveWriteLimits::new(4096, 4, 2048, 4096, 128),
            )
            .expect("synthetic package should write");
        let (space, folder, source_name) = space_with_archive(archive.bytes);
        let observation = inspect_wad_package_member(
            &space,
            InspectWadPackageRequest {
                archive: InspectArchiveResourceRequest {
                    source_folder: folder,
                    source_name,
                    format: ArchiveFormat::Zip,
                    limits: ArchiveReadLimits::new(4096, 4, 2048, 4096, 128),
                },
                member_name: "DOOM1.WAD".to_owned(),
                wad_source_label: "synthetic-package/DOOM1.WAD".to_owned(),
                wad_limits: WadReadLimits::new(2048, 16, 1024, 2048),
            },
            &ZipArchiveProvider,
        )
        .expect("selected synthetic WAD member should inspect");

        assert_eq!(observation.member.normalized_name, "DOOM1.WAD");
        assert_eq!(observation.wad.source.label, "synthetic-package/DOOM1.WAD");
        assert_eq!(observation.wad.lumps.len(), 1);
        assert_eq!(
            space.summary().resources(),
            1,
            "only the package is retained"
        );
    }

    #[test]
    fn missing_member_remains_an_archive_boundary_failure() {
        let archive = ZipArchiveProvider
            .write_archive(
                ArchiveFormat::Zip,
                &[ArchiveWriteEntry::file(
                    "README.TXT",
                    b"context",
                    ArchiveCompression::Stored,
                )],
                ArchiveWriteLimits::new(4096, 4, 2048, 4096, 128),
            )
            .expect("synthetic package should write");
        let (space, folder, source_name) = space_with_archive(archive.bytes);
        let error = inspect_wad_package_member(
            &space,
            InspectWadPackageRequest {
                archive: InspectArchiveResourceRequest {
                    source_folder: folder,
                    source_name,
                    format: ArchiveFormat::Zip,
                    limits: ArchiveReadLimits::new(4096, 4, 2048, 4096, 128),
                },
                member_name: "DOOM1.WAD".to_owned(),
                wad_source_label: "synthetic-package/DOOM1.WAD".to_owned(),
                wad_limits: WadReadLimits::new(2048, 16, 1024, 2048),
            },
            &ZipArchiveProvider,
        )
        .expect_err("missing selected member must stay visible");
        assert!(matches!(error, WadPackageError::Archive(_)));
    }

    #[test]
    fn selects_a_complete_episode_map_block_in_source_order() {
        let manifest = map_manifest(&[
            "PLAYPAL", "E1M1", "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS",
            "NODES", "SECTORS", "REJECT", "BLOCKMAP", "E1M2",
        ]);
        let selection = select_doom_episode_map(&manifest, "E1M1")
            .expect("complete ordered map block should select");

        assert_eq!(selection.marker.index, 1);
        assert_eq!(selection.local_range, 2..12);
        assert_eq!(selection.required_lumps.len(), 10);
        assert_eq!(
            selection.required_lumps[0].kind,
            RequiredDoomMapLump::Things
        );
        assert_eq!(
            selection.required_lumps[9].kind,
            RequiredDoomMapLump::Blockmap
        );
    }

    #[test]
    fn map_selection_keeps_missing_duplicate_and_reordered_lumps_explicit() {
        let missing = map_manifest(&["E1M1", "THINGS"]);
        assert!(matches!(
            select_doom_episode_map(&missing, "E1M1"),
            Err(DoomMapSelectionError::MissingRequiredLump {
                required: RequiredDoomMapLump::Linedefs,
                ..
            })
        ));

        let duplicate = map_manifest(&[
            "E1M1", "THINGS", "LINEDEFS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS",
            "NODES", "SECTORS", "REJECT", "BLOCKMAP",
        ]);
        assert!(matches!(
            select_doom_episode_map(&duplicate, "E1M1"),
            Err(DoomMapSelectionError::DuplicateRequiredLump {
                required: RequiredDoomMapLump::Linedefs,
                ..
            })
        ));

        let reordered = map_manifest(&[
            "E1M1", "LINEDEFS", "THINGS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS", "NODES",
            "SECTORS", "REJECT", "BLOCKMAP",
        ]);
        assert!(matches!(
            select_doom_episode_map(&reordered, "E1M1"),
            Err(DoomMapSelectionError::ReorderedRequiredLumps {
                preceding: RequiredDoomMapLump::Things,
                following: RequiredDoomMapLump::Linedefs,
                ..
            })
        ));
    }
}
