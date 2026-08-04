# Logical Resource Space

## Status

In progress. The source library has substantial behavioral evidence, and the
first provider-neutral identity and address contract now incubates in
`corpus/lib/resource-space`. The contract now includes opt-in bounded mutation
evidence in addition to identity, hierarchy, search, content, and format
bridges. Its relationship to `tokimu-assets`, persistence providers, and any
architectural admission remain unresolved.

## Purpose

Port the useful semantics of the C# `MemoryStore` into a provider-neutral
logical resource space that Tokimu consumers can use on native and WASM
targets. `MemoryStore` remains the historical source name; memory is one
provider rather than the identity of the capability.

The incubating Rust library is named `resource-space`. It is deliberately not
named VFS: the initial contract provides roots, explicit folders, resources,
and deterministic navigation, but does not promise host filesystem semantics
such as mounts, links, permissions, device nodes, or path equivalence.

The port must preserve the distinction between resource content and engine
asset lifecycle. A resource store owns addressable bytes and metadata. The
existing `tokimu-assets::AssetStore` owns asset handles, generations, and
prepared asset lifecycle. A loader may bridge the two, but neither replaces the
other.

## Source Evidence

The source `MemoryStore` provides a mature behavior set backed by approximately
83 tests:

- URI- and path-addressed byte storage;
- text, JSON, and XML convenience operations;
- timestamps, media types, and content hashes;
- copy, move, remove, enumerate, list, and search operations;
- filesystem import and export;
- XML resolver integration.

This evidence is valuable, but the C# API is not itself the Rust contract. The
port should preserve behavior that survives Tokimu's ownership boundaries and
Rust's type model rather than translating every method literally.

## Known Failure Evidence

Earlier `MemoryStore` use exposed four recurring failures that the Rust design
must treat as first-class corpus cases rather than incidental bugs:

1. **Hidden resources were inconsistently discoverable.** Files hidden by a
   leading dot, imported host metadata, or filtering policy could disappear
   from enumeration or become inaccessible through a different API path.
2. **Roots lost stable meaning.** Root strings were normalized, joined, or
   interpreted differently across operations, occasionally changing which
   resource an address denoted.
3. **Two stores could appear to be the same store.** Separate stores could be
   created with the same object name while retaining similar but divergent
   content. Callers could not reliably distinguish store identity, resource
   identity, and content equality.
4. **Folders did not exist as semantic objects.** Prefix searches could suggest
   a hierarchy only when resources happened to occupy it. Empty folders,
   folder metadata, direct child listing, hidden folders, and stable navigation
   could not be represented honestly.

These failures imply that a store name, path string, and byte hash are three
different facts:

```text
store identity
    identifies one store instance or durable logical store

resource identity
    identifies one address inside one named root of that store

content identity
    identifies exact bytes using an algorithm-qualified digest
```

Similar content is never identity. A human-readable store name is never
identity. Hidden status is never inferred by an enumeration side effect. A
folder is not merely a substring shared by resource addresses.

## Governing Boundary

```text
consumer meaning
        |
        v
resource address + requirements
        |
        v
resource store contract
        |
        +-------------------+
        |                   |
        v                   v
in-memory provider    platform adapters
                            |
                            +-- native filesystem
                            +-- browser upload/download

resource bytes
        |
        v
asset loader
        |
        v
tokimu-assets handles and prepared lifecycle
```

- The resource store owns normalized addresses, immutable byte content,
  metadata, explicit visibility, deterministic enumeration, and mutation
  semantics.
- A store registry or factory owns create-versus-open behavior and rejects
  duplicate logical store identities.
- Stable store and root identifiers qualify every resource identity.
- The resource store owns explicit folder nodes, parent/child relationships,
  and deterministic direct-child navigation.
- `tokimu-assets` owns asset identity, generations, handles, loading state, and
  prepared asset lifecycle.
- Platform adapters own filesystem and browser mechanisms.
- JSON and XML adapters own format-specific parsing and serialization.
- Applications own naming policy, quotas, persistence policy, and trust policy.

The store must not become a hidden filesystem, database, asset registry, or
global singleton.

## Intended Destination

Incubate the first implementation under `corpus/lib` while two independent
consumers establish the contract. The likely public destination is a companion
library such as `tokimu-resource-space`, not `tokimu-core`.

Candidate consumers are:

