# Hello UI Theme

## Purpose

`hello-ui-theme` is an architectural corpus test that validates Tokimu's
separation between interface semantics and visual appearance.

Rather than testing controls or layout, this example isolates theming as an
independent concern capable of restyling identical UI semantics without
modifying application logic.

The goal is to discover the minimal semantic vocabulary required for reusable,
consistent interface appearance.

## Core Thesis

Appearance is not semantics.

Applications own meaning.

Themes own appearance.

That boundary matters because it keeps interface concepts stable when the visual
language changes.

## Primary Proof

This example demonstrates that UI appearance is entirely derived from semantic
roles rather than embedded styling decisions.

Application

↓

Semantic UI

↓

Theme Resolution

↓

UiDrawer

↓

Mesh + Text Commands

↓

Renderer

Applications own meaning.

Themes own appearance.

## What This Example Proves

- Semantic controls are theme-independent.
- Appearance derives entirely from themes.
- Controls never hardcode colors or spacing.
- Multiple themes can render identical interfaces.
- Theme changes require no application changes.

## Architectural Assertions

### Semantics precede appearance.

Applications describe:

- Button
- Card
- Toolbar
- Panel
- Status

Applications never describe:

- RGB colors
- Border thickness
- Shadow opacity
- Corner radius

### Themes own visual identity.

Themes determine:

- Color palette
- Surface hierarchy
- Typography
- Spacing
- Borders
- Shadows
- Elevation
- Transparency

Application semantics remain unchanged.

### Controls consume theme roles.

Controls describe semantic intent such as:

- Primary Button
- Danger Button
- Inspector Panel
- Toolbar Surface
- Status Card

Themes resolve those roles into concrete visuals.

### Renderer owns pixels.

The renderer receives:

- colors
- geometry
- text
- transparency

It never understands:

- Primary Button
- Warning Card
- Toolbar

Those remain semantic concepts.

## Example Content

The example intentionally displays semantic surface roles as swatches so the
theme vocabulary can be inspected directly.

Example theme families include:

- Default Dark
- Default Light
- High Contrast
- Blueprint
- Wireframe

Future themes may include:

- Retro
- Terminal
- Minimal
- Game HUD

The interface itself remains identical.

## Concepts Under Test

### Surface Roles

- Background
- Region
- Panel
- Card
- Toolbar
- Overlay
- Selected

### Color Roles

- Primary
- Secondary
- Success
- Warning
- Danger
- Muted
- Text
- Accent

### Typography Roles

- Title
- Heading
- Body
- Caption
- Button
- Status

### Spacing

- XS
- Small
- Medium
- Large
- XL

### Elevation

- Flat
- Contained
- Raised
- Floating

### Borders

- None
- Thin
- Medium
- Heavy

## Non-Goals

This example does not attempt to become:

- a visual editor
- a skinning framework
- a CSS replacement
- an animation system

Its purpose is solely to validate semantic theming.

## Future Growth

Future iterations may explore:

- Runtime theme switching
- Theme inheritance
- User customization
- Animated transitions
- Accessibility themes

These remain intentionally outside the initial corpus test.

## Candidate `UiTheme`

The semantic language becomes easier to reason about when the theme is grouped
by concern.

```rust
pub struct UiTheme {
    pub surfaces: UiSurfaceTheme,
    pub text: UiTextTheme,
    pub spacing: UiSpacingTheme,
    pub borders: UiBorderTheme,
    pub elevation: UiElevationTheme,
    pub radius: UiRadiusTheme,
}
```

Each section should stay semantic rather than numeric.

For example:

```rust
pub struct UiSurfaceTheme {
    pub background: UiSurfaceStyle,
    pub region: UiSurfaceStyle,
    pub panel: UiSurfaceStyle,
    pub card: UiSurfaceStyle,
    pub toolbar: UiSurfaceStyle,
    pub overlay: UiSurfaceStyle,
    pub selected: UiSurfaceStyle,
}
```

The application says:

```text
SurfaceRole::Panel
```

The theme says:

```text
Panel =
    dark gray
    1px border
    medium shadow
```

Tomorrow another theme might say:

```text
Panel =
    parchment texture
    brown border
    no shadow
```

Nothing else changes.

## Relationship To The UI Corpus

Theme sits underneath almost everything.

```text
hello-ui-text
        │
        ▼
hello-ui-theme
    ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
    ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-framework
```

Why?

Because almost every UI primitive consumes a theme, but none of them should
define one.

## Lessons To Observe

Implementation should record observations such as:

- Which visual roles remained stable?
- Which concepts naturally became theme tokens?
- Which styling concerns repeated?
- Which abstractions belonged inside UiTheme?
- Which concepts should remain application semantics?

These observations should feed back into:

- ui-tools
- UI Theme
- UI Drawer
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- The identical interface renders under multiple themes.
- No application code changes when switching themes.
- Controls contain no hardcoded visual styling.
- Appearance is entirely resolved through semantic theme roles.