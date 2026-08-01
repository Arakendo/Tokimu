# ui-tools Design Doc

## Purpose

`ui-tools` is Tokimu's example-side semantic interface vocabulary.

It should provide reusable interface meaning, not reusable application shells.
Examples should still feel distinct, but they should stop reinventing what a
toolbar, inspector, status rail, or card region *is*.

The renderer still owns pixels. `ui-tools` owns intent.

This crate is currently the proving ground for first-party presentation
capabilities. It is not `tokimu-core`, and it does not commit the engine to a
specific font parser, icon library, rasterizer, atlas, or renderer backend.
Stable concepts may graduate into first-party text and icon capabilities when
they survive independent examples and headless tests.

The architectural boundary is:

```text
application intent
    -> ui-tools semantic contract
    -> replaceable provider/backend
    -> renderer or headless consumer
```

Applications communicate intent. Providers communicate implementation.

## API Tiers

The public facade is being staged toward explicit consumer-safe entry points.
The existing root re-exports remain available while corpus callers migrate; they
are compatibility surface, not the preferred direction for new consumers.

```text
ui_tools::consumer
    semantic regions, layout, controls, text intent, themes

ui_tools::diagnostics
    structured UI and text diagnostics

ui_tools::provider
    fonts, icons, rasterization, SVG and other external technologies

ui_tools::lowering
    renderer-neutral commands and presentation geometry
```

Ordinary applications should begin with `consumer` and add a provider or
lowering contract only when they deliberately own that boundary. A semantic UI
spec must not require an SVG parser, a concrete font format, or a tessellator.

The migration remains additive while corpus consumers move deliberately:

- New ordinary consumer code uses `ui_tools::consumer`.
- A consumer imports `provider` or `lowering` only at an explicit technology or
  renderer-neutral lowering boundary.
- Root re-exports remain source-compatible during the corpus migration; they
  are not deprecated until independent consumers establish that the tiered
  paths cover their legitimate use.
- A future removal or deprecation requires an Architectural Review and a
  consumer migration inventory. Module organization alone does not settle an
  engine package boundary.

The tier names describe intended ownership, not an admission of a stable engine
crate. `ui-tools` remains corpus-side incubation until independent consumers
prove which contracts should graduate.

## Resolved Composition Boundary

Ordinary consumers can now describe a small `UiTree` with stable `UiNodeId`
identities, semantic node kinds, content, visibility, enabled state, clipping,
text intent, and spatial intent. Headless resolution produces a
`UiResolvedTree` with final bounds, inherited clips, deterministic layer order,
and bounded diagnostics.

```text
semantic node tree
        ↓
resolved bounds, clips, and fit evidence
        ├── presentation lowering
        └── interaction hit testing
```

The resolved tree is deliberately renderer-neutral. Its first safety contracts
are intentionally small:

- a node may declare a minimum readable size;
- a too-small region remains explicit geometry but reports layout overflow;
- zero or non-finite geometry reports an impossible layout;
- attached `UiTextSpec` values inherit the same resolved bounds rather than a
  consumer maintaining a second text rectangle;
- admitted button hit testing uses the resolved visible, enabled, clipped
  bounds and deterministic child order.

`UiResolvedTree::node` is the read-only bridge for domain-specific content that
must be anchored inside shared UI composition. A consumer may look up a stable
node identity and place its own chart, mesh, or other semantic-domain evidence
inside the resolved bounds. The lookup does not expose mutable UI state,
provider internals, renderer resources, or a second layout mechanism. The
resolved tree remains the sole owner of shell geometry.

Scroll state contributes a descendant translation during resolution rather
than creating a second drawing-only coordinate system. Resolved nodes also
declare normal, overlay, or modal stacking. The stable stacking order is shared
by drawing and reverse-order hit testing, while the topmost modal confines
pointer and focus targets to its subtree. Dismissal remains an application
decision: `ui-tools` reports the modal identity and dismissal reason but does
not mutate application state.

