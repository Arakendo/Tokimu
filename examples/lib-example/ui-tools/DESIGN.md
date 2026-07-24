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
    ├── lib.rs
    ├── controls.rs
    ├── geometry.rs
    ├── layout.rs
    ├── raster.rs
    ├── svg.rs
    └── text.rs
```

## Suggested Internal Structure

Keep the implementation small and role-based:

- `geometry.rs` for rectangles, anchors, margins, and bounds math
- `controls.rs` for buttons, chips, labels, and interaction state
- `layout.rs` for regions, toolbars, sidebars, cards, and framing helpers
- `text.rs` for baseline-aware text layout and text-box contracts
- `raster.rs` for backend-neutral glyph coverage generation
- `svg.rs` for shared SVG/path geometry services
- future `state.rs` only if examples need shared lightweight interaction state

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