- the ASP.NET/WASM Asset Workbench for dropped and generated files;
- SVG/XML processing with referenced resources;
- the external Weaver TypeScript/XSLT3 project at `F:\LocalSource\TS XSLT`
  as a prospective XML-provider consumer, once a bounded integration contract
  is selected;
- glTF external buffers and images;
- corpus artifact capture and inspection.

Promotion requires an Architectural Review if the implementation becomes a
shared Tokimu capability. A convenient crate extraction alone is not admission
evidence.

### Incubation Evidence

The corpus now contains two deliberately one-way bridges:

- `resource-space-assets` loads immutable resource bytes through
  `tokimu-assets::AssetLoader`, allocating a handle only after a successful
  decode;
- its glTF adapter resolves one JSON glTF document's external buffer URIs from
  an explicitly selected resource folder before handing the bytes to the
  existing glTF decoder.

Neither bridge transfers source ownership into the decoder or asset store. A
missing logical buffer and a rejected asset decode remain explicit failures,
while the original resource bytes stay inspectable. Image URI resolution,
cross-folder references, and generic resolver traits remain deferred until
additional independent consumers require them.

`AR-0009` is open to determine whether stable resource/store identity and root
semantics are kernel-native, foundational capability semantics, or remain a
companion-library contract. The review must not promote platform import,
filesystem behavior, or stored bytes merely because identity is foundational.

ADR-0005 permits provisional admission of the narrow, reversible foundational
contract before the normal Rust consumer threshold is complete. The substitute
evidence is the source test suite, repeated maintainer-observed identity/root/
visibility failures, and the concrete consumer set. This exception does not
stabilize the API or admit the contract into `tokimu-core`; `AR-0009` retains
that decision.

## Kernel Admission Discovery

There is credible pressure for some part of this model to become native Tokimu
meaning:

- existing C# consumers repeatedly need a dependable logical resource space;
- Tokimu corpus consumers already need uploaded, related, generated, and
  inspectable resources;
- importers naturally need roots, folders, lookup, and diagnostics before an
  object becomes a prepared asset;
- native and WASM applications need equivalent identity without sharing
  platform storage mechanisms.

The plan must therefore discover the admission shape rather than treating
kernel admission as either automatic or forbidden.

### Candidate Boundary

The current hypothesis is:

```text
possible narrow kernel meaning
    stable logical identity vocabulary
    provider-neutral qualification
    bounded identity diagnostics

foundational resource capability
    roots and explicit folder hierarchy
    address normalization and visibility
    create/open and conflict semantics
    deterministic navigation and mutation

replaceable providers
    in-memory byte retention
    filesystem import/export
    browser upload/download
    persistence and synchronization

applications and consumers
    display names
    quotas and trust policy
    persistence expectations
    interpretation and asset admission
```

This is a study hypothesis, not an accepted dependency boundary. In
particular, a broadly useful `StoreId` does not automatically prove that a
resource-store-specific identifier belongs in `tokimu-core`.

### Admission Questions

Implementation and corpus evidence must answer:

1. Which types express universal engine identity rather than resource-store
   convenience?
2. Can the identity vocabulary remain useful without byte storage, paths,
   folders, or one registry implementation?
3. Which operations must every Tokimu application be able to rely on even when
   providers differ?
4. Does deterministic replay or observation require stable resource identity?
5. Must identity survive process, persistence, package, or transport
   boundaries?
6. Can `tokimu-assets` consume resource bytes without owning resource identity?
7. Do C# consumers and Tokimu corpus consumers converge on the same contract,
   or merely use similar names for different concepts?
8. Can a foundational capability satisfy all pressure without adding anything
   to the trusted core?

### Required Admission Artifact

Maintain an admission matrix while implementing the slices:

| Candidate semantic | C# consumers | Tokimu corpus consumers | Headless | Native/WASM parity | Proposed owner | Evidence status |
| --- | --- | --- | --- | --- | --- | --- |
| Store identity | duplicate-name failure recorded | `registry_uses_stable_store_identity_not_display_name_or_content`; `synchronized_registry_creation_preserves_one_store_per_stable_identity`; `hello-resource-space` | proven | build-and-contract parity: the same Workbench engine compiles for WASM and uses an explicit transient store | under review | Rust and executed C# behavioral evidence; persistent identity remains unproven |
| Root identity | root ambiguity recorded | `roots_and_stores_qualify_identical_relative_addresses`; `changing_a_root_display_name_does_not_change_identity`; Asset Workbench session root | proven | build-and-contract parity: explicit transient selection root compiles for WASM | under review | Rust evidence; real browser chooser run open |
| Folder identity/navigation | missing-folder failure recorded | `empty_and_hidden_folders_are_navigable_by_explicit_query`; `document_bundle_preserves_explicit_navigation_and_replacement_intent`; native directory adapter | proven | partial: same-folder session policy compiles for WASM; nested selection remains unsupported | foundational candidate | native and WASM policy evidence; nested browser policy open |
| Resource key | partial historical pressure | `roots_and_stores_qualify_identical_relative_addresses`; `resource_enumeration_copy_and_move_are_deterministic`; Workbench glTF sidecar resolution | proven | build-and-contract parity: constrained selected names and sidecar resolution compile for WASM | under review | Rust and executed C# behavioral evidence; browser observation remains open |
| Content fingerprint | pending | `content_fingerprints_are_named_diagnostics_not_resource_identity` | proven | provider-specific | capability/provider boundary | diagnostic-only evidence; no identity admission claim |
| Byte retention | partial historical pressure | `resources_retain_shared_immutable_bytes_and_qualified_addresses`; `ResourceSpaceBenchmark`; `repeated_browser_reads_do_not_change_session_retention` | proven | provider-specific: browser boundary returns a copy while session retention remains stable | provider | two consumer implementations; persistence evidence open |
| Import/export | directory round-trip and import/export tests executed | native approved-root export and explicit directory policy; `resource_session_returns_selected_bytes_without_exposing_host_paths` | no | partial: generated WASM/host build passes, real multi-file browser selection open | platform provider | native and WASM boundary evidence; C# behavioral workflow comparison complete |
| Visibility policy | hidden-resource failure recorded | `hidden_resources_remain_directly_addressable_without_leaking_into_visible_lists`; direct navigation and native include/skip/reject policy tests | proven | pending | foundational candidate | native evidence; browser visibility policy not yet needed |

Every row must eventually record concrete tests and consumers, not only a yes/no
opinion. The first stable Rust API should remain relocatable until this matrix
supports an AR disposition.

### Possible Admission Outcomes

AR-0009 must ultimately choose one of these outcomes:

- **No kernel admission:** publish a companion library or capability and keep
  all semantics outside `tokimu-core`.
- **Narrow kernel identity admission:** admit only universal identity and
  qualification concepts; keep hierarchy and storage in a foundational
  capability.
- **Foundational native capability:** make resource-space semantics a standard
  Tokimu capability without placing implementation in the trusted core.
- **Combined kernel store admission:** admit the store itself only if evidence
  proves its full semantics are universal engine truth. This is the highest-risk
  outcome and is not the current default.
- **Relocation or retirement:** replace the provisional model if consumers
  expose a simpler existing owner.

## Goals

- Provide deterministic in-memory storage on native and WASM targets.
- Make address normalization and case sensitivity explicit.
- Make store identity, root identity, and content identity distinct.
- Make hidden-resource visibility explicit and queryable.
- Provide explicit folders, including empty and hidden folders, for navigation.
- Prevent accidental duplicate logical stores through create/open semantics.
- Use immutable shared bytes such as `Arc<[u8]>` or an equivalent bounded type.
- Preserve useful metadata without conflating metadata with decoded assets.
- Support atomic insert, replace, remove, copy, and move behavior.
- Provide explicit limits for untrusted browser and corpus inputs.
- Translate source behavior into Rust tests and property tests.
- Integrate through adapters rather than format-specific methods on the store.

## Non-Goals

- Replacing `tokimu-assets::AssetStore`.
- Transparent disk persistence or database synchronization.
- Async APIs for operations that are intrinsically in-memory.
- Embedding JSON, XML, image, SVG, glTF, or FBX semantics in the base store.
- Operating-system file watching.
- Distributed consistency, locking, or transactions across processes.
- A virtual filesystem with every host filesystem behavior.
- Symbolic links, hard links, mounts, junctions, and host permission semantics
  in the initial folder model.

## Candidate Contract

The exact names remain provisional, but the semantic surface should resemble:

