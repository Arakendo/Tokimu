# Hello UI Panel

## Purpose

`hello-ui-panel` is an architectural corpus test that validates Tokimu's
fundamental region and containment semantics.

Rather than constructing complete application windows, this example isolates
panels as reusable interface regions capable of hosting arbitrary UI content.

The goal is to discover the minimal semantic vocabulary required for
containment throughout the Tokimu UI ecosystem.

## Core Thesis

Panels are where interfaces stop being individual controls and start becoming
structured spaces.

The important move is this:

Application

↓

UiPanelSpec

↓

Layout Resolution

↓

UiDrawer

↓

Mesh + Text Commands

↓

Renderer

Panels own containment.
Renderers own pixels.

## What This Example Proves

- Panels are semantic regions.
- Panels provide containment.
- Panels establish visual hierarchy.
- Panels compose naturally with controls.
- Panels are independent from rendering.
- Future layouts reuse the same panel model.

## Architectural Assertions

### Panels define regions

Panels describe:

- purpose
- bounds
- surface
- contained content

Panels do not describe rendering details.

### Containment is semantic

Controls belong to panels.
Panels belong to layouts.
Layouts belong to applications.

This ownership hierarchy should remain explicit.

### Panels own structure

Panels establish:

- margins
- padding
- content bounds
- optional header
- optional footer

Children consume the resulting layout space.

### Appearance belongs to themes

Panels expose semantic roles such as:

- Workspace
- Sidebar
- Inspector
- Toolbar
- Card
- Overlay

The active theme determines visual appearance.

### Rendering owns geometry

Renderers receive:

- surfaces
- borders
- text
- shadows

Renderers never understand:

- Workspace
- Sidebar
- Inspector

Those remain semantic concepts.

## Example Content

The example intentionally contains only panels.

Example arrangements include:

- Single panel
- Nested panels
- Panel with title
- Panel with footer
- Panel with body
- Transparent panel
- Raised panel
- Flat panel

## Concepts Under Test

### Surface

- Background
- Border
- Elevation
- Transparency
- Theme roles

### Layout

- Margin
- Padding
- Content bounds
- Nested regions

### Hierarchy

- Parent panel
- Child panel
- Region ownership

### Composition

Panels hosting:

- Labels
- Buttons
- Empty content
- Future controls

## Non-Goals

This example does not attempt to become:

- a window manager
- a docking system
- a workspace editor
- a complete application shell

Its purpose is solely to validate semantic panel behavior.

## Future Growth

Future iterations may explore:

- Scrollable panels
- Split panels
- Docking
- Collapsible regions
- Resizable panels
- Floating panels
- Animated transitions

These remain intentionally outside the initial corpus test.

## Relationship To The Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-button`
- `hello-ui-theme`

Later examples should consume panels rather than redefining panel semantics.

Examples such as:

- `hello-ui-toolbar`
- `hello-ui-card`
- `hello-ui-framework`

should reuse panel concepts where containment is the right abstraction, but they are not required to sit beneath panel in a single linear progression.

The broader corpus should read as shared semantic seams:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which containment concepts repeated?
- Which layout rules generalized?
- Which spacing rules became reusable?
- Which panel roles remained stable?
- Which abstractions naturally belonged inside `UiDrawer`?
- Which concepts became candidates for future layout services?

These observations should feed back into:

- `ui-tools`
- UI Theme
- UI Layout
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Panels are described entirely through `UiPanelSpec`.
- Child controls require no panel-specific rendering logic.
- Layout remains independent from rendering.
- Themes fully control appearance.
- Nested panels behave consistently.
- Future examples naturally reuse the same panel implementation.
