# Tokimu Data Structures Library
## Status

Proposed. The intended destination is a broadly useful public Tokimu companion
library. Porting is staged by family so the project can pursue broad coverage
without converting a large C# library into one unreviewable Rust rewrite.

## Purpose

Port the useful data structures from the C# `DataStructures` project into a
well-tested Rust library that Tokimu users can call directly.

Tokimu itself may only need a subset. That is not the sole admission test for
this library: reusable, documented structures can still serve engine users,
tool authors, corpus consumers, and application developers. However, the
library must remain a companion library rather than quietly expanding the
trusted kernel.

## Source Evidence

The source project contains more than 65 structures and approximately 836
tests across these families:

- buffers and caches;
- graphs, disjoint sets, and heaps;
- balanced, indexed, persistent, and spatial trees;
- ropes, piece tables, gap buffers, tries, and string search;
- range-query structures;
- probabilistic structures and sketches;
- streaming statistics and process observations;
- scheduling, simulation, and combinatorial structures;
- selected concurrent structures.

This is enough evidence to justify a serious porting program. It is not enough
to declare every C# API idiomatic or every structure worth permanent support.

## Architectural Position

```text
tokimu-core and capability crates
        use only structures justified by their own semantics

tokimu-data-structures
        public companion library
        reusable algorithms and containers
        no engine ownership

applications and tools
        may use either independently
```

- The library owns data-structure behavior, invariants, complexity contracts,
  iteration, and diagnostics for invalid operations.
- It does not own world state, runtime scheduling policy, renderer resources,
  persistence, or application semantics.
- A structure does not enter `tokimu-core` merely because it exists here.
- ADR-0007 remains binding: statistics structures do not turn kernel
  diagnostics into a general profiler or aggregation service.

The likely destination is `crates/tokimu-data-structures` or a separately
publishable workspace crate. The final crate name and release policy require an
admission review after the foundation slice.

## Broad Port Policy

Broad coverage is an explicit goal, but silent cargo-cult translation is not.
Every source type must appear in a parity manifest with one status:

- `ported` - supported by the Rust library;
- `adapted` - semantics preserved through an idiomatic Rust API;
- `replaced` - an established standard-library or ecosystem type is preferred;
- `deferred` - valuable but not yet at the quality bar;
- `rejected` - unsuitable, redundant, unsafe, or outside scope, with a reason.

No object is omitted merely because the first Tokimu consumer does not need it.
No object is accepted merely because it existed in the source library.

## Proposed Module Families

```text
tokimu_data_structures
    buffers
    caches
    collections
    graphs
    heaps
    trees
    text
    persistent
    spatial
    range_queries
    strings
    probabilistic
    statistics
    simulation
    combinatorial
    concurrent
```

The module tree should communicate concepts rather than becoming a `utils`
drawer. Cargo features should only be added when dependency or build evidence
justifies them; they are not required merely because the module list is broad.

## Goals

- Preserve the useful breadth of the source library.
- Offer idiomatic Rust APIs with explicit invariants and complexity.
- Translate the source tests into durable behavioral evidence.
- Add property, differential, fuzz, and benchmark coverage where appropriate.
- Keep deterministic and randomized behavior explicit.
- Support `no_std`-compatible subsets only if consumers justify the cost.
- Make the library useful independently of Tokimu engine crates.
- Document alternatives when a standard or ecosystem implementation is better.

## Non-Goals

- A miscellaneous helper library.
- Reimplementing every Rust ecosystem crate for branding reasons.
- Promoting every structure into Tokimu's kernel or public facade.
- Hidden wall-clock, thread-pool, filesystem, renderer, or network dependencies.
- Serialization as a mandatory property of every structure.
- One release that ports all source types before any can be used.
- Preserving C# naming, inheritance, exceptions, or allocation behavior.

## Library Quality Rules

- Normal lookup misses use `Option`; invalid operations use structured errors.
- Public operations document expected time and space complexity.
- Iteration order is deterministic where the structure promises ordering.
- Randomized structures accept an explicit seed or random source.
- Time-sensitive structures accept explicit time observations rather than
  reading a hidden clock.
- Unsafe code is denied initially and requires a focused review if introduced.
- Panics are reserved for documented invariant violations, not ordinary input.
- Generic bounds remain as narrow as the implementation honestly requires.
- Benchmarks support claims; they do not become API guarantees accidentally.

