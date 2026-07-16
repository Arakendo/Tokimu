# Tokimu UI Design Document

| Field        | Value |
| ------------ | ----- |
| Status       | Draft — design direction |
| Version      | 0.1.0 |
| Last updated | 2026-07-16 |
| Scope        | Semantic UI presentation, measurement, layout, and widget composition |
| Relates to   | SDD, Architectural Maxims, `docs/Conversations/on-ui.md`, UI corpus examples |
| Source discussion | [`docs/Conversations/on-ui.md`](Conversations/on-ui.md) |

## 1. Purpose

Tokimu UI should let an application express semantic intent while the UI system
handles the spatial details needed to present that intent. An application should
be able to say:

```text
ui.button("Compile")
```

without first deciding the button's width, height, font, padding, alignment,
border, or corner radius.

Those details still exist. They belong to theme, measurement, layout, and
rendering rather than to the application's semantic declaration.

This document records the design direction for that system. It is not yet a
stable public API specification. Concrete examples and implementation pressure
must shape the final traits and data types.

## 2. Design Philosophy

Tokimu UI exists to minimize application decisions while maximizing semantic
clarity.

Applications should describe:

- what exists;
- what it means;
- what actions are possible.

The presentation system owns:

- measurement;
- layout;
- visual construction;
- theme resolution;
- presentation interaction and hit testing.

The renderer owns pixels. The application owns truth. Every abstraction should
reduce repetition without hiding ownership.

This is why intrinsic sizing, nested composition, and deterministic defaults
matter more than a large initial widget catalog. A useful presentation system
removes accidental decisions while leaving semantic decisions visible.

## 3. Central Direction

### 2.1 Defaults express intent; configuration expresses exceptions

The common path should require fewer decisions. A widget's default behavior
should produce a useful result without explicit geometry:

```text
ui.button("Save")
```

Applications may override the result when they have a real spatial requirement:

```text
ui.button("Save").size(SizePolicy::Fill)
ui.button("Save").size(SizePolicy::Fixed(Vec2::new(120.0, 32.0)))
```

Fixed dimensions are valid, but they should not be the vocabulary required for
every control.

### 2.2 Intrinsic sizing is the default

A widget should be able to determine the space it naturally needs. A button's
intrinsic width, for example, is derived from its content and theme:

```text
text measurement
    + horizontal padding
    + border and other decoration
    = desired button width
```

The parent remains responsible for deciding how that desired size fits into the
available space. Intrinsic sizing does not mean that a child can ignore its
parent's constraints.

### 2.3 Useful is cheap; irreducible is expensive

A layout abstraction should become foundational only when the examples show
that the meaning cannot be expressed cleanly by composition. Early work should
prove a small number of concrete layout behaviors before introducing a general
layout language.

## 4. Semantic Ownership

The UI system owns presentation meaning and spatial arrangement. It does not own
simulation truth.

The intended ownership direction is:

```text
Application meaning
  |
  v
Semantic presentation
  |
  v
Theme and visual defaults
  |
  v
Renderer
  |
  v
Platform
```

- Applications and higher-level engines own semantic state and intent.
- UI widgets observe state and emit semantic interaction results.
- Layout owns measurement, constraints, allocation, and final rectangles.
- Themes own visual defaults such as typography, spacing, colors, and surfaces.
- The renderer owns pixels, GPU resources, clipping, and draw execution.
- Platform adapters own window, pointer, keyboard, and display integration.

A UI element may cache layout results for efficiency, but cached geometry is a
representation. It must not become a second source of application or simulation
state.

### 4.1 Ownership matrix

| Concern | Owner |
| ------- | ----- |
| Application state | Application |
| Simulation state | Application / engine runtime |
| Text meaning | UI |
| Text measurement | UI / text capability |
| Layout and constraints | UI |
| Theme defaults | Theme |
| GPU buffers and draw execution | Renderer |
| Windows and displays | Platform |
| Mouse and keyboard source events | Platform |
| Hit testing and event routing | UI |
| Visual drawing commands | Renderer |
| Assets and font data | Asset or font capabilities |

The matrix is an ownership guide, not permission for every owner to become a
new global service. When a concern crosses boundaries, the boundary should be
represented through an explicit input, output, or capability.

## 5. Presentation Tree

