# Tokimu And Tosumu Reciprocal Website Evidence

## Status

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | 2026-08-04 |
| Last updated | 2026-08-04 |
| Owners | Tokimu and Tosumu maintainers |
| Tokimu target | `website/` and website consumer corpus |
| Tosumu target | `third-party/tosumu/docs/` |
| Related plans | [Tokimu Website](tokimu-website.md), [Consumer Corpora](consumer-corpora.md), [Tosumu .NET Resource Space Consumer Migration](tosumu-dotnet-resource-space-consumer-migration.md) |
| Related reviews | AR-0009, AR-0011, Tosumu AR-0001 and AR-0002 |

## Purpose

Tokimu and Tosumu should help each other's public documentation demonstrate
real cross-project composition. Each website remains an independently useful
MkDocs site, while selected lab pages consume versioned evidence produced by
the peer project.

The governing claim is:

> **Each project owns its meaning. The peer project may provide mechanisms,
> observations, or execution evidence without redefining that meaning.**

This plan covers both directions:

```text
Tosumu facts and durable observations
                |
                v
       Tokimu presentation
                |
                v
       visual evidence on either site

Tokimu Resource Space requests
                |
                v
       Tosumu persistence
                |
                v
       durable evidence on either site
```

## Primary Composition Claims

### Tokimu Helps Tosumu

Tosumu can use Tokimu as a presentation provider for bounded storage evidence:

- topology and page-allocation diagrams derived from Tosumu observations;
- WAL, checkpoint, and recovery timelines;
- TQL command/result explorers using versioned JSON envelopes;
- integrity, freshness, and diagnostic-state visualizations;
- accessible vector, text, chart, and interactive inspection controls;
- deterministic screenshots and structural artifacts for documentation.

Tokimu must receive provider-neutral observations or an explicitly documented
Tosumu schema. It must not parse Tosumu pages, infer storage truth from pixels,
or become an alternate database inspector.

### Tosumu Helps Tokimu

Tokimu can use Tosumu as a durability provider for bounded application evidence:

- persisted Resource Space trees, folders, resources, visibility, and metadata;
- save, close, reopen, and compare sessions;
- durable `.tasset` or project-artifact experiments;
- provenance, integrity, and retained-byte observations;
- history or synchronization evidence after Tosumu admits those contracts;
- fixtures shared between native, .NET, and future browser consumers.

Tosumu must store Tokimu-owned semantic records without defining Resource Space,
asset, scene, material, or application meaning.

## Ownership Boundary

| Layer | Owns | Must not own |
| --- | --- | --- |
| MkDocs site | Prose, navigation, stable URLs, static fallback, accessibility | Engine or database semantics |
| Tokimu | Presentation, interaction, Resource Space meaning, provider-neutral diagnostics | Tosumu pages, WAL internals, storage policy |
| Tosumu | Durable storage, integrity, recovery, public inspection facts | Tokimu resources, UI meaning, rendering |
| Evidence adapter | Versioned mapping between one public contract and one artifact schema | A second semantic model or hidden fallback |
| TypeScript | Island lifecycle and browser interaction | Parsing Tosumu storage or redefining Tokimu observations |
| Browser | DOM, Canvas, file selection, fetch, cache | Project semantics |

The dependency direction is always downward through public contracts:

```text
site-specific explanation
        |
        v
versioned evidence manifest
        |
        +--> Tokimu presentation adapter
        |
        +--> Tosumu public observation/provider adapter
```

Neither website may import the other website's generated HTML, navigation,
theme, or build configuration.

## Evidence Exchange Contract

The first shared boundary is a checked-in or reproducibly generated artifact,
not a network service:

```json
{
  "schema_version": 1,
  "producer": "tosumu",
  "producer_revision": "...",
  "fixture_id": "resource-space-basic-v1",
  "capability": "tql-status",
  "observation": {},
  "diagnostics": [],
  "limitations": []
}
```