## Slice 1: Provenance And Complete Parity Ledger

### Deliverables

- [ ] Record source provenance, license, revision, and test counts.
- [ ] Inventory every source type by family and public behavior.
- [ ] Create the port/adapt/replace/defer/reject parity manifest.
- [ ] Identify source types already covered by `std` or mature Rust crates.
- [ ] Select representative source fixtures with clear redistribution status.

### Acceptance Criteria

- [ ] Every source type has exactly one current disposition.
- [ ] Every rejection and replacement records a technical reason.
- [ ] The manifest can be updated without rewriting the plan.
- [ ] No source code or fixture enters Tokimu without provenance.

## Slice 2: Crate Foundation And Test Harness

### Deliverables

- [ ] Scaffold the companion crate with family-based modules.
- [ ] Add crate-level API, safety, panic, determinism, and MSRV policy.
- [ ] Add unit, property-test, fuzz-target, and benchmark conventions.
- [ ] Add a corpus example that enumerates implemented families and status.
- [ ] Add CI checks for formatting, clippy, tests, docs, and selected fuzz seeds.

### Acceptance Criteria

- [ ] The crate has no dependency on Tokimu runtime, rendering, or platform
      crates.
- [ ] An external-style corpus consumer can use the crate directly.
- [ ] Public items require documentation and examples.
- [ ] The initial crate compiles with no unsafe code.

## Slice 3: Foundation Collections

Initial candidates include `RingBuffer`, `LruCache`, `DisjointSet`, graph
primitives, and basic heaps.

### Deliverables

- [ ] Port ring-buffer behavior with explicit full/overwrite policy.
- [ ] Port LRU caching with explicit capacity and eviction observations.
- [ ] Port disjoint-set union with path compression and union policy.
- [ ] Port directed/undirected graph behavior needed by source tests.
- [ ] Port or replace foundational heap variants after ecosystem comparison.

### Acceptance Criteria

- [ ] Source behavior tests are translated or intentionally adapted.
- [ ] Property tests exercise capacity, eviction, connectivity, and ordering.
- [ ] Complexity claims are documented and benchmarked.
- [ ] Empty, duplicate, maximum-capacity, and mutation-sequence cases pass.

## Slice 4: Streaming Statistics And Diagnostics Structures

Candidates include running statistics, histograms, EWMA, moving percentiles,
TDigest, control charts, and change-point detectors.

### Deliverables

- [ ] Port numerically stable streaming mean and variance behavior.
- [ ] Port histogram and quantile-oriented structures with explicit bounds.
- [ ] Port selected trend and change detectors with explicit sample cadence.
- [ ] Add reference-vector and floating-point tolerance tests.
- [ ] Exercise the library from performance-diagnostics tooling without moving
      aggregation ownership into the kernel.

### Acceptance Criteria

- [ ] Results identify count, window, weighting, and approximation semantics.
- [ ] NaN, infinity, empty input, overflow, and reset behavior are explicit.
- [ ] Approximate structures document error expectations.
- [ ] Kernel diagnostics remain usable without this library.

## Slice 5: Spatial And Range Structures

Candidates include KD trees, quadtrees, R-trees, interval trees, segment trees,
Fenwick trees, and sparse tables.

### Deliverables

- [ ] Port one spatial family with point, bounds, and nearest/range queries.
- [ ] Port interval and prefix/range-query foundations.
- [ ] Define update versus immutable-build semantics per structure.
- [ ] Add randomized differential tests against straightforward reference
      implementations.
- [ ] Add dimensionality and coordinate-validity diagnostics.

### Acceptance Criteria

- [ ] Boundary inclusion and overlap semantics are documented.
- [ ] Degenerate, duplicate, empty, and highly unbalanced inputs pass.
- [ ] Query results are deterministic where ties exist.
- [ ] Benchmarks separate build, update, and query cost.

## Slice 6: Text And Persistent Structures

Candidates include rope, text rope, piece table, gap buffer, persistent list,
map, stack, queue, and persistent red-black tree.

### Deliverables

- [ ] Port text-editing structures around byte/character boundary decisions.
- [ ] Port selected persistent structures using Rust ownership deliberately.
- [ ] Add Unicode boundary and mutation-sequence corpus cases.
- [ ] Compare persistent cloning and allocation behavior.
- [ ] Exercise one structure from Tokimu Shell or a text-oriented consumer.

