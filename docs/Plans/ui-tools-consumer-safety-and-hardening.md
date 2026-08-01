# UI Tools Consumer Safety And Hardening

## Status

Proposed implementation plan. Audit baseline recorded 2026-07-31.

## Purpose

Harden `corpus/lib/ui-tools` into a dependable corpus-side presentation
library whose ordinary APIs make readable, responsive, interactive UI easier
to produce than overlapping or unbounded UI.

The immediate trigger was the runtime inspector layout failure. A consumer
assembled reasonable semantic regions but still produced overlapping text,
undersized content, and brittle fixed geometry. Repairing the example exposed
a broader problem: `ui-tools` provides many useful primitives, but it does not
yet provide one complete, constraint-safe path from semantic UI intent to
interaction and renderer submission.

This plan treats the incident as library evidence rather than an isolated
example defect.

> A consumer should have to opt into unsafe presentation behavior, not discover
> it after composing ordinary UI tools.

## Related Sources

- [`corpus/lib/ui-tools/DESIGN.md`](../../corpus/lib/ui-tools/DESIGN.md)
- [ADR-0004: Foundational Presentation Text And Icons](../ADR/ADR-0004-foundational-presentation-text-and-icons.md)
- [AR-0001: Shared Vector Presentation Geometry](../Architectural%20Reviews/AR-0001-shared-vector-presentation-geometry.md)
- [Testing Strategy](../testing-strategy.md)
- [UI Presentation Performance Evidence](../Notes/ui-presentation-performance-evidence.md)
- [Runtime Inspector Layout Audit](../../.workbench/Audits/ui-tools-runtime-inspector-layout-audit.md)
- [Runtime Observation And Command Corpus](runtime-observation-and-command-corpus.md)
- [Performance Diagnostics And Runtime Observation](performance-diagnostics-and-runtime-observation.md)

## Executive Audit Finding

`ui-tools` has strong low-level presentation evidence and broad corpus use, but
its consumer-facing composition path is incomplete.

The library currently contains mature pockets for text, SVG, vector geometry,
font outlines, themes, surfaces, and focused controls. Consumers still commonly
perform important work themselves:

- construct large groups of literal `UiRect` values;
- decide how impossible constraints should fail;
- lower surfaces and text through separate command collections;
- coordinate focus, hit testing, clipping, and scrolling locally;
- decide when layout or geometry should be rebuilt;
- infer whether a result fit without receiving structured diagnostics.

This is why many individually useful APIs did not prevent the runtime inspector
incident. The missing capability is not another control. It is a complete,
observable composition contract.

## Audit Baseline

The following measurements are implementation evidence, not permanent API
claims:

- `ui-tools` currently reports 215 unit tests.
- 137 of those tests are concentrated in SVG, vector, and font-outline modules.
- The broad cross-cutting UI test module contains 31 tests.
- Controls have one local test, text input has two, and scrolling has four.
- The repeatable audit reports 18 root public-export statements and 641 source
  public declarations. Neither is a semantic API count, but together they show
  that the root facade and implementation surface are broad.
- Corpus consumers still contain substantial literal rectangle construction and
  local layout geometry, including the UI corpus, runtime inspector, CGM
  inspector, and file/UI examples.
- `cargo clippy -p ui-tools --all-targets -- -D warnings` now passes. The
  initial repair retained rich SVG/XML diagnostics while compacting nested error
  storage, and removed mechanical lint debt without changing UI behavior.
- The CGM performance investigation showed that a static screen could issue
  thousands of draws and repeatedly allocate renderer bindings before an
  example-specific optimization exposed the problem.

The raw test count therefore overstates consumer safety. Geometry parsing and
tessellation have substantially stronger evidence than composition, input,
overflow, clipping, and steady-state presentation behavior.

## Architectural Boundary

The target flow is:

```text
Application intent
        |
        v
Semantic UI tree
        |
        v
Measurement and constraint-safe layout
        |
        v
Resolved UI tree + structured diagnostics
        |                         |
        |                         +--> interaction and focus routing
        v
Owned renderer-neutral draw list
        |
        v
Renderer adapter and backend execution
```

Ownership remains explicit:

- Applications own their information, commands, and domain state.
- UI semantics own control meaning, composition, state roles, and layout intent.
- Layout owns measurement, constraints, fit results, overflow decisions, and
  resolved regions.
- Interaction owns hit testing, focus, capture, activation, and text-input
  routing over the resolved tree.
- Presentation lowering owns renderer-neutral surfaces, text, icons, vectors,
  clips, layers, and diagnostic provenance.
- Providers own font, icon, SVG, and other external implementation technology.
- Renderers own GPU resources, uploads, batching, cache lifetime, and pixels.

`ui-tools` remains a corpus incubation library. This plan does not admit UI into
`tokimu-core`, create a retained application framework, or promote a stable
first-party crate merely because the current source tree is large.

## Hardening Principles

### Safe Composition Is The Default

Ordinary layout APIs must not silently overlap children or compress below
declared minimum sizes. Impossible constraints return an explicit fit result and
diagnostic.

### Measurement Precedes Placement

Text, icons, and content are measured through provider-neutral contracts before
their containing regions are finalized. Visual bounds, advance bounds, and
layout bounds remain distinct.

### One Resolved Tree Drives Pixels And Input

Rendering and interaction must consume the same resolved geometry. A control
must not be visually located in one rectangle while hit testing uses another.

