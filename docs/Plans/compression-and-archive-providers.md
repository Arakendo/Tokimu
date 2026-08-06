# Compression And Archive Providers

## Status

In progress. The historical C# `CompressionTools` project is pinned and
classified in the
[migration ledger](../Notes/compression-tools-migration-ledger.md). An
incubating provider-neutral byte contract now exists under
`corpus/lib/compression-provider`; no provider or permanent capability boundary
is architecturally admitted yet. The incubation crate includes
`flate2`-backed GZip/raw-Deflate and `brotli`-backed raw-Brotli
implementations. An explicit `resource-space-compression` bridge now composes
bounded transformations with logical lookup and mutation. A bounded ZIP reader
now incubates in `archive-provider`, with a headless corpus proving manifest
inspection and selected-entry reads. An explicit `resource-space-archive`
bridge now proves bounded archive inspection and selected-entry materialization.
The ZIP and TAR providers write deterministic archives from ordered byte entries
under explicit limits and normalized metadata. `hello-archive` now proves that
TAR and GZip compose through bounded bytes without a combined provider. It also
proves deterministic ZIP export of an explicit nested Resource Space subtree.
Archive-backed views, TAR metadata extensions, and broader browser runtime
evidence remain open. The Asset Workbench now supplies bounded ZIP, TAR, and 7z
subtree-import consumers; each is deliberately consumer policy rather than a
base Resource Space behavior. The 7z provider now also creates fresh bounded
archives from validated entries; update, password, multi-volume, and
metadata-parity behavior remain deferred.

### Format Roles

ZIP and TAR are the current canonical archive contracts: each has bounded
provider-neutral inspection and selected reads, deterministic writing, and
logical-content differential coverage. Consumers may select either format
without changing Resource Space semantics. This does not make either container
kernel-native; it records the read/write behavior Tokimu currently promises at
the archive-provider boundary.

7z is a bounded compatibility provider. It may inspect, validate, import, and
create a fresh archive from admitted entries. Regular files use provider-
selected 7z compression (`ArchiveCompression::Other`); ZIP-specific `Stored`
and `Deflate` requests are rejected rather than silently reinterpreted. Update,
password, multi-volume, and metadata-parity behavior remain unsupported.

## Purpose

Port the useful behavior demonstrated by
`F:\LocalSource\ClassLibrary\CompressionTools` into bounded, provider-neutral
Tokimu companion capabilities for:

- compressing and decompressing one byte payload;
- inspecting, reading, and writing archive containers;
- moving archive entries to and from Resource Space without restoring the old
  `MemoryStore` dependency;
- exposing deterministic limits, observations, and diagnostics to native and
  WASM consumers.

This is a semantic migration, not a line-by-line C# translation. The source
project combines codecs, archives, filesystem helpers, backup workflows,
multi-volume naming, and `MemoryStore` integration. Tokimu should preserve the
proven behaviors while separating decisions that have different owners.

## Source Evidence

The source project is MIT licensed and currently demonstrates:

- GZip, Brotli, and Deflate round trips;
- ZIP, TAR, and TAR.GZ creation, listing, and extraction;
- entry inspection without full archive extraction;
- compression ratios and size observations;
- GZip and ZIP signature detection;
- ZIP integrity checks and comparisons;
- individual-entry and whole-archive workflows;
- direct filesystem wrappers and backup helpers;
- `MemoryStore` integration;
- multi-volume archive naming and assembly.

Its tests exercise byte and string round trips, compression levels, archive
entry operations, filesystem workflows, in-place backups, and format
comparisons. Slice 0 must pin the exact source revision and classify every
public behavior before implementation begins.

## Primary Architectural Claim

Applications and capabilities request bounded transformations. Providers own
codec and container mechanisms. Resource Space owns logical resource identity.
Platform adapters own filesystem access.

```text
application or capability bytes
        |
        v
provider-neutral request + limits
        |
        v
compression or archive provider
        |
        v
bytes, manifest, observations, or structured failure
        |
        +-----------------------+
        |                       |
        v                       v
Resource Space adapter     platform/file adapter
logical resources          explicit host mechanism
```

The renderer, world, runtime scheduler, and trusted kernel do not parse
compression or archive formats.

## Separate Semantic Boundaries

### Byte Compression

Byte compression transforms one payload into another. It owns:

- codec identity;
- encode and decode requests;
- bounded decode policy;
- deterministic result observations;
- structured codec failures.

It does not own entry names, folders, archive manifests, files, or backups.

### Archive Containers

Archive containers package ordered named entries. They own:

- archive format identity;
- entry metadata and ordering;
- bounded inspection and extraction;
- safe entry-name validation;
- duplicate-name and integrity policy;
- structured container failures.

They do not own Resource Space identity or host filesystem paths.

### Resource Space Integration

The adapter maps validated archive entries to explicit Resource Space folders
and resources. Resource Space continues to own:

- store, root, folder, and resource identity;
- explicit folder objects;
- visibility and navigation;
- duplicate logical-address policy;
- provider-neutral mutation observations.

An archive entry name is untrusted input, not a Resource Space address until
the adapter validates and lowers it.

Archive integration is **subordinate to Resource Space at the consumer-facing
composition boundary**, not inside the base Resource Space dependency graph.
The bridge may depend on both Resource Space and archive contracts; Resource
Space must not depend on ZIP, TAR, filename extensions, or concrete codec
providers.

