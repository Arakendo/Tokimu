# Hello UI Scroll

## Purpose

`hello-ui-scroll` is an architectural corpus test that validates Tokimu's
viewport and content-bound semantics.

Rather than testing another control type, this example isolates what happens
when content becomes larger than the visible region.

The goal is to discover the minimal semantic vocabulary required for editors,
inspectors, dashboards, and any view that needs to clip and move content.

## Core Thesis

Scroll is not movement for its own sake.

It is the relationship between a viewport and content that exceeds it.

Application

↓

Viewport + Content Bounds

↓

Scroll State

↓

UiDrawer

↓

Mesh Commands

↓

Renderer

Content is larger than the frame.

Scroll reveals a portion of it.

## What This Example Proves

- Viewports can clip content cleanly.
- Content bounds can move without changing content meaning.
- Selection can remain stable while the viewport moves.
- Scroll state is separate from layout state.
- Large lists can be rendered as semantic stacks.

## Architectural Assertions

### Viewport is semantic

Applications describe:

- visible frame
- content extent
- scroll offset
- selected item

Applications do not describe:

- ad hoc clipping math in the renderer
- per-item camera ownership

### Content can exceed the frame

The example should show that the same cards exist whether or not they are
currently visible.

### Scroll state is intentional

Arrow keys and clicks should change the visible portion of the content without
changing the underlying item meaning.

### Rendering owns clipping only

Renderers receive clipped draw commands.

They do not decide what is visible.

## Example Content

The example intentionally displays a tall list of cards inside a clipped frame.

Example interactions include:

- ArrowUp / ArrowDown scrolling
- Left / Right item selection
- Mouse hover over visible items

## Concepts Under Test

### Viewport

- frame bounds
- visible area
- clipping

### Content Bounds

- tall list
- item stack
- selected item

### Scroll

- offset
- target offset
- smooth movement

## Non-Goals

This example does not attempt to become:

- a full listbox widget
- a virtualization engine
- a scrollbar library
- a file browser

It exists solely to validate semantic scroll behavior.

## Future Growth

Future iterations may explore:

- Nested viewports
- Horizontal scroll
- Virtualized lists
- Touch scrolling
- Scrollbar dragging

These remain intentionally outside the initial corpus test.

## Candidate Scroll Model

The semantic model should stay intentionally small.

```rust
pub struct UiScrollModel {
    pub scroll_offset: f32,
    pub target_offset: f32,
    pub selected_index: usize,
    pub hovered_index: Option<usize>,
}
```

The model answers:

- where is the viewport?
- which item is selected?
- which item is hovered?

It does not answer:

- how clipping is implemented
- how the GPU draws the frame
- how input is sourced

## Relationship To The UI Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-theme`
- `hello-ui-button`
- `hello-ui-panel`
- `hello-ui-card`
- `hello-ui-layout`
- `hello-ui-state`

It is the first example that pressures viewport clipping and moving content.

The broader corpus should read as a progression of concerns:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-state  hello-ui-scroll  hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which bounds stayed stable?
- Which content could move independently?
- Which clip region rules generalized?
- Which scroll behavior belonged above the renderer?
- Which viewport concepts should stay shared?

These observations should feed back into:

- ui-tools
- UI Layout
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Content is visibly clipped by a viewport.
- Scrolling changes visibility without changing content identity.
- Selection remains distinct from scroll position.
- Future editors can reuse the same viewport model.