# Tokimu Architectural Review Records

Architectural Review Records preserve architectural questions, corpus pressure,
evidence, findings, dispositions, and reopening criteria.

They occupy the space between informal observations and binding Architecture
Decision Records.

```text
Observation or corpus pressure
    ↓
Architectural Review Record
    ↓
Findings and disposition
    ├── incubate / defer / reject / no change
    └── accept architectural change
            ↓
       ADR or deliberate ADR revision
```

## Relationship To Other Documents

Each document type has one job:

| Document | Owns |
| --- | --- |
| Architectural Review Record | Question, evidence, analysis, findings, disposition, and reopening triggers |
| ADR | An accepted, binding architectural decision and its consequences |
| SDD or subsystem design document | The current intended architecture and contracts |
| Audit | Observed implementation conformance, defects, and risks |
| TODO | Work sequencing and completion tracking |
| Corpus example | Executable architectural or implementation evidence |

An architectural review does not override an accepted ADR. When a review finds
that an ADR should change, the review remains the evidence record and the ADR is
superseded or revised deliberately.

## When A Review Record Is Required

Create a review record when one or more of these apply:

- corpus pressure suggests promoting application behavior into Tokimu;
- ownership between kernel, foundational service, capability, backend, and
  frontend is unclear;
- a new crate, subsystem, cross-layer dependency, or stable semantic contract is
  proposed;
- repeated implementation friction suggests that an existing boundary is wrong;
- an accepted decision may need to be reopened;
- a concept, crate, or contract may need retirement, relocation, or merger;
- a proposal is deferred or rejected but the reasoning should remain durable.

Small local refactors, implementation choices that preserve accepted contracts,
and ordinary bug fixes do not need architectural review records.

## Naming

Records use a stable, independent sequence:

```text
AR-0001-short-title.md
AR-0002-short-title.md
```

The sequence is independent from ADR numbering. A review may produce no ADR,
one new ADR, or a revision/supersession of an existing ADR.

Do not reuse a retired number. Copy `TEMPLATE.md` when opening a record and add
the new record to the index below.

## Statuses

Use one of these statuses:

- **Proposed** -- the question and initial evidence have been recorded.
- **Under Review** -- alternatives and ownership are actively being evaluated.
- **Incubating** -- the direction is plausible, but more examples or consumers
  are required before disposition.
- **Accepted** -- the findings require an architectural decision. The record
  must link the resulting ADR or explicit ADR revision.
- **Deferred** -- no decision is justified now; reopening triggers are recorded.
- **Rejected** -- the proposal should not proceed under the reviewed evidence.
- **No Change** -- the review confirmed the existing architecture.
- **Superseded** -- a later review replaced this record's active findings.
- **Reopened** -- new evidence has started another review cycle. This is a
  temporary active status until a new disposition is recorded.

Closed statuses are not claims of eternal truth. They describe the disposition
supported by the evidence available at that review cycle.

## Required Content

Every review record must contain:

1. the architectural question;
2. the trigger that caused review;
3. concrete evidence, including relevant corpus examples or tests;
4. ownership and dependency analysis;
5. alternatives considered;
6. findings, including uncertainties;
7. a disposition;
8. required follow-up;
9. explicit reopening triggers;
10. an append-only review history.

Evidence should distinguish observation from guarantee. A screenshot may prove
that one backend rendered correctly once; it does not prove a cross-backend
rendering contract.

When a review uses the ADR-0005 maintainer exception process, it must also name:

- which normal evidence is missing;
- which evidence is being substituted;
- why mechanically gathering the missing evidence would add little decision
  value;
- who is accountable for the decision;
- which observable result would reopen it.

## Dispositions

A review should end with one of these outcomes:

- admit or revise architecture through an ADR;
- continue incubation against named corpus pressure;
- defer until named trigger conditions occur;
- reject the proposal with reasons;
- confirm the current boundary without change;
- retire, merge, or relocate an existing concept through an ADR when binding
  architecture changes.

"Discussed" is not a disposition. If evidence is insufficient, use
**Incubating** or **Deferred** and say what evidence is missing.

## Reopening A Review

Corpus pressure can reopen any closed review.

Reopening must preserve the old findings. Do not rewrite the original evidence
or disposition as though it never existed. Instead:

1. change the record status to **Reopened**;
2. append a new review cycle under `Review History`;
3. identify the new evidence and which reopening trigger it satisfies;
4. reassess ownership, alternatives, and consequences;
5. record the new disposition;
6. create or revise the relevant ADR if binding architecture changes.

