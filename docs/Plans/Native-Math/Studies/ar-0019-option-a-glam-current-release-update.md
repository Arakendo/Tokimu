# AR-0019 Option A Current `glam` Update And Effort Study

| Field | Value |
| --- | --- |
| Status | Complete — Pause outcome recorded; candidate fully reviewed but not admitted; production pin unchanged |
| Owner | Tokimu maintainers |
| Date | 2026-08-12 |
| Related review | `AR-0019-native-math-vocabulary-and-foreign-type-boundary.md` |
| Related ADRs | ADR-0005, ADR-0008, ADR-0009, ADR-0010, ADR-0011 |
| Current pin | `glam` 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Intake target | `glam` 0.33.3 at release commit `9928729066db87d97fa779e129469721a289beae` |
| Submodule | `third-party/ring-0/glam` |
| Existing audit | `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md` |

## Purpose

Execute AR-0019 Alternative A's normal maintenance path: update Tokimu's
audited, locally pinned Ring 0 `glam` source from 0.29.3 to the current stable
release identified at plan intake, while retaining the existing five-type
public vocabulary and selected `std`-only feature policy.

The update is also a measured lifecycle study. It must retain the actual labor,
automation time, source-review breadth, failures, rework, and maintainer
judgment required by ADR-0010. The result should let AR-0019 compare Option A's
recurring update burden with C0/C1's implementation and maintenance burden
without reducing either alternative to line count or subjective irritation.

On 2026-08-12, the live crates.io index reported 0.33.3 as current. Its
canonical GitHub release tag `0.33.3` is an annotated tag whose peeled commit is
`9928729066db87d97fa779e129469721a289beae`. Registry metadata reports
`MIT OR Apache-2.0`, Rust 1.68.2, and the canonical repository
`https://github.com/bitshifter/glam-rs`. These are intake facts to verify again,
not substitutes for the revision-specific audit.

## Governing Rule

This is a Ring 0 source change even if public Tokimu code compiles without
modification. Passing tests alone cannot admit it.

```text
upstream release
    -> immutable source identity
    -> complete selected closure and source-delta review
    -> public compatibility + correctness + target evidence
    -> performance/code-quality + security/failure evidence
    -> clean local pin, lockfile, CI, attribution, and audit update
    -> maintainer accept or reject
```

The study must not:

- change Tokimu's public math vocabulary;
- add a new `glam` feature because the release makes it convenient;
- consume a registry or remote-Git copy in the Ring 0 closure;
- edit the submodule source locally as the accepted state;
- suppress new warnings merely to make the update appear clean;
- bundle unrelated Tokimu refactors with the provider update; or
- treat elapsed automation time as maintainer review time.

## Questions To Answer

1. Can 0.33.3 be reconciled to one immutable upstream commit and locally pinned
   source tree?
2. Does `default-features = false, features = ["std"]` still provide the same
   selected closure and the five public types Tokimu exposes?
3. What source, generated-code, unsafe/SIMD, feature, build, legal, and security
   changes occurred between the two admitted candidates?
4. Which Tokimu callers or tests require source changes, and are those changes
   mechanical compatibility repairs or semantic changes?
5. Does the update clear the known generated-swizzle warning without creating
   new strict-warning failures?
6. Do native, actual browser/WASM, correctness, failure, compile-time,
   binary-size, and caller-shaped performance observations remain acceptable?
7. How much active effort and review judgment does ADR-0010 require for this
   real update?
8. Is the completed burden proportionate, or does it materially change the
   future case for Alternative A or ADR-0010's update policy?

## Scope

### In scope

- upstream 0.29.3-to-0.33.3 source and metadata comparison;
- exact submodule gitlink, workspace dependency version, and lockfile update;
- full review of the selected Ring 0 closure;
- the existing five public re-exports: `Mat4`, `Quat`, `Vec2`, `Vec3`, `Vec4`;
- strict native and WASM validation;
- actual browser execution for bounded retained math controls;
- representative camera, CAD/GLB, DOOM observer, transform, collision, picking,
  rendering, and AR-0026 chart controls where already available;
