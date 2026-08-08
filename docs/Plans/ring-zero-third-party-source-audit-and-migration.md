# Ring 0 Third-Party Source Audit And Migration

## Status

Active on 2026-08-07. ADR-0010 is Accepted. The current Ring 0 closure remains
noncompliant until source audit, submodule pinning, local Cargo resolution, and
CI enforcement are complete.

## Purpose

Audit every third-party source that enters Tokimu's Ring 0 trust boundary,
decide whether it remains there, pin accepted source through Git submodules,
make Cargo compile the reviewed local source closure, and enforce that closure
in CI.

This plan performs the migration required by ADR-0010 without assuming that the
current dependencies should all survive it. Each dependency may be retained,
wrapped, replaced, reduced, or moved into an Outer Ring according to the audit
evidence.

## Governing Decisions

- ADR-0003 defines which meaning is Native Tokimu.
- ADR-0005 governs provisional admission and evidence substitutions.
- ADR-0008 applies the full performance and code-quality gate to Ring 0.
- ADR-0009 applies the full verification, containment, and recovery gate.
- ADR-0010 requires exact submodule source, recursive auditability, local Cargo
  resolution, update review, CI enforcement, and a publication strategy.

This plan uses **Ring 0** and **Native Ring** as synonyms. The classification is
about architectural ownership, not crate names or desktop-native execution.

## Desired End State

```text
Tokimu parent commit
    |
    +-- pinned gitlinks
    |     third-party/ring-0/<upstream repository>/
    |
    +-- retained dependency audits
    |     docs/Dependency Audits/Ring 0/
    |
    +-- Cargo path dependencies / local patches
    |     exact reviewed source only
    |
    +-- machine-derived Ring 0 closure
    |     runtime + build + proc-macro edges
    |
    +-- CI enforcement
          no registry, remote Git, dirty, missing, or unapproved source
```

An official Ring 0 build must be explainable from the parent commit without
Cargo silently substituting a registry package that happens to share a version
number.

## Scope

### In scope

- Classifying the compilation roots and feature sets that contain Ring 0 code.
- Inventorying direct and transitive runtime, build-script, and procedural-
  macro sources reachable from those roots.
- Auditing the current `glam` and `serde` closure.
- Reviewing `glam` types in Tokimu's public math API.
- Reviewing whether serialization derives belong in `tokimu-core`.
- Adding accepted source repositories under `third-party/ring-0/` as pinned
  submodules.
- Rewiring Cargo to local submodule paths and proving source selection.
- Adding audit records, source/feature manifests, and CI enforcement.
- Resolving official build, release archive, and crate publication behavior.
- Recording rejected dependencies and their replacement or relocation plans.

### Out of scope

- Auditing the Rust compiler, standard library, Cargo, linker, operating system,
  browser, GPU driver, or hardware implementation under this policy.
- Applying Ring 0 rules to fonts, reference corpora, website packages, or
  optional providers that remain structurally outside Ring 0.
- Updating dependencies merely to reach newer upstream releases.
- Treating vulnerability scanners as a substitute for source review.
- Reorganizing unrelated crates while performing dependency migration.
- Claiming that audited source is therefore correct, secure, or bug-free.

## Current Baseline

`tokimu-core` currently declares:

```toml
glam.workspace = true
serde = { workspace = true, features = ["derive"] }
```

Current direct use is narrow but architecturally significant:

- `tokimu-core::math` publicly re-exports `Mat4`, `Quat`, `Vec2`, `Vec3`, and
  `Vec4` from `glam`.
- `tokimu-core::scene` derives `Serialize` and `Deserialize` on scene-facing
  Native Ring types.

The current Ring 0 dependency closure observed from
`cargo tree -p tokimu-core -e all` is:

| Package | Version | Registry checksum | Role |
| --- | --- | --- | --- |
| `glam` | `0.29.3` | `8babf46d4c1c9d92deac9f7be466f76dfc4482b6452fc5024b5e8daf6ffeb3ee` | Public math representation and implementation |
| `serde` | `1.0.228` | `9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e` | Serialization facade and derives |
| `serde_core` | `1.0.228` | `41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad` | Serialization traits/runtime support |
| `serde_derive` | `1.0.228` | `d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79` | Procedural macro |
| `proc-macro2` | `1.0.106` | `8fd00f0bb2e90d81d1044c2b32617f68fcb9fa3bb7640c23e9c748e53fb30934` | Procedural-macro compatibility layer |
| `quote` | `1.0.46` | `dfbc457d0c7a0759a614551b11a6409e5951f6c7537be1f1b7682b9ae9230368` | Token generation |
| `syn` | `2.0.118` | `1b9ae57f904213ebb649ce6895b8a66c66f0203b9319718f69a5612a065b1422` | Rust syntax parsing for derives |
| `unicode-ident` | `1.0.24` | `e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75` | Identifier classification |

The workspace lockfile also contains `syn` 3.x for other consumers. It is not
part of the observed `tokimu-core` closure and must not be pulled into this
audit merely because it shares a package name. Closure evidence must follow
actual dependency edges and selected features.

Current feature observations:

- `glam`: default features, including `std`;
- `serde`: default, `std`, and `derive`;
- `serde_derive`: its selected `proc-macro2`, `quote`, and `syn` features.

All packages above currently resolve from crates.io. No current package is
approved or grandfathered by ADR-0010.

The active baseline and open findings are retained in
`docs/Dependency Audits/Ring 0/migration-baseline-2026-08-07.md`. The initial
audit harness is `scripts/audit-ring-zero-dependencies.ps1`; its intentionally
failing current result is the machine-checkable record of the registry-source
finding, not a compliance result.

## Repository Grouping Hypothesis

The eight packages likely map to six upstream source repositories:

| Proposed submodule | Packages expected from it |
| --- | --- |
| `third-party/ring-0/glam` | `glam` |
| `third-party/ring-0/serde` | `serde`, `serde_core`, `serde_derive` |
| `third-party/ring-0/proc-macro2` | `proc-macro2` |
| `third-party/ring-0/quote` | `quote` |
| `third-party/ring-0/syn` | `syn` 2.x selected by Ring 0 |
| `third-party/ring-0/unicode-ident` | `unicode-ident` |

This is a discovery hypothesis, not authorization to add the submodules. Each
registry package must be mapped to the exact upstream commit and compared with
the published package before the repository grouping is accepted.

## Evidence Artifacts

The audit must retain:

| Artifact | Purpose |
| --- | --- |
| Ring 0 root and feature inventory | Defines which compilation graphs are trusted |
| Cargo metadata closure report | Proves runtime/build/proc-macro source edges |
| Registry-to-upstream equivalence report | Maps each current package to an exact source revision |
| One audit record per upstream repository | Records ADR-0010 audit evidence and disposition |
| License and attribution inventory | Preserves redistribution obligations |
| Unsafe/build/proc-macro inventory | Identifies privileged or generated behavior |
| Public API exposure report | Records foreign types and migration cost |
| Native/WASM build and behavior report | Proves supported target consequences |
| Performance/size/compile baseline | Supports ADR-0008 decisions |
| Failure and malformed-input report | Supports ADR-0009 decisions |
| Dependency disposition matrix | Records retain/wrap/replace/move/reject outcomes |
| Publication decision | Prevents downstream registry substitution |
| CI enforcement report | Proves source-selection policy is mechanical |

Audit records should use:

```text
docs/Dependency Audits/Ring 0/<library>-<revision>.md
```

## Slice 0: Accept The Governing Boundary And Freeze Expansion

### Deliverables

- [x] Complete review of ADR-0010 and mark it Accepted or revise it.
- [x] Record the eight current packages as the finite migration set.
- [x] Freeze new Ring 0 third-party dependencies and feature expansion.
- [ ] Require ADR-0005 evidence for any urgent exception during migration.
- [x] Announce that current Ring 0 builds are not yet ADR-0010 compliant.

### Acceptance criteria

- [ ] No document or release claim describes current Ring 0 source as fully
      audited.
- [ ] New work cannot expand the migration set without a retained exception.
- [ ] Audit work may proceed without implying that any dependency is accepted.

## Slice 1: Define Ring 0 Compilation Roots

### Deliverables

- [x] Inventory every workspace package and feature set that owns Native Ring
      semantics under ADR-0003 and ADR-0008.
- [x] Distinguish Native contracts from Outer implementations inside mixed
      crates such as render, platform, asset, input, and runtime packages.