Resolved semantics preserve provider-neutral role, label, value, visibility,
enabled, selected, focusable, and focused state. A platform adapter may convert
that snapshot into native accessibility mechanisms later, but those mechanisms
do not define UI meaning. Draw-list provenance retains the same `UiNodeId`, so
inspection tools can correlate semantic intent with lowered presentation
without embedding accessibility metadata in renderer commands.

Theme profiles remain structural and provider-neutral. `UiTheme::default()` and
`UiTheme::high_contrast()` cover the same complete surface, text, control, and
interaction-state inventories. Control styles preserve their semantic role and
state so a later palette or platform adapter does not need to infer danger,
focus, selection, or disabled meaning from numeric opacity. Exhaustive enums
make omitted admitted roles a compile-time error; theme diagnostics reject
invalid token scales and accidentally indistinguishable state output. High
contrast here is corpus evidence, not a claim of native accessibility adapter
integration.

Presentation invalidation is also renderer-neutral. Applications and providers
supply revisions for semantic input, measurement, layout constraints, geometry
policy, and draw-list policy. `UiPresentationRevisionTracker` reports which
dependent stages require rebuilding and bounded rebuild counts for one
observation. It does not cache meshes, allocate GPU resources, or choose runtime
performance budgets; those remain renderer and application responsibilities.

This is not yet a retained widget framework or general event router. It is the
single geometry source that later draw-list and interaction contracts must
consume so pixels and hit testing cannot drift apart.

## Ordered Draw Boundary

`UiDrawList` is the renderer-neutral handoff for admitted surface and text
lowering. It carries a producer revision, deterministic layer and insertion
order, optional `UiNodeId` provenance, and explicit clip push/pop operations.
`UiDrawListBuilder::finish` rejects descending layers, clip underflow, and
unclosed clips before a renderer adapter receives the artifact.

```text
resolved UI geometry
        ↓
ordered surfaces, text, and clip operations
        ↓
renderer adapter
        ↓
backend resources and pixels
```

The draw list is intentionally not a mesh list, glyph atlas, texture binding,
or GPU cache. Renderer adapters own those execution details. Existing
`UiDrawer` surface/text vectors remain a staged legacy API and can be adapted
into one owned list with their established surfaces-before-text order.

`lower_resolved_tree_to_draw_list` is the preferred new-consumer path for the
currently admitted region and text contracts. It uses the same resolved bounds,
visibility, clipping scopes, and layer order as `UiResolvedTree::hit_test`.
Stateful control styling, icons, and general vector content remain explicit
extensions rather than being inferred by the renderer.

## Core Thesis

> `ui-tools` provides reusable interface vocabulary, not reusable application interfaces.

That means the crate should describe interface structure and interaction meaning,
not just rectangles and convenience helpers.

## Semantic Layers

The crate should be organized around a semantic stack:

```text
Geometry

Rect
Insets
Anchor
Padding
Margin
Alignment

↓

Layout

Region
Toolbar
Sidebar
Inspector
Workspace
CardGrid
StatusRail

↓

Controls

Button
Toggle
Chip
Badge
IconSlot
Label

↓

Interaction

Hovered
Pressed
Focused
Selected
Disabled

↓

Example
```

This hierarchy explains where new concepts belong and prevents low-level
geometry from becoming the headline concept everywhere.

## Goals

- Provide a small semantic vocabulary for interface regions and controls
- Keep geometry, hit-testing, and layout math reusable across examples
- Support text-bearing layout contracts and reusable renderer-neutral text geometry
- Make visual hierarchy explicit through named surface roles
- Translate semantic controls into abstract draw commands through a local drawer
- Keep spacing, radius, and elevation meaningful instead of numeric noise
- Preserve example-specific look and feel while reusing interface semantics
- Stay framework-agnostic and renderer-agnostic

## Non-Goals

- Full retained-mode UI system
- Backend-specific GPU submission and window management
- Complex widget trees with app-wide focus routing
- Desktop-style styling systems with exhaustive theming knobs
- Application shell ownership
- Engine-owned UI capability before the vocabulary is proven

