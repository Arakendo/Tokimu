# Hello UI Toolbar

## Purpose

`hello-ui-toolbar` is an architectural corpus test that validates Tokimu's
command organization semantics.

Rather than constructing complete application interfaces, this example isolates
toolbars as semantic collections of related actions.

The goal is to discover the minimal semantic vocabulary required for command
presentation across editors, CAD applications, game tools, dashboards, and
future Tokimu applications.

## Core Thesis

Buttons answer:

> How do I activate one action?

Toolbars answer:

> How do I organize many related actions?

That is a genuinely different architectural seam.

Toolbars are not just buttons in a row. They are a command organization
primitive.

## Primary Proof

This example demonstrates that command collections can be expressed through
semantic specifications rather than manually arranging buttons.

Application

↓

UiToolbarSpec

↓

Layout Resolution

↓

UiDrawer

↓

Mesh + Text Commands

↓

Renderer

Toolbars organize commands.

Renderers draw pixels.

## What This Example Proves

- Commands are grouped semantically.
- Toolbars compose existing controls.
- Layout is independent of rendering.
- Themes determine appearance.
- Applications express intent rather than geometry.
- Toolbars naturally scale from simple to complex interfaces.

## Architectural Assertions

### Toolbars organize actions.

Toolbars communicate:

- related actions
- active tools
- command groups
- separators
- optional titles

They do not communicate rendering details.

### Commands remain semantic.

Applications describe:

- Select
- Move
- Rotate
- Save
- Open

The toolbar decides presentation.

The renderer remains unaware of command meaning.

### Toolbars compose existing primitives.

Toolbars reuse:

- UiButton
- UiText
- UiSurface
- UiTheme
- UiInteractionState

Toolbars introduce no new rendering primitives.

### Layout belongs to the toolbar.

Toolbars determine:

- orientation
- spacing
- grouping
- alignment
- overflow

Individual buttons remain unaware of neighboring controls.

### Appearance belongs to themes.

Themes determine:

- toolbar surface
- separators
- spacing
- button treatment
- elevation

Applications only describe command intent.

## Example Content

The example intentionally contains only toolbar demonstrations.

Examples include:

- Horizontal toolbar
- Vertical toolbar
- Icon placeholder toolbar
- Text toolbar
- Mixed toolbar
- Toolbar with separators
- Toolbar with active tool
- Toolbar with disabled actions
- Toolbar with overflow, future

## Concepts Under Test

### Command Organization

- Primary commands
- Secondary commands
- Action groups
- Separators

### Layout

- Horizontal flow
- Vertical flow
- Alignment
- Spacing

### Interaction

- Active tool
- Hover
- Pressed
- Disabled

### Theme

- Toolbar surface
- Toolbar spacing
- Button styling

### Composition

Toolbars containing:

- Buttons
- Labels
- Flexible spacing
- Future controls

## Non-Goals

This example does not attempt to become:

- a ribbon interface
- a menu system
- a command palette
- a docking framework

Its purpose is solely to validate toolbar semantics.

## Future Growth

Future iterations may explore:

- Overflow menus
- Search bars
- Tool groups
- Drop-down buttons
- Toggle groups
- Responsive layouts

These remain intentionally outside the initial corpus test.

## Candidate `UiToolbarSpec`

The semantic model should stay intentionally small.

```rust
pub struct UiToolbarSpec {
    pub id: UiRegionId,

    pub orientation: UiOrientation,

    pub role: UiToolbarRole,

    pub items: Vec<UiToolbarItem>,
}
```

Items can remain semantic too:

```rust
pub enum UiToolbarItem {
    Button(UiButtonSpec),
    Separator,
    Spacer,
    Label(UiTextSpec),
}
```

The toolbar does not know about rendering.

It only knows:

- order
- grouping
- orientation
- semantic organization

## What Makes A Toolbar Different?

This is worth documenting because it is not just buttons in a row.

| Button | Toolbar |
| --- | --- |
| Represents one action | Represents a collection of related actions |
| Owns interaction | Owns organization |
| Individual control | Structural command region |
| Standalone | Composes controls |

Likewise, compared to a panel:

| Panel | Toolbar |
| --- | --- |
| Organizes regions | Organizes commands |
| Usually contains arbitrary content | Usually contains actions |
| Structural layout | Command layout |

## Relationship To The UI Corpus

At this point the dependency graph starts looking like an actual language.

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar
        │              │              │              │
        └──────────────┴──────────────┴──────────────┘
                       ▼
                 hello-ui-framework
```

Notice the pattern.

- Text proves communication.
- Button proves interaction.
- Panel proves containment.
- Card proves information grouping.
- Toolbar proves command organization.

None of these examples are really about drawing rectangles. Each one is
isolating a different semantic relationship that appears over and over again in
real applications. That is exactly what makes them good architectural corpus
tests. They are testing concepts, not just controls.

## Lessons To Observe

Implementation should record observations such as:

- Which command groupings remained stable?
- Which spacing concepts repeated?
- Which layout rules generalized?
- Which abstractions naturally belonged inside ui-tools?
- Which concepts became reusable across editors?

These observations should feed back into:

- ui-tools
- UI Layout
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Toolbars are described entirely through `UiToolbarSpec`.
- Command grouping remains semantic.
- Layout remains renderer-independent.
- Themes fully determine appearance.
- The same toolbar implementation naturally supports:
  - editors
  - CAD tools
  - asset browsers
  - debug tools
  - game tooling