- warning, compile-time, output-size, and caller-shaped performance comparison;
- license, attribution, advisory, unsafe, generated-source, and target review;
- rollback proof and a revision-specific Ring 0 audit; and
- measured level-of-effort evidence for AR-0019.

### Out of scope

- replacing or wrapping `glam`;
- expanding the public foreign vocabulary;
- adopting new integer, f64, size, serde, mint, rand, bytemuck, or other
  optional feature families;
- editing upstream implementation, accepting a fork, or redesigning Tokimu
  math semantics;
- selecting Option C from the outcome of this update alone;
- weakening ADR-0010; and
- unrelated dependency upgrades or workspace cleanup.

## Level-Of-Effort Method

### Time categories

Retain time per work session in a ledger. Record start/end timestamps and round
only after calculating the duration.

| Category | Includes | Excludes |
| --- | --- | --- |
| Mechanical | Fetching, pinning, manifest/lock edits, command setup | Source interpretation |
| Source review | Changelog, diff, unsafe/SIMD, generated code, features, closure, build behavior | Unattended scanner time |
| Integration repair | Tokimu compile/API/test repairs caused by the update | Unrelated cleanup |
| Validation review | Interpreting correctness, target, performance, warning, and failure results | Unattended build time |
| Documentation | New audit, attribution, AR-0019 result, rollback and evidence records | Prose unrelated to admission |
| Maintainer decision | Final risk review and accept/reject judgment | Agent-generated draft time |
| Automation wall time | Builds, tests, scanners, downloads | Active review while they run |
| Blocked/rework | Tool failures, source-identity mismatch, reverted approaches, repeated work | Expected first-pass validation |

Every row must name the slice, actor (`maintainer`, `agent`, or `automation`),
active minutes, wall-clock minutes, files or source area reviewed, outcome, and
whether the work would recur on a future update.

```text
| Timestamp | Slice | Actor | Active min | Wall min | Scope | Outcome | Recurring? |
```

### Predeclared effort bands

Classify the completed update using active maintainer-plus-agent time. Report
automation and blocked time separately.

| Band | Active effort | Interpretation for this single update |
| --- | ---: | --- |
| Routine | <= 4 hours | Bounded update with little judgment or repair |
| Moderate | > 4 to 12 hours | Material audit/integration work, still manageable |
| High | > 12 to 24 hours | Significant recurring Ring 0 maintenance cost |
| Exceptional | > 24 hours | Update burden requires explicit proportionality review |

These bands are study conventions, not universal engineering estimates. The
final report must also retain source-delta size, manually reviewed files/areas,
unsafe/generated-code changes, integration failures, test reruns, and
maintainer decision time so a fast agent-assisted update is not mistaken for a
small trust review.

### Burden indicators

Record counts without collapsing them into one gamified score:

- upstream commits and releases crossed;
- changed files and lines, separated into implementation, generated source,
  tests/benches/docs, and metadata;
- added/removed/changed unsafe blocks, intrinsics, target cfgs, and SIMD paths;
- selected-feature and dependency-closure changes;
- build script, proc-macro, generated artifact, environment, filesystem, or
  network behavior changes;
- public/API/compiler errors and Tokimu files repaired;
- correctness, target, warning, and performance failures;
- validation retries and why they were necessary;
- review questions requiring maintainer judgment; and
- rollback operations and evidence.

## Work Slices

### Slice 0: Freeze A Clean Baseline

- [x] Commit or otherwise isolate all pre-existing Option C and unrelated
      work; do not mix it with the `glam` gitlink update.
  - [x] Create an ignored detached worktree for candidate compilation without
        changing or committing the dirty production worktree.