The application describes semantic presentation elements. The UI system builds
or observes a presentation tree, then processes that tree through the canonical
phases:

```text
Application meaning
  |
  v
Presentation tree
  |
  v
Measure
  |
  v
Layout
  |
  v
Draw
```

The presentation tree may be retained, rebuilt immediately, or represented by
a hybrid structure. That implementation choice remains open. The enduring
contract is that semantic composition exists before measurement and that the
resulting layout can be inspected independently of drawing.

## 6. Canonical Presentation Pipeline

The intended pipeline is:

```text
Semantic elements
        |
        v
Measure: determine desired size under constraints
        |
        v
Layout: assign final rectangles and resolve relationships
        |
        v
Draw: emit rendering commands for resolved geometry
```

Measurement should not draw. Drawing should not guess geometry. Layout should
not require the renderer to discover how large content wants to be.

A possible future shape is:

```text
const PREVIOUS_ASSET: UiActionId = UiActionId(1)
const TOGGLE_PINNED: UiActionId = UiActionId(2)
const NEXT_ASSET: UiActionId = UiActionId(3)

UiButtonSpec::new(UiButtonId(0), "PREV").with_action(PREVIOUS_ASSET)
UiButtonSpec::new(UiButtonId(1), "PIN").with_action(TOGGLE_PINNED)
UiButtonSpec::new(UiButtonId(2), "NEXT").with_action(NEXT_ASSET)
```
result.

### 4.1 Measure context

Measurement needs an explicit description of the conditions under which a
widget is being measured. The context is the input to the measure phase, not a
global lookup assembled by drawing code.

A possible shape is:

```rust
pub struct MeasureContext {
  pub theme: Theme,
  pub available_space: Size,
  pub dpi_scale: f32,
  pub font_provider: FontProvider,
  pub constraints: Constraints,
}
```

This is illustrative rather than a proposed public API. The eventual context
may use references, separate capability handles, or a smaller set of inputs.
The important contract is that measurement receives enough explicit context to
remain deterministic and testable, including available space, scaling,
typography, theme defaults, and constraints.

The first example-side proof of this boundary lives in `ui-tools` as
`UiMeasureContext`. It is intentionally small and does not yet establish the
final engine capability API.

The example-side flow corpus now includes both `UiHorizontalStack` and
`UiVerticalStack`. Both use `UiMeasurable`, intrinsic measurement, parent
constraints, configurable gaps, and deterministic compression when the
available axis is too small. A single configurable direction type remains a
follow-up concern.

Both stacks now also expose cross-axis `Start`, `Center`, `End`, and `Fill`
alignment, with `Center` preserved as the default. In the centered world
coordinate system, horizontal-stack start/end map to top/bottom and
vertical-stack start/end map to left/right.

Both stacks also support explicit main-axis `Intrinsic` or `Fill` allocation.
Fill distributes remaining space evenly among children while preserving the
configured gaps; intrinsic allocation remains the default.

### 4.2 Layout result

Layout should produce an explicit result that can be inspected, used for hit
testing, and passed to drawing. A possible shape is:

```rust
pub struct LayoutResult {
  pub rect: Rect,
  pub children: Vec<LayoutResult>,
}
```

The final representation may separate the tree from widget state or add
identity, clipping, baseline, overflow, and diagnostics metadata. The
architectural requirement is that final geometry is a visible product of the
layout phase rather than an incidental mutable detail discovered during draw.

Layout results are presentation data. They may be cached and reused, but they
must not become application or simulation state.

`ui-tools` currently provides `UiLayoutResult` as a data-only proof of this
tree-shaped output, plus a deliberately narrow `UiHorizontalStack` proof for
measurable children. The stack measures its children, applies a parent
rectangle, and emits ordered nested child rectangles. `UiMeasurable` is a
measurement-only example contract implemented by buttons and cards; it does
not claim ownership of drawing, identity, or interaction. This is an
example-side validation of recursive allocation, not yet the final engine
capability API.

## 7. Desired Size and Constraints

Every widget should have a meaningful answer to "how much space would you like"
when given the relevant measurement context. A parent then applies available
space and policy.

```text
Child desired size: 84 x 32
Parent available space: 300 x 40
Final child size: 84 x 32
```

When space is limited:

```text
Child desired size: 84 x 32
Parent available space: 60 x 32
Final child size: constrained according to policy
```

The exact constraint model is intentionally open. It must make these cases
explicit:

- unconstrained measurement;
- maximum width or height;
- minimum width or height;
- available fill space;
- content that may wrap or overflow;
- a child whose intrinsic size exceeds its parent;
- a parent whose available space is smaller than its minimum viable size.

Silent overflow and silent fallback should not be the default. Diagnostics or a
visible, deterministic overflow policy should make surprising constraints
understandable.

## 8. Layout Hints

The likely semantic vocabulary is a small size policy rather than arbitrary
rectangles on every widget:

```rust
enum SizePolicy {
    Intrinsic,
    Fill,
    Fixed(Vec2),
    MinContent,
    MaxContent,
}
```

`ui-tools` now provides an example-side `UiSizePolicy` with `Intrinsic`, `Fill`,
`Fixed`, `Min`, and `Max` variants. The names and final set remain provisional;
they must be validated against the layout corpus before becoming foundational
engine API. A policy should describe intent, not leak a particular layout
algorithm into every caller.

Likely defaults:

- controls such as buttons, labels, and icons: `Intrinsic`;
- containers such as panels and workspace regions: a container-specific fill or
  allocation policy;
- application-authored fixed geometry: explicit `Fixed` as an exception;
- content with an application-defined minimum: a minimum constraint combined
  with intrinsic or fill behavior.

A single policy is unlikely to express every relationship. Alignment, gaps,
wrapping, flex allocation, grids, and split resizing should remain separate
layout concerns rather than being hidden inside `SizePolicy`.

## 9. Nested Composition

Nesting controls is a primary requirement of the layout model, not an advanced
feature. A control must be able to contain other controls, and a parent must be
able to size itself from the measured results of its children.

For example:

```text
Card
  -> vertical stack
       -> Text("Build settings")
       -> horizontal stack
      -> Button("Compile")
      -> Button("Cancel")
```

The layout relationship is recursive:

```text
Parent constraints
  |
  v
Measure children
  |
  v
Add padding, gaps, and decoration
  |
  v
Compute parent's desired size
  |
  v
Allocate final child rectangles
```

This should allow a nested control to fit itself automatically while still
allowing an ancestor to constrain, fill, align, wrap, or override it. A card
should not need to know the button's fixed dimensions, and a button should not
need to know the card's eventual rectangle.

The model must support both directions of composition:

- a parent sizing itself around intrinsically sized children;
- a parent providing available space that children use according to their
  policies.

Nested measurement must also remain stable under repeated passes. A child must
not require its parent to draw first in order to discover its size, and drawing
must not mutate the measurements that produced the layout.

## 10. Text Is a Foundational Capability

Text measurement is the first concrete dependency of intrinsic layout. A text
capability must be able to answer questions such as:

```text
measure("Compile", text style, available width) -> desired size
```

The semantic text request should name a role or family, not a font file path.
The existing UI direction distinguishes text role, font provider, font family,
and text direction. Layout should consume the resulting measurement without
knowing where glyph data came from.

Text measurement must eventually account for:

- font family and fallback;
- text role and scale;
- LTR and RTL direction;
- wrapping width;
- line height;
- shaping and glyph clusters;
- truncation or overflow policy.

The example-side bitmap text capability now implements `UiTextOverflow::Wrap`
for whitespace-separated text, explicit newlines, and words that exceed the
available line width. The implementation is intentionally a simple corpus
slice. It also supports basic `Ltr` and `Rtl` visual ordering and resolves
`Start` and `End` alignment relative to direction. It does not yet provide
Unicode bidi segmentation, shaping, or font fallback.

This makes `hello-ui-text` a foundational corpus slice for layout rather than a
standalone rendering demo. Buttons, panels, cards, toolbars, and inspectors will
all depend on the same measurement contract.

## 11. Presentation Construction

The UI API should communicate semantic widgets while the lower-level
construction layer provides consistent visual composition and styling.

`UiDrawer` is a possible implementation name for that lower-level layer, not a
semantic boundary that applications should need to understand. The enduring
concept is presentation construction: turning semantic widgets and resolved
layout into a consistent set of visual primitives.

A future drawer may offer operations such as:

```text
drawer.button(spec)
drawer.panel(spec)
drawer.card(spec)
drawer.toolbar(spec)
```

Those operations can consistently emit primitives in an implementation-defined
order, for example:

```text
surface
    -> shadow
    -> border
    -> icon and text
```

This does not mean the construction layer should own application state or
replace the layout system. It is a presentation construction kit. The layout phase gives it
resolved rectangles; the drawer turns semantic visual specs into renderer
commands.

The boundary should stay clear:

- widgets declare meaning and behavior;
- layout resolves space;
- the drawer resolves themed visual construction;
- the renderer executes primitives.

## 12. Boring Interaction and Actions

The common interaction path should be deliberately unsurprising. An
application should be able to declare a button, give it a stable action
identifier, let the presentation capability perform hit testing, and handle a
semantic event in its normal application update code.

The core presentation data should not store closures or function pointers.
Callbacks capture application state, make otherwise simple widgets harder to
copy or cache, and blur ownership between presentation and the application.
The application owns the action meaning and the state mutation; presentation
owns recognition of the interaction and emission of the event.

The intended shape is illustrative:

```rust
enum UiAction {
  PreviousAsset,
  TogglePinned,
  NextAsset,
}

UiButtonSpec::new("PREV", UiAction::PreviousAsset)
UiButtonSpec::new("PIN", UiAction::TogglePinned)
UiButtonSpec::new("NEXT", UiAction::NextAsset)
```

The exact action representation may be an application-owned enum, a stable
action ID, or another typed command. The important rule is that it is semantic
and stable. Callers should not dispatch behavior by array index or by looking
at button text.

Input processing should follow this path:

```text
Platform input
  |
  v
Presentation hit testing and focus routing
  |
  v
UiEvent::Activated(action)
  |
  v
Application update and state mutation
  |
  v
Measure -> Layout -> Draw when the presentation is affected
```

The resolved presentation should expose semantic hit testing at its boundary,
for example `layout.event_at(pointer, enabled)`, and focused activation through
the same event path. Applications should not need to scan child rectangles or
reconstruct button ordering to obtain an action. The presentation resolves
geometry and interaction routing; the application still decides what the
resulting action means and how state changes. Enabled state belongs to the
resolved control, not only to the input caller, so a disabled control cannot
emit an action through either pointer or focused activation.

The current Tokimu platform input vocabulary exposes `Space` for keyboard
activation. `Enter` remains part of the presentation activation contract so
platform adapters can add it without changing application dispatch code.

At minimum, activation should work consistently for pointer click, keyboard
activation, and an explicitly focused control. Hover, pressed, focused,
selected, and disabled states are visual and interaction state; they must not
silently mutate application state. A disabled control must not emit its action.

The boring path should require only these decisions from an application:

1. Declare a stable action for the control.
2. Build the semantic presentation from current application state.
3. Match the emitted action in the application update path.
4. Mutate application state and allow presentation invalidation to follow.

Convenience adapters may let an application write callback-like code at a
higher layer, but callbacks should remain an adapter over semantic events, not
the ownership model of the foundational presentation capability.

## 13. Spacing and Theme Defaults

Repeated numeric spacing values should not spread through application code.
Spacing belongs to the theme or a semantic design scale:

```text
Spacing::Xs
Spacing::Small
Spacing::Medium
Spacing::Large
Spacing::Xl
```

The final representation may instead be a theme method or token table. The
important contract is that padding, gaps, control heights, and surface rhythm
come from shared semantic defaults.

Theme defaults should cover at least:

- text roles and font families;
- spacing tokens;
- control padding;
- minimum interactive sizes;
- colors and visual states;
- border and corner treatment;
- focus, hover, pressed, disabled, and selected states.

A widget may override a theme value, but the common path should not require
repeating it.

## 14. Relationship to UI State and Input

Layout is related to, but distinct from, application state and input routing.
The existing UI corpus should keep these pressures visible:

- `hello-ui-layout` proves semantic regions become spatial arrangements;
- `hello-ui-state` proves state changes can invalidate and redraw dependent UI;
- `hello-ui-input` proves pointer, focus, keyboard, capture, and tab routing;
- `hello-ui-text` proves measurement, roles, fallback, and direction;
- `hello-ui-icons` proves mixed icon, text, and spacing composition.

