# AR-0011: Tosumu-Backed Tasset Canonical Asset Output

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-03 |
| Last reviewed | 2026-08-03 |
| Scope | Capability / backend / cross-cutting |
| Trigger | Tokimu needs a canonical editable asset output, and Tosumu needs a demanding embedded-database consumer that can apply real corpus pressure |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005 |
| Related reviews | AR-0006, AR-0009 |
| Related evidence | `docs/Conversations/Tokimu Asset (tasset).md`; `docs/Conversations/Tosumu/`; pinned Tosumu source checkout at `third-party/tosumu` (`a84a0c09c05de85751c1456151ac35afeb97da15`); Tokimu importer and consumer corpora |
| Admission exception | None; this review opens investigation and does not admit a stable format or dependency |

## Architectural Question

Should `.tasset` become Tokimu's primary canonical editable asset output, with
Tosumu as its first durable storage provider, while Tokimu continues to own the
asset model and keeps source interchange formats, cooked runtime resources, and
database mechanics behind separate boundaries?

## Context

Tokimu can already inspect or lower several external formats, including SVG,
CGM, glTF/GLB, FBX, and raster images. Those formats remain source or
interchange representations. They cannot, individually, preserve all meaning
that Tokimu may add after import, such as stable resource identity, provenance,
diagnostics, material relationships, animation metadata, collision data,
streaming hints, or later editor-authored semantics.

The proposed pipeline is:

```text
source or interchange format
        |
        v
Tokimu importer and canonical asset model
        |
        v
.tasset logical package
        |
        v
Tosumu storage adapter
        |
        v
Tosumu database and WAL mechanics
```

The corresponding runtime pipeline remains distinct:

```text
.tasset canonical editable asset
        |
        v
cook / prepare
        |
        v
runtime resources and asset handles
```

The `.tasset` proposal previously considered SQLite as a structured,
transactional application-file container. The project maintainer now prefers
to evaluate Tosumu so Tokimu becomes a real consumer of another project rather
than selecting SQLite by default. This is useful corpus pressure, but it does
not by itself justify coupling Tokimu asset meaning to Tosumu's current schema,
query surface, or on-disk format.

Tosumu is currently an embedded, page-based KV engine with B+ tree storage,
overflow pages for large values, write-ahead logging, crash recovery,
transactions, authenticated pages, and machine-readable inspection. Its SQL
layer is intentionally narrow and still developing. Its on-disk format is real
and documented, but explicitly pre-stability and not yet covered by long-term
compatibility guarantees.

Tosumu also currently uses a WAL sidecar. A live `.tasset` therefore cannot be
treated as a safely movable single file merely because the main database has a
single extension. Tokimu needs an explicit close, checkpoint, backup, or export
contract before a `.tasset` can be copied, published, or committed as one
portable artifact.

## Terminology And Scope

In this review, **primary output** means Tokimu's preferred canonical editable
asset representation after import or authoring. It does not mean:

- the preferred source format for artists or external tools;
- a universal interchange format;
- a database for live `World` state;
- a runtime hot-path resource package;
- an editor session, undo log, or application project file;
- the only storage backend that a Tokimu asset model may ever use.

An initial `.tasset` may contain:

- an asset manifest and schema/version declaration;
- stable asset-local object and resource identities;
- canonical semantic records and relationships;
- source provenance and importer identity/version;
- importer and validation diagnostics;
- encoded or canonical binary payloads where appropriate;
- dependency and derivation metadata;
- explicit capability requirements and deferred-feature records.

Whether revision history, derived caches, editor state, or cooked runtime data
belong in the same package remains outside this review until consumer evidence
demonstrates that ownership.

### Candidate Artifact Operating Modes

The storage lifecycle may require two explicitly different artifact modes:

```text
Working asset
    live database
    WAL and local transaction state
    optional local history or pending synchronization state

Portable asset
    closed and finalized
    checkpointed or exported
    self-contained and safe to move or publish
```