## Semantic Surface

Authors should think in interface regions first, then controls, then geometry.

Good headline concepts include:

- `UiRegion`
- `UiPanel`
- `UiWorkspace`
- `UiToolbar`
- `UiSidebar`
- `UiInspector`
- `UiStatusBar`
- `UiTabStrip`
- `UiCard`

Those concepts may still be backed by `UiRect`, but they should not force
authors to think in raw rectangles for every layout decision.

## Layout Vocabulary

Reusable layout concepts should reflect editor and workspace structure:

- `ToolbarLayout`
- `SidebarLayout`
- `InspectorLayout`
- `DockLayout`
- `CardGrid`
- `StackLayout`
- `FlowLayout`
- `SplitLayout`
- `CenteredLayout`
- `StatusBarLayout`

These layouts should describe containment, spacing, and intent.

## System Vocabulary

Some examples should test the systems that connect controls and regions rather
than adding more controls.

Useful system concepts include:

- `Layout`
- `State`
- `Input`
- `Scroll`
- `Animation`
- `Inspector`
- `Dashboard`

These concepts answer questions such as:

- how do semantic regions become spatial arrangements?
- how does input become interaction state?
- how does selection propagate through the interface?
- how do viewports clip and reveal content?
- how do transitions stay semantic rather than decorative?

The key point is that these are still semantic contracts, not renderer-owned
behavior.

## Surface Roles

Interface regions should have semantic surface roles instead of raw color
numbers scattered through examples.

Suggested roles:

- `Background`
- `Panel`
- `Raised`
- `Selected`
- `Accent`
- `Overlay`

Examples can map these roles to different palettes, but the semantic meaning
should stay stable.

## Spacing And Shape

Spacing should be named because it communicates hierarchy.

Suggested concepts:

- `Spacing::XS`
- `Spacing::Small`
- `Spacing::Medium`
- `Spacing::Large`
- `Spacing::XL`

Likewise for shape and containment:

- `Radius::Small`
- `Radius::Medium`
- `Shadow::Raised`
- `Padding::Toolbar`

These names should become visual language, not just style constants.

## Controls

The control vocabulary should grow by semantic need, not by widget checklist.

Current and likely control concepts include:

- `Button`
- `Toggle`
- `Radio`
- `Chip`
- `Badge`
- `IconButton`
- `Tab`
- `ToolbarButton`
- `CardAction`

The goal is not to implement every control quickly. The goal is to name the
semantic families that examples naturally keep recreating.

## Theme And Drawer

`ui-tools` should also include a small drawing translation layer.

The drawer is responsible for turning semantic intent into abstract surface and
text commands. It should not own the renderer, but it should own the logic that
decides which surface role, text role, spacing token, or interaction state gets
emitted for a control.

Recommended pieces:

- `UiTheme`
- `UiSurfaceStyle`
- `UiTextStyle`
- `UiControlRole`
- `UiTextRole`
- `UiDrawer`

The drawer should support a small first set of primitives:

- `surface`
- `border`
- `label`
- `button`
- `card`
- `chip`
- `divider`

That is enough to stop example code from manually assembling every slab,
highlight, and label out of raw quads.

## Cards And Regions

Cards should be formalized as semantic interface regions rather than loose
collections of panels.

Useful card structure:

- header
- body
- footer
- padding
- surface role

That makes it easier for examples to compose content cards without hand-drawing
four rectangles every time.

## Text Contracts

`ui-tools` should own text geometry and placement intent, not backend-specific
glyph rendering. Its font and SVG helpers are renderer-neutral evidence
services that may later become `tokimu-text`, `tokimu-font-*`, or `tokimu-icon`
capabilities.

Good concepts include:

- `UiLabel`
- `UiTextBlock`
- `UiCaption`
- `UiTitle`
- `UiChipLabel`

These types should answer questions such as:

- where does text belong?
- how much space does it reserve?
- how does it align?
- does it clip or wrap?
- what region is it attached to?