Every exchanged artifact must record:

- schema version;
- producer and producer revision;
- stable fixture identity and provenance;
- capability or command identity;
- observation payload;
- typed diagnostics and explicit limitations;
- generation command or retained generation report;
- content fingerprint where practical.

Consumers reject unknown schema versions. They do not guess, scrape terminal
text, or silently substitute hand-authored sample data.

## Initial Lab Matrix

| Lab | Host site | Meaning owner | Provider | First useful proof |
| --- | --- | --- | --- | --- |
| TQL Observation Explorer | Tosumu | Tosumu | Tokimu presentation | Render admitted TQL JSON outcomes with command, facts, diagnostics, and schema version |
| Storage Lifecycle Timeline | Tosumu | Tosumu | Tokimu presentation | Visualize init, put, check, checkpoint, reopen, and failure observations from a deterministic fixture |
| Resource Space Durability | Tokimu | Tokimu | Tosumu persistence | Create folders/resources, persist, reopen, and compare provider-neutral observations |
| In-Memory Versus Durable | Tokimu | Tokimu | In-memory and Tosumu providers | Show identical Resource Space semantics and separately labeled provider evidence |
| Tasset Evidence | Tokimu | Tokimu asset semantics | Tosumu storage | Inspect a bounded canonical-asset experiment without exposing Tosumu records as asset meaning |
| Cross-Project Diagnostics | Both | Producing subsystem | Peer presentation/host | Preserve owner, severity, code, evidence source, and limitation across the boundary |

## Progressive Enhancement And Deployment

Both sites must remain useful when peer artifacts or interactive bundles are
unavailable.

- Static prose and screenshots are committed or generated during the local
  site build.
- Interactive labs load only after explicit activation.
- Peer artifacts are pinned or copied during a controlled build step; pages do
  not hot-link mutable files from the other deployment.
- A missing peer checkout produces a labeled unavailable state, not fabricated
  evidence.
- Each site deploys independently and may publish while the peer site is down.
- Cross-site links use stable public URLs but are never required for local
  navigation or build success.

Live Tosumu execution in a browser is deferred until Tosumu exposes a bounded,
browser-safe public API. Pre-generated Tosumu JSON remains valid structural
evidence, but must be labeled as generated evidence rather than a live database
session.

## Deliverables

- [ ] Publish this reciprocal ownership and evidence-exchange contract in both
      repositories.
- [ ] Define `reciprocal-site-evidence-v1` with schema validation and fixtures.
- [ ] Add one Tosumu-produced artifact rendered by a Tokimu website component.
- [ ] Add one Tokimu Resource Space fixture persisted by Tosumu and presented
      on the Tokimu website.
- [ ] Add static fallbacks, typed unavailable states, and accessible summaries.
- [ ] Add independent site-build and cross-project compatibility checks.
- [ ] Record divergences without silently widening either public contract.

## Implementation Slices

### Slice 0: Boundary And Baseline

**Deliverables**

- [ ] Inventory the current Tokimu island contract, Tosumu MkDocs build, TQL
      JSON schema, Resource Space provider contract, and reusable fixtures.
- [ ] Select one fixture whose license, provenance, and expected observations
      are already known.
- [ ] Record current independent site-build commands and artifact sizes.

**Acceptance criteria**

- [ ] Both sites build without the peer checkout's generated output.
- [ ] Meaning owner, provider, host, and presentation owner are named for every
      proposed lab.
- [ ] No browser or TypeScript layer is assigned database or engine semantics.

### Slice 1: Versioned Evidence Manifest

**Deliverables**

- [ ] Define a small JSON schema and producer metadata.
- [ ] Generate one Tosumu TQL/status artifact through public commands.
- [ ] Validate fingerprints, schema version, bounded diagnostics, and limits.

**Acceptance criteria**

- [ ] Unknown versions are rejected explicitly.
- [ ] The artifact contains no secret payload, physical path, or protected
      storage material.
