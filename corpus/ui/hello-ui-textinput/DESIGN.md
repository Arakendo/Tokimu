# Hello UI Text Input

## Purpose

`hello-ui-textinput` is an architectural corpus test for editable single-line
text. It builds on `hello-ui-text` and `hello-ui-input` without becoming a
text editor.

The example proves that a semantic text field can keep its value, caret,
selection, focus, and editing events separate from rendering and application
meaning.

## Core Thesis

Editable text is a stateful control, not merely a text label.

```text
Application value
        |
        v
Text input model
  value / caret / selection
        |
        v
Keyboard and pointer editing
        |
        v
UiTextSpec + caret presentation
        |
        v
Renderer
```

The application owns the committed value. The text-input model owns transient
editing state. The renderer owns glyphs and pixels.

## What This Example Proves

- A field can receive focus through the shared focus model.
- Pointer placement can establish a caret position.
- Printable input inserts text at the caret.
- Backspace and delete mutate the editing value predictably.
- Left and right movement preserve caret bounds.
- Shift-selection is represented separately from the committed value.
- Enter emits a semantic submit event instead of directly invoking a callback.
- Focused, hovered, disabled, and selected states are visual states only.
- The caret is presentation derived from the editing model.

## Initial Model

The first model should remain single-line and Unicode-agnostic at the editing
boundary while the text renderer is still bitmap-based:

```text
TextInputState
- value
- caret offset
- optional selection anchor
- focused
- enabled
```

Offsets must be bounded and deterministic. The implementation should document
whether offsets are bytes, scalar values, or grapheme clusters. It should not
silently mix those units.

## Input Contract

The example translates platform text and keyboard events into semantic editing operations:

```text
Insert(text)
MoveCaret(Left | Right)
DeleteBackward
DeleteForward
SelectAll
Submit
Focus
Blur
```

The model applies operations. The application handles emitted semantic events.
No closures or renderer callbacks belong in the model.

## Visual Corpus

The window should show the same field through a small state matrix:

- unfocused and empty;
- focused with caret;
- focused with selected text;
- populated value;
- hovered field;
- disabled field;
- submitted value/status.

The value should be shown both inside the field and in a separate status line
so committed value and transient editing state are easy to compare.

## Non-Goals

The first corpus test does not implement:

- multiline editing;
- IME or composition events;
- Unicode grapheme segmentation;
- rich text;
- undo/redo;
- clipboard integration;
- password masking;
- autocomplete;
- platform accessibility adapters.

Those are future capabilities, not reasons to make the first field opaque.

## Success Criteria

The example succeeds when:

- the field can be focused without application-side rectangle scanning;
- editing operations are deterministic and unit-testable;
- selection and caret state survive redraws without becoming application state;
- Enter produces a semantic submit event;
- the same text measurement path used by labels is used by the field;
- the renderer receives text and caret presentation commands, not editing logic.

## Open Questions

- Should the shared model use grapheme clusters before Unicode support lands?
- Does pointer-to-caret mapping belong in text measurement or the input control?
- How should horizontal scrolling keep the caret visible?
- Which selection semantics should be shared before clipboard support exists?
- Does text input need a dedicated capability crate or remain in `ui-tools`?