```text
application
    |
    v
Resource Space archive facade / extension
    |
    v
archive-resource bridge
    |                  |
    v                  v
Resource Space     archive provider
identity + bytes   manifests + entries
```

This permits concise application operations such as:

```text
inspect archive resource
import archive into folder
export folder as archive
open archive as a read-only derived view
```

without changing the meaning of ordinary Resource Space reads and writes.
`read_resource` always returns the bytes retained for that resource. It never
silently decompresses, extracts, mounts, or reassembles an archive.

#### Convenience Operations

The first integration should provide explicit sugar through an extension trait,
facade, or orchestration service rather than methods on the foundational
Resource Space contract itself. Candidate operations are:

- inspect one resource as a bounded archive and return a manifest;
- import selected or all validated entries into an explicit destination folder;
- export an explicit Resource Space subtree through a selected archive provider;
- report provider availability, limits, collisions, and partial-work policy;
- optionally copy one archive entry into Resource Space without mounting the
  complete archive.

Format selection is explicit. Optional detection may inspect bounded magic
bytes, but must not rely only on `.zip`, `.tar`, or similar names.

#### Archive-Backed Views

A later read-only archive view may make entries navigable through Resource
Space-shaped observations without first materializing every payload. Such a
view is derived provider behavior, not an ordinary retained Resource Space
subtree. It must expose:

- the source archive resource identity and content fingerprint or revision;
- archive-provider identity and format;
- normalized entry identity beneath a qualified derived root;
- read-only status and extraction limits;
- invalidation when the source archive changes;
- explicit copy/import operations for materialization into Resource Space.

Writes, rename, and removal are rejected in the initial view. Copy-on-write,
archive repacking, and writable mounts require separate evidence because they
introduce transaction, atomicity, and source-revision questions. Archive views
must not cause Resource Space to admit general host-style mounts or virtual
filesystem semantics prematurely.

#### Automatic Behavior Boundary

Automatic behavior is acceptable only as caller-selected policy, such as
`open_as_archive` choosing a provider after bounded signature inspection. The
following remain prohibited:

- decompression during an ordinary resource read;
- automatic mounting based on a filename extension;
- automatic recompression after a logical mutation;
- archive entries appearing as durable resources without provenance;
- hidden extraction limits, collisions, unsupported entries, or diagnostics.

### Platform And Filesystem Integration

Host adapters may read or write files, enumerate directories, choose backup
locations, and perform atomic replacement. Those mechanisms remain outside the
base compression and archive contracts.

## Provisional Semantic Model

Exact names remain intentionally unstable.

```rust
pub enum CompressionCodec {
    Gzip,
    Brotli,
    Deflate,
}

pub struct DecodeLimits {
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_expansion_ratio: Option<u32>,
}

pub struct CompressionObservation {
    pub codec: CompressionCodec,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

pub struct ArchiveManifest {
    pub format: ArchiveFormat,
    pub entries: Vec<ArchiveEntryObservation>,
}

pub struct ArchiveReadLimits {
    pub max_archive_bytes: u64,
    pub max_entries: u32,
    pub max_entry_bytes: u64,
    pub max_total_output_bytes: u64,
    pub max_path_bytes: u32,
}
```

Compression levels should begin as semantic goals such as `Fast`, `Balanced`,
and `Small` unless evidence proves that provider-native numeric levels must be
exposed. Provider-specific tuning may remain an extension rather than becoming
the portable contract.

## Safety And Determinism Rules

- Every decode and extraction operation has explicit input and output limits.
- Archive entry names containing absolute paths, drive prefixes, parent
  traversal, NUL characters, or invalid normalization are rejected.
- Symlinks, hard links, devices, and other non-regular entries are rejected or
  explicitly deferred until a consumer proves their semantics.
- Duplicate normalized names produce a structured policy outcome; they never
  overwrite silently.
- Malformed, truncated, encrypted, or unsupported content fails explicitly.
- CRC or checksum mismatches remain distinguishable from syntax failures.
- Archive inspection must not require extraction of every payload.
- Archive writing uses stable entry ordering and normalized metadata where the
  admitted format permits deterministic output.
- Detection is advisory and only promised for self-identifying envelopes.
  Raw Brotli and Deflate are not reliably detectable from arbitrary bytes.
- No implementation allocates from untrusted size declarations before limits
  are checked.
- Diagnostics are bounded and structured rather than accumulated without
  limit.

## Initial Incubation Layout

The first implementation should incubate beneath `corpus/lib` so examples and
consumers can pressure the contracts before public extraction.

```text
corpus/lib/
    compression-provider/       # provisional byte-codec boundary
    archive-provider/           # provisional container boundary
    resource-space/             # existing logical-resource capability

corpus/
    hello-compression/           # focused codec corpus
    hello-archive/               # manifest and extraction corpus
```

The implementation may begin in one package for speed, but modules and public
types must preserve the semantic split. A permanent `tokimu-compression`,
`tokimu-archive`, combined support crate, or external companion repository is
an admission result, not a premise.

## Goals

- Replace historical `CompressionTools` dependencies with portable Rust
  semantics rather than recreating the C# utility surface.
- Support deterministic native and WASM byte workflows.
- Make dangerous input limits part of every decode and extraction request.
- Integrate archives with Resource Space without coupling providers together.
- Give Resource Space consumers explicit archive-aware convenience operations
  without widening the foundational Resource Space contract.
