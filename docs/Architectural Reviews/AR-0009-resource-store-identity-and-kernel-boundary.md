# AR-0009: Resource Store Identity And Kernel Boundary

| Field | Value |
| --- | --- |
| Status | Provisionally admitted as a foundational capability candidate |
| Opened | 2026-08-03 |
| Last reviewed | 2026-08-03 |
| Scope | Kernel / foundational service / capability / cross-cutting |
| Trigger | Repeated MemoryStore identity, root, visibility, and missing-folder failures plus proposed native/WASM consumers |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005 |
| Related evidence | `docs/Plans/Standalone/memory-resource-store.md`; C# MemoryStore behavior and tests; Asset Workbench, XML/SVG, glTF, and corpus artifact consumer candidates |
| Admission exception | ADR-0005 provisional admission for a reversible foundational contract; no permanent crate extraction or kernel admission |

## Architectural Question

Which resource-store semantics, if any, must be kernel-native or a foundational
Tokimu capability so applications can rely on stable store, root, folder, and
resource identity plus deterministic navigation, while byte retention,
platform import/export, and format resolution remain replaceable mechanisms?

## Context

Applications repeatedly need to hold addressable resources before those
resources become decoded or prepared assets. Native tools receive files from
disk. Browser consumers receive dropped or uploaded bytes. XML, SVG, glTF, and
other importers resolve related resources. Corpus harnesses retain source and
derived artifacts for inspection.

The existing `tokimu-assets::AssetStore` does not own this concern. It owns
asset handles, generations, loading state, and prepared asset lifecycle. A
resource store instead retains bytes and metadata under logical addresses.

Prior C# `MemoryStore` use exposed failures that are architectural rather than
merely ergonomic:

- hidden resources did not behave consistently across direct access,
  enumeration, and import;
- roots could lose or change meaning through path normalization and joining;
- two stores could be created with the same human-readable name while holding
  similar but divergent content.
- folders did not exist as semantic objects, so navigation depended on occupied
  path prefixes and could not preserve empty or hidden folders.

These failures suggest that applications may require stable, shared identity
semantics before a particular in-memory provider is selected. They do not yet
prove that byte storage belongs in `tokimu-core`.

## Trigger And Evidence

- Corpus examples: `hello-resource-space` and the ASP.NET/WASM Asset Workbench
  engine now use the same provisional contract. The workbench creates a
  transient resource session from explicit selected files and resolves an
  external glTF buffer and image through that session; TypeScript presents the
  result and does not reimplement importer resolution.
- Automated tests: the source C# implementation's executed suite passes all
  83 tests. The provisional Rust implementation has 37 focused Resource Space
  tests, and the Asset Workbench engine has 19 contract tests.
- Audits or diagnostics: prior use observed hidden-entry loss, root ambiguity,
  duplicate logical-store construction, and missing folder semantics.
- Independent consumers: the headless resource-space corpus and Asset
  Workbench engine now prove one provisional Rust contract in two independent
  consumer implementations. The same Workbench engine compiles for
  `wasm32-unknown-unknown`; browser runtime chooser validation remains
  pending.
- Repeated implementation friction: store names, paths, roots, hidden status,
  and content similarity were allowed to stand in for identity in different
  call paths.
- Missing evidence: browser runtime chooser validation, persistence scope,
  nested browser-selection policy, and broader XML reference resolution. A
  corpus-side loader bridge demonstrates one-way lifecycle interaction with
  `tokimu-assets`, the Asset Workbench engine resolves same-folder glTF
  dependencies, synchronized registry creation demonstrates one scoped
  create/open conflict boundary, and the same session contract now has
  build-and-contract parity on the WASM target.

## Ownership Analysis

The review currently distinguishes five concepts:

```text
StoreId
    identity of one logical store

ResourceRootId
    identity of one address root within that store

ResourceKey
    store + root + normalized relative address

FolderId
    identity of one explicit navigable folder inside one root

ContentFingerprint
    algorithm-qualified diagnostic candidate for byte equality; exact equality
    verifies retained bytes and remains distinct from resource or store identity

StoreProvider
    mechanism retaining and retrieving bytes
```

- Applications own display names, trust policy, quotas, persistence policy,
  and when a resource should become an asset.
- A possible foundational identity contract would own stable IDs,
  qualification, create/open conflict semantics, and provider-neutral
  diagnostics.
- A resource-store capability would own addresses, roots, visibility, explicit
  folders, parent/child navigation, metadata, deterministic enumeration, and
  mutation behavior.
- Providers would own in-memory retention, filesystem interaction, browser
  upload/download, database persistence, and synchronization.
- `tokimu-assets` continues to own asset lifecycle and must not become the
  hidden owner of source bytes.
- `tokimu-core` must not acquire filesystem, browser, database, parser, or
  renderer dependencies.

The primary uncertainty is whether stable store/root identity is a kernel truth
needed by many capabilities or simply part of one foundational resource-store
contract close to, but outside, the trusted core.

The implementation plan now requires an admission matrix comparing C# consumer
pressure, Tokimu corpus consumers, headless use, native/WASM parity, and the
proposed owner for each semantic. That matrix is the intended evidence for the
next review cycle rather than a general claim that MemoryStore is universally
useful.

## Dependency Direction

```text
Current candidate implementations:

application/importer
        |
        v
ad hoc maps, paths, names, and provider mechanisms
        |
        +-- may feed tokimu-assets loaders

Proposed boundary under review:

tokimu-core identity (only if admitted)
        ^
        |
resource-store semantic contract
        ^
        |
in-memory / filesystem / browser providers
        ^
        |
applications, importers, corpus consumers

resource-store bytes
        |
        v
tokimu-assets loader boundary
        |
        v
asset handles and prepared lifecycle
```