```rust
pub struct ResourceAddress { /* normalized, provider-neutral identity */ }
pub struct StoreId { /* stable identity, not a display name */ }
pub struct ResourceRootId { /* stable root within a store */ }
pub struct FolderId { /* stable folder identity within one root */ }

pub struct ResourceKey {
    pub store: StoreId,
    pub root: ResourceRootId,
    pub address: ResourceAddress,
}

pub struct ResourceEntry {
    pub key: ResourceKey,
    pub bytes: Arc<[u8]>,
    pub metadata: ResourceMetadata,
}

pub enum ResourceVisibility {
    Visible,
    Hidden,
}

pub enum ResourceNode {
    Folder(FolderEntry),
    Resource(ResourceEntry),
}

pub struct FolderEntry {
    pub id: FolderId,
    pub store: StoreId,
    pub root: ResourceRootId,
    pub parent: Option<FolderId>,
    pub name: ResourceName,
    pub metadata: FolderMetadata,
}

pub enum StoreOpenMode {
    CreateNew,
    OpenExisting,
    CreateOrOpen,
}

pub trait ResourceStore {
    fn get(&self, address: &ResourceAddress) -> Option<ResourceEntry>;
    fn insert(&mut self, entry: ResourceEntry) -> Result<StoreChange, StoreError>;
    fn remove(&mut self, address: &ResourceAddress) -> Result<StoreChange, StoreError>;
    fn list(&self, prefix: &ResourceAddress) -> Vec<ResourceSummary>;
}
```

The contract should prefer explicit changes and errors over bool-returning
operations. Normal misses may use `Option`; invalid addresses, limit failures,
and conflicts should use structured errors.

An implementation must not expose an unrestricted public constructor that can
silently create a second logical store with an existing identity. A registry or
factory should resolve `StoreOpenMode`, return a handle to the existing store,
or emit a structured identity conflict.

Each root owns one distinguished root-folder node. Ordinary folders are
explicit children of that node. Resource and folder names share one child
namespace by default, so one parent cannot contain both a folder and resource
with the same normalized name. This avoids navigation that resolves one name
differently depending on which API is called.

The incubating in-memory provider currently uses caller-supplied stable IDs and
supports **empty-only** removal for both ordinary folders and roots. Recursive
removal remains deferred until resource entries and their mutation guarantees
are present; no hierarchy operation silently deletes descendants.

## Slice 1: Provenance And Behavior Inventory

### Deliverables

- [x] Record source provenance and licensing before copying implementation.
- [x] Inventory every public C# operation and all source tests at the
      source-review level; the source contains 83 test cases at review time.
- [x] Classify identity, hierarchy, format, and adapter behavior as port,
      adapt, replace, defer, or reject in the C# comparison note.
- [x] Record incompatibilities caused by URI, filesystem, case, or async
      assumptions.
- [x] Record every source behavior involving hidden files, roots, store names,
      duplicate construction, and content comparison.
- [x] Record the consequences of the source model having no explicit folders.
- [x] Create a first-party document-bundle fixture independent of the source
      repository, including explicit folders and replacement intent.

### Acceptance Criteria

- [x] Source-level behavior and the executed C# public test suite have a
      recorded disposition; a shared cross-language report format remains
      optional diagnostic work rather than a Slice 9 requirement.
- [x] No C# code or fixtures were copied. `corpus/lib/resource-space/PROVENANCE.md`
      records the maintainer-owned MIT source project, and the first-party
      document-bundle fixture is independently authored.
- [x] The inventory distinguishes semantic behavior from C# convenience APIs.
- [x] Native-only behavior is clearly separated from the base contract.
- [x] The four known failure classes have explicit regression fixtures.
- [x] Folderless hierarchy behavior has explicit empty-folder and navigation
      fixtures.

## Slice 2: Address And Metadata Semantics

### Deliverables

- [x] Define normalized resource addresses and traversal rejection.
- [x] Define stable `StoreId`, `ResourceRootId`, and qualified `ResourceKey`
      semantics.
- [x] Define root creation, lookup, rename, and empty-only removal behavior.
- [x] Define explicit case-sensitive and case-insensitive policies.
- [x] Define explicit visible/hidden metadata and enumeration filters.
- [x] Define resource metadata, media type hints, and timestamps.
- [x] Define deterministic address ordering and segment-aware prefix semantics.
- [x] Add table-driven normalization tests across Windows-like, URL-like, and
      logical resource paths.

### Acceptance Criteria

- [x] Equivalent addresses normalize identically under one selected policy.
- [x] The same relative path under two roots cannot resolve to the same key.
- [x] Changing a display name does not change root identity.
- [x] `..`, invalid separators, empty segments, and ambiguous roots have
      deterministic outcomes.