- Investigate a provenance-preserving read-only archive view after import and
  export behavior is stable.
- Preserve enough observations for corpus artifacts and diagnostics.
- Permit established Rust codec/container crates below Tokimu-owned contracts.
- Prove equivalent logical content across supported providers and formats.

## Non-Goals

- Adding compression or archive parsers to `tokimu-core` or `tokimu-runtime`.
- Porting `MemoryStore`; Resource Space is its semantic replacement.
- A general backup application or directory synchronization service.
- Silent in-place filesystem mutation in the base capability.
- Archive encryption, password management, signing, or key ownership.
- RAR support, 7z update semantics, or multi-volume archive support merely because
  historical volume names imitate those ecosystems.
- Multi-volume archives in the initial correctness slices.
- Transparent compression hidden inside every asset or Resource Space write.
- Filename-driven auto-mounting or implicit decompression during ordinary
  Resource Space reads.
- Writable archive mounts, copy-on-write overlays, or automatic repacking in
  the initial correctness slices.
- Treating compression as integrity, authentication, encryption, or freshness.
- Preserving C# static classes, exception behavior, callback APIs, or naming.

## Slice 0: Provenance And Behavior Ledger

### Deliverables

- [x] Record source repository, exact revision, license, public API inventory,
      and test inventory.
- [x] Classify every source behavior as port, adapt, replace, defer, or reject.
- [x] Separate byte codecs, archive semantics, Resource Space adapters,
      filesystem helpers, and application workflows.
- [x] Identify mature Rust codec and archive provider candidates with license,
      native, and WASM compatibility notes.
- [x] Create first-party tiny, malformed, high-expansion, traversal, duplicate,
      and truncation fixtures.

### Acceptance Criteria

- [x] No source behavior disappears without a recorded disposition.
- [x] No copied code or fixture lacks provenance.
- [x] Provider choices do not redefine the portable semantic contracts.
- [x] Multi-volume, backup, encryption, and filesystem-only behaviors are
      visibly classified rather than silently omitted.

### Current Pause Notes

- Provider review is complete for pinned `flate2` 1.1.9 GZip/raw-Deflate,
  `brotli` 8.0.4 raw-Brotli, and `zip` 8.6.0 stored/Deflate reading. TAR still
  requires exact version, license, feature, native, and WASM review before its
  later slice begins.
- First-party malformed and high-expansion codec cases now exist as tests.
  ZIP evidence now adds tiny valid, traversal, normalized-duplicate, symlink,
  encryption, malformed central-directory, CRC corruption, truncation, and
  bounded-output cases.

## Slice 1: Bounded Compression Contract

### Deliverables

- [x] Define codec identity, encode request, decode request, limits,
      observations, and errors.
- [x] Distinguish explicit codec selection from advisory envelope detection.
- [x] Define semantic compression goals separately from provider-native levels.
- [ ] Define deterministic diagnostics and cancellation behavior where the
      execution boundary supports cancellation.
- [x] Add contract tests independent of a concrete codec implementation.

### Acceptance Criteria

- [x] Callers can request a bounded transformation without filesystem or
      Resource Space dependencies.
- [x] Output limits are enforced while decoding, not after full allocation.
- [x] Raw Brotli or Deflate is never reported as reliably auto-detected.
- [x] Errors distinguish unsupported codec, malformed input, truncation,
      exceeded limits, and provider failure.

### Current Pause Notes

- Cancellation remains open until a real execution boundary can demonstrate
  cooperative cancellation without placing scheduler semantics in this crate.
- The selected flate provider enforces output and expansion budgets while
  collecting decoder chunks; later providers must pass the same contract.

## Slice 2: GZip, Brotli, And Deflate Provider

### Deliverables

- [x] Implement GZip encode/decode through one selected Rust provider.
- [x] Implement Brotli encode/decode through one selected Rust provider.
- [x] Implement Deflate encode/decode and document raw versus wrapped form.
- [x] Port representative source round-trip and compression-observation tests.
- [x] Add native/WASM build and deterministic fixture checks.

### Acceptance Criteria

- [x] Empty, small, Unicode, binary, incompressible, and repetitive payloads
      round trip byte-for-byte.
- [x] Truncated and expansion-limit fixtures fail with the expected category.
- [ ] Equivalent requests produce equivalent decoded content on native and
      WASM targets.
- [x] Provider-native types do not escape the contract.

### Current Pause Notes

- GZip and raw Deflate are implemented through pinned `flate2` 1.1.9.
- Raw Brotli is implemented through pinned `brotli` 8.0.4. The provider maps
  semantic goals to quality 2, 6, and 11 and uses a fixed 22-bit window.
- `brotli` reports both malformed bytes and a truncated valid stream as
  `InvalidData`; the adapter therefore classifies both as malformed rather
  than inventing an unsupported truncation distinction.
- `corpus/hello-compression` now exercises all three codecs and goals through
  the public contract and retains a bounded structural report. This is the
  first application-shaped codec consumer; it is not browser runtime parity.
- WASM evidence currently proves compilation of the same implementation. A
  browser consumer must still prove equivalent decoded bytes at runtime.

## Slice 3: Resource Space Transformation Facade

### Deliverables