No provider dependency may point upward into `tokimu-core`. No format parser
may become a dependency of the resource-store semantic contract.

## Alternatives Considered

### Alternative A: Promote The Entire MemoryStore Into The Kernel

- Benefits: one universally available API; duplicate-store prevention can be
  coordinated globally; every application receives immediate byte storage.
- Costs: expands the trusted core with path, metadata, retention, quota, and
  lifecycle concerns; risks filesystem-like semantics and platform pressure.
- Failure mode: the kernel becomes the owner of bytes and mechanisms merely
  because most applications need resources.

### Alternative B: Admit Only Store And Resource Identity Into The Kernel

- Benefits: stable IDs and qualification become universal while providers and
  byte retention remain replaceable; duplicate logical-store creation can be
  diagnosed consistently.
- Costs: may introduce generic identity before enough unrelated capabilities
  prove it; a kernel registry could become hidden global state.
- Failure mode: identity types exist without a clear lifecycle owner or are
  reused for concepts whose equality rules differ.

### Alternative C: Foundational Resource-Store Capability Outside Core

- Benefits: establishes stable store/root/resource semantics without expanding
  `tokimu-core`; native and WASM providers remain replaceable; fits the
  foundational service ring.
- Costs: applications must opt into or receive the capability; duplicate-store
  prevention is scoped to a registry instance rather than universal process
  state.
- Failure mode: multiple registries recreate the same ambiguity unless scope
  and provenance are explicit.

### Alternative D: Extend `tokimu-assets::AssetStore`

- Benefits: reuses existing asset handles and lifecycle infrastructure; fewer
  public concepts.
- Costs: conflates source bytes, logical resources, decoded assets, and
  prepared resources; external references and failed decodes become awkward.
- Failure mode: asset generations and resource roots acquire incompatible
  ownership semantics behind one API.

### Alternative E: Public Companion Library With No Tokimu Admission

- Benefits: smallest architectural commitment; useful to consumers; easy to
  evolve independently.
- Costs: Tokimu capabilities may invent competing identity and root semantics;
  shared diagnostics and loader boundaries remain application conventions.
- Failure mode: the historical hidden/root/duplicate failures recur in each
  consumer under different APIs.

### Alternative F: Continue Corpus Incubation

- Benefits: lets native, WASM, importer, and asset-loader consumers pressure
  the boundary before any kernel or capability promotion.
- Costs: temporary duplication remains; consumers cannot yet rely on a stable
  contract.
- Failure mode: incubation continues without named tests or a disposition.

## Findings

Current evidence supports these provisional findings:

1. A display name is not store identity.
2. A normalized path is not globally meaningful without store and root
   qualification.
3. A content fingerprint identifies a candidate byte match; exact retained
   byte comparison proves content equality, and neither result is resource or
   store identity.
4. Hidden-resource visibility must be explicit metadata and query policy, not
   an enumeration accident.
5. Folders must be explicit semantic nodes when consumers need navigation;
   occupied path prefixes cannot represent empty folders or folder metadata.
6. Roots and folders are distinct: roots establish address spaces and cannot be
   moved, while folders are navigable nodes inside one root.
7. Create, open, and create-or-open are distinct operations; unrestricted
   constructors cannot preserve logical-store uniqueness.
8. Resource bytes and `tokimu-assets` lifecycle remain distinct concerns.
9. Platform hidden attributes, path syntax, and filesystem mechanisms remain
   provider responsibilities.
10. A resource-to-asset bridge can load immutable bytes through the existing
    `AssetLoader` contract without transferring root, folder, or source-byte
    ownership to `tokimu-assets`; failed decodes leave the resource inspectable
    and allocate no asset handle.
11. A glTF importer can resolve bounded external-buffer and external-image
    references through an explicitly selected logical folder, leaving URI
    interpretation with the importer adapter. Document/accessor validation
    remains with the glTF decoder; image decoding remains outside this bridge.
    This is evidence for provider-neutral byte resolution, not a generic
    cross-format resolver contract.
12. XML parsing and external-resource resolution remain separable. A dedicated
    adapter can use `xml-tools` for XML syntax while resolving one-segment,
    same-folder `href` and `xlink:href` resources through Resource Space. It
    preserves a fragment for the XML or SVG semantic consumer and rejects
    local-only, traversal, nested-path, and URL-like references explicitly.
    This does not admit external SVG `<use>` semantics to `ui-tools`.
13. Mutation evidence can remain an opt-in, bounded, locally ordered concern
    of one resource-space instance. Its sequence identifies capture order only;
    it is not a durable revision, persistence log, or kernel event stream.

The evidence does not yet establish:

- that the entire resource store belongs in the kernel;
- that store identity must be process-global;
- that IDs must survive persistence or transport;
- that content-addressed deduplication belongs in the semantic contract;
- that one registry model works for native tools, WASM consumers, importers,
  and tests.

## ADR-0005 Admission Evidence

ADR-0005 applies directly because the likely ownership is clear enough to begin
a reversible implementation while the normal Tokimu admission evidence remains
incomplete.

### Proposed Meaning

The provisional contract includes:

- stable store, root, and resource identity;
- explicit create, open, and identity-conflict outcomes;
- normalized provider-neutral relative addresses;
- explicit folder nodes, including empty folders and direct-child navigation;
- explicit hidden-resource visibility and query policy;
- structured diagnostics for identity, root, visibility, and quota failures;
- an in-memory provider used to exercise the contract.

