# Categorical Cutout Capability Admission

## Purpose

Implement the narrow caller-declared categorical-cutout capability accepted by
ADR-0013. This plan turns AR-0023's corpus findings into a small stable
renderer surface without promoting continuous blending, source interpretation,
or a shared alpha-policy abstraction.

## Boundaries

The admitted capability is exactly:

- a caller-declared finite `[0.0, 1.0]` threshold;
- an explicit `below` versus `at-or-below` categorical comparison;
- discard of fragments that fail that declaration; and
- ordinary opaque textured-3D depth testing and depth writing for retained
  fragments.

It does not infer intent from pixels or asset formats. It does not admit a
`Blend` sibling, `AlphaPolicy`, public shader authoring, PBR, sorting,
alpha-to-coverage, or a material graph.

## Slice 0 — Freeze the admission boundary

- [x] Record ADR-0013 as Accepted.
- [x] Record AR-0023's split disposition: Cutout admitted for implementation;
      Blend remains incubating.
- [x] Update the SDD's textured-3D boundary.
- [x] Add the concrete public-vocabulary proposal to the ADR-0008/0009 ledger
      before implementation starts.

Acceptance: the plan names only categorical coverage and cannot be read as a
general alpha/material contract.

## Slice 1 — Checked renderer vocabulary

- [x] Introduce the smallest provider-neutral cutout declaration in
      `tokimu-render`; it requires threshold and comparison explicitly.
  - [x] `CategoricalCutout`, `CutoutThreshold`, and `CutoutComparison` are
        dedicated Cutout vocabulary rather than variants of `BlendMode`.
- [x] Reject non-finite and out-of-range thresholds through typed construction
      or typed pipeline validation before backend submission.
- [x] Make a textured-3D cutout declaration select opaque color output and
      ordinary depth-test/depth-write behavior explicitly.
- [x] Preserve existing opaque textured-3D behavior and keep the current blend
      mechanism outside the new public capability.
- [x] Add focused public API and negative tests.

Acceptance: no caller can obtain cutout through source alpha, a default magic
threshold, or an unvalidated raw float.

## Slice 2 — WGPU realization and bounded performance

- [x] Add a dedicated provider implementation for the admitted cutout
      declaration; do not generalize it into an alpha-policy switch.
- [x] Implement both explicit comparison forms in the textured fragment path.
- [ ] Keep pipeline/material cache keys complete for the new declaration and
      retain a diagnostic for unsupported or malformed state.
- [ ] Measure first and warm fixed-scene observations; show no avoidable
      steady-state allocation or per-draw source-alpha classification.
- [ ] Run renderer focused tests and strict focused Clippy.

Acceptance: retained fragments use normal opaque depth behavior, invalid state
does not reach the backend silently, and warm-frame observations are recorded.

## Slice 3 — Cross-target conformance and failure containment

- [x] Replace corpus-local cutout WGSL in the shared alpha fixture with the
      admitted renderer capability.
- [ ] Retain the exact threshold-boundary, zero, one, invalid-threshold, and
      opaque-control regressions.
- [ ] Retain depth/order regressions proving opaque and cutout remain
      order-invariant under ordinary depth behavior.
- [ ] Capture native WGPU and browser/WebGPU observations with target/adapter
      metadata; do not claim pixel identity.
- [ ] Verify failure/recovery behavior in line with ADR-0009.

Acceptance: one fixed source demonstrates that policy is caller intent on both
targets, while malformed declarations are visible and contained.

## Slice 4 — Independent real caller and closeout

- [x] Migrate E1M1's 26 source-selected masked-middle candidates from its
      custom cutout shader to the admitted generic capability in both native
      and browser/WASM callers.
- [x] Preserve Doom's source selection and zero-alpha classification outside
      `tokimu-render`; the focused E1M1 suite and browser `wasm32` check pass.
  - [x] Retain a fresh browser/WebGPU visual observation of the admitted path:
        the reviewed package presents `1835` opaque and `1861` cutout draws at
        `960x600`; pre-admission captures remain historical evidence only.
  - [x] Retain a comparable native Vulkan/AMD visual observation at the fixed
        source-spawn camera: 1,835 opaque plus 26 cutout draws, with the window
        reporting 1,861 submitted draws.
- [ ] Update AR-0023, the DOOM checklist, gate ledger, and SDD with final
      evidence and explicit remaining Blend limits.
- [ ] Complete the ADR-0008/0009 ledger, including any locally justified
      `N/A` entries, and run proportional workspace validation.

Acceptance: E1M1 supplies an independent real caller without WAD vocabulary
crossing the renderer boundary; Cutout may then be marked implemented.

## Parking Criteria

Stop and reopen architectural review if implementation requires a blended
ordering contract, a shared alpha taxonomy, source-format terms in the
renderer, renderer-owned scene sorting, a public shader-resource API, or an
unbounded material system.

## Evidence

- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Plans/alpha-policy-slice6-gate-review.md`
- `docs/Plans/textured-surface-alpha-policy-comparative-corpus.md`
- `docs/Plans/DOOM/E1M1 masked-middle cutout intake evidence.md`