- [x] Read source bytes through public Resource Space semantics.
- [x] Write results through explicit Resource Space mutations.
- [x] Preserve source and result identity in bounded observations.
- [x] Define collision and overwrite policy explicitly.
- [x] Define an archive-aware facade or extension contract without adding
      codec/container methods to the foundational Resource Space API.
- [x] Add explicit inspect and selected-entry-copy requests.
- [x] Add an explicit subtree-export request after traversal and
      archive-writing semantics have executable evidence.
- [x] Add subtree import through a caller-selected, transient Resource Space
      destination policy. This is an explicit consumer operation, not a base
      Resource Space mutation API.
- [x] Report supported formats and provider availability without leaking
      provider-native types.
- [x] Add in-memory and Tosumu-backed consumer evidence through the public
      Resource Space contract. This proves the current consumer-local durable
      host composition; it is not evidence of an independent persistent
      Resource Space provider.

### Acceptance Criteria

- [x] The adapter does not depend on historical `MemoryStore` APIs.
- [x] Compression behavior is identical across the current in-memory and
      Tosumu-backed consumer sessions.
- [x] Failed transforms do not leave an ambiguous partial logical resource.
- [x] Provider-owned persistence details remain outside observations.
- [x] Ordinary Resource Space reads return retained bytes unchanged.
- [x] Format selection or detection is explicit, bounded, and diagnostic.

### Current Pause Notes

- `corpus/lib/resource-space-compression` depends on Resource Space and the
  compression contract; neither foundational contract depends back on the
  bridge.
- One request explicitly selects source and destination identities, encode or
  decode semantics, codec, metadata, and `Reject` or `Replace` collision
  behavior. No filename or media type triggers hidden transformation.
- Transformation completes before insertion or replacement. Unit evidence
  proves malformed decode creates no destination, rejection preserves existing
  bytes, and replacement is explicitly observed.
- `hello-compression` retains source, encoded, and decoded resources and emits
  their addresses in its structural report. Ordinary source lookup remains
  byte-identical after both transformations.
- The .NET Resource Workbench now executes `resource.transform_compression`
  through both an in-memory session and a fresh-process Tosumu-backed session.
  Its `compression-provider-conformance-v1.json` artifact proves matching
  public GZip encode/decode observations, source retention, and restored bytes.
  This is durable host-composition evidence: the consumer-local Tosumu adapter
  persists and restores an `InMemoryResourceSpace` snapshot. It does not prove
  an independently implemented persistent Resource Space provider.
- `corpus/lib/resource-space-archive` depends on Resource Space and the archive
  contract; neither foundational contract depends back on the bridge.
- Inspection preserves source identity and content fingerprint without a
  mutation. Selected-entry copy requires an explicit normalized entry name,
  destination name, metadata, limits, format, and `Reject` or `Replace`
  collision policy.
- Entry decoding and validation complete before mutation. Unit evidence proves
  failed reads create no destination and rejected collisions preserve existing
  bytes.
- `export_resource_subtree` traverses one caller-selected Resource Space folder
  deterministically, lowers child folders and resources to validated archive
  entries, and lets the selected writer own container bytes. The source root is
  intentionally not emitted as a synthetic archive directory.
- Archive-to-Resource-Space subtree import now exists only where the consumer
  owns folder allocation and its transient import destination. The bridge
  validates every entry before materialization, and does not claim a
  caller-neutral batch transaction or durable all-or-nothing hierarchy import.

## Slice 4: ZIP Inspection And Bounded Reading

### Deliverables

- [x] Define archive format, manifest, entry metadata, limits, and diagnostics.
- [x] Inspect ZIP entries without extracting all payload bytes.
- [x] Read one selected regular-file entry by normalized name.
- [x] Reject traversal, absolute paths, duplicate normalized names, unsupported
      entry kinds, encrypted entries, and exceeded limits explicitly.
- [x] Add malformed central-directory, CRC, truncation, and archive-bomb cases.

### Acceptance Criteria

- [x] Manifest inspection is bounded by archive and entry-count limits.
- [x] Entry extraction cannot escape a provider-neutral destination namespace.
- [x] Integrity failures remain distinguishable from unsupported semantics.
- [x] No host filesystem path appears in the base ZIP API.

### Current Evidence

- `corpus/lib/archive-provider` defines provider-neutral ZIP observations and
  bounded selected reads with no filesystem or Resource Space dependency.
- Pinned `zip` 8.6.0 is MIT licensed and built with default features disabled;
  only Deflate support is enabled. Native tests and WASM compilation pass.
- Twenty-three adversarial and positive tests cover safe normalization, duplicate
  names, symlinks, directories, encryption, malformed/truncated central
  directories, CRC corruption, independent archive/count/entry/total-output
  budgets, deterministic writing, bounded writer failures, 7z traversal and
  declared-entry-size rejection, plus fresh 7z writer round-trips and write
  compression-policy rejection.
- `corpus/hello-archive` consumes an immutable first-party ZIP fixture solely
  through `ArchiveProvider`, retains normalized manifest evidence, verifies one
  byte-identical selected read, and proves an archive-input budget rejection.
  The same corpus now retains the fixture in Resource Space, inspects it through
  the explicit bridge, and materializes one selected entry under a
  caller-selected logical name while preserving the source bytes.
- `resource-space-archive` has ten bridge tests and compiles for
  `wasm32-unknown-unknown`; browser runtime parity remains unclaimed. Whole-tree
  Resource Space import/export and archive-backed views belong to later slices.

