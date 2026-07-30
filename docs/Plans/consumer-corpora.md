# Consumer Corpora

## Status

| Field | Value |
| --- | --- |
| Status | Proposed |
| Scope | Downstream application-shaped corpus entries that validate public or incubating Tokimu contracts through realistic composition |
| Source discussion | `docs/Conversations/Consumer Corpora.md` |
| Related governance | `corpus/README.md`, `docs/testing-strategy.md`, `docs/contribution-admission-guide.md` |
| Related decisions | ADR-0003, ADR-0005 |

## Purpose

Tokimu already has focused architectural corpus entries and data corpora:

- focused entries ask whether one behavior or boundary can be expressed
  naturally;
- data corpora ask whether an implementation survives representative external
  input;
- contract and integration tests prove bounded automated assertions.

A remaining gap is application-shaped evidence from the point of view of a
downstream consumer.

A **consumer corpus** asks:

> Can an application compose Tokimu's intended contracts into a useful,
> coherent program without privileged access, duplicated engine mechanics, or
> ownership leakage?

Consumer corpora should expose friction that focused examples cannot:

- several individually sound APIs may compose poorly;
- lifecycle requirements may be inconsistent across capabilities;
- provider selection may leak into application meaning;
- native and WASM paths may require incompatible application structures;
- diagnostics may exist but be difficult for an application or tool to consume;
- an incubating boundary may appear stable in isolation but fail under broader
  application pressure.

The goal is not to build product applications inside the repository. The goal
is to make downstream use observable before Tokimu asks external users to find
the same problems.

## Core Distinction

Consumer corpora are related to, but not interchangeable with, the existing
validation layers.

| Validation form | Primary question |
| --- | --- |
| Unit or contract test | Does this bounded semantic contract behave correctly? |
| Focused architectural corpus | Can Tokimu express this one behavior through the intended boundary? |
| Data corpus | Can this implementation survive representative real-world input? |
| Consumer corpus | Can a downstream application compose several contracts cleanly as a consumer? |
| Production consumer | Does an independently owned application actually rely on this contract? |

A repository-owned consumer corpus remains corpus evidence. It does **not**
become a production or independent non-example consumer merely because it uses
public APIs.

This distinction must remain explicit in Architectural Reviews and admission
decisions.

## Governing Principles

### Consume Before Reaching Through

A consumer corpus should use the same API surface available to its declared
consumer tier. It must not reach into private modules, renderer internals, or
foreign provider objects merely to complete the demonstration.

When the intended API cannot express the application, record the failure as
evidence before adding an example-local escape hatch.

### Applications Own Meaning

The consumer application owns its domain state and intent.

Tokimu may own:

- kernel and runtime semantics;
- capability contracts;
- foundational presentation contracts;
- provider-neutral diagnostics;
- adapter entry points.

The consumer must not move application truth into presentation, transport,
asset, or platform adapters to make composition easier.

### Composition Is The Subject

A focused corpus entry should stop after proving one seam. A consumer corpus
deliberately composes several already-proven seams and studies the interaction
between them.

New low-level behavior should first receive a focused proof unless the behavior
only exists at a composition boundary.

### Friction Is An Artifact

Repeated glue code, lifecycle translation, provider leakage, and diagnostic
blind spots are evidence. Consumer corpora should record this evidence rather
than hide it behind local helper layers.

### Consumer Evidence Is Labeled

Every consumer corpus entry must state whether it consumes:

- stable public Tokimu contracts;
- provisional or incubating contracts from `corpus/lib`;
- backend-specific APIs for an explicitly backend-focused proof.

These tiers provide different architectural evidence and must not be reported
as equivalent.

### Ordinary Runs Remain Deterministic Where Possible

Consumer corpora should control time, randomness, input, assets, and transport
when those inputs are not the subject of the proof.

Interactive presentation may remain available, but important state transitions
should also be observable through deterministic reports or tests.

## Consumer Tiers

### Tier 1: Public Consumer

A public consumer depends only on published-intent workspace crates and their
public APIs.

It may use:

