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