## Slice 5: ZIP Writing And Resource Space Extraction

### Deliverables

- [x] Write ZIP archives from ordered byte entries.
- [x] Lower a Resource Space subtree into validated archive entry names.
- [x] Extract validated entries into explicit Resource Space folders and
      resources.
- [x] Define stable ordering, timestamps, permissions, and metadata policy.
- [x] Add archive-write-read conformance tests.
- [ ] Add provider-crossing Resource Space conformance tests.
- [x] Exercise bounded archive inspection through the same facade against
      in-memory and Tosumu-backed Resource Space sessions.
- [x] Exercise bounded archive subtree import through the same facade against
      in-memory and Tosumu-backed Resource Space sessions.

### Acceptance Criteria

- [x] Equivalent ordered byte input produces deterministic output under the
      documented metadata policy.
- [x] Round trips preserve names and bytes for admitted regular-file entries.
- [x] Archive-to-Resource-Space folder creation is explicit and observable.
- [x] Duplicate or conflicting destinations never overwrite silently.
- [x] Export preserves the same observations as the lower-level archive and
      Resource Space contracts.
- [x] Import preserves the same observations as the lower-level archive and
      Resource Space contracts for the admitted ZIP directory-and-file fixture.

### Current Evidence

- `ArchiveWriter` is separate from `ArchiveProvider`, so read-only providers do
  not falsely claim write support.
- `ZipArchiveProvider` writes only caller-ordered regular files and explicit
  directories using Stored or Deflate compression. It normalizes names, rejects
  duplicates and unsafe paths, fixes timestamps and permissions, and accepts no
  host filesystem metadata.
- Writer limits bound entry input and estimated/final archive output. A bounded
  cursor provides a second output-limit defense while the ZIP is assembled.
- Provider tests prove identical input produces byte-identical ZIP output and
  that the public reader recovers the same normalized names and payload bytes.
- `hello-archive` repeats the same write twice, requires byte identity, reads the
  result through the public archive contract, and records the observation in
  `target/hello-archive/report.json`.
- `resource-space-archive` exports one explicit subtree through a selected
  `ArchiveWriter`. It preserves Resource Space traversal order, emits child
  directories explicitly, rejects logical names that would alter archive
  hierarchy, and returns only archive-owned write observations plus bytes.
  Ten focused bridge tests cover stable hierarchy, unsafe logical names, and
  inspection/copy collision guarantees.
- `hello-archive` creates a nested Resource Space subtree, exports it through
  ZIP, re-inspects the resulting bytes through `ArchiveProvider`, and records
  the one-folder/two-resource/three-entry result in its report.
- `import_archive_subtree` now eagerly reads every admitted regular entry,
  parses every destination component with the selected Resource Space case
  policy, and materializes a caller-selected destination root with a
  caller-provided deterministic folder-ID range. Its observation retains the
  immutable source identity/fingerprint plus created-folder, created-resource,
  and retained-byte counts. A destination-root collision is rejected before
  mutation, so imports never silently overwrite an existing logical entry.
  Full transactional rollback for a later Resource Space capacity failure is
  intentionally not claimed; batch mutation remains pending provider-crossing
  evidence.
- The .NET Resource Workbench bridge exposes the same bounded subtree import
  request. Its `archive-provider-conformance-v1.json` artifact compares the
  in-memory result with a fresh-process Tosumu reopen: ZIP manifest, explicit
  imported root and child folders, retained leaf metadata, and exact entry
  bytes are equal. This is durable host-composition evidence only because the
  current Tosumu adapter persists a consumer-local in-memory snapshot rather
  than providing an independent Resource Space implementation.
- The same bridge now imports the equivalent deterministic TAR fixture into the
  same logical folder/resource tree as ZIP. This confirms that canonical
  container selection stays below Resource Space semantics rather than being a
  ZIP-shaped import contract.
- `TarArchiveProvider` now admits the same bounded regular-file and directory
  subset as ZIP. It fixes write metadata, rejects links and extended record
  kinds explicitly, and reports no invented entry checksum where TAR has no
  portable CRC-32 field. Provider tests compare equivalent ZIP and TAR inputs
  through normalized names, kinds, sizes, and selected payload bytes.
- `resource-space-archive` now also exports an admitted subtree through the
  create-only 7z writer. The bridge forwards the provider-selected
  `ArchiveCompression::Other` request and explicitly rejects ZIP-style Deflate
  requests. This proves archive-provider crossing while intentionally leaving
  the unchecked cross-Resource-Space-provider evidence requirement intact.
- `hello-archive` proves TAR-plus-GZip composition by writing an admitted TAR
  subset, encoding it through the independent GZip codec, bounded-decoding the
  bytes, and inspecting and reading the result through the TAR provider again.
  ZIP may request per-entry Deflate while the current TAR subset requires
  Stored entries, so the corpus supplies separate but logically equivalent
  write requests rather than obscuring that provider-policy difference.
  Neither provider claims the other's format semantics.
- The .NET Tosumu Resource Workbench bridge now exposes explicit bounded
  `resource.inspect_archive` requests. Its headless contract runner retains
  opaque ZIP bytes as an ordinary resource, compares the provider-neutral
  manifest after in-memory and fresh-process Tosumu-backed sessions, and emits
  `target/resource-space-conformance/dotnet-tosumu-resource-workbench/archive-provider-conformance-v1.json`.
  The compared boundary is source bytes and metadata plus normalized manifest
  entries; the .NET host does not inspect ZIP internals. This is durable
  host-composition evidence only, so the independent provider-crossing and
  transactional subtree-import criteria remain unchecked.

