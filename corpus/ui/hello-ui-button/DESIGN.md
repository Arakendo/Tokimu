# Hello UI Button

## Purpose

`hello-ui-button` is an architectural corpus test that validates Tokimu's
fundamental UI control semantics.

Rather than constructing a complete interface, this example isolates a single
button and explores every concept required to make it behave as a reusable UI
primitive.

The goal is to discover the minimal semantic vocabulary necessary for all
future interactive controls.

## Core Thesis

A button is the UI equivalent of `hello-triangle`.

It should not be flashy.
It should not be a widget showcase.
It should be one control that pressures almost every semantic layer of a UI
system.

The important move is this:

Application

↓

UiButtonSpec

↓

Interaction Resolution

↓

UiDrawer

↓

Mesh + Text Commands

↓

Renderer

The renderer owns pixels.
Tokimu owns interaction.

## What This Example Proves

- Buttons are described through semantic specifications.
- Interaction state is independent of rendering.
- Visual appearance is theme-driven.
- Layout and rendering remain separate concerns.
- Controls consume shared text semantics.
- Controls reuse shared surface semantics.
- Future widgets can compose from the same primitives.

## Architectural Assertions

### Controls describe intent

Applications express:

- action
- label
- enabled state
- interaction state

Applications should never manually construct button geometry.

### Interaction precedes rendering

Hover.
Pressed.
Focused.
Selected.
Disabled.

These states are resolved before rendering occurs.

Renderers simply visualize the resolved state.

### Buttons consume shared semantics

Buttons should reuse:

- `UiTextSpec`
- `UiSurfaceRole`
- `UiInteractionState`
- `UiTheme`

Buttons should not reinvent:

- text alignment
- borders
- spacing
- shadows

### Themes own appearance

Applications describe:

- Primary Button
- Secondary Button
- Danger Button
- Ghost Button

The active theme determines colors, borders, spacing, and elevation.

### Rendering owns pixels

The renderer receives:

- mesh commands
- text commands
- colors

It should never understand:

- "Button"
- "Primary"
- "Hover"

Those concepts belong above the renderer.

## Example Content

The example intentionally contains only buttons.

The purpose is to isolate interaction and visual semantics.

The corpus should demonstrate:

- Idle button
- Hovered button
- Pressed button
- Focused button
- Disabled button
- Selected button

Style variations:

- Primary
- Secondary
- Ghost
- Danger

Sizing:

- Small
- Medium
- Large

Alignment:

- Text centered
- Text left aligned
- Icon placeholder
- Icon + text, future

## Concepts Under Test

This example pressures:

### Surface

- Fill
- Border
- Elevation
- Shadow
- Opacity

### Text

- Alignment
- Padding
- Typography role
- Clipping

### Interaction

- Mouse hover
- Mouse press
- Release
- Keyboard focus, future

### Layout

- Padding
- Minimum size
- Content alignment

### Theme

- Surface roles
- Color roles
- State transitions

## Non-Goals

This example does not attempt to become:

- a UI framework
- a menu system
- a window manager
- a dialog library

It exists solely to validate Tokimu's button semantics.

## Future Growth

Later versions may explore:

- Icons
- Split buttons
- Toggle buttons
- Radio buttons
- Menu buttons
- Animated transitions
- Keyboard shortcuts

These are intentionally excluded from the initial corpus test.

## Relationship To Other Corpus Tests

This example intentionally depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-theme`

Later examples should consume buttons rather than redefining them.

Examples such as:

- `hello-ui-toolbar`
- `hello-ui-card`
- `hello-ui-panel`

should build upon this corpus test rather than introducing new button semantics.

The broader corpus should read as a shared foundation, not a strict chain:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-framework
        │
        ▼
hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which concepts naturally generalized?
- Which concepts remained button-specific?
- Which styling concerns repeated?
- Which interaction states became reusable?
- Which abstractions belonged inside `UiDrawer`?
- Which abstractions belonged inside themes?

These observations should feed back into:

- `ui-tools`
- UI Theme
- UI Drawer
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Buttons are described entirely through `UiButtonSpec`.
- Every visual state derives from semantic state.
- Rendering requires no button-specific logic.
- Themes fully control appearance.
- The same button implementation could naturally be reused inside:
  - toolbars
  - dialogs
  - cards
  - inspectors
  - editors
