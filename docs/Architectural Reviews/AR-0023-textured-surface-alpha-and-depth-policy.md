# AR-0023: Textured Surface Alpha And Depth Policy

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | 2026-08-09 |
| Last reviewed | 2026-08-09 |
| Scope | Cross-cutting renderer/material and backend boundary |
| Trigger | AR-0022 established a narrow opaque textured-mesh path, while its alpha audit found that `Textured3d` combines source-alpha blending with depth writes and has no cutout threshold or ordering policy. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0008, ADR-0009 |
| Related evidence | AR-0006, AR-0022 Cycle 10, `tokimu-render` pipeline/shader/backend code, DOOM Slice 5B source coverage facts |
| Admission exception | None |

## Architectural Question

What is the smallest provider-neutral alpha and depth policy, if any, that
`tokimu-render` should admit for textured 3D surfaces without treating encoded
image alpha or Doom masked-middle coverage as renderer-owned semantics?

## Context

The first textured Box corpus profile is intentionally opaque. Its PNG inputs
contain no transparency, and AR-0022 correctly proved supplied UV and sampler
behavior without making an alpha claim.

The renderer already carries two relevant mechanisms: textured shaders preserve
the sampled alpha channel, and `BlendMode::AlphaBlend` maps to conventional
WGPU source-alpha blending. The default `Textured3d` state, however, is
`depth_writing_3d()`, which retains depth writes while using `AlphaBlend`.
That does not define ordering or depth semantics for general transparent
surfaces. Cutout/alpha test has no threshold vocabulary at all.

DOOM adds real future pressure: raster decoding can retain covered pixels as
straight alpha, but a masked middle texture is a source/application fact, not a
license for the renderer to infer discard, blend ordering, or a magic threshold.

## Trigger And Evidence

- Corpus examples:
  - `hello-textured-box` and `hello-textured-box-web` use an opaque initial
    profile and deliberately defer alpha.
  - DOOM Slice 5B needs to classify masked middle behavior before static E1M1
    presentation, but has not selected its policy.
- Automated tests:
  - RGBA8 validation preserves payload bytes; material/pipeline tests cover
    declared blend state and WGPU maps `AlphaBlend` to `ALPHA_BLENDING`.
  - No corpus test proves correct transparent ordering, depth-write behavior,
    or cutout threshold behavior.
- Audits or diagnostics:
  - AR-0022 Cycle 10 identifies the blend-plus-depth-write combination as an
    explicit blocker, not a failure of PNG decoding.
- Missing evidence:
  - A real transparent/cutout source fixture with documented intent.
  - A caller-defined rule for cutout threshold or transparent draw ordering.
  - Native and browser presentation evidence for any selected policy.

## Ownership Analysis

- Asset/format providers own decoded straight-alpha pixels and source coverage
  observations. They do not choose GPU discard or ordering.
- A corpus application owns whether its selected material requests an admitted
  generic alpha policy and owns render ordering where the policy requires it.
- `tokimu-render` may own only a bounded provider-neutral pipeline/material
  declaration and validation for an admitted policy, plus backend realization.
- The WGPU backend owns blend/depth state realization, not alpha semantics.
- Doom owns classification of masked middles, palette selection, and any
  original-behavior claim. The renderer must not receive WAD terms or infer
  Doom policy from alpha bytes.
- This boundary must not become a general material graph, order-independent
  transparency system, source-format alpha heuristic, or renderer-owned scene
  sorting service.

## Dependency Direction

```text
Decoded source alpha / source coverage
        ↓ observed fact
Corpus application selects an admitted generic policy and draw order
        ↓ provider-neutral declaration
tokimu-render validates policy → WGPU realizes blend/depth state
```

No PNG, GLB, WAD, palette, or coverage-provider type crosses into
`tokimu-render`.

## Alternatives Considered

### Alternative A: Keep Textured3d Opaque Only

- Benefits: preserves current demonstrated semantics; no ordering or threshold
  contract is implied.
- Costs: transparent and masked surfaces remain unavailable to future callers.
- Failure mode: callers create ad hoc shaders or silently misuse alpha blend.

### Alternative B: Admit A Bounded Cutout Policy

- Benefits: potentially fits masked/categorical surfaces without transparent
  ordering; can retain depth writes.
- Costs: requires a precise, provider-neutral threshold and source fixture;
  threshold selection can accidentally encode source-format or Doom behavior.