It excludes:

- filesystem and browser mechanisms;
- database or durable persistence;
- JSON, XML, image, model, or other format semantics;
- asset generations and prepared asset lifecycle;
- process-global hidden state;
- a binding claim that any of these semantics belong in `tokimu-core`.

### Decomposition Attempts

The review considered:

- promoting the entire MemoryStore into the kernel;
- admitting identity while leaving storage outside the kernel;
- a foundational capability outside core;
- extending `tokimu-assets::AssetStore`;
- a public companion library with no Tokimu admission;
- continued corpus-only incubation.

The current decomposition keeps identity, addressing, byte retention, platform
mechanisms, format adapters, and asset lifecycle distinct.

### Normal Evidence Missing

- generated WASM/browser evidence for a multi-file external-resource session;
- evidence establishing whether IDs survive a process or persistence boundary;
- evidence that a narrow identity concept is universal engine meaning.

### Substitute Evidence

- 83 executed tests in the source C# MemoryStore;
- repeated maintainer experience with hidden-resource loss, unstable roots,
  duplicate logical stores, and the inability to navigate explicit folders;
- concrete candidate consumers across WASM asset workbenches, XML/SVG
  resolution, glTF external resources, and corpus artifacts;
- accepted ADR-0001 and ADR-0003 ownership constraints that keep platform and
  provider mechanisms out of core;
- a reversible corpus-first implementation plan with explicit graduation and
  reopening criteria.

Mechanically creating artificial consumers before implementing the contract
would add little decision value. The known failures already identify the first
semantics that must be tested. Real consumer evidence is still required before
permanent or kernel admission.

### Accountable Maintainer

The accountable maintainer is the Tokimu project maintainer, Arakendo. Codex
may implement and critically review the provisional contract but is not the
governance principal authorizing permanent admission.

### Risks Accepted

- provisional public names and types may be relocated or renamed;
- the initial registry scope may prove too broad or too narrow;
- store/root IDs may require different persistence semantics;
- consumers may expose a simpler decomposition;
- temporary migration work may be required if the capability is retired or
  folded into another accepted boundary.

### Confirmation Or Retirement Milestone

The provisional contract must be confirmed, relocated, or retired after:

- one native consumer and one generated WASM/browser consumer use it;
- an importer resolves related resources through it;
- a `tokimu-assets` loader consumes bytes without taking resource ownership;
- duplicate-store, hidden-resource, and root-identity regression cases pass;
- empty-folder, hidden-folder, child-listing, rename, and subtree-move cases
  pass;
- AR-0009 records whether the contract remains a companion library, becomes a
  foundational capability, or contributes a narrower identity concept to the
  kernel.

## Disposition

**Incubating with provisional admission under ADR-0005.** Implement the narrow,
reversible foundational contract described above without presenting it as a
stable kernel API. The likely direction is a foundational resource-store
capability with a narrower kernel identity contribution, but the current
evidence is insufficient to bind that split through an ADR.

## Consequences

- The Rust port must not begin as a literal copy of the C# public API.
- Store, root, resource, and content identities must remain distinct in tests
  and diagnostics.
- Folder nodes and direct-child navigation must be preserved independently from
  recursive resource search.
- Hidden resources require explicit import and enumeration policy.
- Creation must flow through an explicit registry/factory boundary.
- `tokimu-assets` remains unchanged until a loader consumer demonstrates the
  bridge.
- Temporary corpus-side types and duplication are accepted while semantics
  stabilize.
- Provisional types must carry an instability or incubation notice and remain
  mechanically relocatable.
- Any kernel proposal must be narrower than platform storage mechanisms.

## Required Follow-Up

- [x] Expand `docs/Plans/Standalone/memory-resource-store.md` with known failure evidence.
- [x] Create regression fixtures for hidden resources, unstable roots, and
      duplicate logical-store construction.
- [x] Create regression fixtures for empty folders, hidden folders,
      direct-child listing, and atomic subtree movement.
- [x] Implement provider-neutral store/root/resource identity types in corpus
      incubation. Content identity remains deliberately deferred.
- [x] Implement explicit create/open registry behavior.
- [x] Exercise one native and one WASM build-and-contract consumer.
- [x] Exercise a corpus importer and `tokimu-assets` loader bridge.
- [x] Complete the admission matrix in
      `docs/Plans/Standalone/memory-resource-store.md`.
- [ ] Attempt a foundational-capability decomposition with no kernel addition
      and record where it succeeds or fails.
- [ ] Reassess kernel-native versus foundational capability ownership.
- [ ] Create an ADR only if the resulting boundary is accepted.

## Reopening Triggers

This review remains active. After a disposition, reopen or supersede it when:

- two independent consumers require stable store/root identity but not shared
  byte-store behavior;
- a target cannot preserve qualified resource identity;
- duplicate logical stores remain possible through the admitted API;
- provider details leak into store or root identity;
- `tokimu-assets` requires ownership of source bytes to satisfy a real loader;
- persistence or transport proves IDs must survive a runtime session;
- a simpler decomposition eliminates the proposed registry or kernel concept.

## Review History

### Cycle 1 -- 2026-08-03

- Status entering review: Proposed
- New evidence: historical hidden-resource, root-ambiguity, duplicate-store,
  and missing-folder failures were recorded alongside multiple candidate
  consumers.