These are provisional operating modes, not admitted file variants. A working
asset must not be advertised as portable merely because its primary database
file has a `.tasset` extension. Conversely, a portable asset must not silently
retain operational state whose interpretation requires the original machine,
process, WAL, protector, or synchronization session.

## Trigger And Evidence

- Corpus examples: Tokimu's SVG, CGM, glTF/GLB, FBX, raster, font, and
  consumer corpora already produce provider-neutral observations and derived
  geometry that a canonical package could preserve.
- Automated tests: Tosumu's accepted `TOKIMU-001` provider-boundary suite
  passes 19 tests, with three explicitly ignored measurement workloads. It
  covers the shared multi-record fixture, 64 MiB values, recovery, backup,
  portable export, verification, structured newer-format and wrong-key errors,
  and database identity isolation. Tokimu has not yet tested a `.tasset`
  semantic round trip through Tosumu.
- Audits or diagnostics: Tosumu exposes structured inspection and verification
  surfaces. Tokimu has established a project-wide preference for explicit
  diagnostics over silent fallback.
- Independent consumers: Tosumu currently has CLI, TUI, WPF, and developing SQL
  consumers, but no Tokimu asset consumer. Tokimu has multiple importer and
  authoring consumers, but no accepted canonical editable asset output.
- Repeated implementation pressure: importer output, source provenance,
  diagnostics, resource relationships, and enriched asset meaning need a
  durable representation that is not defined by any one source format.
- Cross-project evidence: making Tokimu a consumer would exercise large values,
  transactions, reopen behavior, inspection, recovery, migrations, and
  cross-version compatibility against application-shaped data rather than only
  synthetic KV workloads.
- Missing evidence: canonical asset semantics and schema, deterministic
  Tokimu-side round-trip behavior, application-level rollback and recovery,
  Tokimu diagnostic translation, schema migration behavior, native/WASM
  strategy, and comparison with a simpler directory or archive provider.

The Tosumu documents copied into this repository were compared with the
authoritative local checkout on 2026-08-03. The current reproducible source is
the `third-party/tosumu` Git submodule, pinned at
`a84a0c09c05de85751c1456151ac35afeb97da15`. That checkout identifies Tosumu
as pre-audit and pre-stability even though substantial storage and recovery
work is implemented.

The submodule keeps Tosumu as an independently versioned repository. Tokimu
changes that require provider behavior belong in a Tosumu commit first; a
separate Tokimu commit then advances the pinned submodule revision together
with the adapter and corpus evidence that consumes it. A local filesystem path
is not part of this contract.

## Ownership Analysis

### Tokimu asset semantics

Tokimu should own:

- the meaning and lifecycle of a canonical editable asset;
- stable asset, object, resource, and relationship identities;
- schema identity and the semantic version of the `.tasset` contract;
- provenance, diagnostics, capability requirements, and deferred semantics;
- conversion between importer output, canonical asset records, and cooked
  runtime resources;
- compatibility policy and migration meaning between `.tasset` schema
  versions;
- provider-neutral observations used by applications and tools.

These semantics likely belong in an optional Tokimu asset-document capability,
not in `tokimu-core` or `tokimu-runtime`. This review does not name or admit a
new crate before the corpus proves its boundary.

### Tosumu storage semantics

Tosumu should own:

- database creation, open, close, checkpoint, backup, and recovery mechanics;
- pages, B+ tree records, overflow chains, WAL, fsync, and transaction behavior;
- database-local key/value ordering and query execution;
- authenticated storage, keyslots, protectors, and storage-integrity failures;
- physical format versions and Tosumu-format migrations;
- storage inspection, verification, and structured storage diagnostics.

If Tosumu later exposes a semantic change-log or synchronization substrate, it
may also own durable ordering mechanics, stable storage-level change identity,
watermarks, tombstone retention, transport-neutral freshness evidence, and
bounded conflict records. Those mechanisms remain future evidence; this review
does not claim that Tosumu currently provides a production-ready synchronization
contract.

