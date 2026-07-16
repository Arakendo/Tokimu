# Hello UI Dialog

## Purpose

`hello-ui-dialog` is an architectural corpus test that validates Tokimu's
modality and overlay semantics.

Rather than just adding another window, this example isolates what happens when
the interface becomes temporarily focused on one task.

The goal is to discover the minimal semantic vocabulary required for confirm,
cancel, dismissal, and modal focus handling.

## Core Thesis

A dialog is a modality boundary.

It changes what can be interacted with until the dialog is resolved.

Application

↓

Modal State

↓

Overlay + Focus

↓

UiDrawer

↓

Mesh Commands

↓

Renderer

The background is present, but not primary.

The dialog owns attention.

## What This Example Proves

- Overlays can suppress background interaction.
- Focus can stay inside the dialog until dismissed.
- Confirmation and cancellation are distinct outcomes.
- Dialog state can exist without redefining the rest of the UI.

## Architectural Assertions

### Modality is semantic

Applications describe:

- open / closed
- active button
- dismissal behavior
- overlay state

Applications do not describe:

- generic full-screen windows
- arbitrary chrome
- hidden background event routing

### Overlay is separate from content

The backdrop and dialog content are different semantic layers.

### Focus is constrained

The dialog should keep attention on its own action buttons until dismissed.

### Dismissal is explicit

Escape, confirm, or cancel can close the dialog.

Backdrop clicks do not become background interaction.

## Example Content

The example intentionally displays one centered dialog and a backdrop.

Example interactions include:

- Escape to dismiss
- click to confirm or cancel
- reopen when closed

## Concepts Under Test

### Overlay

- backdrop
- centered panel
- blocked background

### Focus

- active action
- hovered action
- modal capture

### Composition

Dialog containing:

- header
- body
- action buttons
- reopen tile

## Non-Goals

This example does not attempt to become:

- a message box library
- a full modal manager
- a generic popover system
- a windowing toolkit

It exists solely to validate semantic dialog behavior.

## Future Growth

Future iterations may explore:

- stacked dialogs
- validation prompts
- async confirmation flows
- non-blocking overlays
- escape routing policies

These remain intentionally outside the initial corpus test.

## Candidate Dialog Model

The semantic model should stay intentionally small.

```rust
pub struct UiDialogModel {
    pub open: bool,
    pub active_button: usize,
    pub hovered_button: Option<usize>,
}
```

The model answers:

- is the dialog open?
- which action is active?
- which action is hovered?

It does not answer:

- how the backdrop is rendered
- how the GPU clips the overlay
- how background widgets are implemented

## Relationship To The UI Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-theme`
- `hello-ui-button`
- `hello-ui-panel`
- `hello-ui-state`
- `hello-ui-input`

It is the first example that pressures modality as its own seam.

The broader corpus should read as a progression of concerns:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-state  hello-ui-input  hello-ui-dialog  hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which dismissal rules stayed stable?
- Which focus boundaries remained strict?
- Which overlay behavior should stay semantic?
- Which modal concepts belong in ui-tools?
- Which dialog behavior should remain application-owned?

These observations should feed back into:

- ui-tools
- UI Layout
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- The dialog captures attention until dismissed.
- Background interaction does not leak through.
- Confirm and cancel remain distinct actions.
- Future modal UI can reuse the same overlay model.