- the `tokimu` facade;
- public first-party capability crates;
- explicitly supported provider crates;
- its own application code and assets.

It must not depend on `corpus/lib`.

This tier provides the strongest repository-owned evidence about API
composition, but it still does not satisfy a requirement for an independently
owned production consumer.

### Tier 2: Incubating Consumer

An incubating consumer may depend on a library under `corpus/lib` to pressure
an unresolved semantic boundary.

It must:

- identify each incubating dependency;
- state which semantics are under review;
- avoid presenting those APIs as stable Tokimu contracts;
- link repeated pressure to an Architectural Review when appropriate.

This tier is useful before promotion, especially when several focused corpus
entries have converged on the same candidate boundary.

### Tier 3: Provider Consumer

A provider consumer intentionally exercises a concrete adapter or external
technology.

It must keep provider-specific behavior below the Tokimu-owned semantic
boundary and identify any provider object that crosses into application code.

Provider consumers validate integration. They do not allow the provider to
define Tokimu semantics.

## Proposed Location

Introduce a dedicated directory only after the first entry proves the category
is useful:

```text
corpus/
  consumers/
    README.md
    headless-observer/
    native-workbench/
    wasm-dashboard/
```

Until Slice 2 is complete, the first consumer may incubate as a normal root
corpus entry. This avoids creating a permanent directory category before one
real consumer proves its shape.

Consumer entries should use purpose-oriented names rather than `hello-*` names
once they intentionally validate multi-capability composition.

## Consumer Record

Each consumer corpus entry should include a `DESIGN.md` containing:

- application purpose;
- primary composition claim;
- consumer tier;
- Tokimu crates and public surfaces consumed;
- incubating libraries or provider APIs consumed;
- application-owned state;
- lifecycle and target assumptions;
- deterministic inputs and observable outputs;
- expected diagnostics;
- known application-side glue;
- architectural friction;
- non-goals;
- success and failure criteria.

An initially hand-authored record is preferred over inventing a manifest schema.
Structured metadata should be added only after at least two entries need the
same fields programmatically.

## Initial Consumer Profiles

The first profiles should maximize architectural contrast rather than merely
increase example count.

### Headless Observer

Composition:

- kernel state;
- runtime scheduling;
- deterministic time;
- commands or rules;
- diagnostics;
- serialization or report output.

Primary pressure:

> Can a useful Tokimu application execute, observe, and report state without a
> window, GPU, or live renderer?

This should be the first implementation because it gives a deterministic,
platform-light baseline and tests the world-first architecture directly.

### Native Interactive Workbench

Composition:

- runtime;
- platform window;
- input;
- rendering;
- assets;
- presentation;
- diagnostics.

Primary pressure:

> Can an interactive tool keep application truth separate from presentation
> while composing several first-party services?

This profile should reuse a small existing corpus workload rather than invent a
new domain application.

### WASM Dashboard

Composition:

- public facade;
- browser platform adapter;
- normalized input;
- presentation;
- diagnostics or observation output.

Primary pressure:

> Can the same application-owned state and presentation intent survive the WASM
> target without a parallel application architecture?

Visible browser validation may remain a target-specific corpus job. Compile and
deterministic semantic checks should remain separately runnable.

### Asset Inspection Tool

Composition:

- asset identity and loading;
- one or more import providers;
- presentation or rendering;
- diagnostics;
- optional saved evidence.

Primary pressure:

> Can a tool inspect heterogeneous imported assets without making provider
> formats part of application truth?

This profile can eventually compose the SVG, glTF/GLB, CGM, and FBX corpus work,
but it must not become a universal asset viewer during the initial plan.

### Networked Observation Consumer

Composition:

- application-owned snapshot;
- replication envelope;
- codec;
- loopback or real transport provider;
- diagnostics.

Primary pressure:

> Can application meaning cross the networking seam while transport remains
> unaware of the payload domain?

This remains a later profile because the networking boundary is still deferred
and incubating.

## Evidence To Collect

Each consumer should produce a concise evidence record containing:

- build target and feature set;
- direct dependency list;
- public versus incubating API usage;
- provider choices;
- lifecycle stages exercised;
- deterministic scenario result;
- emitted diagnostics;
- application-side glue that appears reusable;
- APIs bypassed or reached through;
- unsupported behavior encountered;
- runtime or presentation measurements when relevant.

The initial record may be Markdown plus test output. A shared JSON schema is
deferred until repeated automation needs justify it.

## Composition Matrix

The corpus should not attempt every possible combination. Start with a bounded
matrix that provides independent pressure:

| Concern | Headless observer | Native workbench | WASM dashboard | Asset inspector | Network observer |
| --- | --- | --- | --- | --- | --- |
| Kernel state | Required | Required | Required | Optional | Required |
| Runtime lifecycle | Required | Required | Required | Optional | Required |
| Rendering | None | Native | Browser | Native or CPU evidence | None |
| Input | Scripted | Native | Browser | Optional | Scripted |
| Assets | Optional | Required | Optional | Required | None |
| Diagnostics | Required | Required | Required | Required | Required |
| Persistence/output | Report | Optional | Browser-visible | Evidence artifacts | Snapshot trace |
| Incubating APIs allowed | No | Initially possible | Initially possible | Likely | Yes |

Pairwise contrast is more valuable than multiplying nearly identical
applications.

## First Composite Candidate

`corpus/consumers/aspnet-wasm-asset-workbench` is the first concrete candidate
for the consumer category. It combines the WASM dashboard and asset inspector
profiles:

- ASP.NET 10 owns static hosting;
- TypeScript owns browser interaction and drag/drop;
- Rust/WASM owns bounded file classification and importer execution;
- Tokimu and incubating corpus providers own semantic observations;
- the browser canvas owns pixels produced from provider-neutral preview data.

The entry is a **Tier 2 incubating consumer** while it depends on SVG, CGM,
glTF/GLB, and FBX libraries under `corpus/lib`. It must not be reported as a
public importer API or an independent production consumer.

Its first proof is byte transfer and observable importer diagnostics. SVG and
CGM may additionally lower into provider-neutral contour previews. GLB/glTF and
FBX rendering remains explicitly pending until their scene and mesh boundaries
can be consumed without exposing provider-native records as application truth.

## Implementation Slices

### Slice 1: Inventory Existing Consumer-Shaped Entries

#### Deliverables

- [ ] Review current corpus entries for multi-capability application
      composition.
- [ ] Classify candidates as focused architectural, data-driven, public
      consumer, incubating consumer, or provider consumer.
- [ ] Identify entries that currently mix more than one claim without recording
      a composition purpose.
- [ ] Record candidate reuse for the initial headless and native consumers.
- [ ] Add a short inventory section to this plan.

#### Acceptance Criteria

- [ ] Every proposed initial consumer is tied to existing code or a clearly
      missing composition proof.
- [ ] No existing entry is renamed or moved merely to satisfy the new taxonomy.
- [ ] The inventory distinguishes repository-owned corpus evidence from
      independent production use.

### Slice 2: Build The First Headless Public Consumer

#### Deliverables

- [ ] Create one deterministic application-shaped corpus entry using only
      public first-party APIs.
- [ ] Compose runtime state, scheduling, diagnostics, and report output.
- [ ] Keep domain state application-owned.
- [ ] Add a `DESIGN.md` using the consumer record fields.
- [ ] Add automated checks for the deterministic final state and diagnostics.

#### Acceptance Criteria

- [ ] The consumer runs without a window, GPU, or renderer.
- [ ] The consumer has no dependency on `corpus/lib`.
- [ ] The same scenario produces the same semantic result across repeated runs.
- [ ] The application does not reach into private crate modules.
- [ ] Any missing public composition seam is recorded rather than patched
      through an internal dependency.

### Slice 3: Establish Consumer Corpus Organization

#### Deliverables

- [ ] Evaluate whether the first consumer is materially different from focused
      `hello-*` entries.
