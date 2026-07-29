# Hello UI State

## Purpose

`hello-ui-state` is an architectural corpus test that validates Tokimu's
application state semantics.

Rather than adding more widgets, this example isolates how selection, hover,
binding, invalidation, and redraw connect multiple UI regions.

The goal is to discover the minimal semantic vocabulary required for stateful
interfaces such as editors, inspectors, dashboards, and future tool surfaces.

## Core Thesis

State is not a visual primitive.

It is the connective tissue between input and multiple dependent regions.

Application

↓

UiStateModel

↓

Bindings

↓

UiWorkspaceLayout

↓

UiDrawer

↓

Mesh + Text Commands

↓

Renderer

Input mutates state.

State invalidates views.

Views redraw from the new semantic truth.

## What This Example Proves

- Selection can drive multiple dependent regions.
- Hover is distinct from selection.
- State changes invalidate derived UI.
- Redraw is a consequence of state changes, not a separate semantic layer.
- Host-side diagnostics can reflect state without becoming the state model.

## Architectural Assertions

### State is semantic

Applications describe:

- selected asset
- hovered asset
- pinned state
- revision counter

Applications do not describe:

- pixel deltas
- repaint instructions
- ad hoc imperative redraws

### Bindings connect meaning

When one state value changes, dependent regions may change too:

- inspector updates
- toolbar changes
- status changes
- selection highlight changes

That coupling is semantic, not incidental.

### Invalidation is intentional

The example should show that derived views can be rebuilt only when the
underlying state changes.

That means state owns the trigger; rendering only consumes the result.

### Redraw is a consequence

Renderers receive the updated surfaces and text commands after state changes.

They do not decide which state is authoritative.

## Example Content

The example intentionally displays the same layout while state changes.

Selected asset changes should affect:

- inspector emphasis
- toolbar state
- status bar role
- selection highlight
- host window title

The underlying layout remains stable.

## Concepts Under Test

### Selection

- active asset
- selected region
- pinned focus

### Observation

- hovered asset
- hover-driven hints
- state-dependent feedback

### Invalidation

- cached layout rebuild
- revision bump
- derived view refresh

### Composition

- Inspector updates from selected asset
- Toolbar changes from pinned state
- Status changes from revision state

## Non-Goals

This example does not attempt to become:

- a full reactive framework
- a store library
- a data-binding system
- a change-tracking engine

It exists solely to validate semantic state propagation.

## Future Growth

Future iterations may explore:

- Multi-selection
- Undo / redo
- Shared state across windows
- Derived selectors
- Background synchronization

These remain intentionally outside the initial corpus test.

## Candidate State Model

The semantic model should remain small.

```rust
pub struct UiStateModel {
    pub selected_asset: usize,
    pub hovered_asset: Option<usize>,
    pub pinned: bool,
    pub revision: u64,
}
```

The model answers:

- what is selected?
- what is hovered?
- what is pinned?
- what changed?

It does not answer:

- how the pixels are drawn
- which GPU command gets issued
- which widget implementation owns the click

## Relationship To The UI Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-theme`
- `hello-ui-button`
- `hello-ui-panel`
- `hello-ui-card`
- `hello-ui-toolbar`
- `hello-ui-layout`
- `hello-ui-inspector`

It is the first example that pressures how the system reacts when state changes
instead of when controls are merely clicked.

The broader corpus should read as a progression of concerns:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-state  hello-ui-inspector  hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which state values were actually shared?
- Which views revalidated together?
- Which updates stayed local?
- Which state lived above the renderer?
- Which state should remain application-owned?

These observations should feed back into:

- ui-tools
- UI Layout
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- One state change visibly updates multiple UI regions.
- Cached views are invalidated only when state changes.
- Input and state remain separate from rendering.
- Future inspectors and dashboards can reuse the same state model.