Renderer code can decide how glyphs are submitted to the GPU. `ui-tools` may
own renderer-neutral font rasterization, glyph layout, SVG parsing, path
flattening, and stroke tessellation when those services are needed by several
examples. It should not own a backend, window, or application shell.

The public direction must remain provider-neutral:

- font sources expose identity and metrics, not parser-native objects;
- text layout exposes advances, baseline, visible bounds, and diagnostics;
- icon identity does not require callers to know SVG or Lucide internals;
- texture, atlas, mesh, and GPU upload remain renderer concerns;
- measurement and layout should remain usable without a live renderer.

The bitmap layout path currently serves as the headless proof of that
requirement: it resolves text placement, alignment, clipping, and wrapping into
stable geometry without a window, GPU, texture upload, or renderer state. A
future provider-neutral text layout result may replace or generalize this path,
but the headless property is part of the contract.

The headless report consumes that same layout result used by renderer-facing
example code. This keeps diagnostics, bounds inspection, and future report
adapters from developing a second interpretation of text placement.

## Renderer-Neutral Asset Services

The glyph and Lucide corpus examples are intentionally consumers of shared
services rather than private parsers. The reusable boundary currently includes:

- `raster.rs` for font rasterization and glyph coverage
- `font_outline.rs` for provider-neutral glyph outlines and font-to-vector lowering
- `text.rs` for baseline-aware glyph layout and text placement
- `svg.rs` for path parsing, curve flattening, SVG primitive extraction, and
  stroke tessellation

These services produce geometry or draw-ready data. They do not decide which
example is being rendered, where an icon is placed in a corpus grid, or which
assets a test selects.

The SVG contract is especially important: closed paths must remain closed,
open paths must retain their true endpoints, and joins belong to connected
path topology rather than to per-segment corrective geometry. Lucide assets are
reference data used to pressure this contract, not a source of example-owned
fallback shapes.

The XML-to-SVG boundary is deliberately profile-based. `xml-tools` owns
well-formed XML, decoded attributes, expanded names, spans, and event order;
`svg.rs` owns SVG namespace policy, path grammar, the admitted presentation
state, transforms, and viewport interpretation. The initial SVG profile admits
only `svg` and `g` containers plus `path`, `circle`, `line`, `polyline`,
`polygon`, and `rect` geometry. Text, `defs`/`use`, clipping, paint servers,
filters, masks, animation, scripting, and external resources are diagnosed as
unadmitted SVG semantics rather than silently treated as supported. Parsed XML
events may feed SVG lowering directly so corpus `xml.json` evidence and SVG
records share source identity without duplicating syntax parsing.

## Button Corpus

The button example should be treated as a corpus test for the whole UI stack.
If one button feels right, the same primitives are usually good enough for
cards, toolbars, panels, and other small controls.

The current button should be improved in this order:

1. text rendering and optical centering
2. padding and hit region balance
3. border thickness and border role usage
4. surface hierarchy and state colors
5. elevation and shadow softness
6. corner style
7. hover feedback
8. pressed feedback
9. disabled feedback
10. focus ring or outline
11. typography scale
12. spacing scale
13. icon support
14. alignment variants
15. minimum size rules
16. state machine coverage
17. animation hooks
18. theme separation
19. drawer API simplification
20. visual balance and scaling
21. hit region vs visual rect
22. semantic theme roles
23. composition into a toolbar or small cluster

Useful state coverage for the corpus includes:

- Idle
- Hovered
- Pressed
- Focused
- Disabled
- Selected
- Primary
- Secondary
- Danger
- Ghost
- Icon
- Large
- Small
- Text only
- Icon only
- Toolbar use
- Dialog use
- Card action use

The goal is not to overbuild the button. The goal is to prove the drawer,
theme, surface roles, spacing, and state machine can express a lot of meaning
from one control before the rest of the UI grows upward from it.

## Interaction Model

Hover, selection, toggle, focus, and disabled states should live in a unified
interaction vocabulary instead of being ad hoc example logic.

