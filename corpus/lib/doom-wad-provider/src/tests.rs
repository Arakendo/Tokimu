use super::*;

fn limits() -> WadReadLimits {
    WadReadLimits::new(4096, 16, 1024, 2048)
}

fn synthetic_wad(kind: &[u8; 4], entries: &[(&str, &[u8])]) -> Vec<u8> {
    let directory_offset =
        HEADER_BYTES + entries.iter().map(|(_, bytes)| bytes.len()).sum::<usize>();
    let mut wad = Vec::with_capacity(directory_offset + entries.len() * DIRECTORY_ENTRY_BYTES);
    wad.extend_from_slice(kind);
    wad.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    wad.extend_from_slice(&(directory_offset as u32).to_le_bytes());
    let mut offset = HEADER_BYTES as u32;
    for (_, bytes) in entries {
        wad.extend_from_slice(bytes);
    }
    for (name, bytes) in entries {
        wad.extend_from_slice(&offset.to_le_bytes());
        wad.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        let mut encoded = [0_u8; 8];
        encoded[..name.len()].copy_from_slice(name.as_bytes());
        wad.extend_from_slice(&encoded);
        offset += bytes.len() as u32;
    }
    wad
}

#[test]
fn synthetic_iwad_preserves_order_duplicates_and_source_identity() {
    let wad = synthetic_wad(
        b"IWAD",
        &[("PLAYPAL", &[1, 2]), ("E1M1", &[]), ("PLAYPAL", &[3])],
    );
    let manifest = inspect_wad("synthetic/valid-iwad.wad", &wad, limits())
        .expect("synthetic IWAD should inspect");

    assert_eq!(manifest.kind, WadKind::Iwad);
    assert_eq!(manifest.source.label, "synthetic/valid-iwad.wad");
    assert_eq!(manifest.source.byte_len, wad.len());
    assert_eq!(manifest.lumps.len(), 3);
    assert_eq!(manifest.lumps[0].name, "PLAYPAL");
    assert_eq!(manifest.lumps[1].name, "E1M1");
    assert_eq!(manifest.lumps[2].name, "PLAYPAL");
    assert_eq!(manifest.total_lump_bytes, 3);
    assert!(manifest.namespaces.is_empty());
}

#[test]
fn marker_ranges_project_members_without_promoting_markers_to_resources() {
    let wad = synthetic_wad(
        b"IWAD",
        &[
            ("F_START", &[]),
            ("FLAT1", &[1]),
            ("FLAT2", &[2]),
            ("F_END", &[]),
            ("S_START", &[]),
            ("TROOA1", &[3]),
            ("S_END", &[]),
        ],
    );
    let manifest = inspect_wad("synthetic/namespaces.wad", &wad, limits())
        .expect("well-paired marker ranges should inspect");

    assert_eq!(manifest.namespaces.len(), 2);
    assert_eq!(manifest.namespaces[0].kind, WadNamespaceKind::Flats);
    assert_eq!(manifest.namespaces[0].start_marker_index, 0);
    assert_eq!(manifest.namespaces[0].end_marker_index, 3);
    assert_eq!(manifest.namespaces[0].lump_indices, [1, 2]);
    assert_eq!(manifest.namespaces[1].kind, WadNamespaceKind::Sprites);
    assert_eq!(manifest.namespaces[1].lump_indices, [5]);
}

#[test]
fn malformed_marker_pairs_are_structured_failures() {
    let unclosed = synthetic_wad(b"IWAD", &[("P_START", &[])]);
    assert!(matches!(
        inspect_wad("synthetic/unclosed-marker.wad", &unclosed, limits()),
        Err(WadError::UnclosedNamespaceStart {
            kind: WadNamespaceKind::Patches,
            index: 0,
        })
    ));

    let unmatched = synthetic_wad(b"IWAD", &[("SS_END", &[])]);
    assert!(matches!(
        inspect_wad("synthetic/unmatched-marker.wad", &unmatched, limits()),
        Err(WadError::UnmatchedNamespaceEnd {
            kind: WadNamespaceKind::Sprites,
            index: 0,
        })
    ));

    let nested = synthetic_wad(b"IWAD", &[("F_START", &[]), ("FF_START", &[])]);
    assert!(matches!(
        inspect_wad("synthetic/nested-marker.wad", &nested, limits()),
        Err(WadError::OverlappingNamespaceMarker {
            open_kind: WadNamespaceKind::Flats,
            marker_kind: WadNamespaceKind::Flats,
            first_index: 0,
            second_index: 1,
        })
    ));

    let mismatched = synthetic_wad(b"IWAD", &[("F_START", &[]), ("S_END", &[])]);
    assert!(matches!(
        inspect_wad("synthetic/mismatched-marker.wad", &mismatched, limits()),
        Err(WadError::MismatchedNamespaceEnd {
            open_kind: WadNamespaceKind::Flats,
            end_kind: WadNamespaceKind::Sprites,
            first_index: 0,
            second_index: 1,
        })
    ));
}