If the new evidence substantially changes the question's scope, create a new AR
and mark the earlier record **Superseded** rather than stretching one record
across unrelated decisions.

## Index

- [AR-0001: Shared Vector Presentation Geometry](AR-0001-shared-vector-presentation-geometry.md)
  — Deferred; shared ownership validated, first-party capability promotion
  awaits a production or independent tool consumer
- [AR-0002: Native Execution and Multithreading Ownership](AR-0002-native-execution-and-multithreading.md)
  — Accepted; resulted in ADR-0006
- [AR-0003: XML Document Boundary](AR-0003-xml-document-boundary.md)
  — Incubating
- [AR-0004: Networking Transport Seam](AR-0004-networking-transport-seam.md)
  — Deferred; bounded observation transport remains example-side until a real
  provider or non-example consumer proves capability admission
- [AR-0005: Runtime Observation and Performance Telemetry](AR-0005-runtime-observation-and-performance-telemetry.md)
  — Incubating; narrow kernel performance diagnostics are accepted by ADR-0007
  while aggregation, resource attribution, and broader observation ownership
  remain under review
- [AR-0006: Raster Image Requirement Pipeline](AR-0006-raster-image-requirement-pipeline.md)
  — Under Review; encoded-format, decoded-image, presentation-requirement, and
  renderer-resource ownership is being studied before capability admission
- [AR-0007: Semantic UI Composition Boundary](AR-0007-semantic-ui-composition-boundary.md)
  — Incubating; semantic composition ownership is validated by native and WASM
  consumers, while first-party package extraction awaits provider separation
- [AR-0008: Audio Observation And Visualizer Boundary](AR-0008-audio-observation-and-visualizer-boundary.md)
  — Incubating; deterministic PCM analysis and bounded overload evidence are
  established while capture providers, shader resources, and capability
  admission remain unproven
- [AR-0009: Resource Store Identity And Kernel Boundary](AR-0009-resource-store-identity-and-kernel-boundary.md)
  — Incubating with provisional admission under ADR-0005; a reversible
  foundational identity, folder, navigation, and store contract may gather
  evidence while permanent and kernel admission remain under review
- [AR-0010: Weaver XSLT Resource Resolver Boundary](AR-0010-weaver-xslt-resource-resolver-boundary.md)
  — Proposed; a bounded adapter may test Weaver's explicit URI resolver against
  selected Tokimu Resource Space bytes without admitting XSLT or URI policy
  into Tokimu
- [AR-0011: Tosumu-Backed Tasset Canonical Asset Output](AR-0011-tosumu-backed-tasset-canonical-asset-output.md)
  — Incubating; Tosumu is the preferred first storage provider for a bounded
  canonical `.tasset` study while Tokimu retains asset semantics and runtime,
  interchange, migration, and portability boundaries remain explicit
- [AR-0012: Bundled Native Default Font Provider](AR-0012-bundled-native-default-font-provider.md)
  — Accepted with provisional admission under ADR-0005; resulted in an ADR-0004
  revision selecting Departure Mono as a replaceable first-party native default
- [AR-0013: Observation Shell And Ratatui Presentation Provider](AR-0013-observation-shell-and-ratatui-presentation-provider.md)
  — Incubating; separates provider-neutral observation-session semantics from
  Ratatui terminal and embedded cell-grid presentation mechanics
- [AR-0014: Native Terminal Text Surface And Ratatui Dependency Boundary](AR-0014-native-terminal-text-surface-and-ratatui-dependency-boundary.md)
  — Incubating; measures Ratatui's real dependency cost and studies whether a
  smaller provider-neutral terminal surface exists below shell semantics
- [AR-0015: Ring 0 Provenance Enforcement And Audit Closure](AR-0015-ring-zero-provenance-enforcement-and-audit-closure.md)
  — Under Review; records local ADR-0010 provenance enforcement, the retained
  `glam` source decision, and the CI, release, publication, and update evidence
  still required for closure
- [AR-0016: Native Ring Performance And Code Quality Conformance](AR-0016-native-ring-performance-and-code-quality-conformance.md)
  — Under Review; records evidence that ADR-0008's proportional gate changes
  real Native Ring decisions and retains its measurement and hygiene gaps
- [AR-0017: Ring-Based Verification And Recovery Conformance](AR-0017-ring-based-verification-and-recovery-conformance.md)
  — Under Review; records ADR-0009 verification evidence and the negative,
  containment, and recovery proof still required at future boundaries
