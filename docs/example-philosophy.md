# Tokimu Example Philosophy

| Field      | Value |
| ---------- | ----- |
| Status     | Living guidance |
| Scope      | Why Tokimu examples exist and how they should be chosen |
| Relates to | SDD, Architectural Maxims, Kernel Principles, Semantic Kernel Map, Primitive Ledger |

## Purpose

Tokimu examples are architectural corpus tests.

They are not showcase demos and they are not just API tutorials. They exist to
apply pressure to the architecture, reveal semantic gaps, and provide evidence
for future decisions about kernels, capabilities, and frontends.

That is the same role Tonesu corpus tests play for language meaning.
A Tonesu sentence pressures the semantic model. A Tokimu example pressures the
architectural model.

## How Examples Work

A good Tokimu example should answer one clear question:

> Can the current architecture naturally express this world relation, rule, or
> transformation?

If the answer is yes, the example becomes evidence that the existing model is
sufficient.
If the answer is no, the example reveals a boundary that may deserve a new
primitive, capability, or document.

## Selection Rules

- Prefer one primary claim per example.
- Keep examples small enough to finish and inspect.
- Use procedural content when it is enough to validate the boundary.
- Avoid inventing new foundational concepts unless repeated examples prove they
  are unavoidable.
- Do not split examples into extra crates or layers just to look scalable.
- Do not duplicate an existing proof unless the new example exercises a new
  architectural seam.

## Two Example Families

Tokimu examples tend to fall into two broad families:

- Simulation examples: they pressure world state, rules, motion, input, and
  runtime behavior.
- Representation examples: they pressure how Tokimu expresses geometry,
  assets, vector composition, and other forms of meaning before rendering.

Both families are useful. They simply prove different seams.

Representation examples can also split by semantic responsibility. In the UI
corpus, that usually means:

- Foundations: text, appearance, and surface meaning
- Controls: button, panel, card, toolbar
- Systems: layout, state, input, scroll, animation, inspector
- Composition: inspector, dashboard, framework, and other end-to-end proofs

Those are not hard architectural tiers. They are a reminder that examples are
allowed to test verbs, not just nouns.

## Vocabulary

Useful phrases for example design:

- Architectural pressure
- Corpus test
- Boundary proof
- World sentence
- New evidence
- Primary claim

Avoid framing examples as mere feature demos. If an example does not introduce
new evidence, it is usually a maintenance or regression case, not a new corpus
sentence.

## Feedback Loop

Examples do not only consume architecture.

Implementation feeds lessons back into Tokimu through:

- Primitive Ledger updates
- ADRs
- Kernel Principle revisions
- Capability boundary refinement

Architecture proposes.
Examples validate.
Implementation teaches.

## References

- [Example Corpus Directory Guide](../examples/README.md)
- [Tokimu Architectural Maxims](architectural-maxims.md)
- [Tokimu Kernel Principles](kernel-principles.md)
- [Tokimu Semantic Kernel Map](semantic-kernel-map.md)
- [Tokimu Primitive Ledger](primitive-ledger.md)
- [Tokimu Software Design Document](Tokimu%20Software%20Design%20Document.md)