- [x] Verify the parent worktree and Ring 0 `glam` submodule are initialized,
      pinned to 0.29.3, and clean.
- [x] Capture Rust/Cargo versions, host/target details, submodule identity,
      selected features, Cargo metadata closure, and current audit-script output.
- [x] Run and retain the pre-update warning count, strict Clippy outcome, and
      narrow native/WASM test/build baseline.
- [x] Retain the pre-update selected caller and performance controls from a
      clean isolated baseline.
  - [x] Retain parent transform/stereo throughput, core clean/incremental build
        time, core output size, native layout, and plain-WASM controls on the
        same host/toolchain used for the candidate.
- [x] Start the LOE ledger before any target-source work.

Acceptance:

- [x] The upgrade diff can be separated exactly from existing work through the
      ignored detached candidate worktree.
- [x] Baseline failures and the known `unused_attributes` warning flood are
      recorded rather than attributed to 0.33.3 later.

### Slice 1: Freeze And Reconcile The Target Identity

- [x] Re-query crates.io immediately before execution. If a newer stable
      release exists, record it and require a maintainer choice to keep 0.33.3
      or revise this plan before fetching a different target.
- [x] Verify tag object `fae5594033edc8cf0f385a1bcbbab5205eabe7df`
      peels to commit `9928729066db87d97fa779e129469721a289beae`.
- [x] Verify package version, `.cargo_vcs_info.json`, source tree, checksum,
      license, repository, MSRV, and release/tag identity agree.
- [x] Retain old/new commit, tree, crate archive, and upstream diff references.
- [x] Reject a mutable ref or unexplained registry-versus-repository mismatch.

Acceptance:

- [x] One immutable commit is the sole candidate for the submodule gitlink.
- [x] The audit can reproduce how the registry package relates to that commit.

### Slice 2: Changelog, Closure, And Source-Delta Triage

- [x] Read every upstream release note and migration note from 0.29.3 through
      0.33.3; list breaking, semantic, representation, target, and MSRV changes.
- [x] Produce initial whole-delta, source, unsafe, and target/intrinsic
      statistics.
- [x] Complete the categorized file-review manifest and selected-path manual
      review.
- [x] Compare Cargo manifests and resolved features/dependencies under Tokimu's
      exact `std`-only selection.
- [x] Compare build scripts, proc macros, generated files, prebuilt artifacts,
      environment/filesystem/network behavior, and package layout.
- [x] Inventory added, removed, and changed unsafe, FFI, inline assembly,
      intrinsic, target-cfg, SIMD, allocation, threading, global-state, panic,
      and I/O behavior.
- [x] Identify generated source separately while keeping it inside the review
      burden; generation is not an exemption from Ring 0 review.

Acceptance:

- [x] The selected runtime closure remains locally auditable and explained;
      the separately pinned generator and its development-tool closure are
      retained as review burden rather than misclassified as runtime code.
- [x] Review manifests identify what was inspected and what automated evidence
      supplemented—but did not replace—the source review.

### Slice 3: Candidate Pin And Mechanical Integration

- [x] Move only `third-party/ring-0/glam` to the reviewed release commit.
  - [x] Move only the disposable isolated worktree's `glam` checkout to the
        candidate; leave the production gitlink unchanged.
- [x] Update the workspace dependency version while retaining
      `default-features = false, features = ["std"]`.
  - [x] Apply and verify that exact manifest change in the isolated worktree.
- [x] Update the lockfile without introducing another `glam` source or version.
  - [x] Update and verify the isolated lockfile; Cargo changed only local
        `glam` 0.29.3 to 0.33.3.
- [x] Run Cargo metadata and Ring 0 audit scripts before code repair; retain the
      raw closure difference.
  - [x] Retain the isolated `std`-only one-node closure and the audit's expected
        unpinned-parent violation.