## Slice 6: TAR And TAR.GZ Provider

### Deliverables

- [x] Inspect and read the admitted regular-file TAR subset.
- [x] Write deterministic TAR archives from ordered byte entries.
- [x] Compose TAR with GZip without merging their semantic contracts.
- [x] Classify links, sparse files, and extended headers; preserve platform
      metadata as deferred provider-only evidence.
- [x] Add differential logical-content tests against ZIP.

### Acceptance Criteria

- [x] TAR.GZ visibly composes an archive provider with a byte codec provider.
- [x] Unsupported TAR entry kinds produce explicit diagnostics.
- [x] Equivalent admitted ZIP and TAR fixtures lower to equivalent logical
      manifests and bytes.
- [x] Platform metadata does not silently alter Resource Space semantics.

## Slice 7: Explicit Platform File Adapters

### Deliverables

- [x] Add a native-only bounded read/write adapter outside the archive and
      Resource Space contracts.
- [x] Define atomic create-new publication and partial-file cleanup behavior.
- [x] Keep backup naming and directory-recursion policy consumer-owned.
- [x] Add missing-file, non-file, collision, input/output-limit, and temporary
      cleanup tests.
- [ ] Add portable permission-denied and interrupted-write cases through a
      controllable native-file abstraction or platform-specific consumer.

### Current Evidence

- `archive-file-adapter` reads regular host files through an explicit byte
  limit, then passes the resulting bytes to the selected archive provider.
  Host I/O failures remain distinct from archive failures.
- Output uses a same-directory temporary file, syncs it, then creates a hard
  link at the requested destination. This publishes only when the destination
  does not exist and never performs implicit replacement. The temporary link is
  removed after publication or a failed attempt.
- Cross-platform replacement of an existing file is intentionally deferred.
  It needs a separately reviewed platform policy because standard-library
  rename behavior differs by operating system. Backup naming, replacement, and
  recursive directory operations remain consumer-owned.
- The native tests cover missing inputs, directories presented as inputs,
  bounded reads, collision refusal, and temporary cleanup. Permission denial
  and interrupted writes remain explicitly unclaimed because they need a
  controllable platform boundary to be portable evidence.

### Acceptance Criteria

- [x] Base codec and archive crates remain free of filesystem assumptions.
- [x] In-place replacement is never the implicit default.
- [x] Host failures are labeled separately from codec/container failures.
- [x] Native-only adapters do not weaken WASM compatibility of semantic crates.

## Slice 8: Corpus And Provider Conformance

### Deliverables

- [x] Add codec provider conformance fixtures and result artifacts.
- [x] Add a first archive-provider fixture and manifest artifact with
      reproducibility metadata.
- [x] Add a ZIP-versus-TAR logical differential fixture and result artifact.
- [ ] Add differential fixtures for future archive-provider implementations.
- [x] Exercise one Resource Space consumer through explicit ZIP inspection,
      bounded entry materialization, and deterministic subtree export.
- [x] Exercise one asset-loading consumer through the ASP.NET/WASM Asset
      Workbench's bounded archive import and selected-entry inspection.
- [x] Exercise the ASP.NET/WASM Asset Workbench through bounded ZIP, TAR, and
      7z imports; Rust/WASM validates entries before materializing an explicit
      transient Resource Space subtree and selects a supported imported entry
      for normal inspection.
- [x] Record initial archive provider identity, versions, limits, and input
      hashes.
- [x] Record equivalent provenance metadata for the admitted codec fixtures.
- [ ] Record the equivalent metadata for future archive-provider fixtures.
- [x] Add deterministic adversarial seeds for codec headers, archive metadata,
      and entry names.
- [ ] Add a dedicated fuzz target only when another provider or CI corpus
      runner can consume the shared seeds.

### Acceptance Criteria

- [x] Admitted failure fixtures localize to codec, archive, adapter, platform, or consumer
      boundaries.
- [x] The first archive corpus artifact is reproducible and provenance-aware.
- [ ] Corpus artifacts remain reproducible and provenance-aware across the
      provider matrix.
- [x] ZIP and TAR can be compared without changing consumer semantics.
- [x] 7z consumes bounded inspection, transient import, and fresh create-only
      writer paths without acquiring update or multi-volume semantics.
- [ ] A second implementation of either container format can be tested without
      changing consumer semantics.
- [x] Browser evidence does not rely on browser-native decompression to claim
      Tokimu support.

### Current Evidence

- `hello-archive` writes schema-2 `target/hello-archive/report.json` with a stable
  first-party fixture selection, a BLAKE3 input fingerprint, provider/library
  identities, and the exact read/write bounds used for the run.
- The report preserves structural manifest observations rather than treating
  container bytes as the only correctness oracle. ZIP and TAR may differ in
  container encoding while still be compared through normalized entry
  semantics.
- The corpus writes logically equivalent ZIP and TAR archives, then requires
  normalized entry names, kinds, uncompressed sizes, `docs/readme.txt` bytes,
  and `data.bin` bytes to agree. It deliberately does not require identical
  container bytes, compression metadata, or ZIP-only CRC fields.
