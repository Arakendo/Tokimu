# Checkpoint: Doom Ordered Presentation

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Campaign | DOOM WAD / E1M1 viewer-relative presentation |
| Active plan | `docs/Plans/DOOM/Studies/Doom ordered source occurrence preparation.md` |
| Active review | `docs/Architectural Reviews/AR-0030-source-owned-presentation-preparation-boundary.md` |
| Repository baseline before this checkpoint | `61588c3` |
| Current phase | Slice 6B structural proof complete; integrated visual, live-camera, and browser gates open |

## Resume Here

The current leading dataflow is:

```text
Doom source + explicit runtime snapshot + current view
    -> Doom-private ordered BSP/SEG preparation
    -> exhaustive source-contribution dispositions
    -> surviving whole/partial ordinary Tokimu declarations
    -> prepared full submission to tokimu-render
    -> optional generic conservative filtering only after correctness
```

Do not resume by repairing the legacy screen-column candidate, adding another
world-space sky occluder, or applying AABB/frustum filtering to hide preparation
defects. Those paths answer different questions and have already produced
misleading visual improvements.

The immediate task is to make the structurally proven Slice 6B result a
composition-local, live-camera preparation used consistently by native and
browser callers without duplicating Doom semantics.

## What The Evidence Currently Says

### Established

- Global full submission preserves the complete reconstructed scene and free
  movement, but remote or lazily authored map geometry can appear through sky
  regions because Doom never treated the WAD as a globally visible watertight
  shell.
- Classic Doom visibility is a viewer-relative ordered source protocol. BSP/SEG
  traversal updates horizontal and vertical coverage; floor, ceiling, and sky
  presentation consume that coverage. Sky is not itself the authoritative
  world-space occluder.
- The private ordered source protocol is structurally coherent after correcting
  translation defects. Its fixed source-spawn ledger has no overlapping writes
  or unresolved plane instances.
- Slice 6B now assigns every original wall and plane contribution exactly one
  terminal disposition: whole retained, terminal rejected, partial SEG,
  partial plane, or explicit unresolved fail-open.
- Terminally rejected contributions cannot re-enter the final declaration list.
  Partial contributions are lowered through Doom-private realization code, and
  final handoff conservation is checked.
- Six deterministic rays balance structurally with no generic filter. Five
  known-invalid contributions reach terminal rejection with zero final
  declarations; the retained partial ceiling case produces eight final
  declarations.
- Submission-local geometry (G2) is feasible through a corpus-only unstable
  renderer seam on native and browser WebGPU. Repeated submissions reuse local
  slots under distinct submission identities, invalid state is rejected and
  recovery succeeds, and persistent mesh identity remains separate.

### Falsified or parked

- A depth-bearing world-space sky wall is not a valid general solution. It can
  clip valid hut and building geometry even when its projected coverage matches
  a source ledger.
- Source-height sky tiles alone do not restore Doom visibility. They can either
  mask valid nearby geometry or allow unrelated geometry that should never have
  survived ordered preparation.
- Legacy SEG/screen-column reconstruction is not a free-look presentation
  strategy. It creates a fixed view window, loses floor/ceiling coverage at
  boundaries, and can make nearby walls vanish.
- Whole-object or whole-mesh Boolean admission is too coarse for contributions
  that survive only over bounded source intervals.
- Generic AABB/frustum selection is an optimization candidate, not the source
  of Doom presentation correctness.

### Not yet proven

- The current Slice 6B declaration set is visually complete at source spawn,
  hut/window, exterior-hut, first-door, moving-platform, green-room cutout, and
  EXIT controls.
- Preparation refreshes correctly as the camera moves. The present integrated
  path prepares once at startup and intentionally suppresses ordinary movement.
- Dynamic door and platform snapshots flow through the same live preparation
  lifecycle in E1M1 rather than only synthetic controls.
- Browser E1M1 consumes the same Doom-private preparation implementation. The
  browser application is presently a separate consumer, so browser structural
  parity is still open.
- Ordinary view-local triangles are sufficient for every surviving Doom
  contribution. G2 is demonstrated mechanism, not an admitted stable contract.

## Current Structural Ledger

The retained Slice 6B E1M1 evidence reports:

```text
source SEGs:                 732
whole retained:              16
partial:                     16
terminal rejected:          563
unresolved fail-open:       137

wall source triangles:      303
wall declarations:          321

plane source triangles:     304
plane survivors:             72
plane rejected:             211
plane fragments:            166
plane declarations:         136
degenerate omissions:        30

final opaque declarations:  445
final cutout declarations:    12
generic-stage rejections:      0
```

