# Doom Source-Topology Admission Over Complete Geometry

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Role | Bounded candidate-realization study |
| Status | Proposed |
| Parent review | AR-0030 |
| Controlling plan | [DOOM WAD Checklist](../DOOM%20WAD%20Checklist.md) |
| Preceding study | [Doom Viewer-Relative Presentation Synthetic Conformance](Doom%20viewer-relative%20presentation%20synthetic%20conformance.md) |
| Corpus target | `corpus/campaigns/doom/hello-doom-e1m1/` plus focused synthetic fixtures |
| Next action | Freeze the global-full baseline and inventory the original source-labelled presentation contributions without reconstructing any geometry. |

## Purpose

Test whether Doom-owned source-topology admission over the original complete
presentation geometry is sufficient to remove source-invalid far geometry
without losing the continuity that global full submission already gets right.

The study deliberately intervenes less than the failed Slice 7 realization:

```text
complete source-labelled E1M1 geometry
        ↓
Doom-owned viewer-relative contribution admission
        ↓
original meshes retained unchanged
        ↓
optional generic conservative AABB/frustum selection
        ↓
full submission to tokimu-render
```

The study does not attempt to reconstruct exact Doom screen columns as world
geometry. It asks whether source-invalid participation, rather than incomplete
geometry, is the dominant cause of the remaining full-submission sky leakage.

## Motivation

Global full submission has an important demonstrated property: original floors,
ceilings, and walls meet continuously; near geometry does not disappear; free
look does not reveal a finite preparation rectangle; and dynamic geometry can
continue to use its original source-labelled meshes.

Its principal visible defect is different: distant source regions may
participate through an area that classic Doom's viewer-relative source protocol
would not allow them to reach, particularly around sky presentation.

The failed inverse-world realization fixed neither problem. It converted a
bounded `320 x 200` prepared observation back into approximate world geometry,
introducing cracks, close-wall loss, and a visible view box. This plan keeps the
known-good complete geometry and changes only whether an original contribution
may participate.

## Governing Hypothesis

A Doom-owned topology pass can reject source-invalid original presentation
contributions while admitting complete original contributions unchanged. Once
that source-specific decision is complete, generic conservative selection and
ordinary GPU depth testing may safely handle the remaining Euclidean work.

```text
Doom source topology:
    may this original contribution participate in this view?

generic AABB/frustum:
    can the admitted geometry intersect this camera?

GPU depth/rasterization:
    which submitted fragments win?
```

The hypothesis is falsified when faithful presentation requires only part of
one original contribution to survive and retaining that contribution whole
produces a visible source-invalid result after all unrelated contributions have
been rejected.

## Study-Local Alternatives

These labels belong to this study. They do not alias the existing executable's
historical `--render-strategy=a|b|c` meanings.

| Alternative | Pipeline | Purpose |
| --- | --- | --- |
| A — global full submission | all original complete contributions -> renderer | Correctness and continuity control; known far-sector sky leakage. |
| B — source-topology admitted full submission | Doom admission -> all admitted original contributions unchanged -> renderer | Candidate under test. |
| C — admitted then conservatively filtered | B output -> generic AABB/frustum -> renderer | Tests AR-0030 Alternative F only after B is correct. |

Use explicit CLI names rather than overloaded letters:

```text
--render-strategy=global-full
--render-strategy=topology-admitted-full
--render-strategy=topology-admitted-frustum
```

If compatibility aliases remain, startup metadata must print the resolved
long-form strategy name.

## Unit Of Admission

Do not begin with `visible sector: bool`. A sector is source identity and
semantic state, but it may have multiple presentation occurrences and may be
reached through one opening while excluded through another.

The initial admission inventory operates on original, source-labelled
presentation contributions:

- subsector floor contribution;
- subsector ceiling contribution;
- original SEG/linedef wall contribution, including upper, lower, and middle
  roles;
- caller-declared masked-middle/cutout contribution;
- sky-associated plane contribution; and
- dynamic door or platform contribution tied to an explicit runtime snapshot.

