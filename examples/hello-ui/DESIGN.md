# Hello UI - Example Design Document

| Field | Value |
| --- | --- |
| Purpose | Demonstrate an information-dense 2D interface surface built from Tokimu primitives |
| Primary Proof | Tokimu can express dashboard-style layout, selection state, hover state, and visual hierarchy without a dedicated UI toolkit |
| Secondary Proof | Interface data can stay in Tokimu-owned runtime state while the renderer remains a presentation layer |
| Non-Goals | A text engine, accessibility layer, form controls, retained-mode widget toolkit, or production UI system |

## Purpose

`hello-ui` is Tokimu's interface corpus test. It exists to pressure the current
2D rendering and input surfaces with a compact control-board layout so Tokimu can
collect real data about interface composition before a dedicated UI system
exists.

The example should feel like a control surface, dashboard, or editor shell
rather than a game HUD. It is intentionally shape-driven and stateful so the
interface boundary stays visible.

## What It Proves

- Tokimu can compose layered panels, tabs, cards, and status rails from simple primitives
- Interface state can stay in Tokimu-owned runtime data
- Hover, selection, and focus can be expressed with the current input model
- A small example can expose layout pressure without a formal widget toolkit
- Interface feedback can come from geometry, color, and motion before text exists

## Scene Shape

The first version should use a compact control-board layout:

- a header with three selectable tabs
- a left rail of filters or modes
- a main surface with metric cards and a live chart region
- a right-side inspector with toggles and status indicators
- subtle animation so the board feels like a live interface rather than a poster
- a second density mode that compares a roomy board against a vertically stacked compact board

## Interaction Model

The example should support at least these interactions:

- click a tab to change the active view
- click a rail item to change a filter or mode
- click a card to change focus
- press the arrow keys to cycle tabs and filters
- press Space to switch between balanced and dense layouts
- press Space to toggle a detail mode

The UI does not need text rendering for the first pass. The window title can carry
labels while the geometry, color, and motion prove the interface layout.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- the example may react to input and update interface state, but it does not let the renderer own interface meaning
- if Tokimu later grows a real UI layer, it should still feed Tokimu-owned state rather than hiding meaning inside rendering code

## Visual Style

Prefer strong hierarchy, clear spacing, and obvious active/inactive states.
The goal is not a generic polished app shell. The goal is to see what interface
information Tokimu can already express cleanly.

## Architectural Assertions

This example demonstrates:

- Interface state remains inside Tokimu-owned data.
- Panel composition is driven by current runtime state.
- Density mode is a first-class layout decision and changes the board shape.
- Input changes interface focus and selection.
- Rendering consumes layout state without owning it.
- Simple geometry is enough to validate the first interface pass.

## Next Steps After MVP

Once the base control-board works, useful follow-ons include:

1. a simple text layer for labels
2. real focus traversal between widgets
3. pointer hover hints and drag interactions
4. keyboard shortcuts for repeated interface actions
5. comparison with a future retained-mode or immediate-mode UI layer

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
- [Tokimu Contribution Admission Guide](../../docs/contribution-admission-guide.md)
