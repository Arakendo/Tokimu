# Hello UI Icons

## Purpose

`hello-ui-icons` is an architectural corpus test that validates Tokimu's mixed
visual content semantics.

The example should draw its icon vocabulary from the Lucide submodule in
`third-party/glyph-providers/lucide` so icon shape does not become a
Tokimu-specific invention.

Lucide is the reference corpus, not the final icon system. The semantic API
should stay provider-aware and name-based rather than path-based:

```rust
UiIconSpec {
        provider: IconProvider::Lucide,
        name: "folder",
}
```

That leaves room for future providers such as Heroicons, Material, or project
assets without changing the caller's intent.

Rather than adding another control type, this example isolates how icon shapes,
text-adjacent spacing, and selection state combine in a single visual unit.

The goal is to discover the minimal semantic vocabulary required for buttons,
tiles, toolbars, lists, and editor chrome that mixes glyph-like visuals with
labels.

## Core Thesis

Icons are not standalone ornaments.

They are visual tokens that become meaningful when paired with spacing and
labeling.

Application

↓

Icon Tile Model

↓

Spacing + Selection

↓

UiDrawer

↓

Mesh Commands

↓

Renderer

The tile is a composition.

The icon is only one part of it.

## What This Example Proves

- Icons can coexist with text-adjacent spacing.
- Mixed visual content can stay semantic.
- Selection and hover still apply to icon tiles.
- Icon placeholders can be reused across many layouts.

## Architectural Assertions

### Icons are compositional

Applications describe:

- icon shape
- surrounding spacing
- tile role
- selected state

Applications do not describe:

- per-widget rasterization details
- arbitrary ornamentation

### Label adjacency matters

An icon is often only useful when paired with a label or caption region.

The spacing between those pieces is part of the semantic model.

### Selection still matters

The icon tile should still respond to hover and selection.

### Rendering owns pixels

The renderer draws the shapes and spacing.

It does not decide how icon tiles are interpreted.

## Example Content

The example intentionally displays a small grid of icon tiles.

Example arrangements include:

- small icon + caption
- mixed tile emphasis
- selected icon tile
- hover highlight

## Concepts Under Test

### Mixed Content

- icon block
- caption bar
- spacing

### Tile State

- selected tile
- hovered tile
- active emphasis

### Composition

Icon tiles in:

- toolbars
- lists
- dashboards
- inspectors

## Non-Goals

This example does not attempt to become:

- a font/icon pack
- an asset browser
- a glyph atlas system
- a skinning kit

It exists solely to validate semantic mixed-content behavior.

## Future Growth

Future iterations may explore:

- icon sizes
- badge overlays
- icon label alignment
- animated icons
- icon-only controls

These remain intentionally outside the initial corpus test.

## Candidate Icon Model

The semantic model should stay intentionally small.

```rust
pub struct UiIconTileModel {
    pub selected_tile: usize,
    pub hovered_tile: Option<usize>,
}
```

The model answers:

- which tile is selected?
- which tile is hovered?

It does not answer:

- how the icon glyph is produced
- how labels are rasterized
- how the GPU batches the shapes

## Relationship To The UI Corpus

This example depends on the semantic foundations proven by:

- `hello-ui-text`
- `hello-ui-theme`
- `hello-ui-button`
- `hello-ui-panel`
- `hello-ui-card`
- `hello-ui-layout`
- `hello-ui-state`
- `hello-ui-input`

It is the first example that pressures mixed content as an explicit UI seam.

The broader corpus should read as a progression of concerns:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-state  hello-ui-input  hello-ui-icons  hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which provider model stayed stable?
- Which icon-label pairings remained stable?
- Which spacing patterns generalized?
- Which tile states belonged above the renderer?
- Which icon concepts should remain application-owned?
- Which mixed-content rules belong inside ui-tools?

These observations should feed back into:

- ui-tools
- UI Layout
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Icon tiles visibly mix shape and label-adjacent spacing.
- Selection and hover remain distinct.
- The same icon composition can be reused in multiple UI systems.
- Future control surfaces can adopt the same mixed-content pattern.