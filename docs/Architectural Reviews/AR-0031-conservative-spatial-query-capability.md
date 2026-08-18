# AR-0031: Conservative Spatial-Query Capability

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-17 |
| Last reviewed | 2026-08-17 |
| Scope | Optional capability / corpus incubation / native-WASM portability |
| Trigger | E1M1 produced reusable conservative BVH mechanics with camera, runtime-revision, and portable evidence while naive splitting BSP construction amplified the same geometry by 180.82x. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0014, ADR-0015 |
| Related evidence | AR-0025, AR-0030, `tokimu-spatial-query-study`, E1M1 spatial reports |
| Admission exception | None |

## Architectural Question

Should Tokimu eventually admit an optional conservative spatial-query
capability, and what is the smallest meaning independent consumers must prove
before corpus-local mechanics move into an engine crate?

## Context

The Doom campaign originally asked whether Tokimu should own a BSP. A naive
triangle-splitting BSP conserved the exact prepared representation but expanded
`1,849` triangles into `334,345` fragments. An unsplit median BVH retained the
same `1,849` members in `255` nodes, matched brute-force camera and ray queries,
and supported explicit immutable geometry revisions.

That narrows the candidate from tree topology to conservative queries over
exact finite members. It does not establish Ring 2 ownership, a stable Rust
surface, or render-visibility authority.

## Trigger And Evidence

- Complete E1M1 BVH fingerprint: `599d8ca7411ffd11`.
- Nine actual-camera poses: zero frustum false negatives, zero false positives,
  and zero nearest-ray mismatches; matrix fingerprint `3c80342bb2cfcdf4`.
- Nineteen door/platform snapshots preserve immutable revisions, reject stale
  artifact identities, and match brute-force frustum/ray results.
- Reusable mechanics now live in `corpus/lib/tokimu-spatial-query-study`.
  Source-format conversion remains outside and the crate contains no
  source-format terms.
- The same portable fixture executes natively and through
  `wasm-bindgen-test-runner` on `wasm32-unknown-unknown`. Both assert structure
  fingerprint `a7ab8dffa4f4b487`, revision `0c2f9ba483384480`, identical
  candidates and nearest hit, stale-revision failure, and revised refit behavior.
- A separate portable Doom consumer adapts four exact retained E1M1
  floor/wall triangles and rays. Native and WASM executions preserve subset
  fingerprint `3189fb35dfba3bdc` and resolve all four source-side identities.
- Only one demanding application domain exists. The portable fixture proves
  target parity, not an independent semantic consumer.

## Ownership Analysis

The mechanics index caller-owned geometry and return candidates. They do not
own simulation state, source topology, movement policy, view construction,
visibility, occlusion, presentation membership, or renderer submission.

Today they correctly remain corpus infrastructure. If admitted later, Tokimu
would own only conservative query and revision meaning; applications would own
artifact lifecycle and interpretation, while implementations could own index
machinery. No provider contract is admitted here.

## Dependency Direction

```text
Current:
source/application geometry adapter
    -> corpus-local spatial-query study
        -> tokimu-core math

Possible after admission:
application/runtime composition
    -> optional Tokimu spatial-query meaning
        -> selected portable or specialized implementation
```

The corpus library has no source-provider, renderer, runtime, platform,
window, filesystem, or browser dependency and is not exported by `tokimu`.

## Alternatives Considered

### Admit a BSP capability now

- Benefit: exposes partition topology.
- Cost: no caller requires partition planes or split fragments.
- Failure mode: topology becomes architecture before its semantics are needed.

### Admit a provider-neutral query capability now

- Benefit: matches the smaller demonstrated behavior.
- Cost: one domain and one synthetic fixture cannot stabilize lifecycle, query
  shapes, errors, or provider selection.
- Failure mode: corpus details accidentally become public guarantees.

### Continue corpus-library incubation

- Benefit: shares exact native/WASM mechanics without changing engine
  boundaries and permits independent falsification.
- Cost: consumers use an explicitly unstable corpus API.
- Failure mode: indefinite incubation, constrained by the triggers below.

## Findings

- A portable conservative BVH is useful for the demonstrated workload; BSP
  topology is not presently required.
- Immutable replacement is the simplest evidenced lifecycle. Refit is valid
  only while exact identity and correlation remain stable.