- [x] Compile the narrowest Ring 0/core targets and classify every failure as
      upstream API, Tokimu misuse, changed semantics, toolchain, or unrelated.
  - [x] Compile/test core native and WASM, then classify camera/projection
        deprecations as foreign-vocabulary pressure rather than a compile break.
- [x] Repair only authorized compatibility defects in Tokimu source, keeping
      semantic changes visible and separately reviewed.
  - [x] Apply no compatibility repair before AR-0029 recorded the ownership
        finding and alternatives.
  - [x] Prototype AR-0029 Alternative B for only look-at, GL-depth perspective,
        and GL-depth orthographic construction in the disposable worktree;
        retain six contract tests and representative native/WASM compile evidence.

Acceptance:

- [x] Cargo consumes the initialized local submodule only.
- [x] No feature or public-re-export expansion is hidden in integration repair.
- [x] Every Tokimu source edit has a named 0.29.3-to-0.33.3 cause.

### Slice 4: Public Vocabulary And Numerical Compatibility

- [x] Confirm the same five names and no additional foreign type/trait are
      publicly exposed.
  - [x] In the isolated AR-0029 prototype, repository scanning finds exactly
        the five existing `glam` re-exports; `glam::camera` appears only inside
        the private constructor implementation.
- [x] Compare sizes, alignments, field/column observations, public constants,
      constructors, operators, conversion behavior, and documented semantics
      used by real callers—without promoting observed representation to a new
      Tokimu guarantee.
- [x] Run the retained AR-0019 differential, degenerate, singular, finite,
      projection, inverse, quaternion, transform, and chart controls.
  - [x] Execute all 44 isolated native math-study tests against the 0.33.3
        candidate, including affine differential sweeps, inverse round trips,
        finite camera/projection sweeps, and singular/degenerate observations.
  - [x] Compare a dependency-isolated 14-line native API/behavior fingerprint
        against exact local 0.29.3 and 0.33.3 sources. Layouts, fields,
        constants, conversions, operators, transforms, inverse observations,
        quaternion rotation, and non-finite masks match exactly.
  - [x] Execute the retained plain-WASM Alternative A transform/inverse,
        stereo-camera, and alignment control against both pins; all five
        bounded observations match exactly.
- [x] Exercise representative camera, transform hierarchy, CAD/GLB, picking,
      collision, DOOM observer, orientation, and renderer-boundary paths.
  - [x] Retain strict compilation for migrated native/browser renderer callers,
        the pinned GLB controls, 41 Doom collision/orientation tests, and 13
        directional camera/picking tests. No tolerance was widened.
- [x] Record changed floating-point results or tolerances as findings; do not
      widen tolerances merely to pass the update.

Acceptance:

- [x] Existing Tokimu semantic claims remain true or an architectural finding
      returns to AR-0019 before admission.
- [x] Compatibility evidence covers actual public/corpus callers, not only
      upstream unit tests.

### Slice 5: ADR-0008 Performance And Code-Quality Gate

- [x] Verify the known generated-swizzle warning is gone under strict Clippy.
  - [x] Isolated strict core/render and representative caller Clippy clears the
        prior 4,896 generated diagnostics without suppression.
- [x] Treat any new warning, duplicate implementation, shadow source of truth,
      unbounded work, avoidable allocation, or target-specific complexity as a
      review finding rather than suppressing it.
- [x] Compare clean and incremental compile time, relevant output/binary size,
      allocations, and representative caller-shaped throughput/tail latency
      against Slice 0 on the same host/profile/toolchain.
- [x] Separate noise-sensitive observations from material regressions and
      repeat measurements enough to support the stated conclusion.
- [x] Inspect material performance changes down to the relevant upstream path
      before accepting or rejecting them.
  - [x] Investigate the initial 2.2x stereo-camera observation, identify the
        caller's overwritten default construction and duplicate provider
        normalization, retain both repairs, and rerun three release samples.
  - [x] Resolve the checked-boundary blocker after repaired in-process controls
        show no throughput penalty; retain the approximately seven-percent
        cross-build difference as noise-sensitive rather than claiming parity.

