# AR-0015: Ring 0 Provenance Enforcement And Audit Closure

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-07 |
| Last reviewed | 2026-08-07 |
| Scope | Native Ring / build and release validation / cross-cutting |
| Trigger | ADR-0010 acceptance exposed a registry-resolved Native Ring closure and required machine-checkable provenance enforcement |
| Related ADRs | ADR-0003, ADR-0005, ADR-0008, ADR-0009, ADR-0010, ADR-0011 |
| Related evidence | `docs/Plans/ring-zero-third-party-source-audit-and-migration.md`; `docs/Dependency Audits/Ring 0/`; `scripts/audit-ring-zero-dependencies.ps1`; `scripts/ring-zero-dependencies.json`; `Cargo.toml`; `Cargo.lock` |
| Admission exception | None |

## Architectural Question

Can Tokimu continuously prove that every package entering a declared Ring 0
compilation root comes from a clean, parent-pinned, locally auditable source
tree—and retain enough evidence to close ADR-0010 honestly across local,
continuous-integration, release, and publication workflows?

## Context

ADR-0010 accepts a strict provenance policy for Native Ring third-party source:
the complete runtime, build, and procedural-macro closure must come from
Tokimu-controlled, parent-pinned submodules rather than a registry, remote Git,
or developer-local substitute. The policy applies to source identity and audit
evidence, not merely to package version selection.

At opening, `tokimu-core` directly used `glam` and Serde derives. Its full
closure contained eight registry packages: `glam`, `serde`, `serde_core`,
`serde_derive`, `proc-macro2`, `quote`, `syn` 2.x, and `unicode-ident`.
The initial repair removed unused Serde derives from Native Ring scene
documents, which removed the serialization and procedural-macro chain from the
declared Ring 0 closure. A subsequent root-boundary review added
`tokimu-input`, `tokimu-runtime`, and `tokimu-assets` to the declared roots;
the first two add no foreign source, and the unused `anyhow` dependency in the
asset loader contract was removed rather than admitted. `glam` remains because
it is the existing shared math representation used through `tokimu_core::math`
by rendering and multiple 3D corpus consumers.

Tokimu now pins `glam` 0.29.3 at
`d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` in
`third-party/ring-0/glam`, and Cargo resolves the workspace dependency from
that local source path with only the `std` feature selected. This is meaningful
progress, but it is not itself a complete claim of ADR-0010 compliance.

## Trigger And Evidence

- **Audit baseline:**
  `docs/Dependency Audits/Ring 0/migration-baseline-2026-08-07.md` retains the
  original eight-package closure, checksums, exact upstream mappings, and the
  distinction between `syn` 2.x in Ring 0 and unrelated `syn` 3.x elsewhere in
  the workspace.
- **Retained dependency audit:**
  `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
  records the exact commit, source-tree identity, selected feature set,
  public-vocabulary cost, source comparison, license, advisory check, unsafe
  surface, update process, and reopening triggers.
- **Machine-derived closure:**
  `scripts/audit-ring-zero-dependencies.ps1` obtains Cargo metadata for the
  configured `tokimu-assets`, `tokimu-core`, `tokimu-input`, and
  `tokimu-runtime` roots, follows non-dev runtime/build/proc-macro edges, and
  reports package, version, source, feature set, dependency kind, and target
  condition.
- **Provenance enforcement:** the script rejects registry, remote Git,
  unapproved local-path, missing-submodule, wrong-gitlink, dirty-submodule, and
  `.gitmodules`-ignored Ring 0 sources. It accepts only workspace members and
  configured Ring 0 submodule roots; being somewhere under the repository is
  not sufficient. The retained harness exercises both a normal accepted
  closure and isolated unapproved-path and dirty-submodule rejection cases.
- **Source comparison:** the cached `glam` 0.29.3 package `src/` tree is
  byte-identical to the pinned upstream `src/` tree. Cargo's normalized package
  metadata and registry-only files are tracked as packaging differences rather
  than silently treated as upstream source.
- **Validation:** `cargo test -p tokimu-core`,
  `cargo clippy -p tokimu-core --all-targets -- -D warnings`,
  `cargo build -p tokimu-core --target wasm32-unknown-unknown`, and
  `cargo test -p tokimu-core --locked --offline` succeeded after the repair.
- **Known risk:** the pinned `glam` revision emits upstream generated-swizzle
  `unused_attributes` warnings that Rust describes as future hard errors. The
  current compiler accepts them; the warning is not evidence of a clean future
  toolchain path.

The evidence proves one declared root's current local source selection. It does
not prove every future Native Ring root is configured, all release artifacts
initialize submodules, published consumers receive the same source, or the
retained source is free of defects.

## Ownership Analysis

ADR-0010 owns the binding source-provenance and audit policy. This review owns
the observed implementation conformance, gaps, and reopening criteria.

`tokimu-core` owns the declared Native Ring root and must not regain a registry
or macro dependency without reopening its closure evidence. Cargo workspace
configuration owns local source selection; the parent repository gitlink owns
the exact submodule revision. The audit script is build and CI tooling: it
derives evidence and rejects invalid source selection, but it does not own
engine semantics, dependency policy, or an application runtime.

Dependency audit records own source-specific evidence and dispositions. They
must not become a generic package manager, a substitute for security response,
or a claim that a source inspection proves correctness. Release and publication
behavior remain repository and distribution concerns, not Native Ring runtime
meaning.

## Dependency Direction

```text
Before the repair:

tokimu-core
    |
    v
crates.io package resolution
    |
    v
registry source outside Tokimu's pinned source graph

Current local repair:

tokimu-core
    |
    v
Cargo workspace dependency (explicit path and feature set)
    |
    v
third-party/ring-0/glam parent-pinned Git submodule

Audit script
    |
    +--> Cargo metadata closure
    +--> parent Git submodule and dirty-source state
    +--> configured approved source roots
    |
    v
retained audit report / CI decision

Required CI and release direction:

clean checkout + initialized Ring 0 submodules
    |
    +--> metadata-derived provenance audit
    +--> locked, offline Ring 0 build and tests
    |
    v
official artifact only when the audited local closure was selected
```

Neither `tokimu-core` nor the audit script may depend on registry source,
network availability, a developer-specific checkout, or a foreign build tool to
substitute a missing Ring 0 source during validation.

## Alternatives Considered

### Alternative A: Treat `Cargo.lock` As Sufficient Provenance

- Benefits: no submodules or dedicated audit tooling.
- Costs: pins package resolution and checksums but not the reviewed source in
  Tokimu's parent-pinned graph.
- Failure mode: a local or CI build can compile a registry package while being
  described as audited Ring 0 source.

### Alternative B: Use A Hand-Maintained Dependency List

- Benefits: simple initial documentation.
- Costs: misses feature-selected, procedural-macro, build, and future
  transitive edges.
- Failure mode: the list remains unchanged while the actual trusted closure
  expands.

### Alternative C: Use Only Vulnerability Or License Scanners

- Benefits: broad automated signals.
- Costs: does not establish source identity, semantic ownership, unsafe surface,
  selected features, or local Cargo source selection.
- Failure mode: a clean scanner result is mistaken for architectural admission.

### Alternative D: Replace Every Foreign Ring 0 Dependency

- Benefits: avoids foreign-source provenance after a successful rewrite.
- Costs: reimplements mature math behavior and expands Tokimu's maintenance and
  correctness burden without evidence that the replacement is safer.
- Failure mode: dependency cleanup introduces a larger unreviewed engine
  implementation.

### Alternative E: Continue The Current Narrow Provenance Review

- Benefits: derives the actual closure, pins the justified source, removes
  unjustified Native dependencies, and leaves open work visible.
- Costs: requires CI, release, publication, and update work before closure.
- Failure mode: a locally passing audit is incorrectly treated as a final
  compliance or security claim.

## Findings

- The original eight-package Ring 0 closure was not compliant with ADR-0010;
  the noncompliance was concrete registry resolution, not a documentation gap.
- The `serde` derive chain did not have a current Native Ring consumer beyond
  derives on scene documents. Removing it reduced the trusted closure by seven
  packages without changing scene compilation behavior.
- `glam` has real shared caller pressure and is now source-pinned, locally
  selected, feature-constrained, and accompanied by a retained audit record.
- The metadata-derived audit closes several common provenance escapes: broad
  local-path acceptance, hidden submodule changes, ignored submodule state, and
  registry or remote-Git substitution.
- A clean local source audit proves only the configured root and current
  checkout. It does not prove CI setup, release archives, downstream package
  publication, all future roots, upstream source safety, or compiler longevity.
- The current `glam` warning set is a concrete maintenance risk. It does not
  invalidate the exact source identity, but a future compiler can invalidate
  the selected revision and must reopen the dependency audit.

## Disposition

**Continue under review.** ADR-0010 remains the accepted policy, and the local
repair establishes a credible first implementation slice. This record remains
open until CI, clean-checkout/offline validation, official artifact behavior,
publication behavior, and sustained update handling prove that the policy is
enforced beyond one maintained working tree. No new ADR is required unless
those findings show that ADR-0010's source, publication, or trust model must
change.

## Consequences

- Native Ring dependencies must be declared through explicit local source paths
  and reviewed feature sets, even when other Outer Ring consumers use the same
  package in the workspace.
- Adding a new Native Ring root or source requires updating the machine-readable
  configuration and retaining closure evidence before the code can claim
  provenance compliance.
- The Serde family remains available to Outer Ring persistence, corpus, and
  frontend code; its removal here is not a workspace-wide serialization ban.
- `glam` updates require a parent gitlink change, source comparison, audit
  update, native/WASM validation, and review of the generated-code warning.
- A future public math wrapper or replacement remains possible, but must be a
  compatibility and performance migration with real caller evidence.

## Required Follow-Up

- [x] Establish the first metadata-derived provenance audit for `tokimu-core`.
- [x] Remove the unjustified Serde procedural-macro closure from Ring 0.
- [x] Pin and locally select the retained `glam` source with a dependency audit.
- [x] Add the provenance audit, submodule initialization, and locked offline
      validation to CI (`.github/workflows/ring-zero-provenance.yml`); its first
      hosted run remains evidence to retain before this review can close.
- [x] Add automated negative fixtures proving unapproved and dirty sources are
      rejected with actionable diagnostics.
- [ ] Validate the complete current workspace and official artifact paths do
      not substitute a registry `glam` source.
- [ ] Decide and implement a compliant source-distribution strategy before any
      Ring 0 crate publication.
- [ ] Establish a monitored update or fork decision before `glam`'s current
      compiler warnings become an error.

## Reopening Triggers

- A new Native Ring root, feature, package, build script, or procedural macro
  enters the configured closure.
- Cargo metadata reports a registry, remote Git, missing, dirty, ignored, or
  unapproved local source in a Ring 0 validation.
- A clean checkout, offline validation, CI job, release archive, or publication
  path cannot reproduce the local source selection.
- A relevant advisory, license change, source-tree mismatch, unsafe/FFI change,
  or target-specific behavioral difference affects `glam`.
- The current `glam` warning becomes an error or a newer compatible pinned
  revision changes its selected source or feature closure.
- A proposed math wrapper, replacement, or additional public foreign type
  changes the current `glam` public-vocabulary decision.

## Review History

### Cycle 1 -- 2026-08-07

- Status entering review: Proposed.
- New evidence: ADR-0010 accepted; baseline identified eight registry-resolved
  Ring 0 packages; exact upstream commits were mapped; the Serde derive chain
  was removed from `tokimu-core`; `glam` was source-pinned and locally selected;
  the metadata-derived audit and focused native/WASM/offline validation passed.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: source provenance can be checked mechanically for the declared
  `tokimu-core` root, but the result is local implementation evidence rather
  than complete CI, release, publication, or security closure.
- Disposition: continue under review.
- Resulting ADR or documentation change: no new ADR; ADR-0010 implementation
  evidence is retained in the dependency audits, migration plan, and this
  review record.

### Cycle 2 -- 2026-08-12

- Status entering review: Under Review.
- New evidence: a proposed maintenance update for the pinned `glam` warning was
  investigated rather than applied. The upstream fix first appears in release
  0.30.2; relative to the audited 0.29.3 pin it changes approximately 170 files
  (`29,090` insertions and `10,156` deletions), including generated SIMD and
  swizzle code. The exact pin was restored to
  `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` after inspection.
- Findings: ADR-0010's update gate materially prevented a toolchain-warning
  cleanup from becoming an unreviewed Ring 0 implementation update. Passing
  compilation would not be sufficient evidence for that source delta.
- Disposition: keep the warning/update item open. A future update must create
  a revision-specific dependency audit, review the complete diff and closure,
  and rerun native/WASM, performance, and verification evidence. AR-0019 may
  separately use the recurring audit cost when comparing retained-provider and
  Tokimu-owned math alternatives.
- Resulting ADR or documentation change: no ADR or gitlink change; retained
  provenance/update evidence only.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/ADR/ADR-0011-ring-based-security-authority-and-trust-boundaries.md`
- `docs/Plans/ring-zero-third-party-source-audit-and-migration.md`
- `docs/Dependency Audits/Ring 0/migration-baseline-2026-08-07.md`
- `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
- `scripts/audit-ring-zero-dependencies.ps1`
- `scripts/ring-zero-dependencies.json`
