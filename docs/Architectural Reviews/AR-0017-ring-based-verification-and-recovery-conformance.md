# AR-0017: Ring-Based Verification And Recovery Conformance

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-07 |
| Last reviewed | 2026-08-07 |
| Scope | Native Ring / verification and resilience / cross-cutting |
| Trigger | ADR-0009 is accepted and needs a durable record of contract, corpus, diagnostic, containment, and recovery evidence |
| Related ADRs | ADR-0005, ADR-0008, ADR-0009, ADR-0010, ADR-0011 |
| Related evidence | `crates/tokimu-core/src/scene.rs`; `docs/Dependency Audits/Ring 0/`; Ring 0 audit script and focused native/WASM/offline validation |
| Admission exception | None |

## Architectural Question

Are Native and Outer Ring changes retaining the proportionate verification,
failure-containment, diagnostic, and recovery evidence required by ADR-0009,
without mistaking a successful happy-path build for proof of resilience?

## Context

ADR-0009 separates the jobs of unit, contract, target, corpus, data corpus,
regression, fuzz/fault-injection, and golden evidence. It requires explicit
failure meaning and state containment before a recovery claim is made. Recovery
is a tested transition; it is not an optimistic log or a swallowed error.

The Ring 0 provenance repair is a narrow but useful first conformance case:
the source audit rejects unapproved provenance rather than silently fetching a
replacement, source initialization is validated, and `tokimu-core` is tested
offline after the local dependency migration. The change did not introduce a
runtime recovery protocol, persistence format, parser, or provider lifecycle,
so those sections remain inapplicable to this slice rather than completed
globally.

## Trigger And Evidence

- `cargo test -p tokimu-core` passes 29 focused unit tests covering current
  world, schedule, diagnostics, and scene behavior.
- `cargo test -p tokimu-core --locked --offline` passes after local source
  selection, proving the declared Ring 0 root does not silently obtain a
  registry substitute during that validation.
- The provenance audit rejects registry, remote Git, unapproved local paths,
  missing submodules, changed gitlinks, dirty submodules, and ignored Ring 0
  submodule configuration with actionable diagnostics.
- Native and `wasm32-unknown-unknown` focused builds pass for the local `glam`
  source selection.
- The Serde derives removed from `SceneDoc` and related scene types had no
  workspace serialization caller; the existing scene-compilation tests retain
  the semantic behavior that remains.
- Missing evidence includes automated negative audit fixtures, broader
  malformed-input corpora, fault injection for provider lifecycle paths, and
  explicit recovery claims for future persistence or remote-control work.

## Ownership Analysis

ADR-0009 owns the verification and resilience gate. This AR owns review of how
that gate is exercised and where evidence is absent. Native contracts own their
invariants, structured diagnostic identity, and state-commit behavior. Corpus
and provider code own realistic composition and target observations; they do
not substitute for assertions of kernel invariants.

The Ring 0 audit is build-validation tooling. It owns source-selection
rejection and diagnostics, not runtime crash recovery, a generic transaction
layer, or application policy. ADR-0010 remains the source-policy owner.

## Dependency Direction

```text
Native change
    |
    +--> unit and public-contract evidence
    +--> target and corpus evidence where relevant
    +--> structured diagnostics and containment evidence
    |
    v
retained ADR-0009 change record

Ring 0 source audit
    |
    v
explicit provenance rejection or local-source success
    |
    v
offline focused build and test evidence
```

## Alternatives Considered

### Alternative A: Treat Compilation As Verification

- Benefits: quick feedback.
- Costs: cannot establish rejection behavior, state preservation, diagnostic
  identity, malformed-input handling, or recovery semantics.
- Failure mode: a new failure path becomes invisible until an application hits
  it.

### Alternative B: Require End-To-End Corpus For Every Change

- Benefits: realistic composition evidence.
- Costs: weakens local diagnosis and overburdens small invariant changes.
- Failure mode: broad examples pass while narrow state or error contracts drift.

### Alternative C: Continue ADR-0009's Layered Evidence Model

- Benefits: tests, corpus, diagnostics, containment, and recovery each prove
  their honest boundary.
- Costs: requires maintainers to state absent or inapplicable evidence.
- Failure mode: layers are named but no negative or cleanup path is exercised.

## Findings

- The provenance audit fails closed and its offline validation is meaningful
  ADR-0009 evidence for dependency-source substitution.
- Focused scene tests preserve the behavior remaining after Serde derives were
  removed; no unsupported persistence guarantee was introduced.
- The current work does not prove process crash isolation, provider recovery,
  serialized-state recovery, or hostile-parser behavior. Those must remain
  open until a change actually introduces the relevant boundary.
- The audit lacks a checked negative fixture, so an unapproved-source rejection
  is currently validated by observed behavior rather than a retained automated
  test case.

## Disposition

**Continue under review.** ADR-0009 remains binding and unchanged. The current
evidence is proportionate for a focused source-provenance and unused-derive
removal slice, while the review stays open to ensure future Native work retains
negative, containment, and recovery evidence at its actual boundary.

## Consequences

- Source-provenance validation must remain fail-closed and offline-capable for
  declared Ring 0 roots.
- A future parser, persistence, provider, remote, FFI, or lifecycle change must
  add the relevant negative, cleanup, malformed-input, and recovery evidence;
  this AR does not pre-approve those omissions.
- The project should retain the smallest reproduction of any audit escape or
  containment defect as an automated regression.

## Required Follow-Up

- [x] Retain focused unit, target, and offline evidence for the Ring 0 repair.
- [x] Add controlled negative fixtures for unapproved-path and dirty-submodule
      rejection in the audit harness. A registry-source fixture remains
      follow-up work.
- [ ] Define how CI retains audit diagnostics and closure diffs for failed and
      successful provenance checks.
- [ ] Review future persistence, parser, provider, and remote-control changes
      against the full ADR-0009 containment and recovery sections.

## Reopening Triggers

- An audit can fetch or accept an unapproved source, or a rejection leaves an
  ambiguous diagnostic or partial trusted state.
- A Native change introduces persistence, hostile input, async lifecycle, FFI,
  provider, or remote operation without the corresponding failure evidence.
- A corrected defect lacks a focused regression test.
- A target-specific build exposes divergent failure or diagnostic behavior.

## Review History

### Cycle 1 -- 2026-08-07

- Status entering review: Proposed.
- New evidence: focused unit, native, WASM, and offline validation accompanied
  the Ring 0 repair; the provenance script rejects unapproved source classes.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: current source-selection evidence is proportionate; negative
  fixtures and future runtime-boundary evidence remain open.
- Disposition: continue under review.
- Resulting ADR or documentation change: no ADR revision; this record opened.

## References

- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/Dependency Audits/Ring 0/migration-baseline-2026-08-07.md`
- `scripts/audit-ring-zero-dependencies.ps1`