- [x] Hidden resources remain directly addressable and appear only when the
      selected visibility query includes them.
- [x] No default behavior silently changes between native and WASM.
- [x] Metadata does not expose filesystem-specific types.

## Slice 3: Explicit Folder Hierarchy And Navigation

### Deliverables

- [x] Define one distinguished root folder for every `ResourceRootId`.
- [x] Define stable `FolderId`, normalized child names, and parent identity.
- [x] Implement create, get, rename, move, and empty-only remove folder operations.
- [x] Implement deterministic `list_children`, `list_folders`, and
      `list_resources` navigation.
- [x] Preserve empty folders and folder metadata.
- [x] Apply visibility policy to direct folder navigation explicitly.
- [x] Define collision policy for resources and folders sharing one name.
- [x] Define recursive versus empty-only folder removal.

### Acceptance Criteria

- [x] An empty folder remains visible and navigable.
- [x] Direct-child listing never substitutes an unbounded descendant search.
- [x] Parent navigation reaches the root folder without string manipulation.
- [x] Folder rename or move updates descendant qualification atomically.
- [x] A failed subtree move leaves the complete hierarchy unchanged.
- [x] Hidden folders are addressable directly and enumerated only by explicit
      visibility policy.
- [x] A resource/folder name collision produces a structured conflict.
- [x] Roots remain distinct identities and cannot be moved beneath folders.

## Slice 4: In-Memory Content Operations

### Deliverables

- [x] Implement insert, get, replace, remove, contains, and enumerate.
- [x] Implement copy and move with explicit conflict policy.
- [x] Store immutable shared byte content without unnecessary full copies.
- [x] Add total-byte, entry-count, and per-entry limits.
- [x] Return structured mutation outcomes suitable for diagnostics.
- [x] Distinguish exact content equality from resource and store identity.

### Acceptance Criteria

- [x] CRUD and mutation behavior is deterministic.
- [x] Failed mutations do not partially change the store.
- [x] Enumeration order is stable across repeated runs.
- [x] Limit violations are reported before unbounded allocation.
- [x] Tests run without a filesystem, window, renderer, or network.

## Slice 5: Store Registry And Identity Conflicts

### Deliverables

- [x] Add explicit create, open, and create-or-open operations.
- [x] Reject duplicate `StoreId` creation even when display names or content
      happen to match.
- [x] Permit duplicate display names only when callers retain distinct stable
      identifiers and policy explicitly allows it.
- [x] Add optional algorithm-qualified content fingerprints for diagnostics and
      candidate deduplication without using them as store identity; exact
      equality verifies retained bytes after a fingerprint match.
- [x] Add store origin/provenance metadata suitable for native and browser
      consumers.
- [x] Add concurrent creation tests around the registry boundary; reentrancy
      remains deferred until a consumer introduces callbacks or nested opens.

### Acceptance Criteria

- [x] Two `CreateNew` requests for one `StoreId` cannot produce two stores.
- [x] `OpenExisting` returns the established logical store or a not-found error.
- [x] Similar-but-different content does not compare equal by the selected
      BLAKE3 diagnostic fingerprint in the regression corpus.
- [x] Identical bytes may share storage without merging resource identity.
- [x] Diagnostics identify store ID, display name, origin, and conflict reason.

## Slice 6: Search, Hash, And Observation

### Deliverables

- [x] Add bounded prefix, suffix, and media-type queries justified by consumers.
- [x] Add content hashing with an explicit algorithm identity.
- [x] Add explicit `VisibleOnly`, `HiddenOnly`, and `All` query behavior.
- [x] Keep recursive search distinct from direct folder navigation.
- [x] Add optional mutation observations without introducing a global event bus.
- [x] Add summary statistics for current roots, folders, resources, and
      retained bytes without exposing provider internals.
- [x] Benchmark large listings, repeated reads, and shared-content copies with
      a deterministic headless workload. It records local timing observations
      without claiming a machine-independent performance contract and proves
      copied entries retain separate keys while sharing immutable bytes.

### Acceptance Criteria

- [x] Search is bounded and does not imply undocumented glob semantics.
- [x] Hash results are deterministic and algorithm-qualified.
- [x] Default listing visibility is documented and consistent across APIs.
- [x] Search results identify their parent folder and qualified resource key.
- [x] Observation can be disabled and has explicit ordering.
- [x] Statistics distinguish current values from lifetime counters.

