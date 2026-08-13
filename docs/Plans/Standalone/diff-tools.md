# Diff Tools

## Status

Incubating. The source library now demonstrates bounded structured text diff,
unified-diff parsing and writing, exact and uniquely located fuzzy application,
structural reversal, JSON artifact comparison, and provider-neutral runtime
snapshot comparison in both native and browser-facing consumers. UI-specific
components and the final Tokimu ownership boundary remain intentionally
unadmitted.

## Purpose

Port the provider-neutral parts of the C# `DiffTools` project into a reusable
Rust diff library for corpus evidence, runtime observations, shell tooling, and
consumer applications.

The library should own structured comparison and patch semantics. It should not
own editor widgets, Monaco integration, WinForms presentation, source control,
or domain-specific XML/JSON meaning.

## Source Evidence

The source project includes:

- a unified-diff parser and writer;
- structured documents, files, hunks, and lines;
- exact patch application;
- fuzzy hunk matching;
- conflict detection and diagnostics;
- editor- and UI-specific integrations.

The semantic core is a strong port candidate. UI integrations are reference
consumers, not part of the base Rust library.

## Governing Boundary

```text
left input + right input
        |
        v
normalization policy
        |
        v
edit script
        |
        v
structured diff
        +-------------------+
        |                   |
        v                   v
writer / artifacts     patch application
                            |
                            v
                     result or conflict

presentation consumers
        render structured results without redefining them
```

- Diff semantics own line identity, edit operations, files, hunks, context,
  patch outcomes, and conflicts.
- Normalizers own newline, whitespace, and path comparison policy.
- Domain adapters may lower runtime observations or corpus artifacts to
  comparable forms.
- UI and website consumers own visual presentation and interaction.
- Filesystem and source-control adapters own repository mechanisms.

The first implementation should incubate in `corpus/lib/diff-tools`. A public
`tokimu-diff` companion crate is a likely destination after independent
consumers stabilize the contract.

## Goals

- Produce deterministic structured diffs.
- Parse and write a bounded, documented unified-diff subset.
- Apply patches with exact and explicitly configured fuzzy matching.
- Represent conflicts as data rather than formatted strings.
- Localize corpus divergence to the earliest comparable artifact stage.
- Support native and WASM consumers without filesystem assumptions.
- Preserve source behavioral tests while adopting idiomatic Rust APIs.
- Permit established Rust diff algorithms beneath Tokimu-owned result types.

## Non-Goals

- A Git implementation or repository model.
- Monaco, WinForms, terminal, or web presentation in the base library.
- Collaborative editing, operational transforms, or CRDTs.
- Binary delta compression in the initial plan.
- Semantic XML, JSON, image, mesh, or world-state diffing in the base contract.
- Treating fuzzy application as silent success.
- Unbounded comparison or patch application for untrusted input.

## Candidate Semantic Model

```rust
pub struct DiffDocument { /* ordered file diffs */ }
pub struct FileDiff { /* old/new identity and hunks */ }
pub struct Hunk { /* source/target ranges and lines */ }

pub enum DiffLine {
    Context(String),
    Remove(String),
    Add(String),
}

pub enum PatchOutcome {
    Applied(PatchReport),
    Conflicted(ConflictReport),
    Rejected(PatchError),
}
```

Exact names remain provisional. Structured results must preserve ordering and
provenance well enough for text, JSON artifact, terminal, and web consumers.

## Slice 1: Provenance And Behavior Inventory

### Deliverables

- [x] Record source provenance, license, revision, and test inventory.
- [x] Separate semantic diff code from UI and editor integrations.
- [x] Classify each source behavior as port, adapt, replace, defer, or reject.
- [x] Create first-party exact, malformed, fuzzy, and conflict fixtures.
- [x] Record supported unified-diff dialect assumptions.

### Acceptance Criteria

- [x] No source behavior disappears without a disposition.
- [x] UI-specific code is excluded from the base contract.
- [x] Fixtures and focused model tests cover LF, CRLF, missing final newline,
      empty files, and Unicode.
- [x] Copied fixtures and code have explicit provenance.

## Slice 2: Structured Diff Model

### Deliverables

- [x] Define files, hunks, ranges, line operations, paths, and diagnostics.
- [x] Define normalization policy separately from stored content.
- [x] Define stable ordering and equality semantics.
- [x] Define bounded limits for files, lines, bytes, hunks, and diagnostics.
- [x] Add construction and round-trip model tests.

### Acceptance Criteria

- [x] A structured diff can be consumed without parsing display text.
- [x] Original line endings and missing-final-newline state are representable.
- [x] Limits reject adversarial inputs before unbounded growth.
- [x] Model behavior has no filesystem, UI, renderer, or Git dependency.

## Slice 3: Text Diff Generation

### Deliverables

