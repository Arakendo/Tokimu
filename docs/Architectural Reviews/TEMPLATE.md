# AR-XXXX: Title

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | YYYY-MM-DD |
| Last reviewed | YYYY-MM-DD |
| Scope | Kernel / foundational service / capability / backend / frontend / cross-cutting |
| Trigger | Corpus pressure, dependency change, repeated friction, retirement, or other concrete cause |
| Related ADRs | None |
| Related evidence | Examples, tests, audits, design documents, or issues |
| Admission exception | None, provisional admission, or permanent evidence substitution under ADR-0005 |

## Architectural Question

State one decision-shaped question. If the record contains several independent
questions, split it before review.

## Context

Describe the current architecture and why the question matters now.

## Trigger And Evidence

Record concrete observations separately from architectural guarantees.

- Corpus examples:
- Automated tests:
- Audits or diagnostics:
- Independent consumers:
- Repeated implementation friction:
- Missing evidence:

## Ownership Analysis

Answer:

- What meaning is involved?
- Who owns it today?
- Who should own it if the proposal is accepted?
- Is it kernel-native, foundational, capability-owned, backend-owned, or
  frontend-only?
- What state or truth must it not own?

## Dependency Direction

Describe the current and proposed dependency direction. Identify any upward,
cyclic, platform-specific, or foreign-object dependencies.

```text
Current:

Proposed:
```

## Alternatives Considered

### Alternative A: Name

- Benefits:
- Costs:
- Failure mode:

### Alternative B: Name

- Benefits:
- Costs:
- Failure mode:

Include decomposition, extension of an existing concept, continued incubation,
and doing nothing where they are genuine alternatives.

## Findings

Record what the evidence supports, what it does not support, and any remaining
uncertainty.

If ADR-0005 is used, also record:

- normal evidence missing;
- substitute evidence;
- why mechanically completing the normal threshold adds little decision value;
- accountable maintainer;
- risks accepted;
- reopening trigger.

## Disposition

Choose one:

- Accepted -- ADR required
- Incubating
- Deferred
- Rejected
- No Change
- Superseded

State the result in one direct paragraph.

## Consequences

Describe architectural, dependency, compatibility, portability, diagnostic,
testing, and migration consequences.

## Required Follow-Up

- [ ] Documentation or ADR work
- [ ] Focused implementation slice
- [ ] Corpus example or automated test
- [ ] Migration, retirement, or compatibility work

## Reopening Triggers

Name observable events that justify reopening. Examples:

- a second independent consumer requires the same semantics;
- three unrelated corpus examples repeat the same application-side behavior;
- a target or backend cannot preserve the accepted contract;
- provider details leak through the semantic boundary;
- implementation evidence invalidates a stated finding;
- a simpler decomposition becomes available.

Avoid vague triggers such as "if needed later."

## Review History

### Cycle 1 -- YYYY-MM-DD

- Status entering review: Proposed
- New evidence:
- Participants or reviewers:
- Findings:
- Disposition:
- Resulting ADR or documentation change:

Append later cycles; do not replace earlier cycles.

## References

- `docs/contribution-admission-guide.md`
- Relevant ADRs, design documents, examples, tests, and audits