- Failure mode: a universal-looking threshold becomes a hidden asset policy.

### Alternative C: Admit A Bounded Blended-Surface Policy

- Benefits: preserves continuous source alpha for a real transparent caller.
- Costs: must explicitly define depth-write and draw-order responsibility;
  needs native/browser evidence.
- Failure mode: a convenient pipeline default is mistaken for correct
  transparency ordering.

### Alternative D: Admit General Transparency/Material Infrastructure Now

- Benefits: broad future flexibility.
- Costs: no current evidence supports sorting services, OIT, material graphs,
  texture arrays, or source-format rules.
- Failure mode: solving an unspecified future renderer instead of one caller.

## Findings

1. Straight alpha is an input fact, not a rendering-policy decision.
2. The existing `AlphaBlend` backend mapping is implementation capability, not
   sufficient semantic proof for transparent 3D surfaces.
3. Opaque textured rendering is independently useful and must remain usable
   without this review choosing alpha behavior.
4. Cutout and blended surfaces are separate alternatives: one need not admit
   the other.
5. If cutout is admitted, its discard threshold must be a caller-declared
   generic input or be justified by the caller's explicit contract. The
   renderer must not silently canonize a conventional value such as `0.5`.
6. If blending is admitted, the policy must state whether the application
   orders blended draws or the renderer owns an explicit sorting contract.
   There is no meaningful implicit middle ground.
7. The current evidence does not justify changing the `Textured3d` default or
   adding a source fixture yet.

## Disposition

Proposed. Retain opaque `Textured3d` as the only demonstrated corpus profile.
Do not use source alpha to select blending or discard automatically. This review
will choose, defer, or reject a bounded cutout or blended-surface policy only
when a real caller supplies its required semantics and focused evidence.

## Consequences

- AR-0022 can progress and receive a UV/sampler decision independently of
  alpha.
- DOOM Slice 5B must keep masked-middle behavior visibly deferred until this
  review records a decision that actually fits it.
- Any future policy must provide typed validation and retain native/browser
  presentation evidence; a decoder result alone is insufficient.

## Required Follow-Up

- [ ] Identify the first real caller: cutout, blended surface, or neither.
- [ ] If cutout is proposed, state a provider-neutral threshold policy and add
      a small documented alpha fixture.
- [ ] If blending is proposed, state draw-order and depth-write responsibility
      and add native/browser conformance evidence.
- [ ] Create or revise an ADR only if this review accepts a binding renderer
      contract.

## Reopening Triggers

- a corpus consumer needs visible transparent or categorical masked pixels;
- DOOM Slice 5B selects a concrete masked-middle requirement;
- a native/WASM backend cannot preserve a chosen bounded policy;
- source alpha starts being used as an implicit blend/cutout selector; or
- a cutout threshold is silently assumed rather than declared by a caller; or
- blended draws rely on unspecified ordering or depth-write behavior; or
- a simpler opaque-only decomposition satisfies the caller.

## Review History

### Cycle 1 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: AR-0022 Cycle 10's alpha implementation audit; current
  `Textured3d` shader and render-state implementation.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: the alpha data path is explicit, but transparent/cutout semantics
  are not selected; opaque presentation remains the honest current profile.
- Disposition: Proposed; no renderer contract is admitted.
- Resulting ADR or documentation change: opened this record and linked it from
  AR-0022 and the DOOM Slice 5B plan.

### Cycle 2 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: independent architectural review of the initial record.
- Findings: cutout and blending are not only different implementation paths;
  they create different ownership questions. A cutout threshold cannot become
  canonical merely through convention, and blending must explicitly allocate
  draw-order responsibility to either the application or a renderer sorting
  contract.
- Disposition: retain Proposed and the opaque-only current profile. The review
  now records threshold ownership and blended-draw ordering as mandatory
  admission questions rather than downstream implementation details.
- Resulting ADR or documentation change: none.

## References

- `docs/Architectural Reviews/AR-0006-raster-image-requirement-pipeline.md`
- `docs/Architectural Reviews/AR-0022-textured-mesh-coordinate-and-sampling-boundary.md`
- `docs/Plans/textured-box-glb-png-corpus.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `crates/tokimu-render/src/pipeline.rs`
- `crates/tokimu-render/src/wgpu_backend/pipeline_support.rs`
