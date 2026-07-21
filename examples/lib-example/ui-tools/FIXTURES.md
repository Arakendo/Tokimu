# Font Fixture Policy

The corpus examples use provider assets from the repository's pinned font
submodules. `scripts/prepare-glyph-corpus.ps1` selects named fixtures, records
their source paths and SHA-256 checksums in `target/glyph-corpus/manifest.json`,
and copies only the selected files into the prepared corpus.

This keeps large provider trees out of normal release artifacts while making a
local corpus run reproducible. Examples should resolve a logical provider and
format through `ui-tools`; they should not search a provider submodule directly.

The current policy is:

- submodule-resolved assets for development and extended corpus pages;
- manifest identity and checksums for reproducibility;
- preserved provider licenses and attribution;
- `fixtures/NotoSans-Regular.otf` as the first small checked-in fixture for
  tests that must run without preparation.

The checked-in fixture is intentionally separate from the reference corpus. It
is selected for test coverage and license clarity, not as the product default.
It is copied from the Noto submodule's unhinted OTF fixture and has SHA-256:

```text
7B8A545D63DE82A3325DC3C545B597898C03219BD432B0A18086E7605859C6C4
```
