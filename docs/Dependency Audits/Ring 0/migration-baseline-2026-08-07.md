# Ring 0 Migration Baseline: 2026-08-07

## Status

This is an inventory and finding record, not an admission record. It records
the current noncompliant state required to begin the ADR-0010 migration.

## Scope And Method

The reviewed root is `tokimu-core` with its currently selected features. The
closure was derived with:

```powershell
pwsh -NoProfile -File scripts/audit-ring-zero-dependencies.ps1 -AllowViolations
cargo tree -p tokimu-core -e all
```

The audit harness follows non-dev Cargo metadata edges from the configured
root. It includes runtime, build, and procedural-macro packages. The
`-AllowViolations` switch permits inventory of a known failing baseline; CI and
ordinary validation must omit it and fail closed.

## Observed Closure

| Package | Version | Selected features | Current source | Registry checksum |
| --- | --- | --- | --- | --- |
| `glam` | 0.29.3 | `default`, `std` | crates.io registry | `8babf46d4c1c9d92deac9f7be466f76dfc4482b6452fc5024b5e8daf6ffeb3ee` |
| `serde` | 1.0.228 | `default`, `derive`, `serde_derive`, `std` | crates.io registry | `9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e` |
| `serde_core` | 1.0.228 | `alloc`, `result`, `std` | crates.io registry | `41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad` |
| `serde_derive` | 1.0.228 | `default` | crates.io registry | `d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79` |
| `proc-macro2` | 1.0.106 | `default`, `proc-macro` | crates.io registry | `8fd00f0bb2e90d81d1044c2b32617f68fcb9fa3bb7640c23e9c748e53fb30934` |
| `quote` | 1.0.46 | `default`, `proc-macro` | crates.io registry | `dfbc457d0c7a0759a614551b11a6409e5951f6c7537be1f1b7682b9ae9230368` |
| `syn` | 2.0.118 | `clone-impls`, `default`, `derive`, `parsing`, `printing`, `proc-macro` | crates.io registry | `1b9ae57f904213ebb649ce6895b8a66c66f0203b9319718f69a5612a065b1422` |
| `unicode-ident` | 1.0.24 | `default` | crates.io registry | `e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75` |

`tokimu-core` itself is the only workspace-path package in this closure.
`syn` 3.0.3 occurs elsewhere in the workspace lockfile but is not reachable
from the `tokimu-core` root and is therefore not a Ring 0 migration package.

## Exact Upstream Mapping

The packaged `.cargo_vcs_info.json` records below were verified as retrievable
upstream Git objects on 2026-08-07. The Serde repository supplies three
packages at one revision; every other package has its own repository.

| Repository | Package or packages | Exact commit |
| --- | --- | --- |
| `https://github.com/bitshifter/glam-rs` | `glam` 0.29.3 | `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| `https://github.com/serde-rs/serde` | `serde`, `serde_core`, `serde_derive` 1.0.228 | `a866b336f14aa57a07f0d0be9f8762746e64ecb4` |
| `https://github.com/dtolnay/proc-macro2` | `proc-macro2` 1.0.106 | `58ab776b95a4c2865554badbb6629c50971a9118` |
| `https://github.com/dtolnay/quote` | `quote` 1.0.46 | `bc4caf255fa9e58e025e5ff5a11ca948442c0f7a` |
| `https://github.com/dtolnay/syn` | `syn` 2.0.118 | `f033ef1403b4dbd276d95c26ff05b51d758d7b14` |
| `https://github.com/dtolnay/unicode-ident` | `unicode-ident` 1.0.24 | `5b54a632702b5744a1c40ea01c127c0ac0498172` |

## Direct Use And Public Boundary

- `tokimu-core::math` publicly re-exports `glam::{Mat4, Quat, Vec2, Vec3,
  Vec4}`. This is an explicit public-vocabulary finding, not merely a private
  implementation choice.
- `tokimu-core::scene` uses `Serialize` and `Deserialize` derives on its
  scene-facing Native Ring types. The derives execute `serde_derive` and its
  `proc-macro2`/`quote`/`syn`/`unicode-ident` build-time closure.
- `glam` is supplied by `https://github.com/bitshifter/glam-rs`.
- `serde`, `serde_core`, and `serde_derive` are supplied by
  `https://github.com/serde-rs/serde`.
