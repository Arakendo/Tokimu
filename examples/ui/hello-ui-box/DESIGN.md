# Hello UI Box

## Purpose

`hello-ui-box` is an architectural corpus test that validates Tokimu's most
primitive containment and bounds semantics.

Rather than behaving like a full panel or workspace region, this example isolates
a bare box: a framed area, an inset area, and a label. It asks what remains when
containment is reduced to shape, extent, and nesting.

The goal is to discover the minimal semantic vocabulary required for a generic
boxed region before higher-level layout semantics take over.

## Core Thesis

A box is a boundary before it is a panel.

Application

↓

UiRegion / UiSurfaceRole

↓

UiDrawer

↓

Mesh + Text Commands

↓

Renderer

The box owns bounds.
The renderer owns pixels.

## What This Example Proves

- A box can exist as a primitive framed region.
- Nested boxes still read as semantic containment.
- A box does not need to be a full workspace panel.
- Labels can sit on top of a bare frame without implying a larger shell system.
- Box semantics can remain independent from layout and controls.

## Architectural Assertions

### Boxes define boundaries

Applications describe:

- outer bounds
- inner bounds
- nesting
- labels

Applications do not describe:

- docking behavior
- workspace hierarchy
- control logic

### A box is simpler than a panel

Panels often imply structure, role, and containment rules.

A box is the smallest reusable bounded region that can still carry semantic
meaning.

### Containment is visible

The example should make the nesting relationship obvious without introducing
additional UI systems.

### Appearance remains theme-driven

The box should still use theme-derived surfaces, but the example should stay
focused on the primitive boundary itself.

## Example Content

The example intentionally contains only a few boxed regions.

Example arrangements include:

- Outer box
- Nested inner box
- Accent strip
- Title label
- Caption label

## Concepts Under Test

### Boundary

- Frame
- Extent
- Nesting
- Inset

### Structure

- Outer box
- Inner box
- Label strip
- Label placement

### Composition

Box containing:

- labels
- empty space
- another box

## Non-Goals

This example does not attempt to become:

- a full panel system
- a layout engine
- a docking framework
- a window manager

It exists solely to validate primitive box semantics.

## Future Growth

Future iterations may explore:

- Resizable boxes
- Stacked boxes
- Scrollable boxes
- Box borders with different weights
- Animation between boxed states

These remain intentionally outside the initial corpus test.

## Relationship To The Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-theme`
- `hello-ui-panel`

It sits near panel semantics, but intentionally keeps the claim smaller: box
before panel, frame before structure.

The broader corpus should read as shared semantic seams:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼
hello-ui-box      hello-ui-panel  hello-ui-card  hello-ui-layout
```

## Lessons To Observe

Implementation should record observations such as:

- Which boundary rules stayed obvious?
- Which parts of panel semantics were premature?
- Which box concepts belong in ui-tools?
- Which box concepts should remain application-owned?
- Did the example remain smaller than `hello-ui-panel` while still proving a
  useful seam?

These observations should feed back into:

- ui-tools
- UI Layout
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- A box reads as the smallest useful framed region.
- Nested boxes remain understandable.
- The example does not collapse into panel semantics.
- Future UI examples can reuse the same primitive framing idea.