### Stable Output Is Observable

The library must expose enough revision and diagnostic information to prove that
an unchanged UI does not repeat measurement, layout, tessellation, or command
construction accidentally.

### Failure Is Structured

Overflow, missing providers, invalid focus, clipped interaction, unsupported
composition, and resource limits produce bounded diagnostics. They do not become
silent fallback or unbounded logs.

### Corpus Evidence Protects Public Meaning

Unit tests protect algorithms. External integration tests protect the public
API. Corpus applications and deterministic artifacts protect composition and
presentation behavior.

## Findings And Required Actions

### P0: Constraint Failure Is Not A First-Class Result

Generic stack layouts can proportionally shrink requested child sizes to fit a
parent without reporting that minimum readability was violated. `UiLayoutResult`
does not carry semantic identity, fit state, overflow, or diagnostics.

Required action: make every composition operation return an explicit result
that distinguishes exact fit, adjusted fit, overflow, and impossible layout.

### P0: There Is No Complete Consumer-Safe Composition Path

Consumers can use regions, layouts, controls, text, and drawers, but still have
to connect them manually. This permits local rectangle math and divergent draw
and hit-test geometry.

Required action: introduce a small semantic tree and resolved tree that compose
existing primitives without replacing their provider boundaries.

### P0: The Strict Quality Gate Is Already Red

The package does not satisfy the repository's strict Clippy expectation.

Required action: establish a clean baseline before adding more public surface,
then keep format, lint, tests, and documentation checks mandatory.

### P1: The Public Facade Does Not Express Stability Tiers

The root module exposes controls, layout, SVG parsing, font providers, raster
helpers, tessellation, themes, and test-oriented corpus support together. An
ordinary consumer cannot easily distinguish semantic UI contracts from
incubating provider and geometry internals.

Required action: define explicit semantic, provider, lowering, and diagnostic
API tiers. Migrate root re-exports gradually; do not churn callers only to make
the module tree look tidy.

### P1: Draw Output Is Fragmented

`UiDrawer` exposes separate surface and text command vectors. It lacks one owned
draw artifact with clip/layer ordering, semantic provenance, revision identity,
and bounded lowering diagnostics.

Required action: define an owned `UiDrawList`-style artifact and make renderer
submission an adapter over it.

### P1: Interaction Is Too Button-Specific

Focus and identity are centered on `UiButton` and a small button identifier.
Text input, scrolling, disabled controls, pointer capture, and future controls
cannot rely on one generalized resolved interaction model.

Required action: add provider-neutral control identity and deterministic event
routing over the resolved UI tree.

### P1: Scroll, Clip, Overlay, And Modal Semantics Are Thin

These behaviors combine layout and interaction and therefore expose different
bugs than isolated geometry tests. Existing local coverage is too small to
protect nested clipping, scroll offsets, focus visibility, overlay precedence,
or modal input exclusion.

Required action: implement and test these as composition behavior rather than
renderer tricks.

### P1: The Efficient Path Is Not The Ordinary Path

Performance evidence exists, but consumers do not yet receive standard
invalidation, revision, rebuild, and batching behavior. GPU caching must remain
renderer-owned, while semantic revisions and stable draw output belong above
the renderer.

Required action: make steady-state rebuild behavior measurable and bounded.

### P2: Test Coverage Is Numerically Strong But Behaviorally Imbalanced

Most tests protect geometry providers. Few protect full UI composition through
the external public API across viewport, scale, provider, interaction, and
overflow variations.

Required action: introduce a declared matrix and report results by capability,
not only total test count.

### P2: Consumer Boilerplate Is Repeated Evidence

Repeated rectangle placement, shadow construction, text lowering, renderer
submission, and resize logic indicate missing or awkward shared paths.

Required action: inventory and retire duplication only after the replacement is
proven by multiple independent consumers.

## Implementation Slices

### Slice 0: Establish The Quality And Evidence Baseline

#### Deliverables

- [x] Make strict Clippy pass for `ui-tools` and its tests.
- [x] Record test counts by capability rather than one aggregate number.
- [x] Add a script or test utility that inventories public root exports.
- [x] Inventory direct consumer rectangle construction, manual lowering, and
      renderer submission boilerplate.
- [x] Record current runtime inspector and CGM steady-state presentation stats.