Suggested state model:

```text
Idle
Hovered
Pressed
Focused
Selected
Disabled
```

This should stay lightweight, but it should be explicit enough that examples
can describe state consistently.

## Interface Design Language

`ui-tools` should also express a design philosophy.

Preferred principles:

- Strong visual hierarchy
- Whitespace over borders when possible
- Panels communicate grouping
- Color communicates state, not decoration
- Motion should reinforce interaction
- Elevation should indicate containment
- Active elements should be obvious within one second

These are not implementation details. They are part of the interface grammar.

## Corpus Growth

`ui-tools` should evolve from examples and provide evidence for presentation
capability admission.

A helper is promoted only when:

- multiple examples need it
- the abstraction remains simple
- ownership boundaries stay clear
- the concept is semantic rather than stylistic

Examples pressure `ui-tools`.
`ui-tools` pressures future `tokimu-text` and `tokimu-icon` candidates.

## Current Folder Structure

```text
ui-tools/
├── Cargo.toml
├── DESIGN.md
└── src/
    ├── controls/
    │   ├── button.rs
    │   ├── content.rs
    │   ├── interaction.rs
    │   └── mod.rs
    ├── corpus.rs
    ├── draw.rs
    ├── font.rs
    ├── font_outline/
    │   ├── lowering.rs
    │   ├── mod.rs
    │   ├── provider.rs
    │   ├── types.rs
    │   └── tests.rs
    ├── geometry.rs
    ├── icon.rs
    ├── layout.rs
    ├── lib.rs
    ├── presets.rs
    ├── raster/
    │   ├── bitmap.rs
    │   ├── layout.rs
    │   ├── mod.rs
    │   ├── provider.rs
    │   ├── tests.rs
    │   └── types.rs
    ├── region.rs
    ├── scroll.rs
    ├── svg/
    │   ├── document.rs
    │   ├── mod.rs
    │   ├── path.rs
    │   ├── primitives.rs
    │   ├── semantic.rs
    │   ├── transform.rs
    │   ├── types.rs
    │   └── tests/
    │       ├── document.rs
    │       ├── lucide.rs
    │       ├── mod.rs
    │       ├── path.rs
    │       └── primitives.rs
    ├── tests/
    │   ├── drawing.rs
    │   ├── interaction.rs
    │   ├── layout.rs
    │   └── mod.rs
    ├── text/
    │   ├── bitmap.rs
    │   ├── mod.rs
    │   └── tests.rs
    ├── text_input.rs
    ├── theme.rs
    └── vector/
        ├── builder.rs
        ├── fill/
        │   ├── lyon.rs
        │   └── mod.rs
        ├── geometry.rs
        ├── mod.rs
        ├── stroke.rs
        ├── types.rs
        └── tests.rs
```

## Internal Structure

Keep small, cohesive responsibilities in role-based files:

- `geometry.rs` for rectangles, anchors, margins, and bounds math
- `controls/` for controls, passive content specs, and interaction state
- `layout.rs` for regions, toolbars, sidebars, cards, and framing helpers
- `text/` for baseline-aware text layout and text-box contracts
- `raster/` for font-provider rasterization, baseline layout, and bitmap
  composition
- `svg/` for SVG document lowering, path parsing, and SVG-local transforms
- `vector/` for provider-neutral presentation geometry and tessellation
- `font_outline/` for lowering provider outlines into vector geometry
- future `state.rs` only if examples need shared lightweight interaction state

Use a folder when a capability has multiple independently testable
responsibilities, not merely because a file crosses an arbitrary line count.
The folder's `mod.rs` owns the capability boundary and public exports; internal
files own focused transformations. Tests live beside the capability they
exercise. Cross-capability contract tests live under `src/tests/`, grouped by
the behavior under test.

In particular:

- `svg/path.rs` owns tokenization, path commands, and curve/arc flattening.
- `svg/transform.rs` owns SVG-local affine transform parsing and composition.
- `svg/types.rs` owns structured importer diagnostics and SVG record contracts.
- `svg/semantic.rs` owns namespace-aware XML events, inherited presentation,
  admitted feature classification, attributes, and viewport normalization.
- `svg/primitives.rs` owns primitive point generation and compatibility
  adapters; it does not own document traversal.
- `svg/document.rs` owns XML-to-SVG semantic traversal and primitive lowering.
- `svg/mod.rs` owns only the capability boundary and public importer exports.
- `text/bitmap.rs` is the built-in bitmap provider; it does not define the
  provider-neutral text contract.
- `controls/interaction.rs` owns focus, activation, events, and diagnostics.
- `controls/button.rs` owns button measurement and activation behavior.
- `controls/content.rs` owns passive labels, chips, and card specifications.
- `font_outline/provider.rs` stops font-technology-specific extraction before
  provider-neutral outline contracts.
- `font_outline/types.rs` owns provider-neutral outline and diagnostic types.
- `font_outline/lowering.rs` is the only font-outline module that lowers into
  shared vector geometry.
- `raster/provider.rs` owns provider construction and individual glyph
  rasterization.
- `raster/layout.rs` owns advances, tracking, baselines, and multiline layout.
- `raster/bitmap.rs` owns coverage-buffer composition and color expansion.
- `raster/types.rs` owns the renderer-neutral raster contracts shared by those
  stages.
- `vector/types.rs` and `vector/builder.rs` own paths and their construction.
- `vector/fill/` and `vector/stroke.rs` own independent tessellation paths.
- `vector/fill/lyon.rs` isolates the replaceable Lyon execution backend from
  Tokimu's fill validation, cleanup, and repair policy in `vector/fill/mod.rs`.
- `vector/geometry.rs` contains only numerical helpers shared by those
  tessellators.
- provider-specific parsing must stop before `vector/`.
- renderer submission and GPU cache lifetime must remain outside these folders.

## Invalidation And Renderer Work Evidence

Semantic, measurement, layout, geometry, and draw-list revisions are observed
independently. A changed stage invalidates only its downstream dependents, while
an unchanged observation emits zero rebuild evidence after warmup.

The ordered draw list exposes two deliberately different identities:

- its structural fingerprint includes semantic provenance for corpus artifacts;
- its opaque cache key includes only executable renderer-neutral work.

Renderer adapters may use the cache key to index their own resources, but
`ui-tools` does not own cache lifetime, residency, uploads, or GPU objects.
Draw-list statistics report bounded command counts and conservative contiguous
surface/text batch candidates. Candidate counts are evidence about available
reuse, not guaranteed submit counts or backend performance claims. Adjacent
ordered layers may remain one candidate when style and clip are compatible;
the renderer must preserve execution order and decides whether grouping is
actually legal for its backend.

`UiPresentationWorkEvidence` carries bounded microsecond measurements for
measurement, layout, lowering, and draw-list generation alongside copied
renderer upload, submit, and draw counts. Producers populate the fields they
own. Applications choose budgets and may forward individual observations to
Tokimu's kernel performance diagnostics; `ui-tools` does not own the monitor,
sampling policy, or backend timing guarantee.

## Success Criteria

`ui-tools` is healthy when examples can reuse the same semantic vocabulary for:

- workspace framing
- toolbar and sidebar structure
- inspector and status regions
- button selection and deselection
- card composition
- label placement
- cursor mapping between screen and world space

## Future Path

The likely promotion path is:

```text
Example

↓

Repeated helper

↓

ui-tools

↓

Many examples

↓

tokimu-text / tokimu-icon candidates

↓

Capability

↓

Maybe, rarely, kernel concept
```

That keeps semantic concepts discoverable without forcing them into the engine
too early.

## Boundary Notes

If a future UI system becomes engine-owned, it should only be promoted after
the example-side primitives prove which interface concepts are stable.

Until then, `ui-tools` should stay small, reusable, and obviously driven by
interface semantics rather than application-specific shells.
