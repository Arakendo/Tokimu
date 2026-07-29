# ui-framework Design Doc

## Purpose

`ui-framework` is the consumer example for the example-side UI primitives
layer. Its job is to prove that `ui-tools` can support a coherent reusable UI
style without turning into a full GUI toolkit.

The example should look like a small polished workspace shell with a few clear
interaction zones, not a giant panel test or a generic dashboard clone.

## Goals

- Exercise the reusable `ui-tools` layout and hit-test helpers
- Show a few distinct UI element types in one example
- Include readable text in the UI shell and window title
- Demonstrate hover, active, and idle states with clear visual feedback
- Prove that future examples can consume the same primitives without copying
  geometry code
- Stay visually clean and intentionally framed

## Non-Goals

- Full application framework
- Arbitrary widget nesting
- Rich text layout engine
- Scrollable forms or input-heavy business UI
- A generic engine-wide UI contract before the pattern is proven

## What It Should Deliver

This example should demonstrate reusable UI elements in a way that future
examples can follow:

- header area with a compact title
- toolbar with named buttons
- status badge or state chip
- one or two content cards
- hover and active styling rules
- simple text labels in fixed positions

The current implementation should lean on `ui-tools` for the shell framing,
text-bearing chip placement, and card metadata so the consumer example proves
the shared primitives are useful, not just present.

The current live version is intentionally narrower than the fuller shell above:
it is a one-button corpus test. That is useful because it lets the example
focus on button shape, hit target, text centering, padding, border, shadow,
and state feedback before the rest of the shell grows back in.

## Current Folder Structure

```text
ui-framework/
├── Cargo.toml
├── DESIGN.md
└── src/
    └── main.rs
```

## Suggested Internal Structure

Keep the example focused on composition, not framework internals:

- `main.rs` should own the app loop and event handling
- a small local rendering helper section is fine for composition
- all reusable geometry and button logic should come from `ui-tools`
- example-specific colors, labels, and copy should stay local

## Reused Text Primitives

The consumer example should build from shared metadata rather than hardcoded
panel geometry when possible:

- `UiLabel` for title, subtitle, and footer text positions
- `UiCardSpec` for card titles and bodies
- `UiStateChip` for small status pills or badges

## Reusable Element Coverage

`ui-framework` should deliberately exercise several element types:

- toolbar buttons
- status badge or state chip
- content card or preview tile
- header strip with text
- hover and selected styling variants
- layout-driven spacing and framing

## Text Requirements

This example needs text because the reusable UI layer should not only prove
geometry.

Text should be used for:

- the example title
- the active button label
- the hover label
- short status copy inside the shell
- future small captions on cards or tiles

The text itself can stay simple, but the layout must make text feel part of the
UI instead of decoration.

## Success Criteria

The example is doing its job when it answers these questions clearly:

- Can multiple examples consume the same UI helper crate?
- Can the helper crate support buttons, cards, and labels without hardcoding a
  specific app style?
- Can the example remain readable with a small reusable primitive set?
- Does the visual frame feel intentional rather than like placeholder panels?

## Future Deliverables

Likely next steps for the consumer example include:

1. a dedicated label/text helper from `ui-tools`
2. a card row that uses shared layout metadata
3. more explicit hover and focus treatments
4. a small badge or icon system
5. one more example that uses the same primitives in a different visual style

## Boundary Notes

`ui-framework` should stay a consumer and not become the owner of shared UI
meaning. When a helper proves broadly reusable, move the helper into
`ui-tools` rather than copying it here.