- [x] Define the exact Cargo roots and feature combinations used to derive the
      trusted dependency closure.
- [x] Record dev-only tools separately from code that generates or links Ring 0
      artifacts.
- [x] Record build scripts and procedural macros as trusted build-time code.

### Current root classification

The machine-readable roots in `scripts/ring-zero-dependencies.json` are the
following full crates, with their default feature sets:

| Workspace area | Classification | Audit treatment | Reason |
| --- | --- | --- | --- |
| `tokimu-core` | Native Ring | Root | World, scheduling, time, commands, diagnostics, and math semantics. |
| `tokimu-input` | Native Ring | Root | ADR-0003 classifies normalized input as universal engine meaning. |
| `tokimu-runtime` | Native Ring | Root | ADR-0006 assigns application lifecycle and execution coordination to Tokimu. |
| `tokimu-assets` | Native Ring | Root | Asset identity, handles, lifecycle observations, and the provider-neutral loader contract. Its unused `anyhow` dependency was removed rather than admitted. |
| `tokimu-render`, `tokimu-platform`, `tokimu-wasm` | Outer adapter/composition | Not roots | They integrate renderer, OS, browser, and target mechanisms. Their dependencies are not thereby Native Ring sources. |
| `tokimu-rule`, `tokimu-ts-frontend` | Optional capability / authoring frontend | Not roots | Rule and frontend semantics are not universal engine meaning. |
| `tokimu` facade and corpus/test packages | Composition, evidence, or application code | Not roots | They consume the Native contracts but do not define the Ring 0 closure. |

No declared root currently has a build script or procedural macro edge. The
audit still follows those edge kinds so a future root change cannot bypass the
policy. Dev-only dependencies remain outside this build-source closure unless
they generate, rewrite, or link a Ring 0 artifact.

### Structural decision rule

Cargo resolves dependencies for compilation units, not individual modules. If
a crate mixes Native Ring contracts with an Outer Ring implementation, one of
the following must be true:

1. features structurally exclude the Outer dependency from the Ring 0 build;
2. the crate is split along the ownership boundary; or
3. the full linked dependency closure is audited as Ring 0.

Comments and intended call paths do not remove a linked dependency from the
trusted closure.

### Acceptance criteria

- [ ] A reviewer can reproduce the same Ring 0 roots without guessing from
      directory names.
- [ ] Every mixed crate has a structural classification or an explicit rework
      finding.
- [ ] The closure excludes unrelated lockfile packages and includes selected
      build/proc-macro edges.

## Slice 2: Build The Inventory And Audit Harness

### Deliverables

- [x] Add a machine-readable allowlist of approved Tokimu workspace roots and
      Ring 0 submodule source paths.
- [x] Add a script that invokes Cargo metadata and derives the relevant runtime,
      build, and proc-macro closure.
- [x] Record package name, version, source, feature set, target conditions, and
      dependency kind for every node.
- [x] Reject registry, remote Git, developer-local, and unapproved path sources.
- [x] Detect missing, wrong-revision, and dirty Ring 0 submodules.
- [ ] Produce a stable human-readable diff when the closure changes.
- [x] Add an audit-record template covering every ADR-0010 requirement.

### Acceptance criteria

- [ ] Running the harness against the current tree fails for the known registry
      sources and names all eight migration packages.
- [ ] `syn` 3.x is not falsely attributed to the `tokimu-core` closure.
- [x] A deliberately introduced unapproved source fails with an actionable
      diagnostic in a harness test (`scripts/test-audit-ring-zero-dependencies.ps1`).
- [x] A deliberately dirtied isolated Ring 0 submodule fails with an actionable
      diagnostic without changing the working checkout.
- [ ] The harness contains no hard-coded assumption that there will always be
      exactly eight packages or six repositories.

## Slice 3: Map Registry Packages To Exact Upstream Source

### Deliverables

- [x] Identify the canonical upstream repository for every migration package.
- [x] Find the exact commit corresponding to each current published package.
- [ ] Compare packaged source with the upstream tree and record exclusions,
      generated files, patches, and metadata differences.
- [x] Verify registry checksums against the current lockfile.
- [x] Record whether multiple packages share one upstream commit.
- [ ] Reject or investigate any package that cannot be mapped reproducibly.
- [ ] Record a mirror/fork strategy if upstream availability is insufficient for
      long-term source retrieval.

