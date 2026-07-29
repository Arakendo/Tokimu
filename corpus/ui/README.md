# UI Corpus

The UI corpus pressures Tokimu's foundational presentation contracts. Its
entries are organized conceptually below; directory names remain flat so every
crate keeps a predictable `corpus/ui/<name>` path.

## Foundations

- `hello-ui-text` proves provider-neutral text roles and alignment.
- `hello-ui-theme` proves that appearance can change without changing meaning.
- `hello-ui-box` proves a reusable bounded surface.
- `hello-ui-icons` and `hello-ui-glyph-corpus` exercise provider-facing visual
  data.

## Controls

- `hello-ui-button` proves action interaction.
- `hello-ui-textinput` proves focus, character input, editing, and caret
  behavior.
- `hello-ui-input` pressures routing, focus, hover, and capture semantics.

## Composition

- `hello-ui-panel` proves structural containment.
- `hello-ui-card` proves information grouping.
- `hello-ui-toolbar` proves command organization.
- `hello-ui-layout` proves spatial composition.
- `hello-ui` and `hello-ui-dashboard` compose established pieces without
  redefining them.

## Stateful Systems

- `hello-ui-state` proves application state propagation through presentation.
- `hello-ui-scroll` proves viewport, clipping, and scrolling behavior.
- `hello-ui-animation` proves presentation transitions.
- `hello-ui-dialog` proves overlays and modal focus.
- `hello-ui-inspector` proves data-driven property presentation.

## Provider And Geometry Stress

- `hello-ui-font` compares TTF and OTF glyph execution.
- `hello-ui-font2` compares the curated font-provider corpus.
- `hello-ui-text-vectors` exercises font outlines through shared vector
  geometry.
- `hello-ui-lucide` proves basic Lucide provider consumption.
- `hello-ui-lucide2` runs the larger interactive Lucide geometry corpus.

Numbered names are retained where they identify existing evidence and command
paths. Rename them only when a clearer semantic distinction has been accepted;
do not create additional numbered variants by default.

## Shared Implementation

These entries primarily consume:

- [`../lib/ui-tools`](../lib/ui-tools/)
- [`../lib/ui-framework`](../lib/ui-framework/)
- [`../lib/presentation-geometry-corpus`](../lib/presentation-geometry-corpus/)
- [`../lib/screenshot`](../lib/screenshot/)

Shared code remains corpus infrastructure until its ownership and admission
status are explicitly settled.