These counts prove conservation and disposition, not final visual correctness.

## Important Implementation And Evidence Locations

- Active plan: `docs/Plans/DOOM/Studies/Doom ordered source occurrence preparation.md`
- Slice 6B evidence: `docs/Plans/DOOM/Evidence/Doom E1M1 Slice 6B literal handoff evidence.md`
- Classic renderer dataflow: `docs/Plans/DOOM/Evidence/Classic Doom renderer dataflow and Tokimu preparation seam.md`
- Classic clipping evidence: `docs/Plans/DOOM/Evidence/Classic Doom visibility clipping evidence.md`
- G2 evidence: `docs/Plans/DOOM/Evidence/Doom AR-0030 G2 submission-local geometry evidence.md`
- Doom-private occurrence realization: `corpus/campaigns/doom/hello-doom-e1m1/src/bin/static_scene/presentation/ordered_occurrence.rs`
- Strategy entry point: `corpus/campaigns/doom/hello-doom-e1m1/src/bin/static_scene/render_strategies/ordered_occurrence_prepared_full.rs`
- Deterministic reports: `corpus/campaigns/doom/hello-doom-e1m1/src/bin/static_scene/diagnostics/source_reports.rs`
- Unstable G2 semantic seam: `crates/tokimu-render/src/experimental_submission_local_geometry.rs`
- WGPU G2 realization: `crates/tokimu-render/src/wgpu_backend/experimental_submission_local_geometry.rs`

## Next Steps, In Order

1. Extract or expose one Doom-private preparation unit that both the native and
   browser corpus callers can invoke. Do not copy the preparation semantics into
   TypeScript or a second Rust implementation.
2. Define a composition-local refresh lifecycle:
   current camera plus explicit runtime snapshot -> immutable prepared result ->
   presentation submission -> safe retirement of the previous result.
3. Re-run the six rays through that live seam, then validate source spawn,
   hut/window, exterior hut, first door, moving platform, green-room cutout, and
   EXIT. Retain conservation and named fail-open reasons at every pose.
4. Exercise free look, near-wall movement, and bounded camera jitter. A fixed
   view window, disappearing nearby walls, cracks, or reappearing terminally
   rejected contributions are correctness failures.
5. Establish browser/WASM structural parity using the same Rust-owned
   preparation. Retain target metadata without claiming pixel identity.
6. Only after prepared full submission is correct, run the existing generic
   AABB/frustum selector on its output as the AR-0030 Alternative-F experiment.
   Attribute generic removals separately and preserve source order.
7. Update AR-0030 and the DOOM checklist only after the native/browser matrix
   supports a disposition. Do not stabilize G2 or a renderer preparation API
   from Doom evidence alone.

## Restart Commands

Focused native tests:

```powershell
cargo test -p hello-doom-e1m1 --bin static_scene --quiet
```

Slice 6B structural report:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene --quiet -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=ordered-occurrence-prepared-full `
  --ordered-occurrence-prepared-report `
  --no-walk-collision
```

Six-ray reconciliation:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene --quiet -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=ordered-occurrence-prepared-full `
  --ordered-occurrence-six-ray-report `
  --no-walk-collision
```

The strategy currently uses a fixed reconstruction camera. Treat the launched
window as a bounded visual control, not as evidence that live movement works.

## Validation State At Checkpoint

- `cargo test -p hello-doom-e1m1 --bin static_scene --quiet`: 67 passed.
- Canonical ordered-occurrence structural and six-ray reports: passed and
  balanced.
- `git diff --check`: clean before checkpoint documentation.
- Focused Clippy is not clean because neighboring experimental code currently
  triggers `clippy::large_enum_variant` in
  `hello-doom-visibility-conformance/src/relational_classifier.rs`. Do not
  misattribute that existing warning to Slice 6B.

## Stop And Escalate

Return for architectural judgment if live realization requires any of the
following:

- Doom semantics in `tokimu-render` or `tokimu-platform`;
- a stable/public renderer contract not already admitted;
- renderer-owned scene topology, visibility policy, or persistent simulation
  state;
- browser duplication of Doom preparation authority;
- source contributions that cannot be conserved through whole, rejected,
  partial, or explicit fail-open dispositions;
- evidence that ordinary declarations and the bounded G2 experiment cannot
  represent the required surviving contributions.