## Slice 7: Native And Browser Adapters

### Deliverables

- [x] Add explicit native directory/file import and export helpers.
- [x] Add browser upload and download adapters through the WASM boundary.
- [x] Preserve logical resource addresses rather than leaking host paths.
- [x] Translate host hidden-file attributes or dot-name conventions through an
      explicit import policy.
- [x] Require an explicit target root for every import.
- [x] Preserve imported empty directories when policy requests it.
- [x] Emit structured diagnostics for rejected files, limits, and collisions.
- [x] Add sandbox and path-containment tests for native export.

### Acceptance Criteria

- [x] The base store has no filesystem or browser dependency.
- [x] Native export cannot escape its approved root.
- [x] Browser import obeys byte and entry limits.
- [x] Equivalent imported bytes produce equivalent resource entries.
- [x] Import never changes the selected root because of host path syntax.
- [x] Hidden host resources are imported, skipped, or rejected according to a
      recorded policy rather than disappearing implicitly.
- [x] Host directories lower to folder nodes without leaking host handles or
      platform path objects.

## Slice 8: Format And Asset Bridges

### Deliverables

- [x] Add JSON helpers through `serde` in `resource-space-json`, outside the
      base store. The adapter owns typed conversion and JSON diagnostics while
      Resource Space retains only logical identity, immutable bytes, and caller
      metadata.
- [x] Add XML resolution through a dedicated adapter that consumes
      `xml-tools`; `xml-tools` remains parser-neutral and does not resolve
       resource-store references itself.
- [x] Add a loader bridge into `tokimu-assets` without sharing ownership.
- [x] Exercise one external-resource SVG/XML case.
- [x] Exercise external glTF buffer and image-reference resolution cases.

### Acceptance Criteria

- [x] Format parsing can be replaced without changing store semantics.
- [x] `tokimu-assets` receives bytes through a loader contract, not direct store
      internals.
- [x] A selected logical folder resolves one glTF external buffer without a
      filesystem dependency; a missing buffer reports an explicit bridge error.
- [x] A selected logical folder resolves one external glTF image reference
      without decoding it; a missing image reports an explicit bridge error.
- [x] Missing same-folder and unadmitted local/parent XML references produce
      bounded diagnostics. Recursive XML reference graphs remain deferred.
- [x] A failed decode leaves the original resource available for inspection.
- [x] Typed JSON can round-trip through one explicit logical folder without
      making `serde` or JSON a dependency of the base store.

## Slice 9: Consumer Corpus And Admission Evidence

### Deliverables

- [x] Integrate a transient selected-file resource session into the WASM Asset
      Workbench engine boundary; the browser runtime build remains a separate
      validation step.
- [x] Build the Asset Workbench TypeScript shell, generated WASM bindings, and
      ASP.NET host together after the resource-session integration.
- [x] Bound each Asset Workbench resource session to explicit entry, per-entry,
      and aggregate byte limits; over-limit selection leaves retained session
      state unchanged.
- [x] Add `hello-resource-space` as the first headless, non-UI consumer;
      retain a second independent consumer as a graduation requirement.
- [x] Record existing C# MemoryStore public workflows against the same semantic
      matrix through its executed 83-test suite, including directory round-trip
      behavior and explicit intentional divergence notes.
- [x] Record build-and-contract native/WASM parity evidence: the Asset
      Workbench engine compiles for `wasm32-unknown-unknown`, while the native
      suite exercises the same `ResourceSession` external-buffer contract.
      A real browser chooser observation remains a separate acceptance check.
- [x] Add a deterministic mutation-sequence matrix for core resource
      operations and summary/navigation invariants. General-purpose generated
      property testing remains deferred until this bounded matrix exposes a
      counterexample or a second consumer needs broader sequence generation.
- [x] Populate every admission-matrix row with tests, consumers, and observed
      ownership pressure.
- [x] Emit a deterministic public conformance artifact from the headless
      consumer so a future persistent provider can compare semantic results
      without reproducing in-memory implementation details.
- [x] Capture same-ID create-or-open behavior, descriptor preservation, and
      case-policy mismatch rejection in the public conformance artifact.

### Acceptance Criteria

- [x] One headless consumer and one independent Asset Workbench engine consumer
      use the same semantic contract without TypeScript reimplementing glTF
      dependency resolution.
- [ ] The generated WASM/browser path has been exercised with a multi-file
      external-buffer glTF selection.