The experiment may group contributions for efficient evaluation only when the
group retains the same admission result and source provenance. Grouping must
not silently turn a presentation-occurrence decision back into sector-wide
authority.

## First Executable Admission Algorithm

The first candidate reuses the existing Doom-owned ordered source trace as an
observation authority but does not consume its cells to build geometry:

1. Locate the viewer's source subsector from the explicit source pose and
   current runtime snapshot.
2. Traverse the existing Doom BSP/source relationships near-to-far and retain
   opening, terminal-solid, wall-role, plane-occurrence, and sky-boundary
   provenance.
3. Fold that provenance back onto the original contribution inventory:
   - admit an original wall occurrence when any authorized retained source
     range refers to it;
   - admit an original subsector floor/ceiling occurrence when the ordered
     trace retains any corresponding plane interval;
   - admit cutouts through reachable openings without allowing their texture
     coverage to close source reachability;
   - consume declared current heights for doors/platforms before classifying
     their openings; and
   - keep the sky panorama/background separate from topology admission.
4. Reject only when the source trace retains positive terminal provenance that
   the contribution cannot participate from this prepared view.
5. Treat mere absence from the bounded trace, projection ambiguity, a
   between-column/thin result, or unsupported source semantics as unresolved
   and fail open.
6. Submit the original draw/mesh for every admitted or unresolved contribution.

This is intentionally conservative. If positive terminal provenance cannot
remove the far-sector sky leakage, that is evidence against this algorithm. It
is not permission to reinterpret every unobserved contribution as rejected.
The `320 x 200` trace may inform admission provenance, but its cells never
become vertices, mesh boundaries, scissors, or renderer vocabulary.

## Required Admission Evidence

Every contribution receives exactly one bounded outcome before any generic
filter runs:

```text
admitted
    source reason + traversal/opening provenance

rejected
    source reason + terminal boundary or unreachable provenance

unresolved
    explicit fail-open admission + diagnostic reason
```

Retain at least:

- contribution identity and family;
- sector, subsector, SEG, linedef, sidedef, and wall-role identities where
  applicable;
- prepared-view and runtime-snapshot identity;
- admission result and Doom-owned reason;
- original mesh/resource identity and a structural hash before and after
  admission;
- whether a later generic filter retained or rejected it; and
- bounded representative samples for every reason category.

Unknown source cases fail open into B so the candidate remains conservative.
They must not be silently hidden to improve a screenshot.

## Hard Invariants

- B and C do not clip, subdivide, reverse-project, rebuild, rescale, or otherwise
  mutate admitted original geometry.
- B preserves the relative submission order of surviving contributions.
- C is applied strictly after B and may only conservatively remove B output.
- Sky supplies presentation content; it does not become a generic occluder or
  an invisible world-space depth wall.
- Cutout geometry is not treated as a solid occluder merely because its quad
  covers a screen range.
- Current door/platform heights are explicit runtime-snapshot inputs. The
  topology pass does not own activation, timing, reversal, or movement policy.
- Renderer and platform crates receive no Doom topology, sector, SEG, or sky
  vocabulary.
- A remains runnable throughout the study as the continuity fallback.

## Slice 0 — Freeze Controls And Names

### Deliverables

- [ ] Record the exact A invocation, resolved defaults, package fingerprint,
      embedding, camera poses, draw-family inventory, and current known sky
      leakage.
- [ ] Add distinct long-form executable strategy names for A, B, and C without
      changing the behavior of any existing comparison control.
- [ ] Make startup and first/warm-frame metadata print the resolved strategy and
      ordered stage list.
- [ ] Retain canonical source poses for spawn, hut/window, exterior hut, first
      door, moving platform, green-room one-sided cutout, and EXIT.
- [ ] Record structural hashes for the original complete meshes used by those
      poses.

### Acceptance criteria

- A remains visually and structurally unchanged.
- No existing historical B/C strategy is silently reinterpreted.
- Every observation unambiguously identifies which pipeline ran.

## Slice 1 — Original Contribution Inventory

### Deliverables

- [ ] Establish one private contribution record that references an existing
      original draw/mesh rather than owning reconstructed geometry.