### Acceptance criteria

- [ ] Every package maps to one exact auditable commit or receives a rejection
      disposition.
- [ ] Package/repository differences are explained rather than assumed benign.
- [ ] No floating tag or branch is used as final identity.

## Slice 4: Audit `glam` And The Public Math Boundary

### Deliverables

- [ ] Audit the selected `glam` source, features, license, unsafe/SIMD code,
      target conditionals, allocations, panics, determinism, and build behavior.
- [ ] Measure source size, dependency count, compile cost, linked cost where
      meaningful, and representative math-path performance.
- [ ] Validate supported native and `wasm32-unknown-unknown` builds.
- [ ] Inventory every public Tokimu API that exposes a `glam` type.
- [ ] Compare retain-and-re-export, private-use-plus-wrapper, Tokimu-owned math
      representation, and Outer Ring conversion alternatives.
- [ ] Record serialization and authoring consequences of the chosen public
      representation.
- [ ] Write the retained dependency audit and disposition.

### Acceptance criteria

- [ ] The audit explicitly decides whether `glam` implementation and public
      representation are separate admissions.
- [ ] Performance evidence does not treat one local machine as a universal
      guarantee.
- [ ] If public re-export is retained, migration and upstream change risks are
      accepted explicitly.
- [ ] If wrapping or replacement is selected, a compileable migration sequence
      is defined before API changes begin.

## Slice 5: Audit `serde` And Serialization Ownership

### Deliverables

- [ ] Audit `serde`, `serde_core`, and `serde_derive` at the exact shared source
      revision.
- [ ] Inventory every Native Ring type that derives or exposes Serde traits.
- [ ] Decide whether serialized representation is a Native contract, an
      interchange mechanism, or an Outer Ring concern for each use.
- [ ] Compare derives, manual implementations, a narrower serialization seam,
      and moving scene document translation outward.
- [ ] Audit derive-generated behavior relevant to bounds, errors, recursion,
      unknown fields, defaults, and compatibility.
- [ ] Record native/WASM, compile-time, binary-size, and failure consequences.
- [ ] Write the retained dependency audit and disposition.

### Acceptance criteria

- [ ] “Convenient derives” are not used as the ownership argument.
- [ ] The chosen boundary states whether serialized field shape is stable API.
- [ ] Malformed and incompatible data behavior is tested at the owning boundary.
- [ ] Removal or reduction of derives remains a valid audit outcome.

## Slice 6: Audit The Procedural-Macro Closure

### Deliverables

- [ ] Audit `proc-macro2`, `quote`, `syn` 2.x, and `unicode-ident` at their exact
      source revisions and selected features.
- [ ] Inventory build scripts, generated tables/source, environment reads,
      target conditionals, unsafe code, and compiler-version assumptions.
- [ ] Trace how each package participates in `serde_derive` code generation.
- [ ] Compare the audit and update burden against eliminating or relocating the
      derive dependency.
- [ ] Record source size, compile cost, licenses, advisories, and MSRV/toolchain
      consequences.
- [ ] Write one audit record per upstream repository and a combined closure
      disposition.

### Acceptance criteria

- [ ] The audit covers executed build-time code, not only runtime-linked code.
- [ ] Generated output is attributable to the pinned source and toolchain.
- [ ] The full closure is accepted, reduced, or rejected as one coherent trust
      decision.

## Slice 7: Add Pinned Submodules And Local Cargo Resolution

Begin only for dependencies whose audits conclude they may remain in Ring 0.

### Deliverables

- [ ] Add each accepted upstream repository under `third-party/ring-0/` at its
      exact reviewed commit.
- [ ] Keep Ring 0 submodules visible to dirty-source checks.
- [ ] Use Cargo path dependencies for direct packages.
- [ ] Add local workspace patches for every accepted transitive package.
- [ ] Disable unneeded default features and pin the audited feature set.
- [ ] Update `Cargo.lock` and retain a dependency/source diff.
- [ ] Make missing submodule source fail explicitly without registry fallback.
- [ ] Verify a network-free Ring 0 build after toolchain and submodules exist.

### Acceptance criteria

- [ ] Cargo metadata reports only Tokimu workspace or approved Ring 0 submodule
      paths in the trusted closure.