- [ ] Regeneration from the pinned fixture is deterministic where promised.

### Slice 2: Tosumu Site Uses Tokimu Presentation

**Deliverables**

- [ ] Add a static Tosumu lab page with textual TQL/storage evidence.
- [ ] Add an optional Tokimu-powered visualization for the same artifact.
- [ ] Preserve command identity, source facts, diagnostics, and limitations.

**Acceptance criteria**

- [ ] The page remains useful without JavaScript or WASM.
- [ ] Tokimu draws only from the versioned observation and invents no storage
      facts.
- [ ] Visual and textual evidence identify the same fixture and revision.

### Slice 3: Tokimu Site Uses Tosumu Persistence

**Deliverables**

- [ ] Produce a Tosumu-backed Resource Space session fixture.
- [ ] Compare its provider-neutral observations with the in-memory provider.
- [ ] Present reopen/durability evidence and provider diagnostics separately.

**Acceptance criteria**

- [ ] Resource meaning is identical across providers or every divergence is
      classified.
- [ ] Tosumu-specific facts remain in a separately labeled provider pane.
- [ ] The site does not imply live browser persistence when showing generated
      evidence.

### Slice 4: Build And Compatibility Automation

**Deliverables**

- [ ] Add schema validation to both repositories.
- [ ] Add an explicit artifact import/update command with revision reporting.
- [ ] Add stale-artifact and missing-peer diagnostics.
- [ ] Retain screenshots or structural summaries for review.

**Acceptance criteria**

- [ ] Neither deployment downloads mutable peer artifacts at runtime.
- [ ] A compatibility failure identifies producer, schema, fixture, and first
      divergent field.
- [ ] Independent MkDocs builds remain green when optional labs are absent.

### Slice 5: Accessibility, Safety, And Admission Review

**Deliverables**

- [ ] Verify keyboard operation and equivalent textual results.
- [ ] Bound file size, command count, rendering work, and diagnostic output.
- [ ] Review disclosure, untrusted input, and generated-artifact provenance.
- [ ] Decide whether any adapter has earned reusable support-library status.

**Acceptance criteria**

- [ ] No lab requires pixels to understand its result.
- [ ] Unsupported or unavailable behavior is explicit.
- [ ] Site consumers do not expose secrets, host paths, protected metadata, or
      mutable production databases.
- [ ] Reusable extraction, continued incubation, or parking is recorded from
      evidence rather than convenience.

## Validation

```text
# Tokimu
python -m mkdocs build --strict -f website/mkdocs.yml
npm test --prefix website

# Tosumu
python -m mkdocs build --strict -f third-party/tosumu/mkdocs.yml

# Cross-project
validate reciprocal-site-evidence-v1
regenerate selected fixtures and compare fingerprints
```

## Risks

### Website Coupling

Shared themes or generated site trees could make one deployment depend on the
other. Share versioned evidence contracts and small adapters, not website
internals.

### Stale Evidence

A checked-in artifact can outlive the implementation that generated it. Record
producer revisions and fail compatibility checks when regeneration diverges.

### Misleading Interactivity

An animation can appear to prove more than its source observation. Every visual
must retain a textual statement of what was measured, what was inferred, and
what remains unavailable.

### Secret Or Physical Data Leakage

Tosumu diagnostics may eventually involve protected storage. The exchange
schema admits only reviewed public facts and must undergo disclosure review
before new command families are visualized.

## Completion Criteria

This plan may complete when:

- each site publishes at least one bounded peer-assisted evidence page;
- both pages retain static, accessible, independently deployable fallbacks;
- the same versioned fixtures pass producer and consumer validation;
- no project semantics leak into the peer provider or browser layer;
- differences are classified and retained as architectural evidence;
- remaining live-WASM, synchronization, or reusable-adapter work is explicitly
  completed, parked, or moved to a new plan.