### Acceptance Criteria

- [ ] APIs state whether indices are bytes, scalar values, or grapheme units.
- [ ] Invalid boundaries do not corrupt content.
- [ ] Persistent versions remain unchanged after derived mutation.
- [ ] Undo/redo-style workloads have deterministic tests and benchmarks.

## Slice 7: Trees, Heaps, Tries, And String Search

Candidates include AVL, B/B+, splay, treap, radix, trie, Merkle, suffix array,
and Aho-Corasick structures.

### Deliverables

- [ ] Port balanced-tree families with invariant validators.
- [ ] Port trie/radix structures with explicit key encoding.
- [ ] Port selected multi-pattern and suffix search structures.
- [ ] Add invariant checks usable in tests and debug diagnostics.
- [ ] Differential-test ordered structures against `BTreeMap`/`BTreeSet` where
      semantics overlap.

### Acceptance Criteria

- [ ] Random mutation sequences preserve all structural invariants.
- [ ] Duplicate-key and replacement behavior is explicit.
- [ ] Text search reports byte/character offsets unambiguously.
- [ ] Replacement decisions identify the preferred Rust implementation.

## Slice 8: Probabilistic, Simulation, And Specialized Families

Candidates include Bloom and Cuckoo filters, Count-Min Sketch,
HyperLogLog, MinHash, Markov chains, leaky buckets, critical path, multilevel
feedback queues, Dancing Links, and selected concurrent structures.

### Deliverables

- [ ] Port probabilistic structures with seed and error parameters exposed.
- [ ] Port useful simulation/scheduling structures without assuming Tokimu
      runtime policy.
- [ ] Port combinatorial structures with bounded-input documentation.
- [ ] Evaluate concurrent structures with Rust-specific correctness tooling.
- [ ] Defer structures that cannot meet safety or quality requirements.

### Acceptance Criteria

- [ ] Randomized outputs reproduce under a fixed seed.
- [ ] Error-rate claims have statistical tests rather than exact assertions.
- [ ] Scheduling structures do not mutate Tokimu runtime policy implicitly.
- [ ] Concurrent structures pass Loom or equivalent model tests where feasible.

## Slice 9: Public Usability And Consumer Corpus

### Deliverables

- [ ] Publish a family-oriented guide and selection table.
- [ ] Add at least three independent consumer corpus examples.
- [ ] Add cookbook examples for common engine/tool workloads.
- [ ] Add migration notes from the original C# names and semantics.
- [ ] Review optional serialization and feature-boundary evidence.

### Acceptance Criteria

- [ ] A user can select a structure by workload and guarantees.
- [ ] Examples do not depend on internal test helpers.
- [ ] The crate remains useful without any other Tokimu crate.
- [ ] Documentation distinguishes exact, approximate, mutable, persistent, and
      concurrent semantics.

## Slice 10: Admission, Stabilization, And Release

### Deliverables

- [ ] Review crate naming, versioning, MSRV, and publication policy.
- [ ] Audit public API consistency across families.
- [ ] Resolve all parity-manifest entries intended for v1.
- [ ] Record deferred families and graduation requirements.
- [ ] Add changelog and compatibility policy.

### Acceptance Criteria

- [ ] V1 scope is explicit and does not imply all 65+ structures are complete.
- [ ] Every shipped structure meets the same documentation and test bar.
- [ ] Breaking changes follow semantic versioning and architectural review when
      they affect Tokimu-owned contracts.
- [ ] The library does not create upward dependencies into engine core.

## Graduation Criteria

The companion library is ready for a first stable release when:

- the parity ledger covers the entire source library;
- each included structure has invariant, edge-case, and API documentation;
- representative families have property and benchmark evidence;
- at least three real consumers use structures without private integration;
- unsafe and concurrency choices have explicit review evidence;
- the public module organization remains coherent under broad coverage.

## Open Questions

- Should the library publish independently from the Tokimu engine release?
- Which families justify optional Cargo features?
- Is a `no_std + alloc` subset valuable enough to maintain?
- Which source structures should be replaced by ecosystem implementations?
- How should common allocator and hashing customization be exposed without
  infecting every API?
- Which statistics structures belong here versus a future observation library?