#### Tests

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -p ui-tools --all-targets -- -D warnings`
- [x] `cargo test -p ui-tools`
- [x] Existing runtime inspector and CGM corpus applications build unchanged.

#### Acceptance Criteria

- [x] The package begins the hardening work with a green strict quality gate.
- [x] Future progress can be measured against checked-in capability categories
  and consumer-duplication evidence.

### Slice 1: Define Public API Tiers

#### Progress

2026-07-31 established additive `consumer`, `diagnostics`, `provider`, and
`lowering` entry points. A crate integration test now proves that a small
ordinary layout and text-intent caller compiles through `consumer` alone.
Existing root re-exports remain source-compatible while corpus callers migrate.

The remaining work in this slice is intentional: inventory legitimate root
imports, migrate an independent caller, and only then decide which root
exports can be deprecated. The tier modules provide direction today; they do
not claim that implementation APIs are already hidden from legacy callers.

`hello-runtime-inspector` now imports its frame, split, geometry, and text
intent from `ui_tools::consumer`. Its direct `layout_bitmap_text` use remains
at the renderer-specific boundary, recording the missing owned draw-list and
text-lowering contract that later slices must replace.

#### Deliverables

- [x] Document semantic, layout, interaction, lowering, provider, geometry, and
  diagnostic tiers in `ui-tools`.
- [x] Introduce explicit module entry points for ordinary UI consumers.
- [ ] Keep SVG, font, raster, and tessellation implementation APIs below
  provider/lowering modules.
- [x] Add a deprecation and migration policy for broad root re-exports.
- [x] Add compile-only external-consumer tests using the intended public path.

#### Tests

- [x] External integration tests import only the consumer tier.
- [x] Provider tests prove implementation types do not leak into semantic specs.
- [x] Existing corpus consumers continue to compile during staged migration.

#### Acceptance Criteria

- [x] A new consumer can identify the supported composition API without using
  importer or tessellator internals.
- [x] No engine boundary changes are implied by source organization alone;
  AR-0007 explicitly separates API tiers from capability admission.

### Slice 2: Add Semantic And Resolved UI Trees

#### Progress

2026-07-31 introduced the first provider-neutral `UiTree` and headless
resolver. `UiNodeSpec` records stable identity, parentage, semantic kind and
role, text content, explicit/fill/inset layout intent, visibility, enabled
state, clipping intent, and ordered children. Resolution produces one
`UiResolvedTree` with deterministic bounds, inherited clips, visibility,
pre-order layers, fit status, and source-node provenance. Invalid parentage and
duplicate identities fail explicitly.

The current adapters cover the existing `UiRegion` family (including panel and
toolbar aliases), cards, buttons, and text. The tree intentionally does not yet
own richer constraints or a general interaction router. Its first draw-list
lowering now consumes the same resolved bounds, clips, visibility, text intent,
and deterministic node ordering; later slices must extend that path rather than
recreate geometry in consumers.

#### Deliverables

- [x] Define stable provider-neutral region/control identity.
- [x] Represent semantic role, content, children, layout policy, and state
      without renderer-native objects.
- [x] Define a resolved tree containing final bounds, clip bounds, visibility,
      z/layer order, and fit status.
- [x] Preserve bounded resolution diagnostics and semantic provenance from spec
      to resolved node.
- [x] Provide adapters from current panels, cards, toolbars, buttons, and text.

#### Tests

- [x] Equivalent semantic trees preserve stable identities and resolution.
- [x] Child order and pre-order layers are deterministic.
- [x] Duplicate identities and invalid parentage fail explicitly.
- [x] Bounds, clipping, visibility, and fit resolve without a renderer.
- [x] Disabled controls remain visible while exposing disabled semantic state.

#### Tests

- [x] Tree identity remains stable across equivalent rebuilds.
- [x] Child order and resolved layer order are deterministic.
- [x] Duplicate identities and invalid parentage fail explicitly.
- [x] Headless tree resolution requires no window, GPU, or renderer.

#### Acceptance Criteria

- [x] Rendering and interaction can consume one resolved geometry source.
- [ ] Consumers no longer need parallel rectangle maps for drawing and input.

### Slice 3: Make Layout Constraint-Safe

#### Progress

2026-07-31 introduced the first shared fit contract: `UiLayoutFit` reports
`Exact`, `Adjusted`, `Overflow`, or `Impossible`, and every stack result now
carries the status plus overflow extent. Existing contained stack behavior is
preserved as the explicit default `UiOverflowPolicy::Compress`, which reports
`Adjusted` instead of silently claiming an exact fit. Consumers can select
`UiOverflowPolicy::Preserve` to retain measured child geometry and receive an
`Overflow` result suitable for scrolling, clipping, or a compact fallback.

The existing frame and horizontal split now expose the same vocabulary at their
current level of evidence. A required frame region or split pane that resolves
to zero or non-finite geometry now reports `Impossible`, rather than appearing
as a successful compact layout. `UiResolvedLayout` gives frame, split, stack,
grid, and already-recursive layout results one provider-neutral
`UiLayoutResult` view without discarding their specialized metadata. Semantic
`SpaceBetween` allocation provides toolbar-style spacer behavior without
consumer-authored gap arithmetic. The runtime inspector's initial fit-policy
migration is complete, and a narrow uniform-grid resolver provides row-major
equal-cell composition with explicit fit status. Content measurement, spanning,
and implicit column selection remain outside that grid contract until
independent consumers require them. Full semantic-tree and shared draw-list
migration is tracked separately in Slice 11.

The semantic-tree path now supports minimum, preferred, and maximum sizes.
`UiNodeLayout::Fit` centers a node at its preferred size while ordinary `Fill`
retains its existing meaning. Resolution preserves declared minimums and
reports bounded, node-identified overflow when parent capacity is insufficient;
maximum clamping and preferred fitting report `Adjusted`. Zero or non-finite
resolved geometry is reported as `Impossible`.

The runtime inspector now consumes those fit states as a presentation policy:
when its frame, pane split, or footer is not `Exact`, it renders a compact
observation summary and labels the constrained layout instead of attempting to
draw the full dense inspector below its admitted readable capacity.

#### Deliverables

- [x] Extend layout results with exact, adjusted, overflow, and impossible fit
  states.
- [x] Add minimum, preferred, and maximum size constraints where evidence
  requires them.
- [x] Replace silent proportional compression below minimums with explicit
  overflow or a caller-selected fallback policy.
- [x] Add bounded layout diagnostics with node identity and violated constraint.
- [x] Support frame, split, stack, grid, padding, and alignment behavior
  through explicit fit-result contracts.
- [x] Support spacer behavior and unify frame, split, stack, and grid behind
  one result contract.

#### Tests

- [x] Wide, normal, narrow, and impossible viewport tables.
- [x] Long text, empty content, zero-size parent, large padding, and conflicting
      minimum-size tests.
- [x] Generated tables assert finite, ordered, contained
  rectangles where containment is promised.
- [x] Contained stack layouts contain no unintended sibling overlap.

#### Acceptance Criteria

- [x] Impossible layouts cannot masquerade as successful layouts.
- [x] The runtime inspector remains readable or reports its fallback at every
  admitted viewport size.

### Slice 4: Unify Measurement And Text Fit

#### Progress

2026-07-31: semantic text nodes now retain their full provider-neutral
`UiTextSpec` through headless tree resolution. The resolved text rectangle is
derived from the same final node bounds used by the rest of the tree, preserving
alignment and overflow policy without a second consumer-owned rectangle map.
An additive `resolve_with_text_metrics` path now attaches provider-neutral
`UiTextMeasure` and `UiTextFit` evidence to each resolved text node and reports
overflow, missing glyphs, or provider unavailability through bounded tree
diagnostics. Plain `resolve` remains provider-free; it never selects an
implicit bitmap or system-font fallback. Renderer-neutral draw-list lowering
forwards those text-resolution findings as bounded diagnostics with the same
node provenance.

2026-08-01 completed the explicit bitmap proof-path policy set. `Clip`, `Wrap`,
and `Ellipsis` remain unchanged; `Defer` now emits no glyph presentation when
the complete request cannot fit while preserving pre-policy overflow evidence,
and `ScaleDown` deterministically reduces the requested presentation height to
fit both axes. These remain semantic `UiTextSpec` choices rather than renderer
or provider fallbacks.

2026-08-01 also added fixed-size metrics adapters for the built-in bitmap text
engine and external TTF/OTF rasterizers. All three satisfy the same
`UiTextMetricsProvider` contract, including multiline aggregation, visible
bounds, finite metrics, and bounded diagnostics. The semantic tree consumes
that shared contract without retaining provider identity or parser objects.

2026-08-01 added component-fit evidence after the matrix exposed a real shared
geometry defect: `UiCard` positioned header, body, and footer by independent
percentages, allowing adjacent sections to overlap. Cards now partition their
padded content into ordered, disjoint sections. Multi-viewport tests cover
toolbar controls and status content, while panel headers, card headers, and
status labels are lowered through the admitted bitmap text path and checked
against their owning rectangles. Existing text tests complete the baseline,
multiline, clipping, start/center/end alignment, overflow, and constrained-label
matrix.

#### Deliverables

- [x] Integrate `UiTextFit` with semantic layout rather than leaving it as a
  consumer-selected helper.
- [x] Preserve advance, visual bounds, baseline, ascent, descent, and line-gap
  distinctions.
- [x] Add explicit wrap, clip, ellipsis/deferred, and scale-down policies.
- [x] Return missing-font and missing-glyph diagnostics through the resolved
  node.
- [x] Keep font provider identities out of application semantic roles.

#### Tests

- [x] Equivalent layout contract tests for built-in, TTF, and OTF providers.
- [x] Baseline, multiline, clipping, alignment, and constrained-label tables.
- [x] Missing provider and glyph fallback tests.
- [x] Button, toolbar, panel header, card, and status-line component tests.

#### Acceptance Criteria

- [x] Text cannot silently escape a resolved content box: measured resolution
  reports pre-policy overflow on the owning node.
- [x] Provider changes preserve semantic layout contracts or emit a diagnostic.

### Slice 5: Introduce An Owned Draw List

#### Progress

2026-07-31 introduced the first immutable, renderer-neutral `UiDrawList` and
validated `UiDrawListBuilder`. It preserves semantic-tree source identity where
available, producer revision, monotonically ordered layers, deterministic
insertion order for equal layers, and explicit balanced clip operations. The
legacy `UiDrawer` command vectors can now adapt into the owned list without
changing existing corpus callers; its established surfaces-before-text order is
preserved during that transition.

Resolved-tree lowering now emits this list directly for semantic surfaces and
attached text. The same resolved geometry therefore drives both admitted
button hit testing and the new renderer-neutral presentation handoff. Stateful
button styling, icons, and general vector content remain separate follow-up
contracts rather than implicit lowering guesses.

This increment deliberately admits only the existing surface and text command
contracts. Icon and general vector entries remain deferred until independent
lowering producers require their own renderer-neutral command shape. No GPU
resource, atlas, mesh, bind-group, or cache lifetime enters the draw list.

2026-07-31 migrated `hello-runtime-inspector` from per-rectangle and
per-glyph renderer submissions to a single `UiDrawList` build followed by one
local renderer adapter. Its observation layout remains intentionally local for
now, but its presentation handoff is no longer expressed as parallel surface
and text submissions. The adapter maps semantic surface/text requests to that
example's existing native materials and bitmap glyph path; it does not expose
those backend details through `ui-tools`.

#### Deliverables

- [ ] Extend the renderer-neutral draw artifact from surfaces, text, and clips
  to icon and general-vector producers when their command contracts are proven.
- [x] Carry semantic source identity, draw-list revision, and lowering
  diagnostics.
- [x] Make clip push/pop or equivalent nesting explicit and validated.
- [x] Adapt current `UiDrawer` consumers without admitting GPU resources.
- [x] Define deterministic ordering for equal-layer commands.

#### Tests

- [x] Structural snapshots of draw-list ordering and clip nesting.
- [x] Invalid clip/layer structures fail before renderer submission.
- [x] Resolved-tree lowering preserves text bounds, clip nesting, and source
  identity in one ordered artifact.
- [x] Equivalent semantic trees produce equivalent draw-list fingerprints,
  independent of producer revision.
- [x] A native corpus renderer adapter consumes only the owned draw artifact
  (`hello-runtime-inspector`).

#### Acceptance Criteria

- [x] One native consumer no longer coordinates separate surface and text
  command vectors; legacy corpus callers remain in staged migration.
- [x] The renderer-facing lowering artifact contains commands to execute, not
  UI-tree meaning or backend resources.

### Slice 6: Generalize Interaction Routing

#### Progress

2026-07-31: `UiResolvedTree` now provides the first shared hit-test seam for
admitted interactive nodes. It consults resolved visibility, enabled state,
layer order, bounds, and inherited clips, so a disabled overlay cannot consume
a lower control's activation and an off-clip control cannot activate.

The resolved interaction contract now belongs to `UiNodeInteraction`, not to
the button type alone. Existing button nodes remain activatable by default for
source compatibility, while cards or future controls can explicitly opt into
the same clipped, visibility-aware target resolution without borrowing a
button identity. `UiPointerRouter` now supplies deterministic move, press,
release, capture, and release-to-activate resolution over that same tree. It
reports node identities only; applications still own callback execution,
commands, state mutation, keyboard focus, and text input.

The tree now also exposes provider-neutral resolved focus traversal by
`UiNodeId`. Traversal follows stable resolved pre-order, ignores disabled or
clipped nodes, and clears a retained focus identity when a re-resolved tree no
longer admits that node. The helper accepts already-normalized Enter or Space
activation and returns only the focused identity; platform normalization and
application commands remain outside this helper.

Editable nodes use the same focus identity. `UiTextInputRouter` resolves
normalized edit operations such as character insertion, cursor movement, and
deletion to a focused editable `UiNodeId`; it does not mutate text state,
interpret platform events, or claim IME composition. The existing
`UiTextInputState` remains the small application-owned editing model.

`UiPointerRouter::interaction_state` now derives disabled, pressed, hover,
focus, selected, and idle presentation states from the resolved tree, pointer
router, and focus identity. Application selection remains an explicit semantic
input, while transient capture and hover state take precedence over it.

#### Deliverables

- [x] Replace button-only identity assumptions with explicit node interaction
  capability while preserving button compatibility.
- [x] Resolve hit targets from the same clipped bounds used for drawing.
- [x] Define headless hover, capture, and release-to-activate target state.
- [x] Define pressed, selected, focused, and disabled presentation states.
- [x] Route pointer capture and activation targets deterministically.
- [x] Route normalized keyboard activation targets deterministically.
- [x] Route normalized text-input events deterministically.
- [x] Define focus traversal order and behavior when focused nodes disappear.
- [x] Preserve normalized activation and editing intent rather than exposing
  platform events upward.

#### Tests

- [x] Boundary, overlap, clipping, disabled, and generic activatable-node
  target-resolution tests.
- [x] Capture, drag-outside, release, activation, and capture-cancellation
  tests.
- [x] Resolved-tree focus traversal and focus-loss tests.
- [x] Keyboard activation target tests.
- [x] Text input, space, digits, backspace, and focus-loss tests; composition
  remains explicitly deferred.
- [x] Identical recorded event sequences produce identical semantic outcomes.

#### Acceptance Criteria

- [x] Visual and interactive bounds cannot diverge for resolved-tree lowering.
- [ ] Every admitted control uses one routing contract rather than local polling.

### Slice 7: Harden Scroll, Clip, Overlay, And Modal Composition

Progress: `UiVerticalScroll` now exposes bounded content translation, explicit
hidden/partial/full visibility, deterministic focus-into-view behavior, and
finite-input repair. `UiNodeSpec::with_child_translation` moves descendant
content during tree resolution while preserving the viewport node, so resolved
bounds, inherited clips, draw lowering, and hit testing share one geometry.
The public consumer regression includes a partially clipped scrolled control
and verifies that off-viewport geometry cannot receive input. Resolved nodes
now declare normal, overlay, or modal stacking. Stable stacking order drives
drawing and hit testing; the topmost modal confines pointer and focus targets,
and dismissible modals produce application-owned dismissal requests rather
than mutating state inside `ui-tools`. A combined consumer test proves that an
active modal excludes translated scroll content from pointer and focus routing.

#### Deliverables

- [x] Define scroll viewport, content extent, offset, and visibility contracts.
- [x] Ensure nested clips affect drawing and hit testing identically.
- [x] Define overlay and modal precedence, focus confinement, and dismissal.
- [x] Add focus-into-view behavior or an explicit deferred diagnostic.
- [x] Bound scroll offsets after resize and content changes.

#### Tests

- [x] Nested scrolling and nested clipping tests.
- [x] Off-screen controls cannot receive pointer activation.
- [x] Modal content excludes background interaction.
- [x] Resize preserves or clamps valid scroll state deterministically.

#### Acceptance Criteria

- [x] Scroll and modal behavior compose without example-local coordinate repair.
- [x] Draw, clip, and hit-test evidence agree structurally.

### Slice 8: Harden Theme And Accessibility Semantics

Progress: semantic node specifications now carry optional labels, values, and
selected state. Resolution preserves that meaning alongside derived semantic
roles, visibility, enabled state, and focusability. A focus-aware semantic
snapshot is provider-neutral and modal-aware, and draw-list source identities
remain correlatable with semantic node identities. Theme roles and interaction
states expose complete inventories, control styles retain their semantic role
and state, and both standard and high-contrast profiles validate structurally.
Malformed tokens and indistinguishable state output produce diagnostics.
Platform accessibility adapters remain intentionally open.

#### Deliverables

- [x] Define required theme roles for every admitted control state.
- [x] Add semantic label, role, value, enabled, selected, and focus metadata to
  resolved nodes where meaningful.
- [x] Add a high-contrast corpus theme without claiming platform accessibility
  integration that does not exist.
- [x] Diagnose missing or invalid theme coverage instead of silently using
  arbitrary colors. Admitted enum roles are exhaustive at compile time;
  malformed token scales and state collisions are runtime diagnostics.
- [x] Keep application meaning independent from concrete colors and fonts.

#### Tests

- [x] Theme completeness tests over every control and state.
- [x] Selected, hover, focus, disabled, and danger states remain distinguishable
  in structural theme output.
- [x] Semantic metadata survives layout and lowering.

#### Acceptance Criteria

- [x] A complete theme can style all admitted controls without application-local
  visual constants.
- [x] Accessibility semantics are inspectable even before platform adapters are
  admitted.

### Slice 9: Add Invalidation, Rebuild, And Batching Evidence

Progress: a provider-neutral revision tracker now accepts independently owned
semantic, measurement, layout, geometry, and draw-list revisions. Invalidation
cascades only toward dependent stages and emits bounded per-observation rebuild
counters. An unchanged observation produces zero rebuild evidence after the
initial build. Draw lists now expose an opaque renderer-work cache key that
excludes semantic provenance, plus bounded command counts and conservative
contiguous surface/text batch candidates. Marker-specific batching and
renderer-backed submit budgets remain open. Text, visual-theme, viewport, and
interaction revision helpers now fix their intended invalidation entry points.
Bounded work evidence carries UI-stage microseconds and renderer-observed
upload, submit, and draw counts without moving monitoring policy into UI.
External consumer-contract tests now keep named static screens under declared
draw-candidate and warm-rebuild budgets. A dev-only composition test feeds UI
stage evidence into the kernel's sustained performance monitor and verifies a
bounded, stage-owned diagnostic without adding a production dependency from
`ui-tools` to `tokimu-core`.

#### Deliverables

- [x] Define semantic, measurement, layout, geometry, and draw-list revisions.
- [x] Rebuild only stages invalidated by changed input.
- [x] Expose bounded counters and timings for measurement, layout, lowering,
  draw-list generation, uploads, submits, and draws.
- [x] Define renderer-facing stable cache keys without moving GPU cache ownership
  into UI.
- [ ] Establish batching expectations for repeated surfaces, text glyphs, and
  markers.

Repeated surface and text-style runs now emit candidate counts. Marker
semantics remain deferred rather than introducing a command solely to complete
this checklist.

#### Tests

- [x] An unchanged UI performs zero semantic measurement/layout/geometry rebuilds
  after warmup.
- [x] Text-only, theme-only, resize-only, and interaction-only mutations
  invalidate only declared stages.
- [x] Static corpus screens stay under declared draw-candidate and rebuild
  budgets. Actual backend submit budgets remain renderer-observed evidence.
- [x] Budget violations emit bounded diagnostics after sustained observations.

#### Acceptance Criteria

- [ ] Efficient steady-state behavior is the shared path, not an example-local
  optimization.
- [x] UI diagnostics identify the owning stage without claiming backend timing
  guarantees.

### Slice 10: Build The UI Validation Matrix

#### Deliverables

- [x] Add a headless structural UI corpus runner.
- [x] Emit semantic-tree, resolved-layout, interaction-map, normalized input
  sequence, draw-list, diagnostic, and explicitly observational timing artifacts.
- [x] Emit deterministic CPU image artifacts where meaningful.
- [x] Use `corpus/lib/screenshot` for labeled visual evidence.
- [ ] Keep native-window screenshots explicitly labeled as manual backend
  evidence.
- [x] Version the corpus selection and artifact schemas.

#### Required Matrix

| Dimension | Required Cases |
| --- | --- |
| Viewport | 1920x1080, 1280x720, 900x600, 640x480, 320x568 logical units |
| Scale | 1.0, 1.5, 2.0 |
| Text provider | built-in, TTF, OTF, missing-provider path |
| Content | empty, ordinary, long, multiline, missing glyph |
| Input | pointer, keyboard, text input, capture, disabled control |
| Composition | frame, split, stack, grid, scroll, overlay, modal |
| Mutation | static, interaction, text, theme, resize, content replacement |
| Target | headless, native, WASM compile/boot consumer |

#### Tests

- [x] Structural artifact comparisons are authoritative for layout and routing.
- [x] CPU image comparisons complement rather than replace structural tests.
- [x] Artifacts record schema, generator, provider, scale, viewport, input hash,
  and algorithm identity.
- [x] The first divergent artifact stage identifies the owning diagnostic
  boundary.

#### Acceptance Criteria

- [x] Coverage is reported by behavior and matrix dimension, not only test count.
- [x] A regression can be localized before manually opening a native window.

Implementation note: `corpus/lib/ui-validation-corpus` now runs six selected
semantic screens across all five required viewport sizes and emits deterministic
structural artifacts under `target/ui-validation-corpus`. Its first run exposed
and localized false clipping diagnostics caused by exact floating-point rectangle
comparison in `ui-tools`; containment is now classified with a scale-aware
tolerance and protected by a regression test. The selection now also records a
disabled toolbar control that remains semantic but cannot route input, plus a
combined scroll/modal case that retains background draw evidence while confining
semantics and pointer activation to the modal. A text-entry case and versioned
input-sequence artifacts now cover focus traversal, Enter/Space activation,
space and digit insertion, backspace, submit activation, and drag-outside
pointer capture without exposing platform events. The sixth case composes the
shared frame, horizontal split, vertical stack, and uniform-grid resolvers;
together with the scroll/modal case, the required composition matrix is now
covered without consumer-owned cell arithmetic. Provider/content/mutation/target
and IME composition remain open. Canonical desktop/1.0-scale runs now emit a
deterministic CPU diagnostic BMP and labeled manifest through
`corpus/lib/screenshot`; structural artifacts remain authoritative and the
manifest explicitly rejects GPU-framebuffer equivalence. The runner now executes
every physical viewport at 1.0, 1.5, and 2.0
logical scale and emits a versioned coverage report that distinguishes covered
behavior from partial, open, and manual matrix dimensions. Selected-case runs
produce separately named reports and cannot masquerade as full-selection
evidence.

The fifth case adds provider-neutral content stress and a versioned
`content.json` artifact. Empty, ordinary, long, and multiline strings are now
preserved and classified across the complete viewport/scale matrix. The
missing-glyph row remains open until a real text provider can report glyph
availability rather than the structural runner guessing from Unicode content.

### Slice 11: Migrate Independent Consumers

#### Deliverables

- [x] Migrate `hello-runtime-inspector` first as the triggering consumer.
- [x] Migrate `hello-cgm` as a content-heavy and performance-sensitive consumer.
- [x] Migrate one focused UI composition example such as `hello-ui-state` or
  `hello-ui-layout`.
- [x] Migrate one WASM consumer path.
- [ ] Remove duplicate layout, lowering, submission, and interaction code only
  after all consumers pass.

#### Progress

`hello-runtime-inspector` now builds one runtime-specific semantic view, lowers
it into a `UiTree`, resolves that tree once, and uses the shared
`lower_resolved_tree_to_draw_list` handoff. Frame, horizontal split, row-grid,
fit/fallback, theme, and draw ordering are no longer privately reconstructed by
the consumer. Its remaining local submission adapter is intentionally
renderer-facing and does not own layout or hit-test geometry.

`hello-ui-layout` now uses its workspace preset only to produce semantic region
intent, places those regions beneath a full-viewport root, resolves one
`UiTree`, and submits one owned `UiDrawList`. The migration exposed and fixed a
real ownership error: the central workspace had previously been treated as the
application root, which clipped sibling frame regions. Deterministic lowering
and desktop, 4:3, and square viewport tests now protect that boundary.

`hello-cgm` now lowers one CGM-specific observation into a shared `UiTree` shell
using `UiFrameLayout` and `UiHorizontalSplitLayout`. Header, footer, source pane,
vector pane, surfaces, text, fit evidence, and draw ordering are shared. The
application retains only its domain visuals and anchors those visuals through
stable resolved pane identities. The previous absolute-coordinate shell was
removed rather than retained as a fallback. Desktop, 4:3, and square viewport
tests verify bounded panes and deterministic draw-list output.

The runtime-observation WASM workbench now exposes a versioned semantic UI
snapshot beside its existing observation, presentation, and playback contracts.
Rust/WASM observes the runtime, builds and resolves a `UiTree` headlessly, and
lowers it into provider-neutral structural draw evidence. TypeScript requests
and displays that evidence but does not reconstruct layout meaning. Desktop,
constrained, and narrow browser viewport tests verify finite bounded nodes;
equivalent inputs produce identical serialized output, and the consumer's full
release WASM plus TypeScript build completes through its checked-in build script.

#### Tests

- [x] Runtime inspector resolves and lowers at its desktop and constrained
  viewport matrix without structural diagnostics.
- [x] `hello-ui-layout` resolves and lowers deterministically at its desktop,
  4:3, and square viewport matrix without structural diagnostics.
- [x] `hello-cgm` resolves bounded source and vector panes at its desktop, 4:3,
  and square viewport matrix and lowers deterministically.
- [x] The runtime-observation WASM UI snapshot resolves finite, bounded output
  at desktop, constrained, and narrow browser viewport sizes.
- [x] Every migrated consumer runs at all currently applicable viewport sizes.
- [ ] Runtime inspector and CGM visual artifacts remain readable and bounded.
- [x] Native and WASM consumers preserve the same semantic observation output.
- [x] Consumer source inventory shows a material reduction in literal shell
  geometry and manual command plumbing.

#### Acceptance Criteria

- [x] At least three independent native consumers and one WASM consumer use the
  safe composition path.
- [x] No migrated consumer maintains a second private draw/hit-test layout.

### Slice 12: Admission And Decomposition Review

#### Progress

2026-08-01: AR-0007 accepted semantic UI composition as a coherent
foundational presentation capability candidate based on independent native and
WASM consumers. Extraction remains deferred: the current `ui-tools` package
still co-locates semantic UI with TTF/OTF, XML/SVG, icon, vector, and
tessellation implementations. ADR-0004 remains correct and required no change.
The SDD and incubator design now record this distinction explicitly.

#### Deliverables

- [x] Review whether the proven semantic UI boundary warrants a first-party
  capability package.
- [x] Reassess vector, text, icon, provider, and corpus-support module placement
  using independent-consumer evidence.
- [x] Record accepted, deferred, and rejected findings in an Architectural
  Review.
- [x] Update ADR-0004 or other ADRs only if accepted ownership changes. No ADR
  change was required by this review cycle.
- [x] Update the SDD and `ui-tools` design to match the implemented boundary.

#### Tests

- [x] Dependency checks reject renderer, platform, window, browser, and GPU
  dependencies while reporting current provider dependencies as extraction
  blockers.
- [x] Public consumer tests define the contract that a future package extraction
  must preserve; no extraction occurred in this slice.
- [x] Corpus artifacts remain stable; this review introduced no artifact schema
  changes.

#### Acceptance Criteria

- [x] Packaging follows proven ownership rather than file size or preference:
  promotion of the mixed package was rejected.
- [x] No unresolved architectural question is presented as a settled API;
  extraction and remaining contract gaps are explicitly deferred.

## Test Architecture

### Unit Tests

Protect measurement math, constraints, fit classification, focus transitions,
hit testing, clip intersection, scroll clamping, invalidation, and deterministic
ordering at the narrowest honest boundary.

### Crate Integration Tests

Compile and execute representative UI through only the intended public consumer
API. These tests protect visibility, ownership, and diagnostics against internal
refactors.

### Workspace Contract Tests

Protect engine boundary direction, headless operation, renderer-neutral draw
artifacts, and provider substitution.

### Corpus Applications

Use focused `hello-ui-*` entries to interrogate one behavior each. Use runtime
inspector, CGM, and WASM consumers to prove composition under realistic pressure.

### Structural And Golden Artifacts

Prefer semantic tree, layout, interaction map, draw list, and diagnostic
artifacts as authoritative evidence. Images are complementary presentation
evidence and must declare whether they are deterministic CPU output or manual
backend capture.

### Performance Tests

Record stage-specific work and sustained budget transitions. Do not encode one
machine's elapsed time as a universal guarantee. Stable operation counts and
invalidation behavior are stronger cross-machine contracts.

## Continuous Validation Gates

Every hardening slice must preserve:

```text
cargo fmt --all -- --check
cargo clippy -p ui-tools --all-targets -- -D warnings
cargo test -p ui-tools
cargo test --workspace
```

Target-specific checks are added when their slice is active:

- headless UI corpus artifact generation;
- runtime inspector and CGM smoke runs;
- WASM build and browser boot test;
- deterministic CPU screenshot comparison;
- native manual evidence capture for backend-only behavior.

## Explicit Non-Goals

This plan does not attempt to create:

- a general retained-mode application framework;
- CSS, DOM, or browser layout compatibility;
- a GPU resource cache inside UI;
- a new font, SVG, raster, or icon parser;
- rich-text shaping, bidi, IME, or full accessibility platform integration in
  the first slices;
- speculative controls without a consumer;
- immediate crate extraction or kernel admission.

Deferred features must remain visible in diagnostics and reopening criteria.

## Risks

### Building A Framework Instead Of A Contract

Mitigation: keep the semantic and resolved trees small, data-oriented, and tied
to existing corpus consumers. Reject application lifecycle or domain state from
the UI library.

### API Churn Across Many Corpus Consumers

Mitigation: add adapters and deprecations, migrate representative consumers
first, and remove old paths only after external tests pass.

### Snapshot Brittleness

Mitigation: separate structural contracts from visual evidence, version schemas,
and avoid asserting irrelevant floating-point or backend details.

### Performance Optimization Crossing Ownership Boundaries

Mitigation: UI owns semantic revisions and stable draw artifacts; renderers own
uploads, GPU resources, batching execution, and cache lifetime.

### Hiding Failure Behind Responsive Fallback

Mitigation: every fallback is selected explicitly and recorded in fit results
and diagnostics.

### Test Count Becoming A Vanity Metric

Mitigation: report capability and matrix coverage, known unsupported behavior,
and independent consumer adoption.

## Definition Of Done

This plan is complete when:

- strict format, lint, package, and workspace test gates pass;
- ordinary consumers compose UI through semantic and resolved trees;
- impossible constraints return explicit fit results and diagnostics;
- rendering and interaction consume identical resolved geometry;
- text measurement and fit are provider-neutral and component-tested;
- one owned draw artifact carries ordering, clipping, identity, and diagnostics;
- interaction, text input, scrolling, overlays, and modal behavior have
  deterministic external tests;
- unchanged UI performs no repeated semantic layout or geometry work after
  warmup;
- structural and visual evidence covers the required viewport/provider/input
  matrix;
- runtime inspector, CGM, a focused UI example, and a WASM consumer use the
  hardened path;
- duplicate consumer layout and command plumbing materially decreases;
- architecture, SDD, review records, and actual ownership agree;
- unsupported behavior remains bounded, explicit, and reopenable.

The final measure is not that every UI looks attractive. It is that reasonable
consumer code cannot accidentally produce unreadable, divergent, or needlessly
expensive presentation without receiving concrete evidence about why.