Tosumu must not define what a mesh, material, animation, source diagnostic,
asset relationship, or Tokimu capability requirement means. It must likewise
not define that a material changed, a node was deleted, or an animation cue was
reordered. Those are Tokimu or application semantic changes even if Tosumu
eventually persists and moves their records.

### Adapter semantics

A Tokimu-to-Tosumu adapter should own:

- deterministic key namespaces and record encoding;
- mapping Tokimu schema versions to Tosumu keys and values;
- transactional save and migration execution;
- translation of Tosumu failures into Tokimu-owned asset-storage diagnostics;
- package finalization and portable export rules;
- implementation metadata sufficient to inspect which adapter and Tosumu
  storage versions produced an artifact.

No `tosumu-core` type should cross into Tokimu's author-facing, importer,
runtime-resource, TypeScript, or WASM semantic APIs.

### Applications, importers, and runtime

- Applications own when to import, save, migrate, publish, or cook an asset.
- Importers own source-format interpretation and loss/defer diagnostics.
- `tokimu-assets` owns runtime handle identity, loading state, generations, and
  prepared resource lifecycle; it does not own editable package storage.
- Renderers own prepared GPU resources and never query Tosumu directly.
- `World` state remains authoritative in memory and is not made database-owned.
- Resource Space may supply selected bytes and logical references, but its
  store/root/folder identity must not be conflated with `.tasset` records or a
  Tosumu database identity.

## Dependency Direction

```text
Current:

source formats
    -> importer-specific observations
    -> runtime or consumer-specific lowering

No accepted canonical editable asset package exists.

Proposed investigation:

Tokimu canonical asset semantics
    <- importer adapters
    -> cooker / runtime-resource adapters
    -> asset-storage provider contract
        -> Tosumu adapter
            -> tosumu-core
                -> filesystem mechanism
```

The dependency must remain one-way. `tokimu-core`, `tokimu-runtime`, importers,
renderers, and authoring frontends must not depend on Tosumu. Tosumu must not
depend on Tokimu. A future WASM or browser provider may implement the same
logical `.tasset` semantics without sharing Tosumu's native filesystem
mechanism.

## Alternatives Considered

### Alternative A: Tokimu-Owned Tasset Semantics With Tosumu As The First Provider

- Benefits: applies real corpus pressure to Tosumu; preserves Tokimu ownership;
  gains transactions, large values, inspection, recovery, and structured
  failures; permits later providers.
- Costs: requires a provider boundary, explicit schema codec, migration policy,
  and package-finalization semantics; both projects are evolving.
- Failure mode: the adapter becomes a nominal boundary while Tosumu keys,
  tables, errors, or lifecycle leak into Tokimu's public model.

### Alternative B: Define Tasset As A Raw Tosumu Database Schema

- Benefits: fastest implementation and direct inspection of physical records.
- Costs: freezes an unstable storage layout into Tokimu's public asset contract
  and makes provider replacement or WASM parity difficult.
- Failure mode: a Tosumu schema or format change silently becomes a breaking
  Tokimu asset-model change.

### Alternative C: Use SQLite As The First Provider

- Benefits: mature application-file precedent, broad tooling, stable format,
  strong migration and interoperability story.
- Costs: does not pressure Tosumu; introduces another external storage stack;
  can encourage schema-first asset APIs.
- Failure mode: implementation maturity prematurely settles an asset semantic
  model that has not been proven by Tokimu consumers.

### Alternative D: Use An Inspectable Directory Or Conventional Archive

- Benefits: simple tooling, visible files, straightforward version-control
  diffs, no database dependency, easier browser handling.
- Costs: transactions, relationships, indexing, and atomic migrations must be
  designed separately; many-small-file and partial-write behavior becomes
  application work.
- Failure mode: package mechanics are repeatedly reimplemented by consumers and
  gradually become an undocumented database.

### Alternative E: Keep Canonical Assets In Memory And Defer Tasset

- Benefits: avoids stabilizing both semantic and storage contracts too early.
- Costs: importer enrichment, provenance, diagnostics, and edited assets remain
  ephemeral; Tosumu receives no application-shaped pressure.