- Participants or reviewers: project maintainer and Codex.
- Findings: stable identity, visibility, and create/open semantics require
  explicit study; ADR-0005 permits reversible foundational implementation from
  substitute evidence; byte storage is not thereby proven kernel-native.
- Disposition: Incubating with provisional admission under ADR-0005.
- Resulting ADR or documentation change: expanded Memory Resource Store plan;
  no ADR yet.

### Cycle 2 -- 2026-08-03

- Status entering review: Incubating with provisional admission under ADR-0005.
- New evidence: `corpus/lib/resource-space` now provides an in-memory,
  provider-neutral contract with caller-supplied `StoreId`, `ResourceRootId`,
  and `FolderId`; qualified resource keys; explicit visible/hidden metadata;
  empty-only root and folder removal; deterministic direct-child enumeration;
  immutable shared byte retention; bounded entry and byte limits; and a
  registry that distinguishes create, open, and create-or-open behavior.
- Query and provenance evidence: recursive search is bounded, literal, and
  distinct from direct navigation; it returns qualified resource keys with
  explicit visibility and exact media-type policy. Store descriptors carry
  advisory generated/imported/fixture provenance without accepting host paths
  or acquisition mechanisms as identity.
- Content evidence: entries expose an algorithm-qualified BLAKE3 fingerprint
  for diagnostics and candidate deduplication, while exact equality verifies
  retained immutable bytes after a fingerprint match. Neither result defines
  a resource or store identity.
- Regression evidence: tests cover the historical ambiguity classes. Hidden
  resources remain directly addressable while visibility queries control
  enumeration; roots cannot be removed while they retain direct resources;
  empty folders remain navigable; one registry rejects duplicate `StoreId`
  creation while allowing nonunique display names for distinct IDs; and a case
  policy mismatch is rejected rather than silently reinterpreting addresses.
  A synchronized caller can race two create requests without producing two
  stores for one ID.
- Consumer evidence: `corpus/focused/data-interchange/hello-resource-space` uses only the public
  contract to create two same-named stores with distinct provenance, navigate
  folders, retain hidden content, run a bounded query, and prove that equal
  retained bytes do not merge qualified resource identity.
- Findings: the first implementation supports Alternative C more strongly
  than kernel admission. The registry is explicit and instance-scoped, so it
  does not create process-global identity or storage state. Resource identity,
  content equality, and provider retention remain separate.
- Remaining uncertainty: the headless corpus and Asset Workbench engine now
  provide two independent consumers, including same-folder glTF
  external-buffer and external-image source resolution. The browser runtime
  has not yet exercised the generated WASM session. Native/WASM parity,
  persistence behavior, reentrant callback behavior, image decoding and
  preparation, and cross-folder reference policy remain unproven.
- Disposition: continue corpus incubation. Do not promote the crate or narrow
  identity types beyond the corpus until independent consumers pressure the
  public contract.

### Cycle 3 -- 2026-08-03

- New evidence: `corpus/lib/resource-space-xml` resolves external XML resource
  references through the public Resource Space contract. `xml-tools` continues
  to own parsing, while the adapter owns the intentionally narrow same-folder
  lookup policy and the XML/SVG consumer retains any fragment semantics.
- Regression evidence: the adapter resolves `symbols.svg#notice`, preserves
  the `notice` fragment, reports a missing sibling without discarding the
  source document, and rejects local-only and parent-directory references
  instead of silently widening the selected resource boundary.
- Mutation evidence: a dependency-free, deterministic 125-case operation
  matrix exercises replace, copy, move, insert, and remove sequences. For each
  sequence, public direct navigation and summary counts agree on folders,
  resources, and retained bytes. General-purpose generated property testing
  remains deferred until bounded evidence exposes a counterexample.
- Consumer evidence: `hello-resource-space` now exercises the XML/SVG case in
  addition to the asset-loader and glTF bridges.
- Candidate consumer: the external TypeScript/XSLT3 Weaver project at
  `F:\LocalSource\TS XSLT` is a prospective XML-provider consumer. It could
  exercise selected-resource XML/XSLT inputs and related-resource lookup
  across a TypeScript boundary. Its design documentation has been reviewed in
  AR-0010; this remains a route for future evidence only and has not been
  integrated or counted as admission evidence.
- Findings: format adapters can remain replaceable and format-specific while
  sharing only qualified source-byte lookup. Resource Space does not need to
  understand XML names, SVG elements, fragments, or recursive reference
  graphs to provide this boundary.
- Remaining uncertainty: browser runtime validation of a selected multi-file
  session, recursive or cyclic document references, nested and cross-folder
  policy, persistence behavior, and native/WASM parity remain unproven.
- Disposition: continue corpus incubation. Do not promote the contract or add
  external SVG importer behavior until an independent consumer requires it.

### Cycle 4 -- 2026-08-03

- New evidence: `InMemoryResourceSpace` can optionally retain a bounded journal
  of structured root, folder, and resource mutation outcomes. Observation is
  disabled by default, enabling it starts an explicit local capture session,
  and capacity pressure deterministically removes the oldest record.
- Ordering evidence: successful mutations receive ascending local sequence
  numbers. Failed mutations and exact no-op resource moves emit no successful
  outcome. Consumers may inspect or drain records without accessing provider
  collections.
- Consumer evidence: `hello-resource-space` enables capture for the fixture
  store, observes eleven ordered mutations, and verifies that its same-named
  imported peer retains the default observation-disabled behavior.
- Findings: structured mutation diagnostics do not require a global event bus,
  process-global registry, durable transaction log, or kernel admission. A
  future Tosumu provider may consume or produce equivalent evidence, but these
  observations do not define persistence or replay semantics.