- [ ] If justified, create `corpus/consumers/README.md`.
- [ ] Document naming, tier labels, completion records, and admission rules.
- [ ] Move only entries whose primary claim is genuinely downstream
      composition.
- [ ] Update `corpus/README.md` and `docs/testing-strategy.md`.

#### Acceptance Criteria

- [ ] The new directory, if created, has a responsibility not already owned by
      root corpus entries.
- [ ] Focused examples remain easy to discover.
- [ ] Consumer corpora are not described as production consumers.
- [ ] Paths and workspace membership remain consistent after any move.

### Slice 4: Add A Native Interactive Consumer

#### Deliverables

- [ ] Select a small existing application workload.
- [ ] Compose runtime, platform, input, rendering, assets, presentation, and
      diagnostics through declared APIs.
- [ ] Separate application state updates from presentation lowering.
- [ ] Add deterministic logic tests independent of the live window.
- [ ] Record frame and presentation diagnostics without turning the consumer
      into a benchmark.

#### Acceptance Criteria

- [ ] The live application remains responsive under its intended workload.
- [ ] Rendering does not mutate application truth.
- [ ] Presentation or provider internals do not leak into the application state
      model.
- [ ] Important behavior can be validated without manual visual inspection.
- [ ] Performance warnings remain explicit and attributable to an owning stage.

### Slice 5: Add Target And Feature Pressure

#### Deliverables

- [ ] Define a minimal supported feature matrix for the first two consumers.
- [ ] Compile the headless consumer with rendering-independent features.
- [ ] Add a WASM compile check for one suitable consumer.
- [ ] Record target-specific unsupported behavior explicitly.
- [ ] Avoid target branches in application meaning where adapter selection is
      sufficient.

#### Acceptance Criteria

- [ ] Headless compilation does not pull window or GPU dependencies through an
      accidental feature path.
- [ ] The WASM check uses the same application-owned semantic model as its
      native counterpart.
- [ ] Unsupported target behavior produces deterministic diagnostics.
- [ ] Feature combinations are bounded and documented rather than exhaustively
      multiplied.

### Slice 6: Add Dependency And Boundary Guardrails

#### Deliverables

- [ ] Add a check that Tier 1 consumers do not depend on `corpus/lib`.
- [ ] Add a check or review step for private-path and backend-object leakage.
- [ ] Record direct dependency graphs for reviewed consumer entries.
- [ ] Identify duplicate application-side glue across consumers.
- [ ] Route repeated boundary pressure into an Architectural Review instead of
      immediately extracting a shared crate.

#### Acceptance Criteria

- [ ] A Tier 1 dependency violation fails validation clearly.
- [ ] Provider-specific dependencies are visible in the consumer record.
- [ ] Reusable glue is supported by at least two distinct consumers before
      shared extraction is proposed.
- [ ] Existing ADR ownership boundaries remain intact.

### Slice 7: Add Consumer Evidence Reporting

#### Deliverables

- [ ] Produce one concise report per consumer run or validation job.
- [ ] Include target, features, dependencies, scenario result, diagnostics, and
      known friction.
- [ ] Keep deterministic semantic evidence separate from native screenshots and
      manual observations.
- [ ] Link findings to the relevant plan, Architectural Review, or ADR.
- [ ] Evaluate whether repeated report fields justify a structured schema.

#### Acceptance Criteria

- [ ] A maintainer can identify the first failing composition boundary from the
      report.
- [ ] Reports distinguish stable public APIs from incubating dependencies.
- [ ] Reports do not claim production-consumer evidence.
- [ ] Normal validation does not modify reviewed golden evidence.

### Slice 8: Architectural Review

#### Deliverables

- [ ] Review evidence from at least one headless and one interactive consumer.
- [ ] Identify boundaries confirmed by composition.
- [ ] Identify repeated glue, ownership leaks, and missing lifecycle contracts.
- [ ] Decide whether consumer corpora remain a testing convention or need
      reusable support.
- [ ] Record any capability admission pressure in the owning Architectural
      Review.

#### Acceptance Criteria

- [ ] Findings are based on observed consumer evidence rather than hypothetical
      application needs.
