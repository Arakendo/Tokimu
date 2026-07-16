# Hello UI Text

## Purpose

`hello-ui-text` is an architectural corpus test that validates Tokimu's text
semantics independently from controls, layout systems, and rendering.

Rather than testing buttons or windows, this example isolates text as its own
semantic concern. It establishes how text is described, measured, aligned,
and translated into renderer commands.

The goal is to discover the minimal semantic vocabulary required for text
across every future UI element.

## Core Thesis

Text is a semantic object before it is rendered geometry.

Application

↓

UiTextSpec

↓

Text Layout

↓

Renderer Commands

↓

GPU

The renderer owns glyph rendering.
Tokimu owns text meaning.

## What This Example Proves

- Text is described using semantic specifications.
- Layout occurs before rendering.
- Alignment is deterministic.
- Text measurement is reusable.
- UI elements consume shared text semantics rather than implementing their own.
- Renderer backends remain unaware of UI intent.

## Architectural Assertions

### Text is semantic

UI code should express:

- what text represents
- where it belongs
- how it should align

rather than issuing glyph drawing commands.

### Measurement precedes rendering

Layout depends on measured text.

Renderers consume positioned glyphs rather than performing UI layout.

### Roles own appearance

Examples describe text using semantic roles.

Examples:

- Title
- Heading
- Body
- Caption
- Button
- Status

Themes determine their visual appearance.

### Alignment belongs to layout

Horizontal alignment.
Vertical alignment.
Padding.
Overflow.
Wrapping.

These belong to text layout rather than glyph rendering.

### Renderer ownership stops at glyphs

The renderer knows:

- glyph atlas
- font rasterization
- GPU upload

The renderer does not know:

- buttons
- cards
- toolbars
- titles
- inspectors

## Example Content

The scene intentionally contains only text demonstrations.

Examples include:

- Title text
- Heading text
- Body text
- Button labels
- Status labels
- Left alignment
- Center alignment
- Right alignment
- Baseline alignment
- Vertical centering
- Long text clipping
- Ellipsis
- Multi-line wrapping, if needed

No controls are required.

## Text Contract

The example should naturally reinforce a small semantic model like this:

```rust
pub struct UiTextSpec {
    pub text: String,
    pub rect: UiRect,
    pub role: UiTextRole,
    pub align_x: UiTextAlign,
    pub align_y: UiTextAlign,
    pub overflow: UiTextOverflow,
}
```

Notice what is not in the semantic contract:

- font atlas
- glyph IDs
- UVs
- vertex buffers

Those belong below the semantic boundary.

## Future Growth

Future versions may add:

- Multiple fonts
- Rich text
- Internationalization
- Unicode shaping
- Text direction (LTR / RTL)
- RTL support
- Text selection
- Caret rendering

These are intentionally excluded from the initial corpus test.

## Non-Goals

This example does not attempt to become:

- a text editor
- a font management system
- a document renderer
- a localization framework

It exists only to validate Tokimu's text semantics.

## Lessons To Observe

Implementation should record observations such as:

- Which text concepts remained stable?
- Which concepts were renderer concerns?
- Which concepts naturally belonged inside ui-tools?
- Which concepts became candidates for future UI services?
- Was additional semantic vocabulary required?

These observations should feed back into:

- Primitive Ledger
- UI tools
- Future UI capability design

## Success Criteria

The example succeeds when:

- Every piece of text is described through `UiTextSpec`.
- Buttons, cards, and future controls could consume the same text model.
- Layout decisions occur before renderer submission.
- Renderer APIs remain free of UI semantics.
- Text direction is a semantic switch rather than a font-path concern.
