# Tokimu Documentation

This directory separates current architectural truth, active implementation
work, evidence, and historical material. The location of a document signals
how it should be used and how much authority it carries.

## Source Of Truth

These documents define current architecture and project policy:

- [Tokimu Software Design Document](Tokimu%20Software%20Design%20Document.md)
- [Tokimu TypeScript Design Document](Tokimu%20TypeScript%20Design%20Document.md)
- [Kernel Principles](kernel-principles.md)
- [Semantic Kernel Map](semantic-kernel-map.md)
- [Primitive Ledger](primitive-ledger.md)
- [Architectural Maxims](architectural-maxims.md)
- [Capability Backends](capability-backends.md)
- [Diagnostics Model](diagnostics-model.md)
- [Testing Strategy](testing-strategy.md)
- [Contribution And Admission Guide](contribution-admission-guide.md)

ADRs record accepted architectural decisions. Architectural Review records
capture proposals, evidence, and findings before or after a decision.

- [ADRs](ADR/)
- [Architectural Reviews](Architectural%20Reviews/)

Recent cross-cutting decisions:

- [ADR-0005: Admission Evidence and Maintainer Exceptions](ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md)
- [ADR-0006: Native Execution Policy](ADR/ADR-0006-native-execution-policy.md)

## Active Work

- [Plans](Plans/): executable implementation work. Plans are actionable but do
  not, by themselves, change architecture.
- [Notes](Notes/): working observations, validation results, and research
  records.
- [Roadmap](roadmap.md): milestone order and project-level sequencing.

## Evidence And History

- [Conversations](Conversations/): source discussions and exploratory design
  material. These are evidence and context, not authoritative contracts.
- [Archive](archive/): superseded or intentionally retained historical
  material. Archived documents should not be treated as current policy unless a
  current document explicitly cites them as background.

## Placement Rules

```text
Accepted architectural decision?  -> docs/ADR/
Architecture under review?        -> docs/Architectural Reviews/
Executable implementation work?  -> docs/Plans/
Observation or investigation?     -> docs/Notes/
Source conversation?              -> docs/Conversations/
Superseded historical material?   -> docs/archive/
Current architecture or policy?   -> docs/ root
```

Keep current documents at the root when they are cross-cutting references used
by the SDD, ADRs, contribution guide, or multiple subsystems. Do not move a
document solely to make the tree symmetrical if doing so would weaken links or
obscure its authority.

## Document Lifecycle

```text
Conversation or observation
        -> Note
        -> Plan
        -> Architectural Review
        -> ADR, if accepted
        -> Archive, when superseded
```

The lifecycle is not mandatory for every document. It preserves the difference
between evidence, intended implementation, and accepted architectural policy.
