# ADR-0002: Conceptual Influence Stays Out of the API Surface

## Status

Accepted

## Context

Tokimu, Tosumu, and the Tonesu language project share a naming lineage and some
design instincts. Tonesu in particular places strong emphasis on compositional
meaning, explicit structural relations, and epistemic honesty.

Those ideas are useful for engineering judgment, but they can easily become a
liability if they leak into public APIs as project-specific philosophical
terminology that future contributors must learn before they can navigate the
codebase.

## Decision

Tokimu may inherit conceptual influence from Tonesu and Tosumu, but that
influence remains conceptual rather than terminological.

* Public APIs, crate names, and engine concepts should use plain English.
* Tonesu-derived ideas may shape architecture, boundaries, diagnostics, and
  truth-model discussions.
* Conlang terminology may appear in project-history or design-rationale
  documentation when it adds useful context, but not as a prerequisite for
  understanding core engine code.

## Consequences

The codebase stays accessible to contributors who have no context for the
language project, while still preserving the real design lineage that helped
shape decisions around boundaries, diagnostics, and source-of-truth concerns.