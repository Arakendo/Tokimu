# W3C XML Conformance Fixtures (20130923)

This directory preserves the W3C XML Conformance Test Suite release dated
2013-09-23 for scoped XML parser evidence. It is not a claim that Tokimu
implements the whole XML recommendation or every suite profile.

## Layout

- `xmlts20130923.tar.gz` is the downloaded upstream archive.
- `upstream/xmlconf/` is the archive extracted without local edits.
- `selected/selection-v1.toml` references the small reviewed smoke selection.
  It does not copy upstream cases.
- `selected/feature-matrix.md` maps the current XML profile to the selection.
- `provenance.json` records origin, retrieval date, and archive checksum.
- `LICENSES/` preserves upstream notices relevant to the selected corpus.
- `scripts/verify-w3c-xml-fixtures.ps1` verifies a locally retained archive
  and every source path referenced by the selection without network access.

## Validation Boundary

Structural parser events and diagnostics are authoritative for this fixture
suite. The selected cases are classified as `Accepted`, `Rejected`, or
`UnsupportedByProfile`; unsupported cases demonstrate an explicit boundary,
not a parser failure.

The full upstream corpus remains a future execution tier. It is intentionally
not run as a single pass until each case is classified against Tokimu's
declared XML profile.