- Failure mode: consumer applications each invent incompatible project and
  asset serialization.

## Findings

The evidence currently supports these provisional findings:

1. Tokimu needs a canonical editable asset representation that is distinct
   from source interchange formats and cooked runtime resources.
2. Tosumu is a credible first storage-provider candidate because its embedded
   KV, overflow-value, transaction, recovery, inspection, and structured-error
   behavior matches meaningful `.tasset` pressures.
3. Tosumu is not yet credible as the definition of the `.tasset` compatibility
   contract. Its on-disk format and higher-level schema/query surface are
   explicitly pre-stability.
4. A live Tosumu database plus WAL is not automatically a portable one-file
   artifact. Package finalization and export are part of the adapter contract.
5. Tokimu asset schema migration and Tosumu physical-format migration are two
   different operations and require separate diagnostics.
6. The initial adapter should target the smallest stable Tosumu surface that
   can preserve deterministic keys and values. Depending on a developing SQL
   layer adds risk without current evidence that relational queries are
   required.
7. Authenticated storage is useful integrity evidence, but Tokimu must not
   claim confidentiality or production durability that Tosumu itself does not
   claim.
8. This relationship does not reopen ADR-0001. Persistence remains outside
   `tokimu-core` and `tokimu-runtime`.
9. A working asset and a portable asset are different lifecycle states. The
   former may include live WAL, local history, or pending synchronization state;
   the latter requires an explicit, self-contained finalization result.
10. Tosumu's authenticated-page and protector model could strengthen corruption
    detection for provenance, diagnostics, source material, and unpublished
    work. Authentication proves bounded integrity properties; it does not by
    itself prove secrecy, freshness, authorship, or suitability for hostile
    environments.
11. A future Tosumu semantic change log could support offline movement and
    collaboration, but Tokimu-owned semantic changes must be the replication
    unit. Physical pages and arbitrary provider key mutations are not an
    acceptable public synchronization model for `.tasset`.

The evidence does not yet establish:

- the complete `.tasset` semantic schema;
- whether one database should contain exactly one asset or an asset graph;
- whether editable history, undo state, or derived caches belong inside it;
- whether Tosumu can satisfy browser/WASM consumers;
- whether Tosumu should be mandatory rather than a preferred native provider;
- whether `.tasset` requires SQL, secondary indexes, or only deterministic KV
  records;
- long-term binary compatibility or production-safety guarantees;
- whether authenticated storage satisfies the threat model of any actual
  Tokimu consumer;
- whether live history or pending synchronization state belongs inside a
  working `.tasset`, a sidecar, or a separate project/workspace artifact;
- semantic change identity, actor identity, watermarks, conflict policy,
  tombstone lifecycle, and freshness/witness behavior;
- whether portable export intentionally retains, compacts, or removes local
  history and synchronization metadata.

## Disposition

**Incubating.** Open a bounded `.tasset` corpus using Tokimu-owned canonical
asset semantics and a Tosumu-backed experimental adapter. Tosumu is the
preferred first provider for this study, but neither Tosumu nor `.tasset` is
admitted as a stable Tokimu dependency or primary output contract until the
named round-trip, recovery, migration, and portability evidence exists.

## Consequences

- Tokimu gains a concrete path toward a canonical editable asset without
  making a database part of the kernel.
- Tosumu receives application-shaped pressure from large binary values,
  structured metadata, relationships, transactional edits, reopen, recovery,
  inspection, and migrations.
- Early `.tasset` artifacts are experimental and may require regeneration.
- The provider boundary adds code and diagnostic translation, but preserves
  native/WASM and backend options.
- Portable copying cannot be implemented as an unchecked copy of an open main
  database file.
- Tools and publishing workflows must distinguish a live working asset from a
  finalized portable artifact.
- Authenticated storage failures can become valuable bounded diagnostics, but
  applications must not translate them into stronger security claims than the
  provider guarantees.