A state change may require a new measure or layout pass. An input event may use
resolved geometry for hit testing. Neither should make widget geometry the source
of application truth.

## 15. Presentation Invalidation

Presentation work should be recomputed only when required. The invalidation
scope depends on which input changed:

| Change | Minimum affected work |
| ------ | --------------------- |
| Text content or text style | Measure, then layout and draw as needed |
| Parent available size | Layout, then draw |
| Position or arrangement | Layout, then draw |
| Color or visual state | Draw |
| Theme spacing or typography | Measure, layout, and draw as needed |
| Theme color only | Draw |
| Application state | Depends on semantic impact |
| Platform scale or display metrics | Measure, layout, and draw as needed |

This table describes a target behavior, not an excuse to introduce hidden
incremental state prematurely. A simple full recomputation is acceptable while
examples are proving correctness. Optimization should preserve the same
observable results and make invalidation boundaries diagnosable.

## 16. Presentation Diagnostics

The presentation system should expose enough information to explain why a
control has the size and position it has. At minimum, diagnostics should be
able to report:

- desired size;
- available size;
- allocated and final size;
- parent and child relationships;
- clipping;
- overflow and truncation policy;
- measure time;
- layout time;
- draw time;
- invalidation reason.

A debug view might show:

```text
Button("Compile")
Desired:   84 x 32
Allocated: 72 x 32
Overflow:  ellipsis
Reason:    parent width constraint
```

Diagnostics are especially important for nested controls. A contributor should
be able to inspect the path from a child's desired size to the ancestor that
constrained it instead of inferring layout behavior from pixels alone.

The current UI corpus exposes clipped button text as a structured warning via
`UiButton::diagnostics(theme)` and `UiWorkspaceLayout::diagnostics(theme)`:

```text
UiDiagnostic {
  severity: Warning,
  kind: TextClipped { control, label },
}
```

The same diagnostic collection also catches duplicate control identities and
duplicate action identities:

```text
DuplicateControlId(id)
DuplicateActionId(action)
ZeroSizeControl(id)
FocusableWithoutAction(id)
MissingControlLabel(id)
UnsupportedTextOverflow { role }
```

These warnings catch ambiguous hit testing, focus restoration, and action
dispatch before those ambiguities become user-visible behavior. A zero-size
control warning catches a resolved control that cannot be rendered, hit-tested,
or focused. The semantic completeness warnings catch controls that are enabled
but cannot produce an action, or that have no accessible label. Text specs also
warn when `Ellipsis` is requested but the active text implementation cannot
provide it. Other useful future diagnostics include unconstrained growth and
children placed outside an active interactive clip.

Diagnostic collection is deterministic and does not print or mutate
application state. The application or host decides whether a warning appears
in a developer console, diagnostics panel, test failure, or another user-facing
channel. This keeps the foundational UI layer observable without forcing a
logging policy on every application.

## 17. Determinism

Given identical semantic content, theme, font inputs, constraints, and scale,
the presentation system should produce identical semantic layout results.

Platform differences may affect renderer output, available display metrics, or
font capability availability, but they should not silently change layout meaning.
Native and WASM implementations should therefore share the same layout rules
where their explicit inputs are equivalent.

Determinism includes:

- stable child ordering;
- stable size allocation;
- explicit rounding and scale behavior;
- explicit fallback for unavailable fonts or assets;
- no dependence on draw order to discover measurement;
- no hidden time or global state in measurement and layout.

## 18. Capability Boundaries

Presentation is a capability of the engine, not a second engine hidden inside
the renderer.

### Presentation capability owns

- semantic text and controls;
- text measurement contracts;
- layout and constraints;
- themes and spacing defaults;
- presentation regions;
- focus, hit testing, and interaction routing;
- presentation diagnostics and invalidation semantics.

### Presentation capability does not own

- renderer implementation or GPU resources;
- windowing or display management;
- simulation state or ECS storage;
- application assets as a persistence system;
- a required native platform;
- a particular renderer;
- a particular layout algorithm.

These boundaries preserve the engine's world-first architecture. Presentation
communicates state and intent; it does not become a competing owner of them.

## 19. Service Layer Direction

The foundational presentation capability should remain small. More specialized
services can build above it:

```text
Kernel presentation capability
  |
  v
Advanced UI service
  |
  +--> Docking
  +--> Inspector
  +--> Property editing
  +--> Editor workspace
```

The kernel should provide the semantic and spatial primitives that these
services require. Docking, inspectors, editor workflows, and domain-specific
panels should not force their assumptions into the foundational capability
before repeated examples prove them universal.

## 20. Corpus Relationship

The UI examples are architectural corpus tests. Each example should pressure a
specific boundary and make the next abstraction easier to evaluate:

```text
hello-ui-text
  |
  v
hello-ui-button
  |
  v
hello-ui-layout
  |
  v
hello-ui-state / hello-ui-input
  |
  v
hello-ui-framework
```

- `hello-ui-text` pressures measurement, roles, fallback, and direction.
- `hello-ui-button` pressures intrinsic sizing and semantic actions.
- `hello-ui-layout` pressures nesting, constraints, spacing, and regions.
- `hello-ui-state` pressures invalidation and state-dependent presentation.
- `hello-ui-input` pressures hit testing, focus, capture, and routing.
- `hello-ui-icons` pressures mixed icon, text, and spacing composition.
- future framework examples should pressure reusable presentation construction
  without hiding ownership.

The corpus is evidence for the capability. It is not a mandate to implement a
complete toolkit before the underlying contracts are understood.

## 21. Layout Corpus Direction

The next layout work should be driven by small examples, not by a general
framework implementation. A useful progression is:

1. Intrinsic text and button sizing.
2. Vertical and horizontal stacks with spacing.
3. Nested cards, stacks, and controls that fit themselves automatically.
4. Parent constraints and fill behavior.
5. Wrapping text and bounded content.
6. Workspace regions with sidebar, content, inspector, and status bar.
7. Resizable splits and grids.
8. State-driven invalidation of measured and laid-out content.

Each example should answer a semantic question and expose enough diagnostics to
show desired size, available size, final rectangle, and overflow behavior.

## 22. Things We Explicitly Refuse To Do

The presentation capability will not:

- become CSS or a complete Flexbox clone;
- hide ownership behind implicit global state;
- store simulation state;
- require geometry for every widget;
- depend on a specific renderer;
- require a windowing system;
- force one layout algorithm;
- become a universal constraint solver;
- use animation to conceal layout errors;
- turn renderer-specific geometry into application semantics;
- provide every widget before foundational behavior is proven;
- stabilize a public trait before concrete examples exercise it.

## 23. Open Design Questions

These questions should be resolved by examples and implementation pressure:

1. Is the first implementation retained, immediate, or hybrid?
2. Does measurement return a value, a layout node, or both desired size and
   intrinsic metadata?
3. How are minimum, maximum, and fill constraints represented without creating
   an overly general solver?
4. Where do wrapping, truncation, and overflow diagnostics live?
5. Does `UiDrawer` receive semantic specs, resolved widgets, or only primitives?
6. How are stable identity and state preservation represented across rebuilds?
7. Which spacing tokens are truly foundational rather than theme preference?
8. What layout behavior must remain deterministic across native and WASM?
9. How should accessibility, keyboard navigation, and minimum hit targets affect
   measured size?
10. Which layout concepts belong in a capability crate versus a higher-level UI
    package?

## 24. Initial Acceptance Criteria

The first implementation should be considered useful when all of the following
are true:

- `ui.button("Compile")` produces a correctly measured control without an
  explicit rectangle;
- changing the text changes the desired width through the shared text measure
  path;
- a card or panel can contain stacks, text, and controls whose sizes are
  resolved recursively without fixed child rectangles;
- nested intrinsic controls fit their content while remaining constrainable by
  an ancestor;
- a parent can constrain or fill a child deterministically;
- layout and draw can be inspected independently;
- nested widgets receive stable, non-overlapping final rectangles;
- theme spacing and control defaults are applied consistently;
- input hit testing uses resolved layout geometry;
- state changes invalidate only the presentation work that must change;
- the example remains expressible without embedding simulation state in the UI;
- native behavior has a clear path to WASM parity.

## 25. Design Maxims