- Validation: all 36 `resource-space` tests pass; the headless consumer runs;
  clippy passes for `resource-space`, its XML and asset bridges, and
  `hello-resource-space` with warnings denied.
- Disposition: continue corpus incubation. The new evidence strengthens a
  foundational capability boundary outside `tokimu-core`; it does not resolve
  the remaining native/WASM, persistence, or admission questions.

### Cycle 5 -- 2026-08-03

- New evidence: `corpus/lib/resource-space-json` provides typed serde/JSON
  conversion as a replaceable format bridge. The adapter inserts compact JSON
  bytes through the public store API, preserves caller-provided media types,
  supplies `application/json` only when absent, and reports malformed or
  missing resources explicitly.
- Consumer evidence: `hello-resource-space` stores and resolves a typed project
  manifest next to its glTF and SVG resources using only the public contract.
- Finding: typed serialization can remain above qualified immutable source
  bytes. Resource Space does not need to own serde, JSON canonicalization, or
  application schema meaning to support the bridge.
- Disposition: continue corpus incubation. JSON confirms the format-bridge
  boundary but does not alter the deferred provider, persistence, or kernel
  admission questions.

### Cycle 6 -- 2026-08-03

- New evidence: `hello-resource-space` now executes a deterministic
  in-memory workload of 2,048 direct-child entries, 8,192 repeated reads, and
  512 shared-content copies. It records raw local timings as diagnostic
  evidence only.
- Retention evidence: copied entries keep distinct qualified keys while their
  `Arc<[u8]>` content allocation is shared with the source. Current retained
  byte counts remain logical entry totals, not a claim about provider physical
  allocation or deduplication accounting.
- Finding: the current in-memory provider supports warm repeated reads and
  immutable copy sharing without widening the public semantic contract into a
  cache or performance-policy API.
- Disposition: continue corpus incubation. Native/browser adapter behavior and
  portable persistence remain the next meaningful pressure points.

### Cycle 7 -- 2026-08-03

- New evidence: `resource-space-native` imports an explicitly selected native
  directory beneath a caller-selected logical folder. It maps host directories
  to explicit folder nodes, preserves empty directories when requested, and
  converts dot-prefixed or Windows-hidden entries through an explicit
  include/skip/reject policy.
- Export evidence: logical folders export only beneath a caller-approved
  native root. The adapter canonicalizes each output parent and rejects an
  attempted containment escape; host paths never enter `ResourceKey` or base
  store metadata.
- Failure evidence: rejected hidden entries, symbolic links, invalid logical
  names, provider limits, collisions, and host I/O all remain structured
  native-adapter diagnostics rather than silent omission.
- Finding: host traversal and sandboxing can remain a replaceable adapter
  concern while preserving stable Resource Space identities and folders.
- Disposition: continue corpus incubation. A browser upload/download adapter
  and equivalent native/WASM import evidence are still required before
  admission is reconsidered.

### Cycle 8 -- 2026-08-03

- New evidence: the ASP.NET Asset Workbench's Rust/WASM `ResourceSession`
  accepts browser-selected logical names and byte arrays under explicit entry,
  per-entry-byte, and retained-byte limits. It resolves observations from that
  bounded session and can return one named logical resource as bytes.
- Browser boundary: TypeScript owns the file chooser and the user-initiated
  download gesture. Rust/WASM owns logical name validation, bounded retention,
  and source-byte lookup. Neither layer exposes or stores browser-native file
  handles or host paths in Resource Space.
- Regression evidence: the session tests selected byte return, rejects host
  path syntax as a logical resource name, and continues to reject selections
  beyond the entry budget.
- Finding: browser import and export can use the same provider-neutral
  Resource Space contract as native import/export without adding browser APIs
  to the base store.
- Disposition: continue corpus incubation. Native/WASM parity is now partially
  evidenced; persistent-provider behavior, nested browser selection policy,
  and an independent non-workbench browser consumer remain open.

### Cycle 9 -- 2026-08-03

- New evidence: native import now enforces both explicit empty-directory
  policies. A requested empty hierarchy is retained as navigable folder nodes;
  when retention is disabled, an empty nested hierarchy is pruned after its
  children are considered. The policy therefore changes adapter behavior rather
  than serving as documentation-only configuration.
- Retention evidence: the headless workload continues to verify immutable
  shared-byte copies, while the WASM `ResourceSession` now performs 64 repeated
  selected-byte reads without changing its retained entry or byte summary.
  Returning bytes to JavaScript is intentionally a boundary copy; the claim is
  that the session does not retain a second provider-owned copy per read.
- Validation: focused `resource-space`, `resource-space-native`, and Asset
  Workbench engine tests pass, including the native policy and repeated-read
  regressions.
- Finding: host-directory policy and browser byte delivery remain adapter
  concerns. The provider-neutral store needs only explicit folder semantics,
  immutable byte retention, and bounded public observations.
- Disposition: continue corpus incubation. A real browser run with a selected
  multi-file external-buffer glTF, a documented C# workflow comparison, and
  persistent-provider evidence remain necessary before admission is reviewed.

### Cycle 10 -- 2026-08-03

- New evidence: the Asset Workbench now presents browser multi-file selection
  as a Resource Space operation rather than an implicit frontend detail. Before
  inspection it identifies the chosen document and its same-folder sidecars;
  after import it reports the bounded Rust/WASM session summary.
