# ADR-0005: Admission Evidence and Maintainer Exceptions

## Status

Accepted

## Context

Tokimu uses corpus examples, tests, decomposition attempts, and Architectural
Review Records to keep useful but domain-specific concepts from entering the
kernel prematurely. That discipline is intentionally conservative.

The admission guidance also contains numerical heuristics such as testing a
candidate across three unrelated domains. Those heuristics protect the project
from speculative abstractions, but they are evidence-gathering defaults rather
than a substitute for architectural judgment. A concept may be clearly
cross-cutting before three production capabilities exist, and requiring
artificial examples can produce ceremony without increasing confidence.

Tokimu therefore needs an explicit way to distinguish:

- ordinary implementation inside an accepted semantic boundary;
- provisional admission supported by incomplete evidence;
- permanent admission supported by substitute architectural evidence; and
- an evidence-free exception that should not proceed.

Without that distinction, maintainers may either over-apply the corpus gate to
ordinary functions or bypass it informally without leaving a durable account of
the decision.

## Decision

Corpus evidence remains the default admission path, but numerical corpus
thresholds are rebuttable evidence heuristics rather than absolute
prerequisites.

No admission exception is required when a change implements or extends an
already-admitted concept without changing its meaning, ownership, dependency
direction, or stability contract.

A maintainer may authorize **provisional admission** when the likely ownership
is sufficiently clear to support a reversible implementation but the normal
evidence set is incomplete. The decision must record:

- the proposed meaning and its Includes/Excludes;
- the decomposition attempts already made;
- which normal evidence is missing;
- the substitute evidence supporting implementation;
- the public stability, dependency, and migration risks accepted;
- the evidence or milestone that will confirm, relocate, or retire the concept.

Provisional admission must remain easy to revise. It must not be presented as a
stable kernel contract merely because implementation has begun.

A maintainer may authorize **permanent admission by evidence substitution**
when existing architectural evidence is already sufficient to decide ownership
confidently. The Architectural Review Record or ADR must identify:

- the normal admission requirement being waived;
- why satisfying it mechanically would add little decision value;
- the cross-cutting evidence relied upon instead;
- at least one serious alternative or decomposition considered;
- portability, dependency, determinism, and migration consequences;
- explicit reopening triggers.

An admission exception cannot:

- authorize a dependency direction or ownership relationship forbidden by an
  accepted ADR;
- treat implementation convenience as engine meaning;
- make missing evidence disappear without recording it;
- use reviewer count as a substitute for reasoning;
- silently stabilize an incubating API.

The accountable project maintainer owns the decision. Tools and AI assistants
may provide recorded critical review, decomposition attempts, and counter-
examples, but they are not governance principals or a durable maintainer
quorum.

When future project governance establishes multiple accountable maintainers, it
may add a voting or quorum rule without changing the evidence requirements in
this decision.

## Consequences

Tokimu can make well-supported architectural decisions without manufacturing
corpus cases solely to meet a numerical threshold. Exceptional decisions remain
inspectable because the missing evidence and substitute reasoning are recorded.

The project accepts the risk that maintainer judgment can be wrong. Reversible
provisional implementation, explicit reopening triggers, and retained review
history limit that risk.

This decision does not weaken ADR-0001 or ADR-0003. Kernel additions must still
express universal engine meaning, capability crates must still own optional
domain meaning, and platform or backend mechanisms must not leak upward into
engine-owned semantics.

## References

- `docs/contribution-admission-guide.md`
- `docs/semantic-kernel-map.md`
- `docs/Architectural Reviews/README.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`

