# Hello UI Animation

## Purpose

`hello-ui-animation` is an architectural corpus test that validates Tokimu's
semantic transition behavior.

Rather than testing flashy motion, this example isolates hover, press, expand,
and collapse as meaningful UI transitions.

The goal is to discover the minimal semantic vocabulary required for timed
changes in editors, inspectors, dashboards, and interactive tools.

## Core Thesis

Animation is not decoration first.

It is the timing of a semantic change.

Application

↓

Animation State

↓

Timed Interpolation

↓

UiDrawer

↓

Mesh Commands

↓

Renderer

The state changes.

The renderer sees a transition.

## What This Example Proves

- Hover can have a transition.
- Press can have a transition.
- Expand and collapse can be timed.
- Motion can remain semantic rather than ornamental.
- The renderer only draws the interpolated result.

## Architectural Assertions

### Animation follows semantics

Applications describe:

- open / closed
- hovered / not hovered
- target state
- animation progress

Applications do not describe:

- particles
- camera motion
- unrelated flourish

### Timing is separate from the target state

The target state says where the UI should go.

Animation progress says where it is right now.

### Transitions are visible changes

The example should make it obvious when a panel is growing, shrinking, or
highlighting.

### Rendering owns interpolation output

Renderers receive the result of interpolation, not the policy deciding it.

## Example Content

The example intentionally displays a single transition-heavy card and a toggle
control.

Example interactions include:

- Click to toggle expansion
- Space to toggle expansion
- Hover to emphasize the active region

## Concepts Under Test

### Transition

- progress
- target
- easing, implicit

### Motion

- expand
- collapse
- hover emphasis

### Composition

Animation containing:

- content card
- trigger control
- hover strip

## Non-Goals

This example does not attempt to become:

- an effects library
- a particle engine
- a motion design toolkit
- a timeline editor

It exists solely to validate semantic transitions.

## Future Growth

Future iterations may explore:

- Multiple concurrent transitions
- Delayed transitions
- Shared motion tokens
- Sequence choreography
- Accessibility-friendly reduced motion

These remain intentionally outside the initial corpus test.

## Candidate Animation Model

The semantic model should stay intentionally small.

```rust
pub struct UiAnimationModel {
    pub progress: f32,
    pub target_open: bool,
    pub hovered: bool,
}
```

The model answers:

- what is transitioning?
- where is it headed?
- is it being hovered?

It does not answer:

- how the motion is rendered
- how timing is scheduled globally
- how the GPU draws the result

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

It is the first example that pressures time-based UI transitions.

The broader corpus should read as a progression of concerns:

```text
hello-ui-text
        │
        ▼
hello-ui-theme
        ├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼              ▼
hello-ui-button  hello-ui-panel  hello-ui-card  hello-ui-toolbar  hello-ui-layout  hello-ui-state  hello-ui-input  hello-ui-animation  hello-ui-framework
```

## Lessons To Observe

Implementation should record observations such as:

- Which transitions stayed stable?
- Which values should have been eased?
- Which motion belonged above the renderer?
- Which animation tokens belong inside ui-tools?
- Which transitions should remain application-owned?

These observations should feed back into:

- ui-tools
- UI Layout
- UI Theme
- Primitive Ledger
- Future UI capability design

## Success Criteria

The example succeeds when:

- Hover and expand transitions are visible.
- The target state and animated state remain separate.
- Motion does not become a generic effect system.
- Future interfaces can reuse the same transition model.