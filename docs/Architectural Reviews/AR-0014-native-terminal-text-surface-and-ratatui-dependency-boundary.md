# AR-0014: Native Terminal Text Surface And Ratatui Dependency Boundary

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | 2026-08-05 |
| Last reviewed | 2026-08-05 |
| Scope | Foundational presentation capability / provider boundary |
| Trigger | Ratatui now has native, WASM, and embedded Tokimu consumers, while its dependency and ownership cost remains unmeasured |
| Related ADRs | ADR-0003, ADR-0004, ADR-0005 |
| Related evidence | AR-0013, Tokimu Ratatui website lab, console command window corpus, observation shell corpus |
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

### Alternative E: Copy Or Fork Selected Ratatui Internals

- Benefits: Could initially reduce dependencies and permit Tokimu-specific
  changes.
- Costs: Takes ownership of copied algorithms, compatibility, security,
  licensing notices, bug fixes, Unicode behavior, and ongoing upstream drift.
- Failure mode: Tokimu acquires a hidden terminal framework without evidence
  that the maintenance burden is justified.

This alternative is not admissible without a separate source, license,
maintenance, and differential-cost review.

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
6. Extracting or copying part of Ratatui is not presently justified. A smaller
   upstream boundary or an independently implemented minimal provider must be
   compared before that option can be reconsidered.

The evidence does not yet support admitting a new crate, replacing Ratatui,
forking Ratatui, or making a native terminal surface kernel-native.

## Disposition

**Incubating.** Continue using Ratatui as a replaceable corpus-side provider and
measure it against a behaviorally matched minimal path. Do not extract, fork,
or stabilize Ratatui-derived terminal semantics yet. Revisit capability
admission only after differential cost and cross-provider conformance evidence
identify a smaller durable ownership boundary.

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

- [ ] Build matched native and WASM scenes with Ratatui enabled and disabled,
      then record incremental release size after the same optimization steps.
- [ ] Record cold build, incremental build, startup, allocation, and warm-frame
      observations for the matched scenes.
- [ ] Inventory the exact Ratatui APIs used by each Tokimu corpus consumer and
      classify them as shell meaning, surface mechanics, text behavior, host
      input, or diagnostics.
- [ ] Add conformance cases for ASCII, Unicode width, grapheme clusters,
      combining marks, wide cells, clipping, resize, cursor state, style reset,
      and changed-cell invalidation.
- [ ] Compare the Ratatui provider with one deliberately minimal independent
      terminal-surface implementation without stabilizing either as public API.
- [ ] Inspect newer upstream Ratatui crate and feature boundaries before
      considering any local extraction.
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

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0004-foundational-presentation-text-and-icons.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/Architectural Reviews/AR-0013-observation-shell-and-ratatui-presentation-provider.md`
- `docs/Plans/tokimu-console-command-window-corpus.md`
- `docs/Plans/tokimu-observation-shell-consumer-corpus.md`
- `corpus/consumers/tokimu-website-ratatui-lab/DESIGN.md`
- `corpus/consumers/tokimu-website-ratatui-lab/engine/src/backend.rs`
- `corpus/hello-observation-shell/src/ratatui.rs`
- `third-party/presentation-providers/ratatui/Cargo.toml`
