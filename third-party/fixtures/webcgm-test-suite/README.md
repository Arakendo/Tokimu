# WebCGM 2.1 Test Suite Fixtures

This directory preserves the dated WebCGM 2.1 Test Suite Release 1.2 as
third-party fixture data for Tokimu's CGM presentation-geometry corpus work.

The suite contains static CGM cases, dynamic and DOM-oriented tests, reference
images, HTML operator scripts, XML descriptions, and support material.
Vendoring it does not imply complete CGM or WebCGM conformance. Tokimu's first
selection is limited to bounded static geometry evidence.

## Layout

```text
upstream/                 Verbatim extracted archive contents
inventory.json            Generated source and encoding inventory
selected/                 Versioned Tokimu selection and capability matrix
provenance.json           Source, release, retrieval, and checksum metadata
```

The downloaded ZIP and temporary extraction directories live under `target/`
and are not authoritative fixture copies.

## Source And License

The pinned source is the OASIS-published dated release:

- <https://docs.oasis-open.org/webcgm/test-materials/webcgm21ts/webcgm21-ts-index.html>
- `webcgm21-ts-20100419.zip`
- SHA-256:
  `d540a452d989091db3abd83724ab9d0d9730f57ad792f4db85a04d93103063c9`

The upstream license permits use, copying, and distribution with attribution
and prohibits modifications or derivatives. The complete notice remains at
`upstream/copyright-license.html`. Tokimu selections reference unmodified
upstream files; future reduced or synthetic fixtures must not be represented
as derivatives authorized by this license.

## Preparation

```powershell
pwsh -NoProfile -File .\scripts\prepare-webcgm-corpus.ps1
pwsh -NoProfile -File .\scripts\verify-webcgm-corpus.ps1
```

The preparation script downloads the immutable dated release, verifies its
archive checksum, refuses to replace a differing reviewed upstream tree, and
regenerates `inventory.json`.