Acceptance:

- [x] No performance claim exceeds its measured workload.
- [x] A material Native or WASM regression has a disposition and cannot be
      waived by the convenience of clearing warnings.

### Slice 6: ADR-0009/0011 Failure, Security, And Target Gate

- [x] Run the complete native test suite and malformed/degenerate boundary
      controls relevant to math construction and consumption.
  - [x] Attempt the complete isolated workspace suite, initialize its missing
        Departure Mono fixture, and prove the next `AssetLoader::Error` failure
        occurs identically on the unchanged 0.29.3 production tree.
  - [x] Re-run the complete workspace suite after the unrelated
        `hello-resource-space` and `resource-space-assets` baseline defects are
        resolved outside this update. Both production 0.29.3 and isolated
        candidate 0.33.3 suites pass locked/offline with warnings suppressed
        only for this test execution.
- [x] Build/test the available WASM targets and attempt the retained actual
      browser/DOM math and AR-0026 chart controls, retaining unavailable
      execution honestly under the selected Pause outcome.
  - [x] Execute the dependency-isolated plain-WASM Alternative A control under
        Node for both 0.29.3 and 0.33.3; retain browser/DOM as distinct
        remaining evidence.
  - [x] Release-build the same control with WebAssembly `simd128` enabled and
        execute both pins under Node 22.22.2. All five bounded observations
        match exactly; this is SIMD WebAssembly-engine evidence, not an actual
        browser-SIMD claim.
- [x] Retain unavailable NVIDIA/other-target gaps without generalizing from AMD
      or browser observations.
- [x] Recheck RustSec and other approved advisory sources with date/method;
      inspect upstream security notes and issue references relevant to the diff.
  - [x] Refresh the official RustSec web intake and inspect upstream soundness
        fixes; no named `glam` advisory was returned.
  - [x] Install workspace-local `cargo-audit` 0.22.2 and scan both lockfiles
        against the same 1,216-advisory RustSec database snapshot. Both report
        the same two `quick-xml` vulnerabilities and four non-`glam` warnings;
        no candidate-specific or `glam` advisory appears.
- [x] Reconfirm licenses, notices, patent/redistribution obligations, and any
      attribution changes.
- [x] Review unsafe/SIMD/representation changes for authority, memory safety,
      determinism, target parity, panic, and failure-containment consequences.
  - [x] Complete selected-source manual review and retain the specific
        `Affine3`, quaternion array, sign-mask, scalar/matrix division, SSE2,
        core-SIMD, and tuple-layout findings.
  - [ ] Execute unavailable NEON, wasm64, and SIMD-enabled browser paths rather
        than inferring them from x86-64 and scalar/plain-WASM evidence.
  - [x] Execute the SIMD-enabled WebAssembly control in Node; retain NEON,
          wasm64, and actual-browser SIMD as separate unavailable targets.
    - [x] Execute all 12 candidate WASM conformance tests under Node both with
          default target features and with `simd128`; both suites pass.
    - [x] Install the Windows AArch64 target and compile-check both pins for its
          NEON-enabled target configuration. Execution remains unavailable on
          the x86-64 host, so this is compile-only evidence.
    - [x] Attempt stable wasm64 target installation and retain the explicit
          toolchain result: stable has no prebuilt standard-library artifact
          for `wasm64-unknown-unknown`; no source-built substitute was used.
    - [x] Attempt direct `tokimu-core` WASM test execution and retain the
          harness limitation: the crate compiles, but its ordinary Rust tests
          are not exported to `wasm-bindgen-test-runner` (zero tests observed),
          so this is not counted as core WASM test execution.

Acceptance:

- [x] Native and actual WASM/browser evidence are distinguished from compile-only evidence.
- [x] No advisory, legal, unsafe, or failure finding is silently converted into
      a routine integration task.