- Contract evidence: the `Box.gltf` plus `Box0.bin` fixture pair is documented
  as the repeatable external-buffer case. The existing engine regression proves
  that the Rust/WASM session retains both selected resources and resolves the
  declared buffer without TypeScript parsing or URI resolution.
- Boundary finding: visible selection status is consumer presentation only.
  It reports Resource Space facts supplied by the WASM session and does not
  introduce browser file handles, host paths, or provider collections into the
  base store.
- Validation: focused `resource-space` and Asset Workbench engine suites pass;
  the Workbench TypeScript source type-checks.
- Disposition: continue corpus incubation. The generated browser build still
  needs one observed chooser or drag/drop run using the documented pair. That
  remains distinct from the code-level WASM session regression and is not
  claimed complete here.

### Cycle 11 -- 2026-08-03

- New evidence: source review of the C# `InMemoryResourceStore` and its 83
  test cases is recorded in
  `docs/Notes/resource-space-csharp-memory-store-comparison.md`.
- Convergence finding: both systems support provider-neutral logical source
  bytes, related-resource lookup, inspection, and adapter-owned decoding.
  This is workflow convergence, not public-API compatibility.
- Deliberate replacement finding: the C# store silently overwrites a
  case-insensitive URI dictionary key and reconstructs directories from
  occupied resource prefixes. Resource Space instead requires qualified store
  and root identity, explicit collision/replacement intent, explicit folders,
  and explicit visibility policy.
- Admission impact: the source evidence strengthens the foundational resource
  capability hypothesis. It does not prove kernel admission, because the C#
  model exposes no stable store identity or durable cross-process identity
  contract.
- Remaining evidence: execute a preserved C# document bundle workflow and the
  corresponding Rust fixture matrix; record intentional divergence rather than
  treating URI, stream, async, or XML convenience APIs as base-contract
  requirements.
- Disposition: continue corpus incubation. Source-level comparison is complete;
  runnable cross-language evidence, browser observation, and persistence
  pressure remain open.

### Cycle 12 -- 2026-08-03

- New evidence: `resource-space` now contains the first-party
  `document_bundle_preserves_explicit_navigation_and_replacement_intent`
  fixture. It models the selected C# document-bundle workflow without taking a
  dependency on the C# API or XML parser technology.
- Finding: explicit folders preserve navigable empty state, while deterministic
  child/resource listings and direct lookup remain coherent for the same
  bundle. Duplicate insertion fails before mutation; replacement remains an
  intentional separate operation.
- Validation: `cargo test -p resource-space --all-targets` passes 37 tests.
- Admission impact: the fixture strengthens the provisional foundational
  contract and closes the Rust-side fixture gap. It does not close the runnable
  C# comparison, browser chooser observation, or persistent-provider pressure.
- Disposition: continue corpus incubation. Treat C# URI directory
  reconstruction and silent overwrite as documented intentional divergence,
  pending a captured runnable comparison.

### Cycle 13 -- 2026-08-03

- New evidence: `hello-resource-space` now runs the document-bundle fixture
  alongside the existing asset, glTF, XML, and JSON bridge observations. The
  report records two root documents, five visible folders, an explicitly
  retained empty document folder, and the qualified `common/utilities.xsl`
  resource address.
- Diagnostic finding: expanding the fixture retained 16 mutation observations
  with sequences `4..=19`, rather than an unbounded history. This confirms the
  public observation API behaves as a bounded diagnostic window and should not
  be treated as a synchronization log.
- Validation: `cargo test -p resource-space --all-targets` passes 37 tests;
  `cargo run -p hello-resource-space` completes its bundle and bridge report.
- Admission impact: the same foundational contract now has both a unit-level
  fixture and a headless consumer report. Browser chooser evidence, runnable
  C# workflow evidence, and persistent-provider pressure remain open.

### Cycle 14 -- 2026-08-03

- New evidence: `dotnet test F:\LocalSource\ClassLibrary\MemoryStore.Tests\MemoryStore.Tests.csproj --no-restore`
  passes all 83 existing C# tests. The suite exercises public URI normalization,
  case-insensitive lookup and overwrite, XML/JSON/text helpers, import/export,
  and directory round-trip behavior.
- Comparison finding: C# and Rust now have runnable consumer-side evidence for
  the same broad source-byte/document workflow. The agreement is semantic:
  logical source retention, related lookup, enumeration, format adaptation,
  and host import/export boundaries. It is not API compatibility: Resource
  Space intentionally replaces case-insensitive implicit overwrite and
  inferred folders with explicit case policy, replacement intent, and folder
  nodes.
- Admission impact: the C# comparison requirement is complete at the
  behavioral level. A dedicated common report format remains optional future
  diagnostic work, not a graduation blocker.
- Remaining evidence: observed browser multi-file selection and a
  persistent-provider consumer remain open.

### Cycle 15 -- 2026-08-03

- New evidence: `cargo check --manifest-path
  corpus/consumers/aspnet-wasm-asset-workbench/engine/Cargo.toml --target
  wasm32-unknown-unknown` compiles the same Asset Workbench engine that the
  native regression suite uses. The native suite covers the bounded
  `ResourceSession` external-buffer glTF case, including `Box.gltf` plus
  `Box0.bin`, Rust-side sidecar resolution, and the `resources=2` summary.
- Parity finding: the base Resource Space and selected-file session contract
  has build-and-contract parity across native and WASM targets. This is not a
  claim that a browser chooser has been observed; DOM file delivery and the
  user-initiated download gesture remain browser-adapter behavior.