- [AR-0018: Ring-Based Security, Authority, And Trust Conformance](AR-0018-ring-based-security-authority-and-trust-conformance.md)
  — Under Review; records ADR-0011's build-provenance application while runtime
  authority, hostile-input, and isolation evidence remains open
- [AR-0019: Native Math Vocabulary And Foreign-Type Boundary](AR-0019-native-math-vocabulary-and-foreign-type-boundary.md)
  — Incubating; retains audited `glam` 0.33.3 with ADR-0014 Narrow B while
  continuing the owned-vocabulary study and reusable foreign-type admission method
- [AR-0020: TypeScript Authoring Boundary And Corpus Conformance](AR-0020-typescript-authoring-boundary-and-corpus-conformance.md)
  — Under Review; classifies TypeScript corpus roles and studies enforceable
  TTSDD conformance before promoting the boundary into a binding ADR
- [AR-0021: Geometry Orientation And Facing Conformance](AR-0021-geometry-orientation-and-facing-conformance.md)
  — Incubating; records independent E1M1 and decoded `Box.glb` workbench
  inside-out-facing observations and requires a bounded renderer-orientation
  contract before any global engine conclusion
- [AR-0022: Textured Mesh Coordinate And Sampling Boundary](AR-0022-textured-mesh-coordinate-and-sampling-boundary.md)
  — Accepted; resulted in ADR-0012 admitting checked supplied UVs and declared
  point/linear plus clamp/repeat sampling, while alpha/depth stays in AR-0023
- [AR-0023: Textured Surface Alpha And Depth Policy](AR-0023-textured-surface-alpha-and-depth-policy.md)
  — Accepted in part; ADR-0013 admits caller-declared categorical Cutout while
  continuous Blend remains incubating pending an explicit ordering/depth contract
- [AR-0024: Renderer Failure Observation And Diagnostic Boundary](AR-0024-renderer-failure-observation-and-diagnostic-boundary.md)
  — Accepted; establishes that the AR-0023 empty frame was valid GPU clipping,
  not an uncaught backend error, keeps GL-style Tokimu camera meaning while
  adapting explicitly to WebGPU depth, and retains application-owned resource
  identity/terminal policy without a shared allocator or record owner
- [AR-0025: Comparative Camera Candidate-Selection And Visibility Study](AR-0025-camera-candidate-selection-and-visibility-culling.md)
  — No Change; retains full submission as the renderer fallback, keeps generic
  selection unadmitted, and leaves Doom source protocols provider-local
- [AR-0026: Non-Euclidean Spatial Charts And Authored Angular Topology](AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md)
  — Incubating; explores non-Euclidean simulation as first-class Tokimu meaning
  through locally Euclidean charts, explicit transitions, and authored angular
  deficit/excess without yet assigning Ring placement or public contracts
- [AR-0027: Diagnostic Error Presentation And Standard Error Texture](AR-0027-diagnostic-error-presentation-and-standard-error-texture.md)
  — Accepted as corpus/application-local policy; the Purple stand-in remains
  explicit evidence machinery, while automatic fallback, shared diagnostic
  visual vocabulary, and a standard Tokimu error texture are not admitted
- [AR-0028: Coordinate-Frame Handedness And Directional Conformance](AR-0028-coordinate-frame-handedness-and-directional-conformance.md)
  — No Change; retains the Doom-local orientation-preserving Preserve North
  adapter and comparison controls without admitting a Tokimu-wide cardinal-axis
  convention
- [AR-0029: Camera, View, And Projection Construction Ownership](AR-0029-camera-view-and-projection-construction-ownership.md)
  — Accepted; ADR-0014 admits the demonstrated three-family Narrow B seam while
  keeping `glam::camera` private
- [AR-0030: Tokimu Render Preparation And Submission Framework](AR-0030-source-owned-presentation-preparation-boundary.md)
  — Under Review; must select a reliable Tokimu-render-specific submission
  strategy through Doom, Quake, ordinary retained-3D, and large or multi-view
  campaign pressure while keeping preparation algorithms program-owned
- [AR-0031: Conservative Spatial-Query Capability](AR-0031-conservative-spatial-query-capability.md)
  — Incubating; portable BVH mechanics have native/WASM and E1M1 lifecycle
  evidence, while Ring 2 admission awaits an independent non-Doom consumer
- [AR-0032: Atomic Staged Render Resource-Set Replacement](AR-0032-atomic-staged-render-resource-set-replacement.md)
  — Under Review; semantic, live WGPU, and repeated-pressure evidence support
  narrow set-level transaction semantics while integrated stale rejection,
  physical reclamation, and final API/handle shape remain explicit gates or
  non-decisions
