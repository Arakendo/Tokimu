# AR-0014: Native Terminal Text Surface And Ratatui Dependency Boundary

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-05 |
| Last reviewed | 2026-08-06 |
| Scope | Foundational presentation capability / provider boundary |
| Trigger | Ratatui now has native, WASM, and embedded Tokimu consumers, while its dependency and ownership cost remains unmeasured |
| Related ADRs | ADR-0003, ADR-0004, ADR-0005 |
| Related evidence | AR-0013, Tokimu Ratatui website lab, console command window corpus, observation shell corpus |
| Related plan | `docs/Plans/Standalone/tokimu-tui-tools-corpus-study.md` |
| Admission exception | None |

## Architectural Question

Which terminal-shaped text-surface semantics, if any, should Tokimu own so that
native shells and embedded terminal views do not require the full Ratatui
implementation, while Ratatui remains usable as a replaceable presentation
provider?

## Context

AR-0013 separates provider-neutral observation-shell meaning from Ratatui's
presentation mechanics. It asks whether Tokimu should define a bounded shell
session and whether Ratatui is a useful provider for that session.

This review asks a narrower question below that shell boundary. Several corpus
consumers now need a bounded terminal-shaped surface with styled cells, cursor
state, text width behavior, dirty-cell updates, and deterministic diagnostics.
Ratatui supplies those mechanics and considerably more. Tokimu also already
owns provider-neutral text semantics under ADR-0004 and can rasterize a
provider-resolved font such as Departure Mono.

The resulting pressure is not sufficient to conclude that Tokimu should fork,
copy, or partially extract Ratatui. It is sufficient to examine whether a
smaller native surface contract exists between shell meaning and text
rendering, and whether carrying Ratatui is materially expensive for consumers
that only need that contract.

```text
Observation shell or application semantics
                |
                v
Terminal-shaped presentation provider
                |
                v
Bounded styled-cell surface
                |
                v
Tokimu text provider and renderer
```

This review must not make "terminal" synonymous with a process shell, PTY,
ANSI parser, command language, or host console. Those are separate semantics
and mechanisms.

## Trigger And Evidence

- Corpus examples:
  - `tokimu-website-ratatui-lab` uses Ratatui 0.29 with default features
    disabled, receives changed cells through a corpus-local `TokimuBackend`,
    and rasterizes the retained surface with Tokimu's font path.
  - `tokimu-console-command-window` compares Ratatui cell evidence with a
    Tokimu-owned cell projection and exercises cursor, wrapping, scrolling,
    command history, and Departure Mono rendering.
  - `hello-observation-shell-ratatui` supplies a native Crossterm-hosted
    Ratatui adapter behind an optional corpus feature.
- Automated tests:
  - The website lab rejects undersized grids and tests complete bounded output.
  - Backend tests exercise retained-cell updates through Ratatui's public
    `Backend::draw` seam.
- Audits or diagnostics:
  - The checked-out Ratatui 0.29 source contains approximately 79 Rust files,
    43,340 source lines, and 1.61 MB of Rust source. This describes maintenance
    surface, not linked runtime weight.
  - The feature-minimal dependency graph still includes layout, compact string,
    Unicode width and segmentation, caching, iterator, enum, and proc-macro
    dependencies. It does not include Crossterm in the browser build because
    default features are disabled.
  - The complete release WASM artifact for `tokimu-website-ratatui-lab` is
    607,001 bytes before attributing any portion to Ratatui. This is an
    application measurement, not a Ratatui-only measurement.
  - Ratatui's default feature set includes Crossterm and underline-color
    support; current browser consumers deliberately disable defaults.
- Independent consumers:
  - Native command-window presentation, native observation-shell presentation,
    and the browser Ratatui lab exercise related mechanics through different
    hosts.
- Repeated implementation friction:
  - Corpus code currently owns retained cells, styled-cell normalization,
    cursor evidence, text rasterization, and browser/native input adaptation in
    several nearby forms.
  - Using `TestBackend` as a production bridge was rejected; the lowest stable
    source-backed seam found so far is Ratatui's public `Backend::draw` method.
- Missing evidence:
  - No differential native or WASM build compares the same scene with and
    without Ratatui.
  - No measured compile-time, cold-start, allocation, or warm-frame cost is
    attributable solely to Ratatui.
  - No non-shell consumer has proven that a provider-neutral terminal surface
    should become a first-party capability.
  - Unicode, grapheme, wide-cell, combining-mark, cursor, resize, clipping, and
    style conformance are not yet complete across native and WASM hosts.
  - No maintenance and license review supports copying selected Ratatui
    internals into Tokimu.

## Ownership Analysis

The meaning under review is a bounded terminal-shaped presentation surface:

- dimensions expressed as rows and columns;
- styled cell content and continuation-cell behavior;
- cursor position, visibility, and shape where supported;
- deterministic full-surface and changed-cell observations;
- clipping, resize, width, and invalidation diagnostics.

AR-0013 continues to own the separate question of observation sessions,
command catalogs, requests, history, and command outcomes. Applications and
shell capabilities own those semantics.

ADR-0004 continues to own provider-neutral text measurement, layout, fallback,
font handles, and diagnostics. Font parsing, glyph rasterization, and bundled
font files remain provider concerns. This review does not admit Ratatui's text
model into the foundational text contract.