- [ ] No new foundational service or capability is admitted solely because a
      consumer corpus is large.
- [ ] Any promotion proposal names the repeated semantics and their owner.
- [ ] Reopening triggers are recorded for deferred findings.

## First Working Milestone

The first milestone is complete when:

- existing consumer-shaped corpus entries have been inventoried;
- one deterministic headless consumer uses only public first-party APIs;
- its dependencies and application-owned state are documented;
- it emits a stable result and diagnostics;
- it reveals whether a dedicated `corpus/consumers/` category adds clarity.

This milestone intentionally does not require a native workbench, WASM runtime,
asset inspector, or shared consumer harness.

## Failure Semantics

Consumer failures should identify the owning boundary where possible:

```text
application scenario
    -> semantic state
    -> capability contract
    -> provider or adapter
    -> observable output
```

Useful failure classes include:

- public API cannot express required composition;
- lifecycle ordering is ambiguous;
- provider selection leaks into application meaning;
- target or feature is unsupported;
- deterministic scenario diverges;
- diagnostics are missing or unactionable;
- application must duplicate engine-owned semantics;
- an incubating dependency is mistaken for a stable contract.

The first stage whose contract cannot be satisfied is the owning diagnostic
boundary. A visually incorrect final result is evidence, not sufficient
localization by itself.

## Risks And Mitigations

### Consumer Corpora Become Product Applications

Mitigation:

- keep each entry tied to one composition claim;
- use small application domains;
- reject feature work that does not increase architectural evidence.

### Consumer Count Becomes A Vanity Metric

Mitigation:

- prefer contrasting profiles over duplicate applications;
- require a distinct composition question for each entry;
- report coverage by boundary, target, and provider rather than raw entry count.

### Repository-Owned Consumers Are Treated As Independent Adoption

Mitigation:

- label all consumer tiers explicitly;
- preserve production-consumer requirements in ADRs and reviews;
- never count a consumer corpus as externally owned merely because it uses
  public APIs.

### Shared Helpers Hide API Friction

Mitigation:

- keep the first consumer direct;
- record glue before extracting it;
- require repeated use before adding shared consumer support.

### Consumer Validation Becomes Slow Or Hardware-Bound

Mitigation:

- keep deterministic headless checks separate;
- label native, browser, and hardware jobs;
- avoid requiring interactive runs in the default workspace test tier.

### Target Matrix Grows Without Bound

Mitigation:

- select representative pairwise combinations;
- add a target only when it pressures a named boundary;
- document unsupported combinations explicitly.

## Acceptance Criteria

This plan succeeds when:

- [ ] Tokimu has a documented distinction between focused, data, consumer, and
      production evidence.
- [ ] At least one headless public consumer composes multiple Tokimu contracts.
- [ ] At least one interactive consumer pressures application/presentation
      separation.
- [ ] Consumer tiers and dependencies are visible and enforceable.
- [ ] Important scenarios produce deterministic semantic evidence.
- [ ] Consumer findings feed Architectural Reviews without automatically
      promoting abstractions.
- [ ] The repository structure reflects the category only if real entries prove
      that the distinction improves ownership and discoverability.

## Graduation Criteria

Consumer corpora remain a corpus convention unless repeated implementation
proves reusable engine-owned meaning.

A shared consumer harness or first-party capability may be proposed only when:

- at least two contrasting consumers require the same semantic contract;
- the contract cannot be expressed cleanly through an existing capability;
- extraction removes duplicated engine-owned meaning rather than ordinary
  application glue;
- native and relevant WASM ownership remain coherent;
- an Architectural Review identifies the owner, dependency direction, and
  reopening triggers.

Consumer corpus evidence can support promotion, but it cannot manufacture the
independent production consumer required by an existing admission decision.

## References

- `docs/Conversations/Consumer Corpora.md`
- `corpus/README.md`
- `docs/example-philosophy.md`
- `docs/testing-strategy.md`
- `docs/contribution-admission-guide.md`
- `docs/Architectural Reviews/README.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
