# AR-0016: Native Ring Performance And Code Quality Conformance

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-07 |
| Last reviewed | 2026-08-07 |
| Scope | Native Ring / cross-cutting review practice |
| Trigger | ADR-0008 is accepted and needs a durable evidence record for its proportional performance and code-quality gate |
| Related ADRs | ADR-0003, ADR-0005, ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0010, ADR-0011 |
| Related evidence | `docs/Plans/ring-zero-third-party-source-audit-and-migration.md`; `docs/Dependency Audits/Ring 0/`; focused Native tests and target builds |
| Admission exception | None |

## Architectural Question

Is ADR-0008's full Native Ring gate being applied with concrete ownership,
performance, determinism, hygiene, and retained-evidence decisions—without
turning ordinary Outer Ring work or mechanical edits into checklist theater?

## Context

ADR-0008 makes Native Ring changes expensive in proportion to their ecosystem
impact. Its gate is deliberately broader than benchmarking: it requires an
explicit invariant, a search for existing vocabulary, bounded work, visible
ownership, target awareness, hygiene, and evidence. Outer Rings retain a
smaller gate unless they cross a Native contract, alter process-wide behavior,
or occupy a measured hot path.

The current Ring 0 provenance repair exercises this distinction. Removing
unneeded Serde derives from `tokimu-core` reduced trusted build-time code.
Pinning `glam` preserves existing math representation while recording its
public-type commitment, selected features, unsafe surface, target behavior, and
compiler-warning risk. Neither change establishes a universal performance
budget or makes a local timing result an engine-wide guarantee.

## Trigger And Evidence

- `tokimu-core` has focused unit tests; `cargo test -p tokimu-core` passed for
  the provenance-repair change.
- `cargo clippy -p tokimu-core --all-targets -- -D warnings` and the native and
  WASM focused builds passed for the changed root.
- The local Ring 0 audit proves a source and feature reduction: eight foreign
  packages in the original closure became one local, pinned `glam` package.
- The retained `glam` audit identifies 78 Rust source files containing unsafe
  code and records future compiler warnings from upstream generated swizzles.
- No before/after runtime benchmark, binary-size measurement, or broad
  workspace performance report was collected because this slice changes source
  provenance and removes unused derives rather than claiming a runtime speedup.

## Ownership Analysis

ADR-0008 owns the binding proportional gate. This AR owns evidence of its use,
recurring friction, and whether its checklist is producing decisions rather
than ritual responses. `tokimu-core` and admitted Native contracts own the
invariants that require the full gate. Corpus and provider code own their local
implementation choices until they cross a Native contract or otherwise meet
the escalation rule.

No new profiler, benchmark framework, code-quality service, or performance
policy belongs in the kernel because of this review. Applications and tools may
measure their workloads; Native contracts retain only the provider-neutral
diagnostic meaning admitted by ADR-0007.

## Dependency Direction

```text
Native Ring change
    |
    v
change-local ADR-0008 evidence and review record
    |
    +--> focused tests / target validation / measurement when applicable
    +--> ADR-0007 diagnostic contract for sustained budget pressure
    |
    v
retained decision evidence

Outer Ring change
    |
    v
minimum gate
    |
    +--> full gate only for a Native crossing, process-wide behavior,
         or a measured hot path
```

## Alternatives Considered

### Alternative A: Require One Benchmark For Every Change

- Benefits: superficially uniform evidence.
- Costs: encourages meaningless timing and delays non-performance changes.
- Failure mode: benchmark output becomes a substitute for ownership review.

### Alternative B: Rely On General Rust Tests And Lints

- Benefits: familiar tooling.
- Costs: does not expose duplicate semantics, target assumptions, unbounded
  work, or the distinction between Native and Outer ownership.
- Failure mode: a tidy build admits a costly or misplaced kernel contract.

### Alternative C: Continue ADR-0008's Proportional Change Record

- Benefits: evidence follows actual risk and architectural ownership.
- Costs: reviewers must state why a check is not applicable.
- Failure mode: routine `N/A` answers become unexamined ceremony.

## Findings

- The Ring 0 repair demonstrates that ADR-0008 can drive a material design
  reduction: an unused derive chain was removed instead of being source-pinned
  only because it was already present.
- Focused test, lint, native, WASM, offline, and provenance evidence are
  appropriate for this dependency and contract-boundary change.
- The `glam` compiler-warning set is a retained code-quality and toolchain
  compatibility risk; successful current compilation does not close it.
- The project still needs a repeatable change-record location or convention for
  full-gate answers on future Native Ring behavior and performance changes.
- No evidence yet supports a change to ADR-0008's accepted proportionality.

## Disposition

**Continue under review.** ADR-0008 remains binding and unchanged. This record
will collect a small sample of real Native and Outer Ring change evidence,
including cases where a checklist item changed the design or was honestly not
applicable, before any checklist revision is considered.

## Consequences

- Future Native Ring work must retain its applicable ADR-0008 answers with its
  change, audit, plan, or review evidence.
- Local tests and lints do not by themselves justify performance claims.
- The current `glam` warning risk must be revisited before a compiler or source
  update can make it a build failure.

## Required Follow-Up

- [x] Retain ADR-0008 evidence for the first Ring 0 provenance repair.
- [ ] Define a lightweight, discoverable location for full-gate evidence in
      future Native Ring changes.
- [ ] Sample several future completed changes for repeated `N/A` answers or
      checklist items that never influence a decision.
- [ ] Add measurement artifacts when a Native change claims a runtime,
      allocation, binary-size, or compile-time effect.
- [ ] Revisit the pinned `glam` warning before a compiler update.

## Reopening Triggers

- A Native change introduces a measurable regression, duplicate semantic
  abstraction, unbounded work path, or target-specific assumption.
- A sampled review shows repeated ceremonial checklist answers.
- An Outer Ring hot path or contract crossing evades full-gate review.
- The `glam` warning becomes a compiler error or its pinned source changes.

## Review History

### Cycle 1 -- 2026-08-07

- Status entering review: Proposed.
- New evidence: the Ring 0 repair removed seven unnecessary trusted packages,
  retained one pinned math package, and recorded focused Native validation.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: ADR-0008 produced a real decomposition decision; broad measurement
  and checklist-maintenance evidence remain open.
- Disposition: continue under review.
- Resulting ADR or documentation change: no ADR revision; this record opened.

## References

- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/Plans/ring-zero-third-party-source-audit-and-migration.md`
- `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