Ratatui owns its widget model, layout behavior, buffer diffing, style types,
and backend protocol. Those foreign types must not cross a future Tokimu public
contract merely because the current corpus adapter consumes them.

Renderers own pixels, GPU resources, batching, and presentation execution. A
terminal surface must not own simulation truth, command authority, host
processes, PTYs, filesystem access, or renderer resources.

If accepted after further evidence, a minimal terminal surface would be a
presentation capability or provider-facing contract, not kernel truth and not
an engine-core dependency on Ratatui.

## Dependency Direction

```text
Current:

corpus consumer
    -> Ratatui widgets and Buffer
    -> corpus-local TokimuBackend / projection adapter
    -> ui-tools font provider and rasterization
    -> browser canvas or native renderer

Proposed study boundary:

application or observation shell semantics
    -> provider-neutral bounded terminal surface
        <- Ratatui provider
        <- possible minimal Tokimu-native provider
    -> ADR-0004 text contracts and replaceable font provider
    -> renderer or browser presentation adapter
```

`tokimu-core` and `tokimu-runtime` must not depend on Ratatui, Crossterm,
terminal host APIs, font parsers, or renderer-native resources. A Ratatui
provider may depend downward on a future terminal-surface contract; the
contract must not depend upward on Ratatui.

## Deployment Composition Hypothesis

The corpus now distinguishes a provider contract from a provider payload.
Ratatui remains an optional presentation provider; a consumer that needs only
an already-resolved or independently produced cell surface must not inherit it
transitively. A consumer that intentionally uses Ratatui layout may opt into a
separate build feature and must record its own browser payload and native
execution observations.

This is not a decision to extract Ratatui code. If the evidence eventually
supports a Tokimu-owned layer, it should be an authored, provider-neutral
surface handoff such as bounded styled cells, cursor state, full frames, and
validated deltas. Ratatui widgets, layout, `Buffer`, backend traits, terminal
hosting, ANSI/PTY handling, and Unicode-layout policy remain upstream/provider
concerns unless independent corpus pressure proves otherwise.

Native delivery may tolerate a richer optional provider composition than a web
island, but it does not waive measurement. Deployment policy belongs to the
consumer: each consumer selects its optional providers and records an explicit
payload and runtime budget instead of making one global claim that native is
free or that web must carry every provider.

## Alternatives Considered

### Alternative A: Keep Ratatui As The Only Terminal Presentation Provider

- Benefits: Uses a mature widget, layout, style, buffer, and backend ecosystem;
  avoids rebuilding terminal UI mechanics; preserves the working corpus path.
- Costs: Carries a nontrivial dependency and compile surface even with default
  features disabled; small consumers may need only a fraction of its behavior.
- Failure mode: Ratatui types become the accidental Tokimu contract, or all
  terminal-shaped consumers pay for mechanics they do not use.

### Alternative B: Define A Minimal Provider-Neutral Terminal Surface

- Benefits: Gives Ratatui and smaller providers one bounded output contract;
  reuses Tokimu text and renderer ownership; supports deterministic diagnostics
  without adopting shell semantics.
- Costs: Risks duplicating Ratatui concepts under different names; requires
  Unicode, width, style, cursor, resize, and invalidation contracts to be stated
  precisely.
- Failure mode: A prematurely stabilized cell API cannot preserve real Ratatui
  behavior or grows into a second terminal framework.

### Alternative C: Implement A Small Tokimu-Native Provider Alongside Ratatui

- Benefits: Supplies differential size and behavior evidence; may suit simple
  command windows and diagnostics while Ratatui remains available for richer
  scenes.
- Costs: Creates implementation and conformance work; may repeat mature layout,
  wrapping, and Unicode behavior.
- Failure mode: The small provider quietly accumulates widgets and becomes an
  inferior Ratatui fork.

### Alternative D: Seek A Smaller Upstream Ratatui Feature Or Crate Boundary

- Benefits: Retains upstream maintenance and ecosystem compatibility while
  reducing linked and compile-time surface if Ratatui exposes a stable smaller
  layer.
- Costs: Depends on upstream architecture and version stability; the needed
  boundary may not exist in the pinned version.
- Failure mode: Tokimu designs around an unstable upstream internal boundary or
  delays its own semantic decision waiting for upstream changes.

### Alternative E: Maintain A Tokimu-Specific Ratatui Fork Or Vendor Experiment

- Benefits: Could create a first-class seam for bounded surface capture,
  changed-cell delivery, direct Tokimu rendering, and target-specific feature
  composition without waiting for an upstream crate boundary. It may also make
  browser and native delivery costs independently measurable.
- Costs: Takes ownership of the forked scope, compatibility, licensing notices,
  security fixes, Unicode behavior, upstream rebases, and any divergence from
  Ratatui's public behavior.
- Failure mode: Tokimu acquires a hidden terminal framework, or a short-lived
  local patch becomes a permanent fork without a defined API, maintainer, or
  exit path.

This alternative is eligible only as a bounded, optional provider experiment;
it is not an admission of Ratatui-derived mechanics into `tokimu-core`,
`tokimu-runtime`, or a public Tokimu contract. Before implementation, the
experiment must record all of the following:

- the pinned upstream revision, license and attribution obligations, and a
  maintained divergence ledger;