### Slice 7: Enforcement, Clean Checkout, And Rollback

- [x] Run the Ring 0 dependency audit against the candidate and verify exact
      gitlink, clean submodule, selected features, and offline local-source closure.
  - [x] In a fresh disposable worktree, stage the exact candidate gitlink and
        replayed integration patch; the audit passes with only local `glam`
        0.33.3 and feature `std` in the selected closure.
- [x] Validate a clean checkout/submodule initialization path without registry
      or remote Git substitution for Ring 0 source.
  - [x] The uninitialized control first failed explicitly while canonical Git
        access was unavailable, then cloned the exact parent submodule from the
        declared canonical URL when access was authorized. Cargo subsequently
        resolved the staged candidate through the local submodule path only.
- [x] Run formatting, strict workspace Clippy, workspace tests, locked/offline
      validation, and relevant website/browser builds.
  - [x] Run `cargo fmt --all -- --check` in the isolated candidate and replay
        control. The clean control required the exact nested code-generator
        checkout because Cargo formatting traverses upstream's workspace; this
        remains tooling burden, not selected runtime closure.
  - [x] Pass locked/offline core tests and metadata plus focused strict Clippy,
        native caller, plain-WASM, and browser-build gates.
  - [x] Re-run complete workspace tests after the unrelated
        `AssetLoader::Error` baseline is repaired; both revisions pass. The
        candidate also passes strict whole-workspace Clippy. Production
        0.29.3 retains its measured 4,896 upstream generated-source warnings;
        Tokimu-owned math-study lint findings were repaired independently.
  - [x] Attempt the already-built browser fixture after restarting the harness;
        retain the absence of an attachable browser as an explicit unavailable
        target rather than substituting Node or blocking completion of Pause.
    - [x] Verify an official workspace-local Node 22.22.2 control and the
          configured Codex Node 24.14 runtime both satisfy the harness minimum.
    - [x] Update the machine-wide Node 22.x installation from 22.21.0 to the
          official 22.23.2 x64 MSI, verify its published SHA-256 and OpenJS
          Foundation signature, and retain the failed non-admin/elevated retry
          as Option A tooling effort.
    - [x] Restart the browser bridge/session so it consumes the configured
          runtime. The restarted session accepts Node 22.23.2 and the candidate
          server reaches HTTP readiness; browser discovery then reports zero
          available browser instances, so actual WebGPU execution remains a
          distinct environment gap.
- [x] Demonstrate rollback by restoring the parent manifest/lock/gitlink to the
      recorded 0.29.3 state in a disposable branch/worktree or equivalent
      recoverable control, then reapply the candidate deterministically.
- [x] Confirm release attribution and source-initialization documentation remain correct.
  - [x] Both license texts are byte-identical, the declared license is
        unchanged, and the existing canonical submodule URL remains the source
        initialization mechanism.

Acceptance:

- [x] The candidate is reproducible from the parent revision and fails clearly
      when required source is absent or dirty.
- [x] Rollback does not depend on an unrecorded local cache or mutable ref.

### Slice 8: Revision Audit, LOE Report, And Maintainer Decision

- [x] Create a new revision-specific audit named for the reviewed candidate
      commit; do not rewrite the 0.29.3 audit to imply it covered new source.
- [x] Update the old audit with a candidate-study pointer while retaining it as
      the production audit; no supersession is claimed under Pause.
- [x] Produce an Option A update report containing the completed time ledger,
      burden indicators, failures/rework, automated wall time, active effort
      band, and recurring-versus-one-time cost assessment.
- [x] Update AR-0019 with one result: admit 0.33.3, reject and retain 0.29.3,
      or pause with explicitly missing evidence.
- [x] State whether the measured burden changes the future case for A, C, or
      ADR-0010 proportionality. Do not revise those decisions inside this plan.