- Semantic before visual.
- Measure before layout.
- Layout before draw.
- Defaults express intent.
- Configuration expresses exceptions.
- Applications own truth.
- Presentation owns communication.
- Useful is cheap; irreducible is expensive.
- Nested controls fit themselves, subject to explicit parent constraints.
- Diagnostics should explain behavior, not merely report failure.

## 26. Working Mantra

> **Simple things should be simple. Complex things should remain possible.**
>
> **Defaults should express intent. Configuration should express exceptions.**

Tokimu UI should reduce decisions without removing control. The system earns its
abstractions by making future buttons, checkboxes, property grids, tree views,
and docking panels simpler because they inherit sound measurement, layout,
spacing, theme, state, and input foundations.

## 27. Architectural Maxim

> **Presentation communicates application state. Presentation does not become
> application state.**

This applies to widgets, layout results, visual transitions, hit-test geometry,
and renderer commands. They are representations and observations of application
meaning, not a second owner of that meaning.

## 28. Capability Lifecycle

Presentation is a capability that participates in the application lifecycle.
It is not only a draw helper and it is not an independent update loop. The
normal cycle is:

```text
Application Update
  |
  v
Presentation Build
  |
  v
Measure
  |
  v
Layout
  |
  v
Draw
  |
  v
Input and Interaction Events
  |
  v
Application Update
```

Application state is the source of meaning. Presentation is rebuilt or updated
from that meaning, measured under explicit conditions, laid out into inspectable
geometry, and drawn through renderer-facing commands. Input uses the resolved
presentation result to produce semantic events that return to application
update. A simple implementation may recompute the full presentation each
cycle; incremental work is an optimization of this contract, not a different
ownership model.

Initialization and shutdown are part of the lifecycle as well. Presentation
must be able to report unavailable theme, text, renderer, or platform
capabilities explicitly, and it must release presentation-owned caches and
subscriptions without owning the application or simulation lifetime.

## 29. Stable Identity

Presentation identity exists to preserve presentation interaction state across
rebuilds. It does not own application state.

Stable identity enables:

- focus preservation;
- hover and pressed-state continuity;
- selection and capture continuity;
- animation continuity;
- cached measurement or layout reuse;
- diagnostics that can name a presentation element.

Identity should be semantic and caller-stable, not derived from a transient
array index, draw order, pointer address, or rectangle. A repeated list item
may need an application-provided key. If an identity disappears, its
presentation state may be discarded; application data must remain unaffected.

The identity contract should remain small. It must not become an alternate
state store or require every widget to expose a framework-specific lifecycle.
When identity is unavailable, the system should have a deterministic fallback
with reduced state preservation rather than silently associating state with a
different element.

## 30. Accessibility Philosophy

Accessibility should emerge from semantic presentation, not be added as a
second rendering layer. A control that declares its role, label, value,
enabled state, action, focus behavior, and relationships gives accessibility
adapters meaningful information before any pixels are produced.

The presentation capability should therefore preserve semantic information
through build, identity, layout, and interaction. Visual styling is not the
accessibility contract. Renderer output alone must not be the only way to
discover that something is a button, heading, status, list, or editable value.

The first implementation need not solve every platform accessibility API, but
it should avoid APIs that make semantic access impossible later. Keyboard
activation, focus order, minimum hit targets, readable labels, and explicit
disabled state belong in the interaction contract rather than being renderer
afterthoughts.

Focus traversal should be deterministic and based on resolved presentation
order, while the focused value itself remains a stable semantic identity. The
traversal must skip disabled or non-actionable controls and wrap predictably at
the ends. Applications may choose when to request traversal, but should not
need to reconstruct the presentation's control order.

## 31. Coordinate Systems and Scaling

Presentation crosses explicit coordinate-system boundaries:

```text
World or Application Space
      |
      v
Viewport and Display Space
      |
      v
Presentation Space
      |
      v
Local Widget Space
```

Transforms should occur at named boundaries. A widget should not need to know
whether its final pixels are produced by a native window, a browser canvas, or
an off-screen target. Hit testing must apply the inverse of the same explicit
transforms used for drawing.

DPI and display scaling belong to the measure/layout inputs and renderer or
platform conversion boundaries. They must not be hidden global factors. Given
the same semantic content, theme, constraints, scale, and font inputs,
measurement and layout should remain deterministic. Rounding should be
explicit at the boundary where continuous presentation geometry becomes pixel
geometry.