- the exact first-party seam being tested, initially limited to direct bounded
  surface capture or changed-cell delivery rather than widgets, layout DSLs,
  terminal hosting, PTYs, or command semantics;
- matched upstream-versus-forked native and browser builds with linked payload,
  startup, warm-frame, allocation, and dependency-graph observations;
- a compatibility fixture suite that compares the chosen upstream and forked
  scenes through the same provider-neutral corpus-local surface handoff; and
- an explicit exit: retire the experiment if upstream provides the needed seam,
  preserve it as an optional provider if the fork remains the right mechanism,
  or propose a Tokimu-owned contract only after independent non-Ratatui
  consumers prove the boundary.

Forking is therefore a way to evaluate provider integration and deployment
composition, not a shortcut around the evidence needed for capability
admission. No Ratatui or fork-specific type may cross the public Tokimu
boundary during this experiment.

### Alternative F: Continue Corpus-Side Incubation

- Benefits: Preserves the working Ratatui adapters while gathering honest
  measurements and conformance evidence; introduces no stable public contract.
- Costs: Some adapter duplication remains and consumers cannot yet rely on a
  first-party terminal surface.
- Failure mode: Incubation becomes indefinite and corpus-local shapes diverge
  without comparison.

## Findings

The current evidence supports these provisional findings:

1. Ratatui is not a tiny dependency in source or compile surface, even when
   terminal-host defaults are disabled.
2. The current 607,001-byte website-lab WASM artifact does not establish
   Ratatui's incremental binary cost. A matched baseline is required.
3. Ratatui's public backend seam can feed a Tokimu-retained bounded surface
   without using `TestBackend` as a production bridge.
4. Multiple consumers need related cell, cursor, width, resize, and diagnostic
   behavior, but they do not yet prove a permanent first-party capability.
5. Shell meaning, terminal-surface mechanics, text semantics, font providers,
   and rendering execution are separate ownership boundaries.
6. A Tokimu-specific Ratatui fork may be worth evaluating when the public
   adapter fights Ratatui ownership, a smaller upstream boundary is unavailable,
   or direct renderer capture and target-specific composition require a stable
   local seam. That remains a provider experiment, not capability admission.

The evidence does not yet support admitting a new crate, replacing Ratatui,
forking Ratatui, or making a native terminal surface kernel-native.

## Disposition

**Incubating.** Continue using Ratatui as a replaceable corpus-side provider and
measure it against a behaviorally matched minimal path. A scoped fork or vendor
experiment may proceed only under Alternative E's source, maintenance,
compatibility, deployment-cost, and exit criteria. Do not stabilize
Ratatui-derived terminal semantics yet. Revisit capability admission only after
differential cost and cross-provider conformance evidence identify a smaller
durable ownership boundary.

## Consequences

- AR-0013 remains authoritative for observation-shell and command-session
  semantics.
- Ratatui remains outside `tokimu-core` and `tokimu-runtime`.
- Existing corpus-local `TokimuBackend` adapters may continue to gather
  evidence but are not a stable public API.
- New terminal-shaped consumers should record which behavior comes from
  Ratatui, Tokimu text, a font provider, host input, and the renderer.
- Size discussions must distinguish source size, dependency count, compile
  cost, unoptimized artifacts, optimized artifacts, and incremental linked
  cost.
- A future minimal provider must be tested for semantic parity rather than
  judged only by screenshots or binary size.

## Required Follow-Up

- [x] Build matched native and WASM scenes with Ratatui enabled and disabled,
      then record incremental release size after the same optimization steps.
      On the local development machine, the native linked payload changed from
      `6,191,104` to `6,298,624` bytes (`+107,520`), and the WASM artifact
      changed from `319,158` to `454,152` bytes (`+134,994`). These are local
      observations, not cross-target budgets.
- [x] Record startup, allocation, and warm-frame observations for the matched
      scenes. The retained CPU frame cache loads and rasterizes once, then
      recorded `255` cache hits in each 256-iteration local composition run;
      independent and Ratatui composition averaged `505 us` and `909 us`.
      Native rendering also records startup and periodic CPU-side warm-frame
      observations.
- [ ] Record comparable cold and incremental build observations for the matched
      scenes. This remains open because local build-cache state is not a stable
      cross-host build-cost measure.
- [x] Inventory the exact Ratatui APIs used by each Tokimu corpus consumer and
      classify them as shell meaning, surface mechanics, text behavior, host
      input, or diagnostics.
- [x] Add conformance cases for ASCII, clipping, resize, cursor state, style
      reset, and changed-cell invalidation.
- [ ] Add paired-provider conformance cases for Unicode width, grapheme
      clusters, combining marks, and wide cells. Explicit continuation is
      covered where an independent producer supplies it, but Ratatui's public
      buffer API does not expose equivalent continuation metadata.
- [x] Open `corpus/focused/observation/hello-terminal-surface` as a headless local baseline for
      full-frame, delta, epoch, extent, cursor, and continuation-cell
      lifecycle evidence. Its types remain explicitly corpus-local.
- [x] Compare the Ratatui provider with one deliberately minimal independent
      terminal-surface implementation without stabilizing either as public API.
      The comparison is limited to bounded terminal and dashboard fixtures;
      it is not universal layout or visual-equivalence evidence.