#[test]
fn pwad_is_recognized() {
    let wad = synthetic_wad(b"PWAD", &[("PATCH", &[7])]);
    assert_eq!(
        inspect_wad("synthetic/valid-pwad.wad", &wad, limits())
            .expect("synthetic PWAD should inspect")
            .kind,
        WadKind::Pwad
    );
}

#[test]
fn unknown_and_truncated_headers_are_structured_failures() {
    assert!(matches!(
        inspect_wad("synthetic/short.wad", b"IWA", limits()),
        Err(WadError::TruncatedHeader { .. })
    ));
    let mut unknown = synthetic_wad(b"IWAD", &[]);
    unknown[..4].copy_from_slice(b"NOPE");
    assert!(matches!(
        inspect_wad("synthetic/unknown.wad", &unknown, limits()),
        Err(WadError::UnknownSignature { .. })
    ));
}

#[test]
fn directory_and_lump_ranges_are_bounded() {
    let mut directory = synthetic_wad(b"IWAD", &[]);
    directory[8..12].copy_from_slice(&4000_u32.to_le_bytes());
    assert!(matches!(
        inspect_wad("synthetic/truncated-directory.wad", &directory, limits()),
        Err(WadError::DirectoryOutOfBounds { .. })
    ));

    let mut lump = synthetic_wad(b"IWAD", &[("DATA", &[1])]);
    let directory_offset = read_u32(&lump, 8) as usize;
    lump[directory_offset..directory_offset + 4].copy_from_slice(&4000_u32.to_le_bytes());
    assert!(matches!(
        inspect_wad("synthetic/out-of-bounds-lump.wad", &lump, limits()),
        Err(WadError::LumpOutOfBounds { .. })
    ));
}

#[test]
fn overlapping_ranges_and_limits_are_rejected() {
    let mut overlap = synthetic_wad(b"IWAD", &[("ONE", &[1, 2]), ("TWO", &[3, 4])]);
    let directory_offset = read_u32(&overlap, 8) as usize;
    overlap[directory_offset + DIRECTORY_ENTRY_BYTES..directory_offset + DIRECTORY_ENTRY_BYTES + 4]
        .copy_from_slice(&(HEADER_BYTES as u32 + 1).to_le_bytes());
    assert!(matches!(
        inspect_wad("synthetic/overlap.wad", &overlap, limits()),
        Err(WadError::OverlappingLumpRanges { .. })
    ));

    let large = synthetic_wad(b"IWAD", &[("DATA", &[0; 8])]);
    assert!(matches!(
        inspect_wad(
            "synthetic/limit.wad",
            &large,
            WadReadLimits::new(4096, 16, 4, 2048)
        ),
        Err(WadError::LumpSizeLimitExceeded { .. })
    ));

    let count_limited = synthetic_wad(b"IWAD", &[("ONE", &[]), ("TWO", &[])]);
    assert!(matches!(
        inspect_wad(
            "synthetic/count-limit.wad",
            &count_limited,
            WadReadLimits::new(4096, 1, 1024, 2048)
        ),
        Err(WadError::LumpCountLimitExceeded { .. })
    ));

    let total_limited = synthetic_wad(b"IWAD", &[("ONE", &[1, 2]), ("TWO", &[3, 4])]);
    assert!(matches!(
        inspect_wad(
            "synthetic/total-limit.wad",
            &total_limited,
            WadReadLimits::new(4096, 16, 1024, 3)
        ),
        Err(WadError::TotalLumpBytesLimitExceeded { .. })
    ));
}

#[test]
fn malformed_names_and_input_limit_are_rejected() {
    let mut malformed = synthetic_wad(b"IWAD", &[("DATA", &[1])]);
    let directory_offset = read_u32(&malformed, 8) as usize;
    malformed[directory_offset + 8..directory_offset + 16]
        .copy_from_slice(&[b'A', 0, b'B', 0, 0, 0, 0, 0]);
    assert!(matches!(
        inspect_wad("synthetic/malformed-name.wad", &malformed, limits()),
        Err(WadError::MalformedLumpName { .. })
    ));

    let valid = synthetic_wad(b"IWAD", &[("DATA", &[1])]);
    assert!(matches!(
        inspect_wad(
            "synthetic/input-limit.wad",
            &valid,
            WadReadLimits::new(1, 16, 1024, 2048)
        ),
        Err(WadError::InputLimitExceeded { .. })
    ));
}