## 32. Clipping and Child Visibility

Clipping is a layout and presentation relationship, not an accidental renderer
side effect. A parent establishes a child visibility region:

```text
Parent Layout Result
  |
  v
Clip Region
  |
  v
Child Layout Results
  |
  v
Draw Commands
```

Child geometry may remain useful for hit testing, diagnostics, and scrolling
even when part of it is outside the current clip region. Drawing must apply the
resolved clip deliberately, and hit testing must define whether clipped
children are interactive. The default should be that content outside an
interactive ancestor's clip is not hit-testable unless an explicit overflow
policy says otherwise.

Clipping is foundational for scrolling, inspectors, trees, editors, and
virtualized content. It should therefore be represented in layout or
presentation results rather than reconstructed independently by every renderer
adapter.

The current corpus slice provides `UiRect::intersection` and an optional
`UiDrawer` clip. Surfaces and text emitted while that clip is active are
intersected before becoming draw commands; fully hidden content emits no
command. This is an implementation step toward clip regions carried by
presentation results, and does not yet define scrolling or overflow behavior.

The first shared scrolling primitive is `UiVerticalScroll`. It owns viewport
extent, content extent, a clamped positive downward offset, content-to-viewport
translation, visible-rectangle intersection, and scroll-aware hit testing. It
does not own input devices, animation policy, scrollbar rendering, nested
scroll routing, or virtualization. Those concerns remain application or
higher-level presentation adapters until real callers justify them.

`hello-ui-scroll` now uses this primitive for offset clamping, smooth target
motion, item visibility, hover hit testing, and scrollbar position. The example
still owns its keyboard mapping and visual scrollbar policy, which keeps the
shared model focused on scroll geometry rather than platform input.

## 33. Presentation Model and Widget Model

The widget model and presentation model are related but not identical:

```text
Widget or Semantic Element
      |
      v
Presentation Node
      |
      v
Layout Node or Layout Result
      |
      v
Draw Commands
```

Widgets express meaning, actions, and semantic relationships. Presentation
nodes carry the data needed to measure, identify, arrange, and describe those
widgets. Layout results carry resolved geometry and child relationships. Draw
commands are renderer-facing output.

These layers may be represented by one compact structure in an early
implementation, but their ownership contracts should remain distinct. Keeping
the distinction visible allows Tokimu to choose a retained, immediate, or
hybrid implementation later without making application semantics depend on
renderer commands or layout cache representation.

## 34. Engine Integration

Presentation is a peer capability in the engine architecture:

```text
Simulation and Application State
    |
    v
Presentation Capability
    |
    v
Renderer Capability
    |
    v
Platform Capability
```

The application and simulation provide meaning. Presentation observes that
meaning, resolves semantic interaction and geometry, and emits renderer-facing
commands. The renderer executes visual work. The platform provides windows,
display metrics, and source input. No lower layer becomes the owner of a
higher layer's truth merely because it processes its output.

This boundary must hold for native and WASM paths. Presentation should depend
on explicit capabilities and data rather than requiring a particular windowing
system, GPU backend, or persistence service. A renderer adapter may reject an
unsupported presentation feature with an explicit diagnostic; it should not
silently redefine semantic layout.

## 35. Bootstrapping and Degraded Presentation

Presentation should remain capable of presenting engine diagnostics before
higher-level presentation services are available. Bootstrapping must not
require the advanced editor, docking service, asset browser, or application
theme to be healthy before a useful failure can be shown.

The foundational capability should provide a deliberately small degraded path
for messages such as:

```text
Presentation Service Failed

Reason:
Missing renderer capability.

[Diagnostics]    [Retry]
```

The degraded path may use simpler typography, fixed fallback geometry, and a
minimal action set. It must still preserve ownership: diagnostics come from the
engine or application, presentation displays them, and actions return as
semantic events. Bootstrapping is a resilience boundary, not permission to
make the presentation capability own service recovery or application state.

## 36. Capability Maxim

> **Every capability should reduce future work.**

Tokimu presentation earns its place by making buttons, panels, text, layout,
stateful interaction, and future controls easier to build without hiding the
decisions that matter. New abstractions should be judged by the concrete work
they remove from the next example and the number of unrelated assumptions they
avoid imposing on the engine.