- [x] Confirm that normal dependency trees for `tokimu-core` and
      `tokimu-runtime` contain neither Ratatui nor corpus crates. Ratatui stays
      optional behind `tui-tools` feature gates.
- [ ] Inspect newer upstream Ratatui crate and feature boundaries before
      considering any local extraction.
- [ ] If the public adapter continues to impose avoidable copying or target
      cost, open a scoped upstream-versus-fork feasibility experiment under
      Alternative E. Record the pinned source, license obligations, divergence
      ledger, matched fixtures, cost comparison, and exit decision.
- [ ] Reopen AR-0013 if this study changes the observation-shell provider
      boundary rather than only the terminal surface below it.
- [ ] Open an ADR only if the evidence admits a stable Tokimu-owned terminal
      surface or changes an accepted presentation boundary.

## Reopening Triggers

- Differential builds show that Ratatui imposes material size, compile, startup,
  or frame cost on a required Tokimu target.
- A second provider preserves the same bounded terminal-surface behavior without
  Ratatui types.
- Three independent non-corpus consumers require the same terminal-surface
  contract.
- Ratatui exposes a stable smaller upstream crate or feature boundary that
  materially changes the dependency analysis.
- Unicode, cursor, resize, or style evidence proves that a proposed minimal
  contract cannot preserve provider behavior.
- Ratatui details leak into ADR-0004 text contracts, shell semantics, or public
  Tokimu APIs.

## Review History

### Cycle 1 -- 2026-08-05

- Status entering review: Proposed
- New evidence: direct `Backend::draw` consumers now exist in native and WASM
  corpus paths; a feature-minimal dependency inventory and complete website-lab
  WASM artifact measurement were recorded.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: Ratatui is substantial enough to measure, but current artifacts do
  not isolate its incremental cost. A terminal surface may exist below shell
  semantics, but extraction or capability admission is premature.
- Disposition: Incubating.
- Resulting ADR or documentation change: no ADR; this review records the
  measurement and ownership study separately from AR-0013.

### Cycle 2 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: `hello-terminal-surface` now isolates the proposed lower
  lifecycle without Ratatui, a window, font provider, or renderer. It proves
  that a complete frame establishes the baseline; only matching epoch and
  extent deltas may modify it; and continuation cells remain layout metadata.
- Findings: snapshot/delta lifecycle is sufficiently concrete to compare
  providers, but it is not yet a public Tokimu contract. Ratatui remains one
  future producer rather than the vocabulary's owner.
- Disposition: Continue incubation through Ratatui and independent-producer
  adapters.
- Resulting ADR or documentation change: no ADR; added
  `docs/Plans/Standalone/terminal-surface-provider-study.md` and the corpus baseline.

### Cycle 3 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the optional `ratatui-producer` feature of
  `hello-terminal-surface` renders a bounded fixture through Ratatui's
  `TestBackend`, lowers all resolved cells into the corpus-local frame, and
  derives a changed-cell delta from a second Ratatui render. The same local
  replica accepts the full frame and delta without receiving a Ratatui type.
- Findings: Ratatui can act as a producer of the candidate full-frame/delta
  lifecycle. The adapter also proved that explicit empty cells are necessary
  for a producer to clear prior content. Cursor state is fixture-owned while
  using `TestBackend`; style and clipping evidence remain open rather than
  being erased from the future contract.
- Disposition: Continue incubation with an independent producer and a richer
  style/clipping candidate before considering extraction.
- Resulting ADR or documentation change: no ADR; Slice 3 of
  `terminal-surface-provider-study.md` is partially completed.

### Cycle 4 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: a dependency-free fixture producer now emits the same local
  full-frame and changed-cell lifecycle as the optional Ratatui adapter.
  Both reconstruct through `TerminalSurfaceReplica`; rejected stale updates,
  invalid bounds, duplicate damage, and orphan continuation cells leave the
  retained surface unchanged. Clearing a wide-cell lead now requires clearing
  its continuation in the completed transactional update.
- Findings: the candidate lifecycle is no longer justified solely by Ratatui.
  The fixture deliberately lacks Ratatui styling, wrapping, width behavior,
  and widget composition; these remain provider behavior and future pressure
  on the candidate contract. No Ratatui type crosses either producer boundary.
- Disposition: Continue incubation with styled-cell, clipping, cursor-host,
  and presentation evidence before considering extraction.
- Resulting ADR or documentation change: no ADR; Slice 4 of
  `terminal-surface-provider-study.md` is partially complete.

### Cycle 5 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the Ratatui adapter now preserves provider-resolved foreground,
  background, and supported emphasis modifiers in corpus-local `ResolvedCell`
  records. The retained-surface replica accepts a style-only changed-cell delta,
  proving that visual provider state does not need to be reconstructed by the
  consumer from text content.
- Findings: style is a meaningful part of the resolved terminal-surface handoff,
  but a minimal provider need not offer Ratatui-equivalent styling to participate
  in lifecycle conformance. Cursor remains fixture-owned under `TestBackend`,
  and clipping remains unproven until Tokimu rasterizes a bounded cell surface.
- Disposition: Continue incubation through presentation and clipping evidence;
  do not promote the corpus vocabulary or extract Ratatui concepts.
- Resulting ADR or documentation change: no ADR; Slice 3 style evidence is
  complete in `terminal-surface-provider-study.md`.