- This is a format differential, not yet an implementation-provider matrix. A
  second implementation of either format remains required before the broader
  acceptance criteria can close. The ASP.NET/WASM Asset Workbench now supplies
  bounded ZIP, TAR, and 7z import consumers: TypeScript transfers only selected
  bytes, Rust/WASM validates every admitted entry before mutation, materializes
  an explicit transient folder/resource tree, and selects the first supported
  imported entry for ordinary inspection. The browser presents the semantic
  observation without browser-native archive decoding. The workbench engine
  also builds successfully for `wasm32-unknown-unknown`. The 7z provider now
  proves fresh bounded archive creation through the public writer trait:
  directories round-trip, regular files use provider-selected compression, and
  unsafe names, declared oversize entries, ZIP-style compression requests, and
  output-limit violations fail explicitly. This remains a create-only contract;
  the production Workbench has no update, password, or multi-volume 7z flow. A
  manual browser file upload remains separately labeled evidence rather than an
  inferred result.
- `hello-compression` now writes `target/hello-compression/report.json` with
  the selected fixture fingerprint, Flate/Brotli provider identities, and the
  exact round-trip and bounded-decode limits used for the artifact. Its codec
  matrix remains provider-neutral while the provenance makes later comparison
  possible.
- `hello-archive` now gives the create-only 7z compatibility writer its own
  structural evidence: the same ordered logical tree is rebuilt, inspected,
  compared with ZIP at the logical-manifest boundary, and read back through a
  selected entry. The artifact records the provider identity and its observed
  byte-stable rebuild. This is fresh-output evidence only, not a claim that
  existing 7z archives can be updated safely.
- `resource-space-archive` now exercises the same create-only writer through
  the ordinary subtree-export bridge. The bridge forwards
  `ArchiveCompression::Other` without format guessing, and the provider
  rejects a ZIP-style Deflate request. This proves Resource Space does not
  absorb 7z compression policy merely because it can export a logical tree.
- Failure localization is covered at the narrowest honest boundary: codec
  output-limit rejection remains a codec result; malformed ZIP bytes remain an
  `ArchiveError` after the native adapter reads them; missing or non-file host
  paths remain adapter/platform results; and the Resource Space bridge exposes
  its own consumer-level failures. Future browser evidence must preserve the
  same distinction rather than wrapping all failures as a generic load error.
- `resource-space-archive` supplies the Resource Space composition proof. Its
  fixture tests retain source identity during inspection, require a caller to
  select both archive entry and destination before materialization, preserve
  rejected-collision bytes, and export a subtree deterministically. Ordinary
  Resource Space reads remain byte-faithful and do not acquire archive behavior.
- The .NET Tosumu workbench proves durable reopen behavior and compares public
  Resource Space observations for ordinary resource operations and the GZip
  compression round trip with an in-memory session. It does **not** yet satisfy
  the remaining independent-provider archive/compression criterion: its
  consumer-local Tosumu adapter persists and restores the in-memory Resource
  Space model rather than independently implementing Resource Space or the
  archive bridge contract. That distinction keeps the unchecked
  provider-conformance items honest.
- `archive-provider` retains named ZIP structural and unsafe-entry-name seeds;
  `compression-provider` retains named malformed Gzip and Brotli prefixes.
  The current tests assert that these cases remain classified provider failures
  rather than becoming successes or generic cross-boundary errors. They are
  deliberately source-level deterministic seeds, not a claim of broad fuzzing
  coverage yet.

## Slice 9: Advanced Archive Evidence

### Scope Decision

ZIP and TAR remain the canonical archive contracts under active development:
bounded read/write behavior, deterministic output, and logical-content
equivalence. 7z now has a narrower create-only compatibility writer; its
inspection, selected-read, and fresh-write behavior may receive safety or
interoperability hardening, but update, password, multi-volume, and
metadata-parity work are not Slice 9 work. They require new consumer pressure
and an explicit plan revision.

### Deliverables

- [ ] Re-evaluate ZIP/TAR append or update semantics only after a consumer
      proves that deterministic repacking is insufficient.
- [ ] Re-evaluate ZIP/TAR multi-volume creation, naming, assembly, and failure
      policy only after a consumer proves it needs those semantics.
- [ ] Re-evaluate random-access streaming and non-seekable providers.
- [ ] Re-evaluate archive comparison and integrity-report APIs.
- [x] Prototype a read-only archive-backed derived view after import/export
      semantics stabilize.
- [x] Record qualified entry identity, archive revision/fingerprint,
      invalidation, and materialization behavior for that view.
- [x] Compare eager import with lazy archive-backed navigation without claiming
      that both have the same persistence semantics.
- [ ] Open focused reviews for encryption or authenticated packaging if real
      consumers require them.

### Acceptance Criteria

- [ ] No advanced behavior is admitted solely for source parity.
- [ ] Multi-volume semantics have at least one real ZIP/TAR consumer and a
      malformed-set corpus before admission. 7z multi-volume support remains
      outside the current compatibility contract.
- [ ] Encryption remains separate from compression and Resource Space identity.
- [ ] Deferred features retain explicit reasons and graduation triggers.
- [x] A derived archive view cannot be mistaken for an ordinary mutable or
      durable Resource Space subtree.
- [ ] Writable mounts, repacking, and copy-on-write remain deferred unless a
      real consumer proves their transaction requirements.

### Current Evidence