- [x] C# and Rust consumer comparison distinguishes real convergence from API
      resemblance.
- [x] Neither consumer reaches into provider internals; both compile as
      separate consumers over the `resource-space` public contract.
- [x] Warm repeated reads do not duplicate retained content unexpectedly; the
      headless corpus verifies shared immutable copy storage and the browser
      session verifies repeated selected-byte reads retain one bounded entry.
- [x] The Asset Workbench identifies the selected document and same-folder
      sidecars, then exposes the bounded Rust session summary after inspection
      so browser multi-file glTF evidence is directly observable.
- [x] Evidence identifies whether each candidate is universal, foundational,
      provider-owned, or application-owned.
- [x] The conformance artifact explicitly distinguishes public semantic
      comparison from persistence, transaction, synchronization, or
      cross-process identity evidence.
- [x] A provider comparison can verify that stable store identity is never
      inferred from display names or content, and that reopening cannot silently
      replace an existing descriptor or case policy.

## Slice 10: Architectural Admission Disposition

### Deliverables

- [x] Update `AR-0009` with the completed admission matrix.
- [x] Attempt decomposition without kernel changes and record the result.
- [x] Map every provisional public type and operation to its proposed owner.
- [x] Select one explicit admission outcome from this plan: provisional
      foundational-capability admission under ADR-0005, outside
      `tokimu-core`.
- [x] Record compatibility and migration consequences: retain the contract in
      `corpus/lib/resource-space`, preserve public semantic conformance, and
      defer crate extraction, facade re-exports, and a durable provider trait
      until Tosumu or another independent persistent provider applies pressure.
- [x] Decide ADR handling: no ADR is created or revised because the outcome is
      provisional under ADR-0005 and introduces no binding permanent boundary.
- [x] Decide provisional API handling: no APIs are relocated or retired in this
      outcome; their current corpus housing is the intentional reversible
      location until persistent-provider evidence arrives.

### Current Decomposition Attempt

The present consumers succeed without a `tokimu-core` addition. This is a
positive result for the proposed foundational-capability boundary, not a final
admission decision. The following ownership map is intentionally provisional
until a persistent provider and the observed browser selection either preserve
or challenge it.

| Provisional public surface | Proposed owner | Reason |
| --- | --- | --- |
| `StoreId`, `ResourceRootId`, `FolderId`, `ResourceKey` | foundational resource capability | They qualify logical resources but do not express universal simulation identity. |
| `ResourceAddress`, `ResourceName`, case policy | foundational resource capability | Normalization and traversal rules are resource addressing semantics, not platform paths. |
| `ResourceMetadata`, visibility, deterministic navigation, search, mutation observation, summaries | foundational resource capability | Both consumers need these provider-neutral facts before format or asset admission. |
| `InMemoryResourceSpace`, `InMemoryResourceSpaceRegistry`, immutable retained bytes | replaceable provider | These are one implementation of retention and create/open mechanics. |
| `ResourceSpaceLimits` | application/consumer policy | Limits reflect a browser session or workload budget rather than universal storage truth. |
| native import/export and browser upload/download | platform adapters | They translate host mechanisms into logical entries and must not leak paths or handles upward. |
| JSON, XML, glTF, and `tokimu-assets` bridges | format/asset adapters | They consume resource bytes without owning hierarchy or source identity. |

The decomposition currently conforms to ADR-0001 and ADR-0003: no filesystem,
browser, database, parser, renderer, or byte-retention implementation enters
`tokimu-core`.

### Provisional Disposition

Under ADR-0005, Resource Space is provisionally admitted as a **foundational
capability candidate**. This admits provider-neutral logical resource meaning:
store/root/folder/resource qualification, normalized addresses, visibility,
deterministic navigation, bounded observations, and diagnostics. It does not
admit a persistent storage implementation, byte-retention strategy, filesystem
or browser mechanism, database dependency, synchronization, encryption,
freshness, format parser, or asset lifecycle into the capability contract or
the trusted core.

The capability remains physically incubated in `corpus/lib/resource-space`
until a Tosumu-backed or otherwise independent persistent provider consumes
the public contract. The conformance artifact is the comparison surface; it
does not prescribe in-memory storage layout or registry mechanics. A future
crate extraction is intentionally a packaging migration, not evidence that the
semantic boundary has changed.

### Acceptance Criteria