- [x] Do not commit a source-pin movement under Pause. Production manifests,
      lockfile, and gitlink remain on the accepted 0.29.3 revision; the
      disposable candidate state remains evidence only.

Acceptance:

- [x] A reviewer can distinguish mechanical upgrade work, trust review,
      validation, documentation, automation, blocked time, and final judgment.
- [x] The update is never described as easy or hard solely from elapsed time,
      diff size, test success, or subjective frustration.
- [x] The selected pause is a complete, evidence-bearing outcome with named
      owners and resumption conditions; it does not imply admission or rejection.

## Planned Evidence Artifacts

- `docs/Dependency Audits/Ring 0/glam-9928729066db87d97fa779e129469721a289beae.md`
  if that exact candidate is admitted or fully audited;
- `corpus/lib/tokimu-math-study/results/<date>-option-a-glam-update-baseline.md`;
- `corpus/lib/tokimu-math-study/results/<date>-option-a-glam-update-source-review.md`;
- `corpus/lib/tokimu-math-study/results/<date>-option-a-glam-update-validation.md`;
- `corpus/lib/tokimu-math-study/results/<date>-option-a-glam-update-loe.md`;
- Cargo metadata/feature/closure comparisons and scanner outputs with commands;
- native/WASM/browser and caller-shaped performance observations; and
- AR-0019 cycle entry with the maintainer disposition.

## Stop And Escalation Conditions

Return to the maintainer before continuing if:

- a newer target would replace the frozen 0.33.3 candidate;
- registry and canonical-source identities cannot be reconciled;
- the selected closure gains a runtime, build, proc-macro, or remote source;
- the `std`-only feature no longer supplies the existing vocabulary;
- license, attribution, MSRV, build behavior, unsafe/SIMD, or generated-source
  changes exceed what the audit can explain confidently;
- compatibility repair would change stable Tokimu semantics or public API;
- native/WASM results diverge materially;
- a material performance, correctness, security, or failure regression remains unresolved;
- an upstream patch/fork is proposed; or
- active effort reaches the Exceptional band before the remaining work and
  decision value are re-estimated.

## Completion Criteria

The plan is complete when one of these evidence-bearing outcomes is recorded:

1. **Admit current A:** Tokimu builds Ring 0 from the exact locally pinned
   0.33.3 commit, all gates pass, the new audit is accepted, rollback is proven,
   and the complete LOE report is retained.
2. **Reject current A update:** Tokimu remains on audited 0.29.3, the candidate
   and reason are bounded, no partial source/manifest state remains, and the
   incurred LOE is retained as AR-0019 evidence.
3. **Pause:** the exact missing evidence, risk, owner, and resumption condition
   are recorded without describing either revision as newly admitted.

None of these outcomes selects Option C or changes ADR-0010 automatically.

After one of these outcomes is recorded, create a separate AR-0019 Alternative
B plan and run it as an independent study. That follow-up must distinguish:

- **Narrow B:** Tokimu owns only corpus-pressured semantic construction such as
  camera/view/projection behavior while the existing five foreign math types
  and `glam` implementation remain;
- **Full B:** Tokimu owns wrappers for the five public math types while `glam`
  remains the private numerical implementation.

AR-0029 and this update's 86-site vocabulary finding become inputs to that
study. They do not pre-authorize Full B, count as the completed B experiment,
or interrupt the current plan before its A disposition and LOE are recorded.

## References

- `docs/Architectural Reviews/AR-0019-native-math-vocabulary-and-foreign-type-boundary.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/ADR/ADR-0011-ring-based-security-authority-and-trust-boundaries.md`
- `docs/Plans/Native-Math/Studies/ar-0019-option-c-owned-math-and-bulk-compute.md`
- `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
- `https://crates.io/crates/glam/0.33.3`
- `https://github.com/bitshifter/glam-rs/tree/9928729066db87d97fa779e129469721a289beae`
