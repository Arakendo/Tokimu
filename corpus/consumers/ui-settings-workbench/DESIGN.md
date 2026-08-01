# UI Settings Workbench

## Purpose

`ui-settings-workbench` is an application-shaped consumer corpus for Tokimu's
provider-neutral UI contracts. It verifies that an ordinary native consumer can
compose a settings editor without recreating layout, hit testing, focus, text
input routing, disabled-state, or presentation lowering.

This is not another isolated widget demonstration. It is pressure from a
consumer that owns mutable application state and uses several UI capabilities
together.

## Primary Composition Claim

```text
Application-owned settings model
        |
        v
Semantic UiTree
        |
        +--> layout and diagnostics
        +--> pointer, focus, and text routing
        +--> renderer-neutral draw list
        |
        v
Native renderer adapter
```

At no point does:

- `ui-tools` own project settings or validation policy;
- the application maintain separate hit-test rectangles;
- disabled actions remain pointer or keyboard targets;
- the renderer infer control meaning.

## Capability Pressure

- editable project and author fields;
- activatable quality and diagnostics controls;
- dirty-state-driven Apply and Reset availability;
- pointer and keyboard focus through the same resolved identities;
- responsive wide and compact arrangements;
- deterministic headless resolution and draw-list lowering.

## Controls

- Up/Down: move semantic focus.
- Left/Right: move the caret while editing.
- Enter/Space: activate the focused action.
- Mouse: focus, edit, and activate resolved controls.
- Text input, Backspace, and Delete: edit the focused field.

## Acceptance Criteria

- Wide and compact scenes resolve without layout diagnostics.
- All interactive behavior routes through resolved `UiNodeId` values.
- A clean model exposes disabled Apply and Reset actions.
- Text input mutates only the focused editable field.
- Applying settings updates the saved snapshot and returns to a clean state.
- Repeated lowering produces the same structural draw-list fingerprint.