- `proc-macro2`, `quote`, `syn`, and `unicode-ident` are supplied by their
  respective `https://github.com/dtolnay/<repository>` repositories.
- Package metadata records `MIT OR Apache-2.0` for every package except
  `unicode-ident`, whose metadata additionally includes `Unicode-3.0`.

## Findings

### R0-001: All foreign Ring 0 packages resolve from the registry

**Severity:** migration blocker.

Every foreign package in the observed closure has the crates.io registry source
in Cargo metadata. None is a parent-pinned submodule under
`third-party/ring-0/`, and Cargo is not configured to resolve any of them from
local audited source. This violates ADR-0010's source and pinning requirements.

**Closure action:** map each package to an exact upstream commit, audit the
repository revision, then add only accepted repositories as submodules and
redirect every closure package through local paths or patches.

### R0-002: No retained source audit exists

**Severity:** migration blocker.

No per-revision audit record yet establishes selected source, generated or
build-time behavior, unsafe/FFI surface, advisories, performance, failure
evidence, owner, or update/removal procedure.

**Closure action:** complete the records described by
`docs/Dependency Audits/Ring 0/README.md` before admitting source. A repository
may be rejected, moved, wrapped, replaced, or removed; audit completion does
not imply retention.

### R0-003: `glam` is a public Ring 0 vocabulary commitment

**Severity:** architectural decision required.

The public `tokimu-core::math` re-export means upstream type semantics and
representation are visible to callers. Its implementation audit and public API
admission must be decided separately under ADR-0010: retain and re-export,
retain privately and wrap, replace, move conversion outside Ring 0, or remove
the dependency.

**Closure action:** do not expand public `glam` exposure while this finding is
open. Record an explicit final disposition in the `glam` audit.

### R0-004: Serde derives make a macro-heavy build-time closure trusted code

**Severity:** architectural decision required.

Serialization derives on Native Ring scene types select four build-time
packages in addition to the Serde runtime packages. The build-time code is in
scope even though it is not linked into the delivered executable.

**Closure action:** decide whether those derives represent an admitted Native
serialization contract, a narrower engine-owned format seam, or an Outer Ring
translation concern. Do not add new Ring 0 derives or Serde features while the
decision is open.

### R0-005: Source and feature enforcement did not exist

**Severity:** tooling finding; initial control now present.

Before this baseline, no repository command derived the Ring 0 closure and
rejected registry, remote Git, unapproved local path, missing submodule, or
dirty-submodule source. `scripts/audit-ring-zero-dependencies.ps1` and
`scripts/ring-zero-dependencies.json` now provide that initial control. With an
empty approved-source list, ordinary execution intentionally reports the eight
registry violations above.

**Closure action:** populate approved submodule paths only as corresponding
audits close, then add this command to CI after the first local-source slice.

## Not Yet Reviewed

- Exact upstream commit mapping and package-to-upstream-tree comparison.
- Source-tree hashes, generated code, build scripts, procedural-macro behavior,
  unsafe/FFI, and source-size evidence.
- Advisory, license notice, patent, and redistribution review beyond Cargo
  package metadata.
- Native/WASM build evidence after local source redirection.
- Publication and release source-identity behavior.

These are open audit work, not clean findings. Tokimu must not claim
ADR-0010 compliance until the full migration plan is complete.

## Remediation Update

After this baseline was captured, `tokimu-core` removed its unused Serde
derives. The serialization and procedural-macro packages no longer enter the
Ring 0 closure. `glam` was then pinned at the exact audited source commit under
`third-party/ring-0/glam`, and Cargo was rewired to that local path. The active
closure is now `tokimu-core` plus the approved local `glam` submodule.

This closes the initial registry-source finding for the active Ring 0 closure.
It does not close ADR-0010's remaining CI, release, publication, and ongoing
audit requirements.

## Reopening Triggers

- Any feature, package, or source change reachable from `tokimu-core`.
- Any new public foreign type or trait exposure from a Native Ring API.
- A relevant upstream advisory, license change, source-identity discrepancy, or
  native/WASM behavior difference.
- Addition of local source, a Cargo patch, a submodule update, or a change to
  the audit harness.