### Cycle 6 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: `hello-terminal-surface` now lowers both its independent
  fixture and its Ratatui `TestBackend` fixture into deterministic bounded
  `288x120` CPU RGBA artifacts through `ui-tools::UiFontRasterizer` and
  Departure Mono. The corpus records distinct fingerprints for each producer,
  and fourteen tests cover retained style state, style-only deltas, bounded edge
  clipping, and the Ratatui-produced path.
- Findings: a resolved cell extent plus cells is sufficient for a consumer to
  execute a bounded presentation surface without Ratatui types crossing the
  raster boundary. The raster adapter does not decide terminal width, wrapping,
  or continuation placement. The minimal fixture's default styling remains
  provider-local behavior rather than a reason to weaken the candidate model.
- Limits: this is a corpus-local CPU reference artifact, not native, GPU, or
  WASM-equivalence evidence. Cursor position and italic emphasis are retained
  by the candidate model but are not yet painted by this adapter. No standalone
  terminal host or public terminal-surface API is justified by this result.
- Disposition: Continue incubation. Gather matched native/WASM artifacts and
  further host/cursor evidence before considering capability admission or a
  dependency-boundary change.
- Resulting ADR or documentation change: no ADR; Slice 5 CPU presentation
  evidence is recorded in `terminal-surface-provider-study.md` and
  `corpus/focused/observation/hello-terminal-surface/RESEARCH.md`.

### Cycle 7 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: `hello-terminal-surface` compiles as a
  `wasm32-unknown-unknown` `cdylib`. Its only WASM export returns an
  independent-fixture summary, so Ratatui, normalized cells, and font-provider
  implementation types remain corpus-internal.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: the candidate surface model and Tokimu CPU text-raster path are
  buildable at the WASM boundary without admitting a terminal surface API or
  leaking the optional Ratatui producer into browser-facing types.
- Limits: compilation does not prove a browser execution path, native/WASM
  render equivalence, or Ratatui's incremental size and runtime cost.
- Disposition: Continue incubation. A browser-hosted artifact and matched
  native/WASM evidence remain required before any admission decision.
- Resulting ADR or documentation change: no ADR; Slice 5 now records explicit
  WASM compile evidence while retaining its uncompleted cross-host criterion.

### Cycle 8 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: `hello-terminal-surface` now builds a local browser presenter
  from its WASM `cdylib`. Rust/WASM exports only the independent fixture's
  dimensions, RGBA bytes, and diagnostic summary. The browser uses `ImageData`
  to display that opaque buffer and reported the same `288x120` raster and
  `851900b19a379bcd` CPU fingerprint as the native reference fixture.
- Findings: a browser can host the corpus-local terminal presentation artifact
  without receiving resolved cells, Ratatui state, or font-provider objects.
  Rust/WASM owns the raster; the browser owns display only. This strengthens the
  lower presentation handoff but does not establish native/GPU image parity, a
  terminal host, or a public terminal-surface contract.
- Disposition: Continue incubation. Record matched renderer and cost evidence
  before reconsidering a capability or dependency-boundary decision.
- Resulting ADR or documentation change: no ADR; Slice 5 browser-display
  evidence is recorded in `terminal-surface-provider-study.md` and
  `corpus/focused/observation/hello-terminal-surface/RESEARCH.md`.

### Cycle 9 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the native `hello-terminal-surface` binary now takes the same
  deterministic CPU RGBA artifact used by the browser proof and submits it as
  one sRGB texture on one Tokimu `Texture2d` quad. Startup logs retain the CPU
  fingerprint, raster dimensions, and texture-upload count while explicitly
  reporting `framebuffer_readback=false`.
- Findings: native renderer execution can remain below the terminal and
  font-raster boundaries. It consumes only opaque pixels; it neither receives
  Ratatui types nor reimplements terminal layout, cell resolution, or text
  rasterization. This is useful independent execution evidence without making
  the renderer a terminal-surface owner.
- Limits: no GPU framebuffer readback, CPU/GPU pixel comparison, native/WASM
  parity claim, standalone terminal host, or public terminal-surface API is
  established. Ratatui remains an optional corpus producer dependency.
- Disposition: Continue incubation. The evidence strengthens the layered
  presentation handoff but is insufficient for dependency admission or a
  native terminal capability decision.
- Resulting ADR or documentation change: no ADR; the CPU-to-native-renderer
  handoff is recorded in `terminal-surface-provider-study.md` and
  `corpus/focused/observation/hello-terminal-surface/RESEARCH.md`.

### Cycle 10 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: release `wasm32-unknown-unknown` builds of
  `hello-terminal-surface` measured `336349` bytes for the independent
  configuration and `336346` bytes with `ratatui-producer` enabled. The
  browser-facing `cdylib` deliberately exports only the independent fixture
  raster; the optional Ratatui producer is not linked into that artifact.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: the public browser boundary remains independent of Ratatui types
  and dependency code. The nearly identical artifacts are expected confirmation
  of that export boundary, not evidence that Ratatui itself has no cost.
- Limits: no executable Ratatui producer dependency-size, startup, or frame
  measurement was made. This result does not establish provider parity, a
  standalone terminal host, or a native terminal capability.
- Disposition: Continue incubation. Retain Ratatui as an optional provider and
  collect matched executable-cost and real-consumer evidence before considering
  an ADR or capability admission.
