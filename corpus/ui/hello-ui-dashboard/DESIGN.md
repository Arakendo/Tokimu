# Hello UI Dashboard

## Purpose

`hello-ui-dashboard` is an architectural corpus test that validates Tokimu's
composition semantics.

Rather than introducing a new primitive, this example isolates how layout,
state, cards, and controls come together into a real interface shape.

The goal is to discover the minimal semantic vocabulary required for dashboards,
editor workspaces, and composite tool surfaces.

## Core Thesis

A dashboard is not a new widget family.

It is composition made visible.

Application

↓

Composite Workspace

↓

UiWorkspaceLayout

↓

UiDrawer

↓

Mesh Commands

↓

Renderer

Multiple semantic seams become one workspace.

## What This Example Proves

- Layout, cards, and controls can compose into a workspace.
- Multiple semantic regions can coexist without becoming a shell framework.
- Dashboard state can remain lightweight.
- The same primitives can support editor-like composition.

## Architectural Assertions

### Composition is semantic

Applications describe:

- header
- sidebar
- content
- inspector
- cards
- status

Applications do not describe:

- arbitrary chrome
- hidden widget frameworks
- layout-only implementation details

### Regions keep their meaning

The dashboard reuses the same region vocabulary from layout, state, and
inspector examples.

### Cards provide structure inside the workspace

Cards are used as subregions, not as a separate application shell.

### Rendering remains downstream

The renderer only receives the composed result.

## Example Content

The example intentionally displays a multi-region workspace with cards and a
simple selection state.

Example interactions include:

- Tab cycling
- Card selection
- Keyboard navigation
- Hover emphasis

## Concepts Under Test

### Workspace

- header
- toolbar
- sidebar
- canvas
- inspector
- status bar

### Composition

- cards inside workspace
- button strip inside header
- status feedback

### State

- active tab
- active card
- hovered button

## Non-Goals

This example does not attempt to become:

- a general app shell framework
- a docking system
- a plugin host
- a state management library

It exists solely to validate semantic composition.

## Future Growth

Future iterations may explore:

- Nested dashboards
- Live metrics panels
- Filtered card grids
- Resizable regions
- Multi-pane composition

These remain intentionally outside the initial corpus test.

## Candidate Dashboard Model

The semantic model should stay intentionally small.

```rust
pub struct UiDashboardModel {
    pub active_tab: usize,
    pub active_card: usize,
    pub hovered_button: Option<usize>,
}
```

The model answers:

- which tab is active?
- which card is active?
- which header action is hovered?

It does not answer:

- how the workspace is rasterized
- how state is persisted
- how the renderer batches draw calls

## Relationship To The UI Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-theme`
- `hello-ui-button`
- `hello-ui-panel`
- `hello-ui-card`
- `hello-ui-toolbar`
- `hello-ui-layout`
- `hello-ui-state`
- `hello-ui-input`
- `hello-ui-inspector`

It is the first example that pressures end-to-end composition.

The broader corpus should read as a progression of concerns:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-state  hello-ui-input  hello-ui-inspector  hello-ui-dashboard  hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which composition patterns repeated?
- Which region names stayed stable?
- Which card groupings proved useful?
- Which workspace behaviors belonged above the renderer?
- Which dashboard concerns should remain application-owned?

These observations should feed back into:

- ui-tools
- UI Layout
- UI State
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- The workspace reads as one composed semantic object.
- Layout, cards, and controls remain distinct concerns.
- Selection and hover stay lightweight.
- Future editor dashboards can reuse the same composition pattern.