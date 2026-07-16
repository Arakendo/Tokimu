# Hello UI Inspector

## Purpose

`hello-ui-inspector` is an architectural corpus test that validates Tokimu's
property editing semantics.

Rather than showing another window of controls, this example isolates how a
selected object becomes a set of editable properties with context-sensitive
layout.

The goal is to discover the minimal semantic vocabulary required for editors,
debug tools, CAD tools, and future authoring surfaces.

## Core Thesis

An inspector is not just a panel with labels.

It is a property system made visible.

Application

↓

UiInspectorModel

↓

Property Schema

↓

UiWorkspaceLayout

↓

UiDrawer

↓

Mesh Commands

↓

Renderer

Selection determines which properties exist.

Properties determine which edits matter.

## What This Example Proves

- A selected object can expose a schema of editable properties.
- Property groups can change based on object kind.
- Inspector fields are semantic, not just visual rows.
- Editing state can remain separate from rendering.
- Context-sensitive layouts can reuse the same inspector region.

## Architectural Assertions

### Inspectors expose properties

Applications describe:

- selected object
- property group
- editable field
- dirty state

Applications do not describe:

- pixel positions
- widget-local form logic
- ad hoc label rendering

### Schema follows selection

Different object kinds should surface different property groups.

That means the inspector is driven by the semantic object model, not by a fixed
list of controls.

### Editing is stateful

The currently active property matters.

Changing a field should be visible in the inspector state, the host title, and
the selected object summary.

### Layout belongs to the inspector

The inspector decides:

- property grouping
- field ordering
- spacing
- emphasis

The renderer only draws the result.

## Example Content

The example intentionally displays a small set of objects and their property
groups.

Example interactions include:

- object selection
- field selection
- keyboard cycling
- dirty-state toggling

## Concepts Under Test

### Selection

- selected object
- selected field
- hovered object
- hovered field

### Schema

- object kind
- property list
- context-sensitive fields

### Editing

- dirty flag
- active field
- property focus

### Composition

Inspector containing:

- object summary
- field rows
- property groups
- status feedback

## Non-Goals

This example does not attempt to become:

- a full form framework
- a serialized schema engine
- a database editor
- a property grid library

It exists solely to validate semantic inspector behavior.

## Future Growth

Future iterations may explore:

- Nested property groups
- Expandable sections
- Validation errors
- Undo / redo
- Multi-object inspection

These remain intentionally outside the initial corpus test.

## Candidate Inspector Model

The semantic model should stay intentionally small.

```rust
pub struct UiInspectorModel {
    pub selected_object: usize,
    pub selected_field: InspectorField,
    pub hovered_object: Option<usize>,
    pub hovered_field: Option<InspectorField>,
    pub dirty: bool,
}
```

The model answers:

- what object is selected?
- what field is active?
- what changed?

It does not answer:

- how the properties are drawn
- what the underlying storage backend is
- which renderer implementation is active

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

It is the first example that pressures property schema and editing flow as a
distinct architectural seam.

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

- Which properties belonged to the object kind?
- Which fields stayed stable across selections?
- Which edits should have been tracked as state?
- Which schema rules belonged inside ui-tools?
- Which inspector behavior should remain application-owned?

These observations should feed back into:

- ui-tools
- UI Layout
- UI State
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Object kind changes the property schema.
- Selecting a field changes visible inspector state.
- Rendering remains separate from property semantics.
- Future editors can reuse the same inspector model.