- Revision checking is correctness evidence, not an optimization.
- Native/WASM parity is established for the same fixture and implementation.
- Query results have no visibility, occlusion, or presentation authority.
- A second independent application-shaped consumer remains necessary before
  Ring placement or a public contract can be judged.

## Disposition

**Incubating.** Retain `tokimu-spatial-query-study` in `corpus/lib`. Do not
create a Ring 2 crate, provider trait, capability descriptor, renderer
integration, or facade export. Gather independent evidence first.

## Consequences

- E1M1 diagnostics consume the shared implementation while source adaptation
  and policy stay local.
- Cross-target tests preserve fixed identities and fingerprints.
- The experimental Rust surface may change with its evidence; it is not a
  compatibility guarantee.
- The historical BSP plan remains evidence; this review now owns the
  prospective capability question.

## Required Follow-Up

- [ ] Add an independent non-Doom application-shaped consumer with a genuine
      frustum, ray, or candidate need.
- [ ] Test a member shape or update lifecycle materially unlike prepared
      static triangles.
- [ ] Add unresolved/query-budget vocabulary only if a real caller requires it.
- [ ] Compare another implementation only if replacement becomes a requirement.
- [ ] Open or revise an ADR only after Ring placement and stable semantics are
      supported by evidence.

## Reopening Triggers

- A non-Doom consumer independently needs the same query/revision semantics.
- A caller requires partition planes, split fragments, portals, or occlusion.
- Native and WASM diverge in identity, fingerprint, or revision behavior.
- Immutable replacement materially misses an explicit runtime budget.
- Source or renderer vocabulary leaks into the corpus library.

## Review History

### Cycle 1 -- 2026-08-17

- Status entering review: Proposed
- New evidence: complete E1M1 bake/query/runtime reports, corpus extraction,
  native fixture, and executed WASM fixture.
- Participants or reviewers: maintainer and Codex implementation review.
- Findings: mechanics are portable; capability admission and BSP semantics are
  unearned.
- Disposition: Incubating in `corpus/lib`.
- Resulting ADR or documentation change: none; engine boundaries are unchanged.

### Cycle 2 -- 2026-08-17

- Status entering review: Incubating.
- New evidence: six retained E1M1 sky-leak rays query the exact global
  prepared-triangle BVH and the final Doom ordered handoff in one shadow
  replay. Every BVH result matches the brute-force nearest-triangle oracle.
- Findings: the BVH correctly reports all six suspect triangles as
  geometrically relevant, while Doom omits five and retains one only as a
  partial plane occurrence. Conservative spatial relevance is therefore not
  visibility, occlusion or source-presentation authority.
- Disposition: no change to capability placement or status. A future spatial
  capability must keep this negative guarantee explicit; it cannot be used to
  repair the Doom sky leak by filtering submission.
- Resulting ADR or documentation change: none; AR-0030 retains ownership of
  the Doom-private preparation/handoff question.

### Cycle 3 -- 2026-08-18

- Status entering review: Incubating.
- New evidence: a Doom-private view-cell/aperture sidecar carried 632
  actual-camera, path-qualified clipped-view states over the existing exact
  BVH for the six retained E1M1 rays. The graph and aperture inventories were
  deterministic and had zero containment failures.
- Findings: path-qualified view state is mechanically viable, but physical
  aperture transfer cannot retain a known-good subsector 104 ceiling
  occurrence. The complete Doom ordered-source oracle remains necessary for
  retained members outside transfer and covered members inside reached cells.
- The BVH continues to answer only conservative geometric questions. Cell,
  aperture, sky and ordered-source facts do not acquire node-level rejection
  or acceptance authority.
- Disposition: no change. Do not extract a view-cell, aperture, portal or
  occlusion contract from this corpus. AR-0026 may later compare path-qualified
  state mechanics using independent authored-chart evidence, but E1M1 does not
  admit shared implementation.
- Resulting ADR or documentation change: none; the experiment is parked in
  AR-0030 and its controlling study.

## References

- `docs/ADR/ADR-0015-source-unit-cohesion-size-pressure-and-decomposition.md`
- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Architectural Reviews/AR-0030-source-owned-presentation-preparation-boundary.md`
- `docs/Plans/DOOM/Tokimu BSP capability setup plan.md`
- `docs/Checkpoints/2026-08-17-tokimu-spatial-runtime-sequence-release.md`
- `docs/Checkpoints/2026-08-18-doom-custom-bvh-view-transfer-shadow.md`
- `corpus/lib/tokimu-spatial-query-study/README.md`