- [x] Select or implement a deterministic edit-script algorithm.
- [x] Lower edit scripts into files and hunks with configurable context.
- [x] Add explicit whitespace and newline normalization policies.
- [x] Add repeated-line and highly dissimilar input stress cases.
- [x] Benchmark small interactive and large artifact workloads separately with
      a deterministic input-size probe. Its elapsed time is diagnostic evidence,
      not a cross-machine pass/fail contract.

### Acceptance Criteria

- [x] Identical inputs produce an empty structured change.
- [x] Repeated runs produce byte-for-byte equivalent structured artifacts.
- [x] Normalization policy is recorded in output metadata.
- [x] Algorithm identity and version are observable for golden comparisons.

## Slice 4: Unified Diff Parsing And Writing

### Deliverables

- [x] Parse the admitted unified-diff subset into structured results.
- [x] Write structured results in one canonical form.
- [x] Preserve paths, ranges, context, and final-newline markers.
- [x] Emit location-aware diagnostics for malformed input.
- [x] Add parse-write-parse corpus tests.

The admitted parser preserves the standard final-newline marker as an explicit
old/new `TextFormat` fact. Canonical writing emits the marker after the final
affected source or target line, and exact in-memory application validates the
declared source final-newline state before retaining the declared target state.
Line-ending conversion remains deliberately unrepresentable in unified output.

### Acceptance Criteria

- [x] Canonical output is deterministic.
- [x] Malformed ranges, truncated hunks, and count mismatches are rejected.
- [x] Unknown extensions are preserved or rejected explicitly, never guessed.
- [x] Round trips preserve admitted semantics even when formatting normalizes.

## Slice 5: Exact Patch Application

### Deliverables

- [x] Apply hunks against exact expected source ranges and context.
- [x] Report each applied, skipped, and rejected hunk structurally.
- [x] Make multi-file application atomic by explicit policy.
- [x] Add structural reverse-application support; inverse documents reuse the exact applicator rather than introducing a second mutation path.
- [x] Add overlap, ordering, and path-collision tests.

### Acceptance Criteria

- [x] Successful exact application produces the expected target content.
- [x] Failed application does not silently produce partial content.
- [x] Hunk order and overlap rules are deterministic.
- [x] The applicator can operate entirely on in-memory strings or bytes.

## Slice 6: Fuzzy Matching And Conflicts

### Deliverables

- [x] Add bounded offset and context matching behind explicit configuration.
- [x] Score and report fuzzy decisions.
- [x] Represent ambiguous and conflicting candidates structurally.
- [x] Add resource limits for candidate search.
- [x] Add adversarial repeated-context fixtures.

### Acceptance Criteria

- [x] Fuzzy application is never reported as exact.
- [x] Equal candidates produce ambiguity rather than arbitrary selection.
- [x] Search limits terminate pathological inputs deterministically.
- [x] Single-hunk fuzzy outcomes retain the candidate locations and explicit
      no-match or ambiguity reason; multi-hunk conflict reporting remains a
      later transaction concern.

## Slice 7: Corpus Artifact Integration

### Deliverables

- [x] Compare presentation-geometry corpus JSON artifacts structurally.
- [x] Compare JSON artifact fields while ignoring only explicitly selected
      volatile paths.
- [x] Emit compact machine-readable and human-readable change summaries.
- [x] Identify the first supplied pipeline stage whose authoritative artifact diverges without inferring stage order from implementation details.
- [ ] Keep image comparison in the screenshot/image evidence layer.

### Acceptance Criteria

- [x] Ordered artifact evidence identifies the earliest supplied owning diagnostic boundary.
- [x] Volatile-field policy is explicit and artifact-specific.
- [x] A report does not claim semantic equality from text formatting alone.
- [x] Diff generation remains usable in headless CI and WASM where applicable.

## Slice 8: Runtime Observation And Shell Consumers

### Deliverables

- [x] Add an adapter for runtime observation snapshots.
- [x] Add a before/after view to the Tokimu observation shell or inspector.
- [x] Preserve typed observation meaning outside the base text diff model.
- [x] Add revision, command, and snapshot provenance to consumer output.
- [x] Exercise one native and one browser-facing consumer. The native inspector
      shows bounded snapshot status while the WASM workbench retains and
      exposes the same serialized `ObservationDiffReport` after each pair of
      observations.

### Acceptance Criteria

- [x] Diff tools do not become owners of world state or command execution.
- [x] Typed adapters can evolve independently from text algorithms.
- [x] The same structured result supports terminal and web presentation.
- [x] Missing or incompatible snapshot schemas produce diagnostics.

## Slice 9: Public Library Review

### Deliverables

- [x] Audit API consistency, naming, complexity, and panic behavior.
- [ ] Review whether the contract has enough consumers for `tokimu-diff`.
- [x] Document replacement dependencies and algorithm licenses.
- [x] Add cookbook examples for compare, parse, write, apply, and structured
      artifact comparison. Fuzzy conflict presentation remains documented by
      its typed API and will receive a dedicated example only if a consumer
      needs a display convention.