- [ ] Map original floor, ceiling, wall-role, cutout, sky, door, and platform
      draws to source identities.
- [ ] Detect and report original draws that cannot be correlated to an
      admission contribution.
- [ ] Prove that inventory creation does not upload, replace, or mutate meshes.
- [ ] Retain counts by family and bounded duplicate/unresolved samples.

### Acceptance criteria

- Every original draw is correlated, explicitly presentation-global, or
  retained as unresolved/fail-open.
- The inventory reproduces A exactly when all contributions are admitted.
- Original resource identities and structural hashes are unchanged.

## Slice 2 — Synthetic Topology Admission

### Fixtures

- [ ] Connected rooms with an open source aperture: both room contributions
      remain admissible.
- [ ] Connected records behind a source-terminal solid boundary: the far
      contributions are rejected with source provenance.
- [ ] Paired-sky and one-sky controls: sky does not independently close or open
      source reachability.
- [ ] Vertical aperture: upper/lower wall roles and plane contributions are
      admitted independently where the source opening permits them.
- [ ] Masked middle: the contribution remains admitted while transparent texels
      do not authorize it as a solid topology occluder.
- [ ] Declared closed/open door and low/raised platform snapshots: admission
      consumes current heights without implementing movement policy.
- [ ] Ambiguous or unsupported topology: fail open and retain a bounded reason.

### Acceptance criteria

- Headless results are deterministic and preserve source identity.
- Rejecting a contribution never changes an admitted contribution's geometry.
- Runtime snapshots cause only the expected admission changes.
- Native presentation fixtures pass before E1M1 promotion.

## Slice 3 — Partial-Survival Falsifier

Revisit the earlier far-SEG observation whose screen-domain reconstruction
survived on two separated column intervals. This slice asks a different
question: after source-invalid surrounding contributions are removed, can the
whole original contribution remain and let ordinary depth produce the correct
final view?

### Deliverables

- [ ] Run A and B over the same partial-survival fixture using identical
      original geometry.
- [ ] Retain which unrelated contributions B rejects and why.
- [ ] Demonstrate whether the complete retained contribution creates any
      visible source-invalid pixels/regions that ordinary depth cannot resolve.
- [ ] Repeat under bounded camera jitter and a near-view movement control.
- [ ] Retain a negative control where partial removal is deliberately required.

### Decision gate

- **Pass:** whole-contribution admission produces the required final semantics;
  proceed to E1M1.
- **Fail:** one contribution demonstrably requires partial survival; stop B/C
  promotion and return the evidence to AR-0030. View-local fragmentation has
  then earned investigation.

Increasing source observation resolution, reconstructing geometry, or patching
the offending contribution is not an allowed repair in this slice.

## Slice 4 — E1M1 Source-Topology Candidate

### Deliverables

- [ ] Run B at every retained canonical pose with A available side-by-side.
- [ ] Verify spawn-room floor/wall and ceiling/wall continuity.
- [ ] Verify close walls remain present and free look exposes no finite view
      rectangle.
- [ ] Verify the hut remains intact while source-invalid far sectors no longer
      appear through terminal sky presentation.
- [ ] Verify the first door, moving platform/elevator, green-room one-sided
      cutout, and EXIT observations retain their demonstrated semantics.
- [ ] Make the debug console report `admission`, `admission-reason`, and
      `generic-filter` separately for `LOOK` hits.
- [ ] Retain contribution waterfalls and bounded rejection samples for every
      pose.
- [ ] Measure first/warm frame work, preparation time, draw count, resource
      uploads/replacements, and admission churn during movement.

### Acceptance criteria

- No cracks, missing adjacent floors/ceilings, disappearing close walls, or
  finite prepared view box occur relative to A.
- No source-invalid far geometry is visible through the canonical hut/sky
  observations.
- Door/platform state changes recompute bounded admission without rebuilding
  unchanged original meshes.
- Any unresolved contribution remains visible and diagnostically fail-open.

## Slice 5 — Generic Conservative Post-Filter

Begin only after B passes Slice 4.

### Deliverables

