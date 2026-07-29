# Hello UI Input

## Purpose

`hello-ui-input` is an architectural corpus test that validates Tokimu's input
to interaction semantics.

Rather than testing controls themselves, this example isolates how cursor,
keyboard, focus, hover, and capture become meaningful UI interactions.

The goal is to discover the minimal semantic vocabulary required for routing
events through editors, inspectors, dashboards, and future tool surfaces.

## Core Thesis

Input is not a visual primitive.

It is the path from raw events to semantic interaction.

Application

↓

Input State

↓

Focus Routing

↓

UiWorkspaceLayout

↓

UiDrawer

↓

Mesh Commands

↓

Renderer

Events become interaction state.

Interaction state becomes visible feedback.

## What This Example Proves

- Mouse routing can select semantic targets.
- Keyboard navigation can move focus.
- Hover is separate from capture.
- Input state can be reused across multiple regions.
- Rendering consumes routed state rather than owning it.

## Architectural Assertions

### Input is semantic

Applications describe:

- cursor position
- pressed buttons
- pressed keys
- hover target
- focus target
- capture state

Applications do not describe:

- raw GPU commands
- widget-local event loops
- ad hoc input decoding in the renderer

### Routing precedes rendering

The example should show that events first update input state and then drive
focus decisions.

That means the renderer sees the result, not the event stream itself.

### Hover and focus differ

Hover follows the cursor.
Focus follows interaction or keyboard navigation.

Those semantics should remain distinct.

### Capture is explicit

Capture indicates that a target is actively holding input attention.

It should not be conflated with hover or selection.

## Example Content

The example intentionally displays a small set of focusable regions.

Example interactions include:

- cursor hover
- left-click focus
- keyboard cycling
- capture toggle
- focus highlight

## Concepts Under Test

### Mouse

- move
- press
- hover
- capture

### Keyboard

- arrow navigation
- space activation
- focus cycling

### Routing

- hit testing
- target resolution
- active target changes

### Composition

Input driving:

- focus region
- inspector region
- capture strip
- status feedback

## Non-Goals

This example does not attempt to become:

- a full input abstraction layer
- a text editing system
- a shortcut manager
- a keybinding editor

It exists solely to validate semantic input routing.

## Future Growth

Future iterations may explore:

- Tab order
- Gamepad navigation
- Pointer capture edge cases
- Drag-and-drop gestures
- Multi-window routing

These remain intentionally outside the initial corpus test.

## Candidate Input Model

The semantic model should stay intentionally small.

```rust
pub struct UiInputModel {
    pub cursor: [f32; 2],
    pub focus: FocusTarget,
    pub hovered: Option<FocusTarget>,
    pub captured: bool,
}
```

The model answers:

- what is under the cursor?
- what has focus?
- what is captured?

It does not answer:

- how the pixel stream is drawn
- how the event source is implemented
- which GPU surface is active

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
- `hello-ui-inspector`

It is the first example that pressures how events become semantic interactions.

The broader corpus should read as a progression of concerns:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-state  hello-ui-input  hello-ui-inspector  hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which events became state?
- Which targets remained distinct?
- Which focus rules generalized?
- Which capture behavior belonged above the renderer?
- Which input concerns stayed application-owned?

These observations should feed back into:

- ui-tools
- UI Layout
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Input events route into semantic interaction state.
- Focus and hover remain distinct.
- Clicks can select or capture targets.
- Future editors and dashboards can reuse the same routing model.