- [x] Record deferred binary and domain-semantic diff work.
- [x] Prove a second domain can consume only the public structural JSON API:
      `hello-resource-space` compares its provider-neutral
      `resource-space-provider-conformance-v1` artifact without importing
      Diff Tools parser, patch, fuzzy-match, or storage internals.

### API Review Notes

- Public generation, parse, apply, JSON comparison, and artifact comparison
  entry points return typed results or typed outcome reports. They do not
  panic in response to caller-controlled documents or artifacts.
- The only production `expect` calls are canonical unified-diff writes into
  an owned `String`; Rust's string formatting sink is infallible. They are
  deliberately local to serialization and do not cross a public boundary.
- Quadratic LCS generation is guarded by `max_edit_matrix_cells`; parser,
  fuzzy-candidate, JSON-difference, and artifact-recursion limits remain
  explicit configuration rather than implicit resource assumptions.
- Binary delta algorithms, semantic/domain-aware diffs, and automatic
  three-way merge remain deferred. The current textual and structured
  contracts report their bounded result rather than silently approximating
  those behaviors.
- A deterministic malformed-input mutation corpus now runs in ordinary
  `cargo test` against parser and exact-application boundaries. Dedicated
  coverage-guided fuzzing now has an isolated `cargo-fuzz` harness and seed
  corpus. Sustained runs and minimized crash promotion remain required
  graduation evidence.

### Acceptance Criteria

- [ ] At least three independent consumers use the structured contract.
- [x] No current consumer depends on private parser or algorithm internals.
- [x] Public parser/applicator behavior is documented and fuzzed at its
      bounded input boundaries.
- [ ] Promotion does not introduce upward dependencies into Tokimu core.

### Fuzz Harness Status

- [x] Standalone `cargo-fuzz` parser/applicator harness with valid, malformed,
      Unicode, and final-newline seed inputs.
- [x] Deterministic malformed-input mutation coverage runs in ordinary CI
      tests without requiring the fuzzing toolchain.
- [x] Verify a bounded coverage-guided Windows run with the Visual Studio ASan
      runtime available on `PATH`: 209,460 inputs in 16 seconds with no crash
      or sanitizer finding.
- [x] Add `scripts/run-diff-tools-fuzz.ps1` so Windows contributors discover
      the matching ASan runtime and run the bounded target without retaining a
      machine-specific `PATH` workaround. It bounds input size and RSS to the
      same admitted workload envelope. Generated corpus expansion remains
      ignored until a minimized regression is deliberately promoted.
- [x] Complete a one-minute Windows coverage-guided run: 677,802 executions,
      642 coverage counters, 2,191 features, and no crash or sanitizer
      finding. The generated working corpus remains local and ignored.
- [ ] Run a sustained coverage-guided fuzzing session and promote every
      minimized regression case into the ordinary fixture or test corpus.

## Emerging Third Consumer

`hello-resource-space` emits a deterministic, provider-neutral
`resource-space-provider-conformance-v1` JSON artifact and now compares that
artifact through the public `diff-tools::compare_json` contract. The comparison
is deliberately consumer-owned: Diff Tools learns neither Resource Space
storage nor mutation semantics.

This is boundary evidence, not yet independent persistent-provider graduation
evidence. A Tosumu-backed or otherwise independent provider and its
conformance result remain the material proof.

## Validation Matrix

```text
case                         generation  parse/write  exact apply  fuzzy apply
identical                    required    required     required     required
single-line edit             required    required     required     required
insert/delete at boundaries  required    required     required     required
repeated context             required    required     required     required
unicode                      required    required     required     required
mixed line endings           policy      required     required     required
missing final newline        required    required     required     required
malformed/truncated          n/a         reject       reject       reject
ambiguous fuzzy match        n/a         n/a          reject       conflict
resource limit               bounded     bounded      bounded      bounded
```

## Graduation Criteria

Diff tools may graduate from corpus incubation when:

- structured diff, parse/write, and patch contracts are stable across three
  independent consumers;
- exact, fuzzy, and conflict outcomes are unambiguous;
- parser and applicator fuzzing has no known unbounded path;
- corpus artifact and runtime observation consumers preserve domain ownership;
- UI and source-control mechanisms remain adapters;
- source behavior has a complete parity disposition.

## Open Questions

- Should inputs be UTF-8 strings only or support opaque line bytes?
- Which edit-script algorithm best balances stability and readable output?
- Should multi-file application default to atomic or per-file outcomes?
- Does a general typed-tree diff deserve a separate future review?
- Should normalization metadata travel in every `DiffDocument` or only reports?
- Is `tokimu-diff` a Tokimu workspace crate or an independently released
  companion package?