- Resulting ADR or documentation change: no ADR; the boundary measurement is
  recorded in `terminal-surface-provider-study.md` and
  `corpus/focused/observation/hello-terminal-surface/RESEARCH.md`.

### Cycle 11 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: release native corpus executables measured `6166016` bytes
  without the optional producer and `6278144` bytes with
  `ratatui-producer` enabled, an increase of `112128` bytes. Unlike the
  browser-export comparison, this executable links the optional producer.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: the optional Ratatui producer has a measurable, bounded size cost
  in this runnable corpus binary while remaining outside Tokimu public APIs.
  The browser and native measurements together distinguish export hygiene from
  optional-provider linkage.
- Limits: this is corpus-local size evidence only. It does not measure startup
  cost, frame cost, a standalone terminal host, or whether a real independent
  consumer needs a stable terminal-surface capability.
- Disposition: Continue incubation. Preserve Ratatui as an optional provider;
  collect startup, frame, conformance, and independent-consumer evidence before
  considering a binding ADR or capability admission.
- Resulting ADR or documentation change: no ADR; the executable comparison is
  recorded in `terminal-surface-provider-study.md` and
  `corpus/focused/observation/hello-terminal-surface/RESEARCH.md`.

### Cycle 12 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: `hello-terminal-surface --features ratatui-producer --
  --measure` repeated the same 24x6 `READY` to `DONE`
  full-frame/delta/replica/CPU-raster lifecycle 256 times for both the
  independent fixture and optional Ratatui producer. Each run rejects differing
  dimensions or raster fingerprints.
- Local observation: the independent producer took `244841 us` total (`956 us`
  average); the Ratatui producer took `457465 us` total (`1786 us` average).
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: both producers remain deterministic through the same resolved
  surface and CPU-raster boundary. The comparison makes optional producer work
  visible without opening a native window or treating host-specific timings as
  a stable performance contract.
- Limits: these are local CPU observations, not budgets. Native startup,
  sustained renderer-frame cost, GPU capture/readback, terminal-host behavior,
  and independent-consumer pressure remain unmeasured.
- Disposition: Continue incubation. Ratatui remains an optional provider while
  the study collects the remaining execution and consumer evidence.
- Resulting ADR or documentation change: no ADR; the measurement method and
  local observation are recorded in `terminal-surface-provider-study.md` and
  `corpus/focused/observation/hello-terminal-surface/RESEARCH.md`.

### Cycle 13 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the native `hello-terminal-surface` viewer now consumes the
  resolved RGBA8 raster through Tokimu's ordinary textured-quad execution
  path. It emits a startup-ready observation and periodic warm-frame renderer
  observations with CPU-side acquisition, preparation, encoding, submit, and
  presentation call durations plus resource-churn counters.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: native execution can be observed at the renderer boundary without
  admitting terminal composition, Ratatui types, font-provider internals, or
  terminal-host state into the renderer. The renderer receives only a resolved
  raster input.
- Limits: instrumentation is implemented, but no bounded native-machine run
  has yet been retained as evidence. The fields are CPU call observations, not
  GPU-completion, display-latency, framebuffer-readback, or cross-platform
  performance guarantees.
- Disposition: Continue incubation. Record one bounded native observation,
  then reassess it together with independent-consumer pressure rather than
  treating instrumentation alone as graduation evidence.
- Resulting ADR or documentation change: no ADR; the plan and corpus research
  record the execution boundary and its measurement limits.

### Cycle 14 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the terminal-surface study now has a direct, bounded Ratatui
  producer path, matched deterministic surface/raster fixtures, and separate
  browser-export and native executable cost observations. The current public
  adapter remains corpus-local and preserves a provider-neutral resolved
  surface boundary, but the study makes the cost and control of the provider
  integration itself an observable concern.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: a Tokimu-specific Ratatui fork or carefully vendored experiment is
  not ruled out when the public adapter cannot provide a stable direct capture
  seam, changed-cell delivery, or target-specific feature composition without
  avoidable copying or deployment cost. Such a fork would be an optional
  presentation-provider experiment, not evidence for admitting Ratatui,
  terminal widgets, or terminal-host semantics into Tokimu core or runtime.
- Required safeguards: any experiment records its pinned upstream revision,
  license and attribution obligations, divergence ledger, narrow initial seam,
  matched upstream-versus-fork fixtures, native and browser cost comparison,
  compatibility result, maintainer responsibility, and an explicit exit
  decision. Its initial scope may cover only bounded surface capture and
  changed-cell delivery; it must exclude widget APIs, layout DSLs, PTY/ANSI
  hosting, and command semantics.
- Limits: no fork has been created or selected. No independent non-Ratatui
  consumer yet proves that a first-party terminal-surface contract should
  exist, and no provider-specific type may cross the public Tokimu boundary.
- Disposition: Continue incubation. A scoped upstream-versus-fork feasibility
  experiment is permitted only if the current adapter demonstrably blocks the
  required seam or delivery target. Retire it if upstream satisfies the need;
  otherwise preserve it as an optional provider until independent consumer
  evidence justifies any broader capability decision.
- Resulting ADR or documentation change: no ADR; Alternative E and the
  required follow-up now record the fork/vendor experiment criteria.

### Cycle 15 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the generic `tui-tools` raster seam (`rasterize_cells` plus
  `rasterize_surface`) now turns normalized terminal cells into deterministic
  CPU RGBA. Native `hello-tui-tools` supplies a `Surface`, while the optional
  `tui-tools` Ratatui bridge maps a Ratatui `Buffer` into `TuiRasterCell`; both
  consume the same Tokimu raster seam.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings:
  - Ratatui `TestBackend` composes terminal cells only. It is a feature-gated
    comparison oracle and does not render presentation pixels.
  - Ratatui-specific color and modifier translation remains in the optional
    bridge rather than the base contract. The website selects a font and blits
    the resolved RGBA frame; it does not interpret Ratatui styles.
  - Generic `tui-tools` has no default Ratatui dependency and selects no font.
    Caller-facing presentation chooses Departure Mono through
    `UiFontRasterizer`.
  - Consumer tests assert output dimensions, RGBA length, and a repeatable CPU
    fingerprint for native-surface and website-Ratatui-buffer inputs.
- Limits: this establishes deterministic CPU evidence only; it does not claim
  browser-canvas or GPU-framebuffer equivalence, a provider-neutral terminal
  host, or kernel admission.
- Disposition: Continue incubation.
- Resulting ADR or documentation change: no ADR; shared CPU renderer-seam
  evidence is retained by the TUI corpus and website adapter tests.

### Cycle 16 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the optional `tui-tools` Ratatui oracle now renders the same
  caller-owned status dashboard through the corpus-local projection and a
  Ratatui `TestBackend` at `48 x 14` and `64 x 18`. The paired report requires
  the title, subtitle, section headings, field labels and values, and footer
  to remain observable through both composition paths.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: caller-owned dashboard facts can survive supported layout and
  resize pressure without promoting Ratatui border glyphs, padding, styles, or
  sizing rules into a Tokimu contract. `TestBackend` remains an internal
  composition oracle; the generic CPU raster seam remains the concrete
  renderer-facing evidence boundary.
- Limits: no input, focus, view-state, undersized-layout expectation matrix,
  Unicode/wide-grapheme, terminal-host, or visual-equality parity has been
  established.
- Disposition: Continue incubation. The result supports a small corpus-local
  composition path alongside Ratatui as an optional provider, but does not
  justify capability admission or a provider decision.
- Resulting ADR or documentation change: no ADR; the TUI corpus plan records
  the supported-extent dashboard comparison and remaining evidence gaps.

### Cycle 17 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the paired status-dashboard oracle now routes both the
  corpus-local surface and the Ratatui `TestBackend` buffer through the same
  CPU cell-to-RGBA raster seam. At `48 x 14` and `64 x 18`, each route produces
  the expected RGBA dimensions, non-empty pixels, and a repeatable
  provider-local fingerprint.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: `TuiRasterCell` is a sufficient renderer-facing handoff for the
  current bounded dashboard evidence. Ratatui remains responsible for its
  buffer composition; the Tokimu raster path remains responsible for
  rasterization. Cross-provider pixel equality is intentionally not a contract
  because borders and styles remain provider-local.
- Limits: no native terminal-host rendering, input/focus/view-state comparison,
  undersized-layout expectation matrix, Unicode/wide-grapheme corpus,
  texture/GPU upload, or visual parity has been established.
- Disposition: Continue incubation. The renderer seam is now exercised by
  independent composition paths without admitting a shared terminal capability
  or Ratatui dependency.
- Resulting ADR or documentation change: no ADR; the TUI corpus plan records
  the renderer-seam evidence and remaining gaps.

### Cycle 18 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: paired dashboard reports now retain typed expected-divergence
  records for provider-local border composition and style composition. Each
  record requires a non-empty reason; no generic ignore-differences path is
  available.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: expected divergence is reviewable evidence, not failure
  suppression. Missing caller-owned facts, invalid output dimensions, empty
  raster frames, and nondeterministic output remain failures even when
  provider-local visual choices differ.
- Limits: the records do not establish cross-provider visual parity, terminal
  hosting, input/focus/view-state parity, undersized-layout behavior, or a
  shared terminal capability.
- Disposition: Continue incubation. The corpus can now distinguish explicit
  provider-local variation from a semantic regression without admitting
  Ratatui types or visual styling into a shared contract.
- Resulting ADR or documentation change: no ADR; the TUI corpus plan records
  the narrow expected-divergence policy.

### Cycle 19 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the paired status-dashboard semantic and CPU-raster oracle
  paths now share one explicit `8 x 6` minimum extent. The boundary matrix
  accepts that minimum and rejects `7 x 6`, `8 x 5`, and `1 x 1` with the same
  deterministic diagnostic, including the received dimensions.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: a paired provider comparison needs an admitted complete-fixture
  extent. That admission rule is not a universal terminal-layout rule and does
  not replace the Tokimu-authored dashboard's separate diagnostic degraded
  layout for smaller surfaces.
- Limits: the matrix does not establish input/focus/view-state parity,
  Unicode/wide-grapheme behavior, native terminal hosting, cross-provider
  visual equality, or a shared terminal capability.
- Disposition: Continue incubation. The corpus now distinguishes an explicit
  oracle precondition from local degraded-layout behavior without treating
  either as a provider failure.
- Resulting ADR or documentation change: no ADR; the TUI corpus plan records
  the paired minimum-extent boundary and its remaining evidence gaps.

### Cycle 20 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: three bounded consumers now meet at the same lower
  normalized-cell and CPU-raster seam. `hello-terminal-surface` composes a
  transcript and prompt, while `hello-tui-tools` composes a non-shell status
  dashboard with no prompt, transcript, command history, or viewport policy.
  The website Ratatui corpus enables the optional `tui-tools::ratatui-bridge`,
  which converts an already-composed Ratatui `Buffer` into the same normalized
  cells before rasterization. The bridge displaced duplicated buffer style
  mapping and CPU frame allocation in the website consumer and the Ratatui
  oracle.
- Local measurements: on the 2026-08-06 development machine, the selected
  WASM artifacts measured `319,158` bytes without the optional Ratatui
  producer and `454,152` bytes with it, a `134,994` byte delta. Native release
  executables measured `6,187,520` and `6,290,432` bytes respectively. One
  256-iteration CPU composition run averaged `966 us` for the independent
  producer and `1,809 us` for the optional Ratatui producer. These are local
  observations, not cross-machine budgets or admission thresholds.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings:
  - A small caller-owned `Surface` and normalized-cell raster seam has
    independent terminal and non-shell dashboard pressure.
  - Ratatui can remain an optional composition provider above that seam without
    introducing Ratatui types into the base `tui-tools` contract.
  - The website retains font selection and Canvas presentation, while Ratatui
    retains widget layout, terminal composition, and its native style
    vocabulary.
  - The current size and CPU measurements establish an ordering between these
    two local corpus paths only. They do not select a default provider.
- Limits: the evidence does not establish shared focus, viewport, action, or
  session semantics; native terminal hosting; wide and combining text behavior
  across providers; GPU or browser-framebuffer equivalence; or two independent
  non-Ratatui consumers needing the same richer composition contract.
- Disposition: Continue incubation. The lower renderer-facing handoff is now
  stronger evidence, but neither a public Tokimu terminal capability nor a
  provider decision is admitted.
- Resulting ADR or documentation change: no ADR; the TUI corpus plan records
  the shared bridge, independent consumer evidence, and retained provider and
  consumer ownership.

### Cycle 21 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: the lower normalized raster-cell seam now accepts an explicit
  continuation bit. `hello-terminal-surface` resolves the independent `A界B`
  fixture into a wide-grapheme lead plus a trailing continuation, and the
  shared raster paints the continuation background without emitting glyph ink
  or text decorations.
- Findings: continuation is resolved layout metadata supplied by a producer,
  not a width policy calculated by the raster seam. This strengthens the
  provider-neutral handoff without making Unicode width, shaping, or grapheme
  segmentation native TUI semantics.
- Limits: Ratatui 0.29 clears public `Buffer` cells after a multi-width
  grapheme to ordinary blanks and exposes no continuation marker. Its public
  `skip` flag is graphics-diff behavior, not width metadata. The bridge does
  not infer a continuation from a blank cell, so cross-provider wide and
  combining-text equivalence remains unproven.
- Disposition: Continue incubation. Preserve explicit continuation only where
  a provider can resolve it; do not admit a global Unicode policy or Ratatui
  implementation detail.
- Resulting ADR or documentation change: no ADR; the TUI corpus plan and
  terminal-surface research record the bounded continuation result.

### Cycle 22 -- 2026-08-06

- Status entering review: Incubating.
- New evidence: `tui-tools` now declares Ratatui only through optional
  `ratatui-bridge` and `ratatui-oracle` features. Normal dependency trees for
  `tokimu-core` and `tokimu-runtime` contain neither Ratatui nor corpus crates.
  Matched local measurements also now retain the current native linked-payload
  delta (`+107,520` bytes), WASM artifact delta (`+134,994` bytes), one-load
  retained-raster cache behavior, and bounded CPU composition ordering
  (`505 us` independent; `909 us` Ratatui) without claiming a deployment
  budget.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: the dependency boundary is now verified rather than merely
  intended. A small caller-owned surface and raster handoff can remain
  Tokimu-local while Ratatui remains an optional composition provider above
  it. AR-0013 continues to own shell and session semantics.
- Limits: cold and incremental build measurements, native raw terminal input,
  GPU completion and framebuffer evidence, and paired Unicode/combining-text
  behavior remain open. The measurements do not select a default provider or
  admit a public terminal capability.
- Disposition: Continue incubation. Close the dependency-boundary question but
  retain the provider-selection and richer cross-host behavior questions.
- Resulting ADR or documentation change: no ADR; the TUI corpus plan and this
  review now distinguish completed boundary evidence from the remaining study.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0004-foundational-presentation-text-and-icons.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/Architectural Reviews/AR-0013-observation-shell-and-ratatui-presentation-provider.md`
- `docs/Plans/Standalone/tokimu-console-command-window-corpus.md`
- `docs/Plans/Standalone/tokimu-observation-shell-consumer-corpus.md`
- `docs/Plans/Standalone/terminal-surface-provider-study.md`
- `corpus/consumers/tokimu-website-ratatui-lab/DESIGN.md`
- `corpus/consumers/tokimu-website-ratatui-lab/engine/src/backend.rs`
- `corpus/focused/observation/hello-observation-shell/src/ratatui.rs`
- `third-party/presentation-providers/ratatui/Cargo.toml`