- Admission impact: native/WASM parity is no longer an unqualified open item.
  The remaining browser evidence is one bounded manual selection of the
  documented `Box.gltf`/`Box0.bin` pair, with visible Workbench status and
  Rust session summary preserved as evidence.
- Remaining evidence: observed browser multi-file selection and a
  persistent-provider consumer remain open.

### Cycle 16 -- 2026-08-03

- New evidence: the plan's admission matrix now cites the concrete public
  contract tests and consumers behind every candidate semantic. This includes
  registry identity and synchronized create/open behavior, qualified root and
  resource keys, explicit folder navigation, diagnostic-only content
  fingerprints, immutable-byte retention, platform import/export boundaries,
  and visibility policy.
- Boundary finding: `hello-resource-space` and the Asset Workbench compile as
  separate consumers of the `resource-space` public contract. Neither reaches
  into `InMemoryResourceSpace` internals, browser file handles, filesystem
  paths, or importer-owned state. The Workbench's `ResourceSession` is an
  application-facing transient provider, not a public persistence contract.
- Admission impact: the matrix no longer lacks test or consumer traceability.
  The remaining question is not whether the in-memory contract works, but
  whether persistent identity and provider behavior preserve the same semantic
  boundary when a real persistence consumer, such as Tosumu-backed `.tasset`,
  arrives.
- Remaining evidence: one observed browser selection of `Box.gltf` with
  `Box0.bin`, plus an independent persistent-provider consumer, remain open.

### Cycle 17 -- 2026-08-03

- New evidence: the plan records a type-and-operation ownership map and an
  explicit no-kernel decomposition attempt. Store/root/folder/resource
  qualification, normalized addressing, visibility, navigation, bounded
  search, mutation observations, and summaries remain a foundational resource
  capability candidate. In-memory retention, registry implementation,
  platform import/export, and format or asset bridges remain replaceable
  providers or adapters.
- Decomposition result: both current consumers work with no `tokimu-core`
  addition. `ResourceSpaceLimits` remains application or consumer policy, not
  a universal storage semantic. This preserves ADR-0001 and ADR-0003 by
  excluding filesystem, browser, database, parser, renderer, and byte-store
  implementation dependencies from the trusted core.
- Admission impact: no final outcome is selected. The decomposition is now a
  concrete, falsifiable candidate for the persistent-provider consumer to
  test, rather than an aspirational ownership diagram.
- Remaining evidence: one observed browser selection of `Box.gltf` with
  `Box0.bin`, plus an independent persistent-provider consumer, remain open.

### Cycle 18 -- 2026-08-03

- New evidence: the built ASP.NET Asset Workbench host serves the static shell,
  TypeScript application module, generated Rust/WASM loader, and generated WASM
  binary successfully over its local HTTP boundary. The checked responses were
  `200` for `/`, `/app/main.js`,
  `/tokimu/tokimu_asset_workbench_engine.js`, and
  `/tokimu/tokimu_asset_workbench_engine_bg.wasm`.
- Boundary finding: host delivery verifies that the generated application can
  be reached as browser resources without changing Resource Space ownership.
  It does not prove DOM file-chooser delivery, multi-file selection, or the
  user-initiated download gesture.
- Provenance finding: Resource Space independently implements recorded
  semantics from the maintainer-owned MIT C# MemoryStore project. No source
  code or fixtures were copied; the first-party document-bundle fixture has no
  external redistribution dependency.
- Admission impact: source provenance and delivered WASM artifacts are no
  longer open implementation questions. One observed browser selection of
  `Box.gltf` with `Box0.bin`, plus an independent persistent-provider consumer,
  remain open.

### Cycle 19 -- 2026-08-03

- New evidence: `hello-resource-space` writes the deterministic
  `resource-space-provider-conformance-v1` artifact beneath
  `target/resource-space-conformance/`. It records qualified store/root/resource
  identity, visible and hidden behavior, explicit folder navigation, the
  bounded mutation-observation window, and existing public adapter outcomes.
- Boundary finding: the artifact is built entirely from public Resource Space
  observations. It contains no retained source bytes, backing collections,
  host paths, browser handles, database records, or provider-specific durable
  state. A future Tosumu-backed provider can compare its externally visible
  semantics with this fixture without being forced to reproduce the in-memory
  implementation.
- Limitation: a matching report does not prove durability, transactions,
  cross-process identity, synchronization, encryption, or freshness. Those
  remain separate persistent-provider evidence rather than properties inferred
  from an in-memory conformance snapshot.
- Admission impact: comparison criteria are now concrete enough for a second
  provider consumer. No provider trait, kernel admission, or persistence API
  is accepted by this cycle.
- Remaining evidence: one observed browser selection of `Box.gltf` with
  `Box0.bin`, plus an independent persistent-provider consumer, remain open.

### Cycle 20 -- 2026-08-03

- New evidence: the headless provider-conformance artifact now records stable
  create-or-open behavior. Reopening the same `StoreId` with the same case
  policy yields the existing logical store, preserves its original descriptor,
  and rejects a changed case policy explicitly.
- Boundary finding: stable logical store identity is not inferred from a
  display name, content identity, or a later descriptor supplied by a caller.
  This directly addresses the historical duplicate-store failure mode without
  requiring a persistent provider to reproduce the current in-memory registry.
- Limitation: this remains local registry evidence. It does not establish
  durable identity, cross-process coordination, transactions, synchronization,
  encryption, or freshness.
- Admission impact: the second-provider comparison boundary is more concrete,
  but no provider trait, kernel admission, or persistence API is accepted by
  this cycle.
