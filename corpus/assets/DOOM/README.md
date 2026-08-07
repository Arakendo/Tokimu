# Shareware Package Fixtures

This directory records Doom and Heretic shareware packages as bounded corpus
evidence. The canonical ZIP archives live under `../archive/DOOM/`; the
metadata and derived internal fixtures live here. Local extracted files are
inspection copies, not independent redistribution units.

Tokimu consumers should access a WAD as a logical archive member:

```text
shareware ZIP
    -> archive provider
    -> Resource Space
    -> logical WAD resource
    -> WAD provider
```

This keeps the historical package, documentation, and applicable notices
together. It also prevents a corpus consumer from turning a convenient loose
WAD path into an accidental distribution contract.

## Package Inventory

| Package | Bytes | SHA-256 | Relevant members |
| --- | ---: | --- | --- |
| `../archive/DOOM/DOSBOX_DOOM.ZIP` | 2,357,547 | `9ed3172e728d403962f874eaba93b4b973af1e57a8608bd803fc6e02d137fbc6` | `DOOM1.WAD`, `README.TXT` |
| `../archive/DOOM/DOSBOX_HERETIC.ZIP` | 2,794,870 | `f4ca7bffd27ab3e671beb3cadee7a39c3b7b8c330e5e0591f4a42c2f7b6bb944` | `HERETIC1.WAD`, `LICENSE.DOC`, `VENDOR.DOC`, `README.TXT` |

## Compact Corpus Packages

The canonical archives contain DOS executables, launchers, configuration, and
distribution-era support files that do not contribute to Tokimu's parser or
Resource Space evidence. Generate bounded internal fixtures with:

```powershell
pwsh -NoProfile -File scripts/prepare-doom-corpus-packages.ps1
```

The script verifies each source archive hash, copies selected members directly
from the ZIP, assigns fixed entry timestamps, and embeds a provenance record.
It produces:

- `packages/doom-shareware-corpus-v1.zip` (1,810,639 bytes, 23.2% smaller)
- `packages/heretic-shareware-corpus-v1.zip` (2,366,228 bytes, 15.3% smaller)

The WAD dominates each package's compressed size, so the reduction is bounded.
The useful result is a precise Resource Space fixture without DOS executables,
launchers, multiplayer utilities, configuration, or order forms.

These compact ZIPs are derived test inputs, not historical distribution
packages. They are internal-only pending an explicit derived-package review.
Do not publish them through the website or release artifacts. The complete,
unchanged source ZIPs remain the authoritative provenance and potential
redistribution units.

The extracted WAD observations are:

| Member | Role | Bytes | Header | Lumps | Directory offset | SHA-256 |
| --- | --- | ---: | --- | ---: | ---: | --- |
| `DOOM1.WAD` | Doom 1.9 shareware semantic target | 4,196,020 | `IWAD` | 1,264 | 4,175,796 | `1d7d43be501e67d927e415e0b8f3e29c3bf33075e859721816f652a526cac771` |
| `HERETIC1.WAD` | Comparative WAD-container evidence | 5,120,920 | `IWAD` | 1,374 | 5,098,936 | `3ab2f21828877e49e5eb3220785aaf8798050b7c4132003b5db7b8f3678bede4` |

`DOOM1.WAD` is additionally pinned by MD5
`f0cefca49926d00903cf57551d901abe`, matching the Doom 1.9 shareware IWAD.
`HERETIC1.WAD` is pinned by SHA-1
`b4c50ca9bea07f7c35250a1a11906091971c05ae`.

A bounded inventory found `E1M1` through `E1M9` in both WADs and no negative
or out-of-file lump ranges. This qualifies the fixtures; an importer must still
validate every read independently.

## Provenance And Distribution Findings

Engine source licensing and game-data licensing are separate. Doom's GPLv2
engine source release and the later GPL releases of Heretic/Hexen source do not
license these WADs.

### Doom

The Doom package does not contain a standalone `LICENSE.DOC`. Its included
`README.TXT` is release documentation, not a substitute license. Debian's
preserved Doom shareware copyright record supplies supplemental provenance and
records the unchanged, no-fee redistribution terms and later rights-holder
clarification used by this corpus review.

That record is external evidence. It must not be inserted into the archive and
then represented as an original package member. The package remains non-free;
the finding does not make its data open source or public domain, permit
derivative data, or cover registered/commercial Doom WADs.

### Heretic

The Heretic package is self-describing: it contains `LICENSE.DOC`,
`VENDOR.DOC`, and `README.TXT` beside `HERETIC1.WAD`. The locally preserved
license permits royalty-free electronic distribution while requiring the
software to be distributed in compressed form. Accordingly, Tokimu treats the
complete ZIP package, not a naked `HERETIC1.WAD`, as the reviewed distribution
unit.

The package documents are pinned as follows:

| Member | SHA-1 | SHA-256 |
| --- | --- | --- |
| `LICENSE.DOC` | `c97b176fe0458039219eb426ad315dc5ff155324` | `d91e4c0571f67cde0ffdf0fe8e5958bd57debda2d31e823c84ca4463c0761279` |
| `VENDOR.DOC` | `a4360e93169602b3daa7e87364e1c341cbc02282` | `c8efc8346f7176828e939a6986bcb83eac490f2b93ca4c98d78069bb10606c4a` |

The removed `heretic license/LICENSE.DOC` inspection copy was verified as
byte-identical to the package member before cleanup.

## Repository And Publication Policy

- Preserve the ZIPs unchanged as the canonical package artifacts.
- Treat `packages/*-corpus-v1.zip` as generated internal derivatives, never as
  substitutes for the canonical packages.
- Keep extracted WADs and extracted package trees out of release artifacts.
- Do not publish bare WAD download endpoints.
- Mount archives through the archive provider when a consumer needs a member.
- Keep Doom and Heretic semantics separate even though both use WAD containers.
- Review any CI or website publication separately; this record does not claim
  that publication is already enabled.
- Keep Tokimu-authored synthetic fixtures for unrestricted deterministic CI.

This is an engineering provenance record, not legal advice.

## Review References

- <https://doomwiki.org/wiki/Licences>
- <https://doomwiki.org/wiki/Shareware>
- <https://sources.debian.org/src/doom-wad-shareware/1.9.fixed-2/debian/copyright/>
- <https://doomwiki.org/wiki/DOOM1.WAD>

See `inventory-v1.toml` for machine-readable package and member identities.