- [ ] No registry or remote Git source remains in runtime/build/proc-macro edges.
- [ ] The submodules are clean and at parent-pinned commits.
- [ ] Native and WASM builds consume the same admitted semantic source closure.
- [ ] Each dependency migration lands as a small compileable increment.

## Slice 8: Enforce The Policy In CI

### Deliverables

- [x] Initialize required Ring 0 submodules explicitly in CI.
- [x] Run the metadata-based source audit before Ring 0 compilation.
- [x] Fail on missing, dirty, wrong-revision, registry, remote Git, or unapproved
      path source.
- [ ] Retain a readable dependency/feature diff when the closure changes.
- [ ] Run focused Native tests plus workspace validation from ADR-0008 and
      ADR-0009.
- [x] Add native and WASM source-selection checks.
- [x] Document the local developer command matching CI.

### Acceptance criteria

- [ ] CI cannot pass by fetching a registry substitute for a missing submodule.
- [ ] A submodule bump necessarily changes the parent repository diff.
- [ ] A feature-induced transitive dependency change is visible and fails until
      audited.
- [ ] Security-update exceptions remain bounded and greppable.

## Slice 9: Resolve Release And Publication Behavior

### Deliverables

- [ ] Inventory official binaries, source archives, and crates that contain or
      expose Ring 0 code.
- [ ] Ensure official binaries build from the pinned local source closure.
- [ ] Decide how source archives initialize or include Ring 0 submodules.
- [ ] Keep Ring 0 crates unpublished until downstream source identity is solved.
- [ ] Evaluate reviewed-source packaging, Tokimu-controlled audited releases,
      or another mechanism with equivalent evidence.
- [ ] Prove `cargo package`/publish behavior cannot silently replace local source
      while being described as ADR-0010 compliant.
- [ ] Record attribution and source-offer requirements in release artifacts.

### Acceptance criteria

- [ ] Every official artifact identifies the Ring 0 source revisions it uses.
- [ ] Clean release validation reproduces the audited source selection.
- [ ] Published consumers, if enabled, receive the reviewed closure rather than
      ordinary registry substitutions.
- [ ] If no compliant publication mechanism is selected, publication remains
      blocked explicitly.

## Slice 10: Close The Migration

### Deliverables

- [ ] Complete a final ADR-0008 performance and hygiene review.
- [ ] Complete a final ADR-0009 verification and resilience review.
- [ ] Run the source audit, focused tests, workspace validation, target builds,
      and applicable corpus checks.
- [ ] Update the SDD, dependency guidance, CI documentation, and accepted ADR
      summaries where the final boundary changes them.
- [ ] Record unresolved risks and reopening triggers.
- [ ] Mark each original migration package retained, wrapped, replaced, moved,
      or removed.
- [ ] Remove the temporary migration warning only after CI proves compliance.

### Acceptance criteria

- [ ] The machine-derived closure contains no unapproved source.
- [ ] Every accepted upstream repository has a current audit record.
- [ ] Every public foreign type exposure has explicit disposition.
- [ ] Native/WASM and clean-checkout validation pass.
- [ ] Official release and publication claims match actual source behavior.
- [ ] Tokimu can accurately claim ADR-0010 compliance.

## Dependency Disposition Matrix

Maintain this table during the audit. `Undecided` is the only valid initial
state; completing a source review does not force retention.

| Dependency/repository | Current role | Initial state | Final disposition | Evidence |
| --- | --- | --- | --- | --- |
| `glam` | Math implementation and public types | Undecided | Retain | `glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md` |
| `serde` family | Serialization traits and derives | Undecided | Move out of Ring 0 | Removal of Native Ring derives; baseline audit |
| `proc-macro2` | Derive build-time support | Undecided | Remove from Ring 0 | Removed with Native Ring derives; baseline audit |
| `quote` | Derive token generation | Undecided | Remove from Ring 0 | Removed with Native Ring derives; baseline audit |
| `syn` 2.x | Derive syntax parsing | Undecided | Remove from Ring 0 | Removed with Native Ring derives; baseline audit |
| `unicode-ident` | Derive identifier support | Undecided | Remove from Ring 0 | Removed with Native Ring derives; baseline audit |

Permitted final dispositions:

```text
retain    accepted in Ring 0 under pinned audit
wrap      implementation retained, public foreign API removed
replace   smaller or Tokimu-owned implementation selected
move      responsibility relocated behind an Outer Ring contract
remove    requirement and dependency eliminated
reject    source cannot meet Ring 0 admission requirements
```

## Validation Commands

The exact audit script name may be selected during Slice 2. The completed plan
should support a validation sequence equivalent to:

```powershell
git submodule status --recursive
git diff --submodule=log --check
cargo metadata --format-version 1 --locked
cargo tree -p tokimu-core -e all
cargo tree -p tokimu-runtime -e all
pwsh -NoProfile -File scripts/audit-ring-zero-dependencies.ps1
cargo test -p tokimu-core
cargo test -p tokimu-runtime
cargo build -p tokimu-core --target wasm32-unknown-unknown
cargo build -p tokimu-runtime --target wasm32-unknown-unknown
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Where supported after source initialization, the focused Ring 0 validation must
also succeed without network source retrieval:

```powershell
cargo test -p tokimu-core --locked --offline
cargo build -p tokimu-core --target wasm32-unknown-unknown --locked --offline
```

Long-running corpus, browser, backend, and hardware checks remain separately
invocable and must report executed, skipped, unavailable, or deferred status
honestly under ADR-0009.

## Risks And Mitigations

### Registry packages do not match an upstream checkout exactly

Mitigation: retain package/upstream diffs, identify generated or omitted files,
and reject mappings that cannot be reproduced confidently.

### Procedural-macro auditing dominates the effort

Mitigation: treat removing or relocating derives as a first-class alternative.
Do not keep a large build-time closure merely because the runtime code is small.

### Path patches change feature unification or package behavior

Mitigation: compare Cargo metadata, features, lockfile, generated code, tests,
and native/WASM outputs before and after each source redirect.

### Public `glam` types make replacement expensive

Mitigation: separate implementation retention from public vocabulary admission.
If wrapping is selected, migrate in compileable compatibility slices and avoid
adding further public exposure during the audit.

### Mixed crates make Ring 0 closure ambiguous

Mitigation: use feature-isolated builds or split the ownership boundary. Do not
pretend module-level intentions override package-level linkage.

### Submodule availability or upstream history changes

Mitigation: record canonical upstream and a Tokimu-controlled mirror/fork
strategy where availability risk is material. The parent gitlink remains exact.

### Security updates become slower

Mitigation: automate diff generation and advisory checks while preserving human
review. Use ADR-0005 provisional admission only for bounded urgent mitigation.

### Publication substitutes registry source downstream

Mitigation: keep Ring 0 crates unpublished until a reviewed distribution model
preserves source identity for consumers.

### The audit becomes checklist theater

Mitigation: retain concrete source diffs, measurements, tests, and disposition
decisions. Periodically prune audit steps that never affect a decision, while
updating ADR-0010 rather than weakening it through custom.

## Stop Conditions

Stop admission and choose replacement, relocation, or rejection when:

- exact source cannot be mapped or retrieved reproducibly;
- license or redistribution obligations are incompatible with Tokimu;
- opaque binaries or unauditable generated code enter the trusted closure;
- unsafe, build-time, macro, or transitive behavior cannot be bounded and
  explained;
- native/WASM behavior violates the admitted Ring 0 contract;
- Cargo cannot be prevented from substituting unreviewed source;
- a dependency owns specialized domain meaning that belongs in an Outer Ring;
- public API coupling creates more permanent cost than the dependency's value;
  or
- the full closure is too large or volatile for maintainers to audit honestly.

## First Useful Stopping Point

The audit has a useful intermediate stopping point after Slices 0-6 when:

- Ring 0 roots are explicit;
- the harness identifies the complete current closure;
- every current package maps to exact upstream source;
- `glam`, the `serde` family, and the procedural-macro repositories have
  retained audits and dispositions; and
- the project knows which dependencies will be retained, wrapped, replaced,
  moved, or removed.

At that point Tokimu has decision-quality evidence, even though source rewiring,
CI enforcement, publication, and final compliance remain incomplete. Do not
claim ADR-0010 compliance until Slices 7-10 are complete.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/testing-strategy.md`
- `docs/contribution-admission-guide.md`
- `docs/Tokimu Software Design Document.md`
- `Cargo.toml`
- `Cargo.lock`
