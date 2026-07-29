# Hello UI Layout

## Purpose

`hello-ui-layout` is an architectural corpus test that validates Tokimu's
spatial arrangement semantics.

Rather than testing controls or appearance, this example isolates how semantic
regions become layout decisions.

The goal is to discover the minimal semantic vocabulary required for headers,
workspaces, sidebars, inspectors, card grids, and status bars.

## Core Thesis

Somebody positions these things.

This example asks who that somebody is.

Application

↓

UiWorkspaceLayout

↓

UiDrawer

↓

Mesh + Text Commands

↓

Renderer

Layout organizes regions.

Renderers draw pixels.

## What This Example Proves

- Semantic regions can become spatial arrangements.
- Split regions remain semantic rather than hardcoded geometry.
- Stacks, flows, and grids belong to layout, not rendering.
- Spacing can be expressed without touching renderer code.
- Resizing can reflow regions while preserving meaning.

## Architectural Assertions

### Layout precedes rendering

Applications describe:

- header
- workspace
- sidebar
- content
- inspector
- status bar

Applications do not describe:

- pixel coordinates
- ad hoc margin math
- per-control positioning

### Regions remain semantic

Layouts organize:

- splits
- stacks
- rows
- columns
- card grids

Regions retain their meaning even when the viewport changes.

### Spacing belongs to layout

Layout determines:

- gutters
- padding
- alignment
- resizing
- overflow boundaries

Controls should not know where their neighbors are placed.

### Themes own appearance

Layout emits semantic regions that themes turn into visual surfaces.

The layout system does not own:

- color
- borders
- shadow style
- typography style

## Example Content

The example intentionally displays only layout regions.

Example arrangements include:

- Header over workspace
- Sidebar / content / inspector split
- Card grid below the main area
- Status bar at the bottom
- Responsive resizing

## Concepts Under Test

### Regions

- Header
- Workspace
- Sidebar
- Content
- Inspector
- Status Bar

### Structure

- Split regions
- Stacks
- Flow
- Grids

### Spacing

- Gutters
- Padding
- Alignment
- Resizing

### Composition

Layout containing:

- Buttons
- Cards
- Labels
- Future controls

## Non-Goals

This example does not attempt to become:

- flexbox
- a general constraint solver
- a docking framework
- a window manager

Its purpose is solely to validate semantic layout behavior.

## Future Growth

Future iterations may explore:

- Nested split panes
- Collapsible regions
- Resizable dividers
- Scroll-aware layout
- Layout transitions

These remain intentionally outside the initial corpus test.

## Candidate Layout Vocabulary

The semantic model can stay small if it names the actual arrangements:

```rust
pub struct UiLayoutSpec {
    pub workspace: UiRegion,
    pub header: UiRegion,
    pub sidebar: UiRegion,
    pub content: UiRegion,
    pub inspector: UiRegion,
    pub status_bar: UiRegion,
}
```

The model does not need to know about pixels first.

It needs to know:

- which regions exist
- how they relate
- what they contain
- how they reflow when size changes

## Relationship To The UI Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-theme`
- `hello-ui-button`
- `hello-ui-panel`
- `hello-ui-card`
- `hello-ui-toolbar`

It is the first example that pressures how those pieces are arranged.

The broader corpus should read as a progression of concerns:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout
        │              │              │              │              │
        └──────────────┴──────────────┴──────────────┴──────────────┘
                               ▼
                      hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which split arrangements repeated?
- Which region names stayed stable?
- Which spacing values generalized?
- Which layout rules belonged in ui-tools?
- Which decisions stayed above the renderer?

These observations should feed back into:

- ui-tools
- UI Theme
- UI Drawer
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- The same region vocabulary reflows cleanly under resize.
- Layout remains separate from rendering.
- Theme changes do not alter region semantics.
- Future dashboards and inspectors can reuse the same spatial model.