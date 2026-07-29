# Hello UI Card

## Purpose

`hello-ui-card` is an architectural corpus test that validates Tokimu's
information grouping semantics.

Rather than constructing complete interfaces, this example isolates cards as
reusable information containers capable of presenting related content with a
clear visual hierarchy.

The goal is to discover the minimal semantic vocabulary required for
information presentation across dashboards, inspectors, browsers, editors,
and future application interfaces.

## Core Thesis

A card is not a prettier panel.

Panels prove containment.
Cards prove information grouping.

The important move is this:

Application

↓

UiCardSpec

↓

UiDrawer

↓

Mesh + Text Commands

↓

Renderer

Cards organize information.
Renderers draw pixels.

## What This Example Proves

- Cards represent semantic information groups.
- Cards compose naturally from existing UI primitives.
- Cards establish visual hierarchy.
- Cards separate presentation from information.
- Cards are reusable across multiple interface styles.
- Themes own appearance.

## Architectural Assertions

### Cards represent related information

Cards communicate:

- a logical grouping
- optional title
- optional content
- optional actions
- optional status

They do not communicate rendering details.

### Cards compose existing primitives

Cards consume:

- `UiPanel`
- `UiText`
- `UiButton`
- `UiTheme`

Cards introduce no new rendering concepts.

### Hierarchy belongs to layout

Cards establish:

- Header
- Body
- Footer, optional

This structure should remain semantic rather than visual.

### Themes own appearance

Examples describe cards through semantic roles such as:

- Information
- Status
- Metric
- Preview
- Inspector

The active theme determines:

- colors
- borders
- elevation
- spacing
- shadows

### Rendering owns pixels

Renderers receive:

- surfaces
- borders
- text
- icons, future

Renderers never understand:

- Metric
- Preview
- Status

These remain application semantics.

## Example Content

The example intentionally displays only cards.

Example variations include:

- Simple card
- Card with title
- Card with subtitle
- Card with body
- Card with footer
- Card with action button
- Metric card
- Status card
- Preview card
- Inspector card

## Concepts Under Test

### Information Hierarchy

- Title
- Subtitle
- Body
- Footer

### Composition

Cards containing:

- Text
- Buttons
- Empty state
- Status indicators

### Surface

- Background
- Border
- Elevation
- Transparency

### Layout

- Internal padding
- Section spacing
- Content alignment

### Theme

- Card roles
- Surface roles
- Typography roles

## Non-Goals

This example does not attempt to become:

- a dashboard framework
- an inspector system
- a property grid
- an analytics UI

It exists solely to validate semantic information grouping.

## Future Growth

Future iterations may explore:

- Image cards
- Expandable cards
- Scrollable cards
- Card grids
- Card animations
- Context menus

These remain intentionally outside the initial corpus test.

## Relationship To The Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-button`
- `hello-ui-panel`
- `hello-ui-theme`

Later examples should consume cards rather than redefining them.

Examples such as:

- `hello-ui-toolbar`
- `hello-ui-framework`

should reuse card concepts where information grouping is the right abstraction, but they are not required to descend from card in a strict chain.

The broader corpus should read as a shared foundation, not a single ladder:

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

- Which card anatomy remained stable?
- Which spacing rules repeated?
- Which information roles naturally emerged?
- Which concepts generalized into `ui-tools`?
- Which abstractions belonged inside `UiDrawer`?

These observations should feed back into:

- `ui-tools`
- UI Theme
- UI Layout
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Cards are described entirely through `UiCardSpec`.
- Card appearance is entirely theme-driven.
- Cards compose existing semantic primitives.
- Information hierarchy remains renderer-independent.
- Future dashboards, inspectors, and editors naturally reuse the same implementation.