- `resource-space-archive` exposes `ArchiveDerivedView` as a separate,
  read-only projection. It carries the source `ResourceKey`, source content
  fingerprint, format, and validated entry observations, but it does not
  allocate Resource Space folders or retain entry payloads. A caller may read
  one selected entry through the view, but that read still does not
  materialize it.
- Reopening after a source replacement yields a different fingerprint, which
  gives callers an explicit invalidation signal. Copy and subtree import remain
  the only materialization operations. This is not yet lazy payload navigation,
  a writable mount, or a persistence model.

## Slice 10: Admission Review And Extraction

### Deliverables

- [ ] Compare evidence from codecs, archives, Resource Space, assets, native,
      and WASM consumers.
- [ ] Decide whether byte compression and archives graduate separately,
      together, remain incubating, or move to external companion projects.
- [ ] Record dependency direction and public stability expectations.
- [ ] Update the SDD, relevant Architectural Review, and ADR if ownership is
      accepted permanently.
- [ ] Remove duplicate corpus-side implementations after the decision.

### Acceptance Criteria

- [ ] At least two independent consumers use each contract proposed for
      graduation.
- [ ] Provider implementation types do not leak into consumer APIs.
- [ ] Resource Space, platform, persistence, and codec/container ownership
      remain distinct.
- [ ] The result is a documented architectural decision, not merely a crate
      extraction.

## Validation Matrix

| Boundary | Required evidence |
| --- | --- |
| Byte codecs | Round trip, malformed input, truncation, limit enforcement |
| Detection | Positive GZip/ZIP signatures and explicit unknown results |
| ZIP inspection | Manifest bounds, duplicate names, CRC, encrypted entries |
| Entry safety | Traversal, absolute path, drive prefix, NUL, long name |
| Archive writing | Stable ordering, metadata policy, read-after-write |
| Resource Space | In-memory and persistent provider conformance |
| Archive facade | Explicit inspect/import/export behavior; unchanged ordinary reads |
| Derived view | Qualified provenance, read-only policy, invalidation, materialization |
| Platform adapter | Atomic output, permissions, collision, interruption |
| Native/WASM | Equivalent decoded bytes and structured diagnostics |
| Adversarial | High expansion, excessive entries, malformed size fields |

## Risks And Mitigations

### Risk: A Utility Drawer Reappears

Mitigation: keep codecs, archives, Resource Space, platform files, and backup
workflows in separate ownership boundaries even if one incubation crate hosts
the first implementation.

### Risk: Convenience Becomes Hidden Transformation

Mitigation: expose archive operations through an explicit facade or extension,
keep ordinary Resource Space reads byte-faithful, and require caller-selected
inspection, import, export, or derived-view behavior.

### Risk: Archive Input Escapes Its Destination

Mitigation: validate archive names before lowering, reject unsafe entry kinds,
and target provider-neutral sinks rather than concatenating filesystem paths.

### Risk: Decompression Exhausts Memory Or Time

Mitigation: require limits, enforce them during streaming decode, bound
diagnostics, and add high-expansion corpus fixtures before public use.

### Risk: Provider APIs Become The Public Contract

Mitigation: translate provider errors and options at one adapter boundary and
exercise conformance through consumer-owned types.

### Risk: Compression Is Mistaken For Security

Mitigation: state explicitly that compression supplies neither secrecy,
authenticity, integrity provenance, nor freshness. Tosumu and future package
security remain separate capabilities.

## Open Questions

- Should compression and archive semantics graduate as separate crates?
- Is a streaming source/sink contract required initially, or can bounded byte
  buffers establish the first honest contract?
- Which semantic compression goals are portable enough across providers?
- Should archive manifests preserve original unsafe names for diagnostics while
  refusing to lower them?
- Are explicit directory entries meaningful, or should folders be derived from
  validated regular-file names?
- Which ZIP timestamp and permission fields belong in portable metadata?
- Does archive update/append deserve a semantic contract or remain a provider
  extension?
- Does the bounded create-only 7z writer remain interoperable with independent
  readers, and do real inputs expose unsupported compression or encryption
  methods that need explicit diagnostics?
- Does Resource Space need transactional batch mutation before extraction can
  guarantee atomicity?
- Which consumers justify multi-volume support beyond historical parity?
- Should decoded-content fingerprints be standard observations or supplied by
  diagnostics consumers?

## Graduation Criteria

The effort is ready for permanent admission only when:

- codec and archive contracts each have independent consumers;
- all untrusted decode and extraction paths enforce explicit limits;
- Resource Space integration uses only its public semantics;
- native and WASM evidence agree for the admitted byte workflows;
- archive entry safety and duplicate policies are deterministic;
- providers can evolve without changing application meaning;
- no active consumer depends on the historical C# `CompressionTools` or
  `MemoryStore` implementations;
- relevant architecture documents record the final ownership and dependency
  direction.

## Recommended First Pause Point

Complete Slices 0 through 3 first. At that point Tokimu will have bounded byte
compression plus Resource Space pressure without prematurely accepting archive
topology, filesystem behavior, or multi-volume complexity. ZIP work should
begin only after that smaller contract is stable.

The compression-specific portion of this pause point is now complete for the
existing durable host composition. The remaining evidence is an independently
implemented persistent Resource Space provider; the archive-specific facade
items intentionally move with Slice 4 rather than being represented by empty
APIs.