- [x] The disposition names what enters the kernel, what remains foundational,
      and what remains provider/application behavior. Nothing enters the
      trusted core in this provisional outcome.
- [x] Decide kernel admission: none is made. The provisional outcome preserves
      a zero-dependency trusted core and leaves Resource Space capability-local.
- [x] ADR-0005 substitute evidence is identified separately from completed
      normal evidence: historical C# failures and 83-test behavioral evidence,
      two independent Rust consumers, native/WASM contract parity, and the
      deterministic public conformance artifact support the reversible
      capability admission while persistent-provider evidence remains open.
- [x] No API is stabilized merely because corpus implementation already uses
      it: there is no facade export, independent capability crate, or durable
      provider trait in this provisional outcome.
- [x] The resulting dependency direction conforms to ADR-0001 and ADR-0003.
- [x] The review can state what belongs in the library and what remains adapter
      behavior.

## Graduation Criteria

The resource store may graduate from corpus incubation when:

- two independent consumers use it without format-specific leakage;
- native and WASM behavior is deterministic where equivalent;
- address, conflict, quota, and observation policies are explicit;
- store, root, resource, and content identity remain separate;
- folder identity and direct-child navigation remain stable under empty,
  hidden, renamed, and moved folder cases;
- duplicate logical stores are impossible through the public creation API;
- hidden-resource behavior is explicit and consistent across direct access,
  enumeration, import, export, and format resolution;
- the `tokimu-assets` ownership boundary remains intact;
- source behavior has a complete port/defer/reject ledger;
- tests cover ordinary, malformed, adversarial, and resource-limit cases.

## Remaining Evidence Before Graduation

The provisional admission closes the current ownership question without
claiming graduation. Only these evidence gaps remain material before a
permanent capability decision:

- [ ] Observe the ASP.NET/WASM Asset Workbench selecting `Box.gltf` together
      with `Box0.bin`, and retain the visible session summary as browser-boundary
      evidence.
- [ ] Complete persistent-provider proof with Tosumu or another independent
      provider: compare public behavior with
      `resource-space-provider-conformance-v1`, classify each divergence as
      provider-only, a contract refinement, or rejected semantics, then reopen
      `AR-0009` for permanent extraction, continued incubation, or retirement.

## Current Answers And Deliberate Deferrals

### Resolved By Current Evidence

| Question | Current answer | Evidence / boundary |
| --- | --- | --- |
| Where is case policy selected? | Per logical store, immutable after create/open. | Address normalization and registry mismatch tests reject a later conflicting policy. |
| Which Resource Space identities are kernel-native? | None in the provisional outcome. | `StoreId`, root, folder, and resource qualification remain capability-local. |
| May display names collide? | Yes. Distinct stable `StoreId` values remain authoritative. | Registry tests allow duplicate display names and reject duplicate IDs. |
| Is visibility one portable bit? | V1 uses explicit `Visible` / `Hidden` state and query policy. | Extension beyond this pair requires new consumer pressure; host hidden-file flags do not leak upward. |
| Are folders independently stable? | Yes. `FolderId` is independent of its current address. | Rename and move preserve the folder identity while derived resource addresses change. |
| How is folder removal handled? | V1 exposes empty-only removal. Recursive removal and transactions are not admitted. | This prevents implicit destructive subtree behavior and leaves durable atomicity provider-owned. |
| Are mutation observations foundational? | They are optional, bounded diagnostic observations in the capability contract, not a sync log or global event bus. | The deterministic mutation window is exercised by `hello-resource-space`. |
| May entries share content? | Yes, provider storage may share immutable bytes, while each copy has a separate logical key. | Copy benchmarks and identity tests keep byte equality distinct from logical identity. |
| Which metadata is authoritative? | The store is authoritative for its retained visibility, media-type hint, and timestamps at the observation boundary; provenance, host attributes, and decoded-format claims remain advisory. | `ResourceMetadata` is provider-neutral and does not infer decoded asset truth or host path semantics. |

### Deferred Without Blocking Graduation

- **Persistent `StoreId` scope:** Tosumu must establish whether a stable store
  ID survives reopen and process boundaries, and how durable create/open
  conflicts are reported.
- **Virtual or derived folders:** no current consumer needs them. They remain
  outside V1 rather than coexisting speculatively with explicit materialized
  folders.
- **Content-addressed deduplication or aliases:** provider optimization may
  share bytes, but no public alias or content-addressed identity model is
  admitted without a consumer requiring it.