- Remaining evidence: one observed browser selection of `Box.gltf` with
  `Box0.bin`, plus an independent persistent-provider consumer, remain open.

### Cycle 21 -- 2026-08-03

- Decision: Resource Space is provisionally admitted under ADR-0005 as a
  **foundational capability candidate**, not as trusted-core meaning. The
  accepted semantic boundary is store/root/folder/resource qualification,
  normalized logical addressing, explicit visibility, deterministic
  navigation, bounded observation, and public diagnostics.
- Explicit exclusions: retained bytes, create/open registry mechanics,
  filesystem import/export, browser file selection, database persistence,
  synchronization, encryption, freshness, format bridges, and asset lifecycle
  remain provider, platform, adapter, or application concerns. `tokimu-core`
  receives no Resource Space type or dependency from this decision.
- Packaging consequence: the public contract remains in
  `corpus/lib/resource-space` during incubation. No `tokimu-resource-space`
  crate, facade re-export, durable provider trait, or persistence API is
  stabilized until an independent persistent provider confirms that the
  present surface is sufficient.
- Compatibility consequence: a future provider compares public behavior using
  `resource-space-provider-conformance-v1`; it is not required to reproduce
  in-memory collections, retained-byte sharing, or registry internals. A
  provider may add durable diagnostics, but cannot silently reinterpret store
  identity, root qualification, address normalization, folder navigation, or
  visibility semantics.
- Reopening triggers: reopen this review if Tosumu requires a semantic not
  represented by the conformance report, durable create/open changes store
  identity or conflict behavior, browser multi-file selection exposes a
  different resource-session shape, or another consumer requires a public
  provider contract incompatible with the current boundary.
- Remaining evidence: an observed browser selection of `Box.gltf` with
  `Box0.bin`, plus Tosumu-backed persistent-provider evidence. Those results
  determine whether the capability is promoted into its own crate, refined,
  or retired; they do not authorize kernel admission by themselves.

### Cycle 22 -- 2026-08-03

- Resolution cleanup: the plan now records current answers rather than leaving
  resolved design choices as open questions. Case policy is per logical store
  and immutable after create/open; duplicate display names are valid for
  distinct `StoreId` values; folders retain independent `FolderId` values;
  V1 visibility is explicit `Visible` / `Hidden`; and folder removal remains
  empty-only.
- Observation and content finding: mutation observations are optional bounded
  capability diagnostics, not a global event bus or synchronization log.
  Providers may share immutable bytes between distinct logical entries, but
  no public content-addressed alias model is admitted.
- Metadata finding: retained visibility, media-type hints, and timestamps are
  observable store metadata. Provenance, host attributes, and decoded-format
  claims remain advisory, so the store cannot silently become an asset parser
  or filesystem model.
- Remaining question reduction: only browser multi-file selection and durable
  persistent-provider behavior remain graduation evidence. Virtual folders,
  recursive transactions, richer visibility, and public content-addressed
  aliases are intentionally deferred until an independent consumer needs them.

### Cycle 23 -- 2026-08-04

- New evidence: the Tokimu-owned .NET Resource Workbench now runs the focused
  `provider-operation-fixture-v1` against both the in-memory provider and a
  Tosumu-backed provider through separate bridge processes. The emitted
  `resource-space-provider-conformance-v1` artifact compares summary facts,
  folder navigation, hidden-resource filtering before mutation, move and
  visibility results, returned metadata, and exact resource bytes.
- Boundary finding: the compared artifact is built only from public Resource
  Space observations. Tosumu durable reopen and `provider.inspect` evidence
  remain separately labeled provider-only facts; no host path, Tosumu key,
  page, WAL, or record representation crosses into the shared contract.
- Result: the persistent-provider graduation evidence is now materially
  satisfied for the focused operation profile. The provider did not require a
  new Resource Space semantic or a wider provider trait.
- Remaining evidence: interrupted-write, corruption, transaction, and
  resource-limit behavior remain durable-provider evidence rather than
  in-memory conformance requirements. Browser multi-file selection of
  `Box.gltf` with `Box0.bin` remains the other outstanding consumer-boundary
  observation.
- Admission impact: this strengthens the provisional foundational-capability
  finding, but does not yet extract a permanent crate or admit persistence to
  the trusted core.

### Cycle 24 -- 2026-08-04

- Migration finding: the historical C# `MemoryStore` is now explicitly
  classified as source evidence for Resource Space rather than a library to
  port wholesale. Its useful logical-resource semantics map to the
  provisionally admitted Resource Space boundary; its in-memory storage model,
  filesystem-shaped convenience surface, and host-specific helpers do not.
- Ownership finding: the migration is into Tokimu's foundational capability
  layer during incubation, not directly into `tokimu-core`. This preserves the
  accepted exclusions for retained bytes, persistence, platform mechanisms,
  and host UI while still retiring the historical duplicate-root, hidden-item,
  duplicate-store, and missing-folder failure modes.
- Guardrail: every remaining historical `ClassLibrary` abstraction must be
  classified independently as a Tokimu capability, consumer-local host glue,
  separately owned provider/support project, or rejected/deferred work. No
  broad source migration is implied by the `MemoryStore` disposition.

## References

- `docs/Plans/Standalone/memory-resource-store.md`
- External candidate: `F:\LocalSource\TS XSLT` (Weaver TypeScript/XSLT3
  project; reviewed in AR-0010, unintegrated prospective XML consumer)
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `crates/tokimu-assets/src/store.rs`
- `docs/contribution-admission-guide.md`
