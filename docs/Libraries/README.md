# External Corpus Libraries

## Purpose

This directory records how Tokimu acquires, scopes, validates, and reports
external test corpora. Each document describes one source ecosystem without
making that source format canonical Tokimu meaning.

## Current Snapshot

Measured on 2026-07-28:

| Corpus | Acquisition | Selection | Executable implementation | Current denominator |
| --- | --- | --- | --- | --- |
| W3C SVG 1.1 | complete and checksum-pinned | 62 manifest entries representing 40 conformance SVGs | active structural runner; 50 SVG cases and 60 total reviewed goldens | 525 conformance SVG documents under `upstream/svg` |
| Khronos glTF Sample Assets | complete and revision-pinned | 6 logical models / 6 selected source variants | bounded inspector and decoder; 5 selected variants decode, 1 stops explicitly | full pinned-revision model inventory remains open |
| W3C XML 2013-09-23 | complete and checksum-pinned | 4 profile cases | active bounded parser/document smoke corpus | 3,078 upstream XML files |
| WebCGM 2.1 | complete and checksum-pinned | 15 cases | acquisition and classification only | 353 CGM files, 232 static case IDs |
| ufbx FBX data | complete and revision-pinned | 23 files / 14 logical scenes | acquisition and classification only | selected ufbx dataset; no universal FBX denominator |

These counts measure different things. A passing-case percentage must not be
reported as source coverage, conformance, compatibility, or architectural
admission.

## Required Plan Shape

Every external corpus document should contain:

- current status with a dated, measured snapshot;
- source identity, revision or archive hash, license, and redistribution policy;
- authoritative fixture layout and preparation/verification commands;
- separate denominators for upstream sources, selected cases, derived cases,
  and executable cases;
- goals, non-goals, ownership, and dependency direction;
- feature matrix and explicit unsupported boundaries;
- incremental slices with deliverables and acceptance criteria;
- structural, diagnostic, differential, and visual evidence policy;
- completion and graduation criteria;
- highest-return next targets and deferred edge cases.

## Shared Rules

1. Ordinary tests are offline and do not mutate authoritative fixtures.
2. Preparation is explicit, pinned, and reproducible.
3. Derived and synthetic fixtures never increase upstream coverage.
4. Structural artifacts are authoritative at structural boundaries. Images are
   complementary evidence.
5. The first divergent stage owns the diagnostic investigation.
6. Source-format objects stop at importer boundaries.
7. Corpus existence does not admit a first-party capability or crate.
8. Coverage is always reported against a named, pinned denominator.

## Validation

Run fixture integrity checks independently:

```powershell
pwsh -NoProfile -File .\scripts\verify-w3c-svg-fixtures.ps1
pwsh -NoProfile -File .\scripts\verify-khronos-gltf-corpus.ps1
pwsh -NoProfile -File .\scripts\verify-w3c-xml-fixtures.ps1
pwsh -NoProfile -File .\scripts\verify-webcgm-corpus.ps1
pwsh -NoProfile -File .\scripts\verify-ufbx-fbx-corpus.ps1
```

Then run the implementation-specific tests named by each corpus document.

## Audit Result

The 2026-07-28 sanity check found:

- the SVG verifier incorrectly treated derived fixture IDs as upstream paths;
- one SVG manifest entry named `shapes-line-02-t.svg` although the pinned
  upstream source is `shapes-line-02-f.svg`;
- SVG documentation mixed unique upstream coverage, manifest entries, and
  runner registration;
- glTF documentation understated the implemented decoder and example evidence;
- XML had a pinned standards corpus but no peer library overview;
- CGM had two completed reproducibility criteria left unchecked;
- FBX already matched the expected planning structure after its initial
  acquisition slice.

The fixture verifier and documentation were updated as part of that audit.
