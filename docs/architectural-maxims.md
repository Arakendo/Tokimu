# Tokimu Architectural Maxims

| Field      | Value |
| ---------- | ----- |
| Status     | Living guidance |
| Scope      | Enduring architectural principles that guide design decisions across Tokimu. |
| Relates to | SDD, Kernel Principles, Semantic Kernel Map, Capability Backends, Future Workspace Layout, Contribution Admission Guide |

## Purpose

Tokimu's detailed design documents describe *how* the architecture works.

This document records the shorter principles that repeatedly guide architectural
judgment.

Maxims are not strict rules. They express the direction Tokimu should naturally
move unless compelling evidence suggests otherwise.

## Semantic Ownership

### Who owns this meaning?

Ownership precedes implementation.

Before asking where code belongs, determine who owns the semantic meaning being
introduced.

## Primitive Admission

### Useful is cheap. Irreducible is expensive.

Utility alone does not justify becoming foundational.

Kernel concepts should be admitted only after repeated attempts at decomposition
have failed.

### Admit unavoidable meaning.

Kernel concepts become permanent cognitive infrastructure.

Only concepts that are genuinely unavoidable should become part of Tokimu's
worldview.

## Layering

### The workspace reflects semantic ownership, not implementation convenience.

Workspace organization should communicate ownership, not historical accident.

### Dependencies inherit worldviews.

Depending on another crate is not merely a compilation decision.

It is accepting that crate's semantic model.

Dependency direction should therefore point toward more universal meaning.

### Keep implementation below semantics.

Libraries, frameworks, and external technologies implement Tokimu semantics.

They should not define them.

## Capability Philosophy

### Own meaning. Delegate implementation.

Tokimu should own semantic models while allowing multiple implementations behind
stable capability contracts.

### Frontends author meaning.

They do not own runtime truth.

Authoring surfaces should target Tokimu semantics rather than invent parallel
runtime behavior.

## Evolution

### Evidence before permanence.

Architectural additions should be justified through examples, tests,
implementation pressure, or repeated semantic failure.

### Architecture is allowed to learn.

Concepts may be retired when better understanding reveals simpler ownership or
clearer boundaries.

Retirement should be guided by evidence rather than attachment.

## Simplicity

### Prefer composition over promotion.

When existing concepts compose successfully, avoid creating new foundational
ones.

### Distinguish reality from representation.

Identifiers are not objects.

Models are not worlds.

Presentation is not simulation.

Representations should never quietly become reality.

## Design Culture

### Let implementation challenge theory.

Architecture should be tested by real implementations.

Good theory survives contact with reality.

### Clarity over cleverness.

Simple semantic boundaries are usually more valuable than sophisticated
mechanisms.

### Protect the learner.

Every permanent concept becomes part of someone else's mental model.

Reduce cognitive burden whenever composition can achieve the same result.

## Closing Thought

Architecture is not the accumulation of abstractions.

It is the disciplined admission of meaning.