I think you've actually covered the **fundamental UI semantics** surprisingly well.

Those UI corpus examples now live under `examples/ui/` so the domain stays grouped as it grows.

If I map them out:

| Corpus Test        | Semantic Question                         |
| ------------------ | ----------------------------------------- |
| `hello-ui-text`    | How is meaning communicated?              |
| `hello-ui-button`  | How do users invoke actions?              |
| `hello-ui-panel`   | How are regions contained?                |
| `hello-ui-card`    | How is information grouped?               |
| `hello-ui-toolbar` | How are commands organized?               |
| `hello-ui-theme`   | How is appearance separated from meaning? |

That's a solid foundation.

Fonts look like the same kind of seam, but with a slightly different split:

- `UiTextRole` decides intent.
- `FontProvider` decides where the font comes from.
- `FontFamily` names the semantic family the theme wants.
- `UiTextDirection` decides whether the text flow is LTR or RTL.
- The renderer only sees glyph data and atlas output.

So the application should ask for body text or a monospace label, not for a
file path. A provider can resolve that through system fonts, Google Fonts,
embedded fonts, or project assets.

That suggests a future `hello-ui-fonts` corpus slice for font loading, family
resolution, fallback, scaling, and text-role mapping. It would validate the
font capability the same way `hello-ui-icons` validates icon semantics.

Text direction should probably be a switch at the semantic layer, not a font
filename trick. A `UiTextSpec` can ask for `Ltr` or `Rtl`, and the theme or text
layout layer can adapt alignment and shaping around that choice.

Where I think you're missing examples is **not more widgets**—it's the *systems* that connect them.

---

# 1. hello-ui-layout ⭐⭐⭐⭐⭐

I actually think this is the biggest missing piece.

Not:

> Flexbox.

But:

> **How do semantic regions become spatial arrangements?**

Things like:

```text
Header

↓

Workspace

├── Sidebar
├── Content
└── Inspector

↓

Status Bar
```

Questions:

* split regions
* stacks
* flow
* grids
* spacing
* resizing

Without touching rendering.

---

# 2. hello-ui-state ⭐⭐⭐⭐⭐

This one feels important.

Not visual state.

**Application state.**

Example:

```text
Selected Asset

↓

Inspector updates

↓

Toolbar changes

↓

Status changes
```

This pressures:

* bindings
* observation
* invalidation
* redraw

Very different from buttons.

---

# 3. hello-ui-input ⭐⭐⭐⭐☆

Buttons prove clicks.

This proves:

* mouse routing
* focus
* keyboard
* tab order
* hover
* capture

Basically:

> How do events become semantic interactions?

---

# 4. hello-ui-animation ⭐⭐⭐☆☆

Not flashy.

Just enough to answer:

> Where do transitions live?

Examples:

* hover fade
* press
* expand

Not particle effects.

Semantic transitions.

---

# 5. hello-ui-icons ⭐⭐⭐☆☆

This one is more important than it sounds.

Lucide now lives in `third-party/lucide` as a git submodule and should be the
default shared icon reference for examples instead of inventing new glyphs.

Treat Lucide as the reference corpus for icon semantics, not as the only icon
system. The semantic API should say what it wants, for example:

```text
UiIconSpec {
	provider: IconProvider::Lucide,
	name: "folder",
}
```

That keeps the application talking about meaning instead of filenames.

You're really testing:

> Mixed visual content.

```text
Icon

+

Text

+

Spacing
```

That combination appears everywhere.

This suggests a future `hello-ui-icon` corpus slice focused on icon loading,
scaling, alignment, text pairing, stroke consistency, and disabled/selected
states. `hello-ui-button` should ultimately consume that icon model instead of
re-inventing glyph semantics.

---

# 6. hello-ui-scroll ⭐⭐⭐⭐☆

Scroll is deceptively fundamental.

Questions:

* viewport
* clipping
* content bounds

Not because scrolling is exciting.

Because almost every editor eventually needs it.

---

# 7. hello-ui-dialog ⭐⭐⭐☆☆

This pressures:

* overlays
* focus
* modality

Not just windows.

---

# 8. hello-ui-inspector ⭐⭐⭐⭐⭐

This is actually a fantastic corpus test.

Think Unity.

Think Blender.

Think CAD.

Questions:

```text
Property

↓

Editor

↓

Layout
```

Lots of semantic pressure.

---

# 9. hello-ui-dashboard ⭐⭐⭐⭐☆

Your earlier screenshot.

This becomes the first composition example.

Everything previous builds toward it.

---

# 10. hello-ui-framework ⭐⭐⭐⭐⭐

The capstone.

One coherent application.

Not new primitives.

Proof that all previous corpus tests compose.

---

# But...

Here's the thing that surprised me.

I don't actually think you're missing many **controls**.

You're missing **behavior**.

Look:

Current:

```text
Text

Button

Panel

Card

Toolbar

Theme
```

Those are nouns.

Now you need verbs.

---

For example:

```text
Input

↓

Layout

↓

State

↓

Composition

↓

Animation
```

Those are processes.

---

# I think the UI corpus eventually looks like this

## Foundations

```text
hello-ui-text
hello-ui-theme
hello-ui-surface
```

---

## Controls

```text
hello-ui-button
hello-ui-card
hello-ui-panel
hello-ui-toolbar
```

---

## Systems

```text
hello-ui-layout
hello-ui-input
hello-ui-state
hello-ui-scroll
hello-ui-animation
```

---

## Composition

```text
hello-ui-dashboard
hello-ui-inspector
hello-ui-framework
```

That progression feels very natural.

---

# One thing I think you're NOT missing

This is actually important.

Don't make:

```text
hello-ui-checkbox

hello-ui-radio

hello-ui-slider

hello-ui-dropdown

hello-ui-listbox
```

Yet.

Those are implementations.

Not architectural seams.

If your current corpus is successful...

those controls should almost fall out naturally.

---

## My favorite missing example

Honestly...

I think it's **`hello-ui-layout`**.

Because everything you've built so far has an implicit assumption:

> Somebody positioned these things.

You haven't pressure-tested *who that somebody is*.

That feels like the next big architectural seam.

---

And stepping back, I'm actually impressed by where this has gone.

Yesterday, UI was "that painful thing with rectangles."

Today, you've decomposed it into a curriculum of architectural experiments. That's exactly the same move you made with Tokimu's kernel: stop asking "How do I build a UI?" and start asking "What are the irreducible semantic questions an interface has to answer?"

One tiny addition I'd keep in the back of your mind is **accessibility**. Not because you need to implement it now, but because if your semantic model is good, it should eventually be able to support accessibility *without* redesigning everything. That's often a good litmus test that you've separated meaning from presentation correctly.