- Future synchronization may reuse Tosumu ordering and durability mechanisms,
  but Tokimu remains responsible for semantic change and conflict meaning.
- Tools must expose both Tokimu semantic diagnostics and Tosumu storage
  diagnostics without confusing one for the other.
- Source files remain authoritative import provenance; `.tasset` records
  Tokimu's durable understanding and may be regenerated or migrated according
  to explicit policy.

## Required Follow-Up

- [ ] Define a minimal provider-neutral `.tasset` semantic model and schema
      version independent of Tosumu keys or SQL tables.
- [ ] Build one corpus entry that imports a known GLB asset, stores the
      canonical model through Tosumu, closes, reopens, and compares a
      deterministic semantic observation.
- [ ] Preserve source provenance, importer version, diagnostics, resource
      identities, relationships, and at least one large binary payload.
- [ ] Test atomic multi-record save and rollback on injected application error.
- [ ] Test Tosumu crash/recovery behavior at `.tasset` transaction boundaries.
- [ ] Define and test close/checkpoint/backup/export behavior that produces a
      safely movable artifact despite the WAL sidecar.
- [ ] Corrupt selected storage bytes and verify Tosumu storage failures become
      bounded Tokimu asset-storage diagnostics without partial semantic output.
- [ ] Define the consumer threat model and test which corruption, wrong-key,
      protector, and authenticated-reopen failures Tosumu can diagnose.
- [ ] Version the Tokimu asset schema separately from the Tosumu physical
      format and adapter codec versions.
- [ ] Exercise one semantic schema migration transactionally and verify both
      success and rollback.
- [ ] Compare the Tosumu adapter with a minimal in-memory or directory-backed
      test provider to prove that the canonical asset model is not
      Tosumu-shaped.
- [ ] Measure save, reopen, scan, large-value, and package-size behavior using
      corpus fixtures rather than synthetic-only records.
- [ ] Document native/WASM scope. If Tosumu remains native-only, preserve an
      explicit alternate provider or transfer representation for browser
      consumers.
- [ ] Re-review whether `.tasset` is one asset, one asset graph, or another
      bounded document only after the first consumers expose actual pressure.
- [ ] Define working-versus-portable artifact state and prove that finalization
      either resolves or deliberately preserves WAL, history, and pending sync
      state.
- [ ] If synchronization evidence becomes concrete, model one Tokimu semantic
      change independently from Tosumu pages and raw key mutations before
      admitting change-log or conflict APIs.
- [ ] If the evidence supports adoption, create an ADR defining `.tasset`
      semantics, provider ownership, compatibility, and migration guarantees.

## Initial Corpus Matrix

| Case | Pressure | Expected evidence |
| --- | --- | --- |
| GLB box round trip | Minimal mesh and metadata | Equivalent canonical observation after reopen |
| Animated GLB round trip | Animation relationships and payloads | Stable clip/channel identities and payload hashes |
| SVG or font vector asset | Multiple contours and geometry payload | Equivalent vector semantics, not source-format leakage |
| Raster-backed material | Large binary and color interpretation metadata | Stable image identity, payload hash, and requirement semantics |
| Failed transaction | Atomicity | No partially updated canonical asset |
| Crash boundary | WAL and recovery | Prior or committed asset state, never mixed state |
| Corrupt page/value | Failure translation | Structured storage diagnostic and no fabricated asset |
| Wrong key or protector | Authenticated open failure | Bounded integrity diagnostic without fabricated semantic output |
| Schema migration | Semantic compatibility | Deterministic migrated observation or full rollback |
| Portable export | Sidecar lifecycle | Reopen succeeds from the exported artifact alone |
| Working-to-portable transition | Artifact operating mode | Final artifact declares and satisfies its retained-history and sync-state policy |
| Semantic edit record | Future synchronization boundary | Tokimu change identity and meaning survive without exposing pages or provider keys |
| Alternate test provider | Boundary integrity | Same semantic observation without Tosumu types |

## Reopening Triggers