- [ ] Feed B's admitted, unchanged contributions to the existing conservative
      AABB/frustum selector.
- [ ] Preserve B survivor order and admission provenance through C.
- [ ] Attribute every removed contribution specifically to the generic stage.
- [ ] Compare B/C at the canonical poses, during camera motion, and after
      dynamic snapshots change.
- [ ] Retain selection CPU time, submitted draw reduction, and warm-frame
      resource churn.

### Acceptance criteria

- C is visually and structurally equivalent to B at every required observation.
- Generic rejection produces no false negatives and never repairs a B defect.
- Static warm-frame resources do not upload or replace because of either stage.
- AABB/frustum remains generic and acquires no Doom admission reasons.

## Slice 6 — Native And Browser Parity

- [ ] Provide the same A/B/C strategy selection in the Rust-owned Browser
      WebGPU fixture without moving WAD parsing or admission semantics into
      TypeScript.
- [ ] Retain equivalent contribution counts, reason categories, strategy names,
      runtime-snapshot identities, and structural hashes on native and browser.
- [ ] Capture bounded native and browser visual observations for the synthetic
      partial-survival fixture and the canonical E1M1 hut/sky pose.
- [ ] Record adapter/backend/build metadata and avoid pixel-identity claims.
- [ ] Confirm camera jitter causes no resource churn or admission instability
      beyond explicitly changed source relationships.

## Slice 7 — Decision And Closeout

- [ ] Produce an A/B/C matrix covering correctness, continuity, source
      fidelity, diagnostics, resource churn, CPU cost, draw reduction, native/
      browser parity, and implementation complexity.
- [ ] State whether whole original contribution admission is sufficient.
- [ ] Update AR-0030 with the result and its effect on Alternative F.
- [ ] Update the synthetic conformance study so the failed inverse-world
      realization and this candidate cannot be confused.
- [ ] Update the DOOM WAD Checklist with the accepted/parked next action.
- [ ] Park or remove experimental implementation only after retaining its
      evidence and preserving A as an explicit fallback.

## Validation Commands

Run proportionately after each compileable slice:

```powershell
cargo fmt --all
cargo test -p hello-doom-visibility-conformance
cargo test -p hello-doom-e1m1 --bin static_scene
cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
cargo clippy -p hello-doom-e1m1 --bin static_scene -- -D warnings
```

For native E1M1, use the documented canonical package invocation plus the
long-form `--render-strategy` selected by the slice. Browser commands must be
recorded in the relevant fixture README once implemented; a successful WASM
compile is not browser execution evidence.

## Stop And Escalate Conditions

Return to AR-0030 before continuing when:

- correct final presentation requires partial removal of one original
  contribution;
- admission requires renderer-owned Doom topology, callbacks, or mutable scene
  state;
- a generic selector must understand Doom admission reasons;
- B changes geometry, material semantics, UVs, ordering, or renderer resources;
- a source ambiguity cannot remain fail-open without invalidating correctness;
- native and browser require different semantic admission results; or
- measured preparation/churn materially violates ADR-0008 expectations.

Ordinary implementation defects, missing diagnostics, fixture wiring, and
local source-correlation gaps may be repaired within this plan while the hard
invariants remain intact.

## Completion Criteria

This study completes with one of two honest results:

1. **Sufficient:** B removes source-invalid participation while preserving the
   complete-geometry continuity of A, and C safely composes afterward. AR-0030
   gains executable evidence for source preparation followed by ordinary
   conservative selection and rendering.
2. **Insufficient:** a retained falsifier proves whole original contribution
   admission cannot express the required view. AR-0030 gains evidence that
   partial/view-local presentation is earned complexity rather than a design
   preference.

Neither result admits a public topology filter, visibility provider, or render
framework by itself.

## Related Records

- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Architectural Reviews/AR-0030-source-owned-presentation-preparation-boundary.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0013-caller-declared-categorical-cutout-surfaces.md`
- `docs/ADR/ADR-0015-source-unit-cohesion-size-pressure-and-decomposition.md`
- `docs/Plans/DOOM/Studies/Doom viewer-relative presentation synthetic conformance.md`