Reopen or advance this review when one or more of these occurs:

- the initial GLB-to-Tosumu `.tasset` round trip is deterministic;
- a second independent authoring or importer consumer requires the same
  canonical asset semantics;
- Tosumu cannot preserve a required payload, transaction, migration, or
  portability contract;
- Tosumu details leak into Tokimu-owned or TypeScript/WASM APIs;
- the WAL sidecar cannot be reconciled with portable file output;
- a simpler directory/archive provider satisfies the same requirements with
  materially less complexity;
- a browser consumer requires direct `.tasset` access that Tosumu cannot
  support;
- Tosumu publishes a stable format/migration contract suitable for durable
  Tokimu artifacts;
- `.tasset` scope expands toward projects, live world state, undo history, or
  cooked runtime packages and requires a separate ownership review.
- Tosumu exposes a semantic change-log, watermark, conflict, or freshness model
  that can be tested without making physical pages or raw keys the Tokimu
  replication unit.

## Review History

### Cycle 1 -- 2026-08-03

- Status entering review: Proposed
- New evidence: the `.tasset` design conversation, copied Tosumu design files,
  and the authoritative local Tosumu checkout were reviewed against Tokimu's
  accepted persistence and capability boundaries.
- Participants or reviewers: project maintainer and Codex implementation
  assistant.
- Findings: canonical asset meaning should remain Tokimu-owned; Tosumu is a
  promising but pre-stability storage provider; WAL portability, schema
  separation, and migration require direct corpus evidence.
- Disposition: Incubating.
- Resulting ADR or documentation change: none; this review opens a bounded
  implementation study.

### Cycle 2 -- 2026-08-03

- Status entering review: Incubating.
- New evidence: design discussion identified authenticated storage and future
  synchronization-shaped history as meaningful reasons to study Tosumu beyond
  ordinary blob persistence.
- Participants or reviewers: project maintainer, Monday review notes, and Codex
  implementation assistant.
- Findings: working and portable artifacts require separate lifecycle claims;
  authenticated storage provides bounded integrity evidence rather than an
  automatic confidentiality or freshness guarantee; future replication must
  operate on Tokimu semantic changes rather than Tosumu pages or raw keys.
- Disposition: Remain Incubating. Add threat-model, artifact-finalization, and
  semantic-change evidence without admitting synchronization or stronger
  security guarantees.
- Resulting ADR or documentation change: AR-0011 expanded; no ADR opened.

### Cycle 3 -- 2026-08-03

- Status entering review: Incubating.
- New evidence: Tosumu accepted `TOKIMU-001` for its provider-owned scope. The
  complete `cargo test -p tosumu-core --test provider_boundary` suite passed:
  19 tests passed and three measurement workloads remain intentionally ignored.
- Participants or reviewers: project maintainer and Codex implementation
  assistant.
- Findings: Tosumu now has sufficient public-boundary evidence to act as the
  first experimental durable provider. Backup, portable export, verification,
  large values, recovery, and structured error classification are provider-side
  evidence, not unresolved preconditions for Tokimu's first adapter.
- Disposition: Remain Incubating. Begin a Tokimu-owned adapter and corpus
  round-trip without treating the accepted provider boundary as a permanent
  `.tasset` format or mandatory engine dependency.
- Resulting ADR or documentation change: Tosumu `TOKIMU-001` marked accepted;
  this review's remaining follow-up list is now consumer-side evidence.

## References

- `docs/Conversations/Tokimu Asset (tasset).md`
- `docs/Conversations/Tosumu/DESIGN.md`
- `docs/Conversations/Tosumu/architecture.md`
- `docs/Conversations/Tosumu/concepts.md`
- `docs/Conversations/Tosumu/file-format.md`
- `docs/Conversations/Tosumu/safety-and-limits.md`
- `docs/Architectural Reviews/AR-0006-raster-image-requirement-pipeline.md`
- `docs/Architectural Reviews/AR-0009-resource-store-identity-and-kernel-boundary.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/Tokimu Software Design Document.md`
