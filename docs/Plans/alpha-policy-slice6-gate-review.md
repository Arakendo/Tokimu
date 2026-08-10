# Alpha-Policy Slice 6 Gate Review

Date: 2026-08-09  
Scope: AR-0023 Slice 6 pre-admission review and ADR-0013 Cutout admission.  
Status: Cutout is accepted for implementation; Blend remains incubating. The
concrete Cutout implementation gate is open.

## Classification

The shared fixtures and E1M1/GLB callers are Outer Ring corpus consumers. The
WGPU backend is an Outer Ring provider. A future provider-neutral public alpha
declaration in `tokimu-render` would cross an established capability boundary,
so this review applies ADR-0008 and ADR-0009's **full** gates to any such
proposal.

No candidate implementation has been added to that public boundary. Existing
`BlendMode`, `PipelineRenderState`, and custom WGSL are pre-existing renderer
mechanisms. The corpus uses them as experimental machinery; their presence does
not constitute an admitted textured-3D alpha capability.

## Candidate Status

| Candidate | Stable crossing proposed? | Full gate status | Local conclusion |
| --- | --- | --- | --- |
| Opaque control | No change | N/A | Remains the ordinary textured-3D control. |
| Cutout threshold/discard | Yes — ADR-0013 | Implementation gate open | The admitted crossing is caller-supplied finite threshold/comparison plus categorical discard and ordinary opaque depth behavior. Concrete API, measurements, migration, and target evidence remain required. |
| Blended 3D surface | No | Pre-admission evidence retained; implementation gate pending | Existing `AlphaBlend` realizes mechanics, but a stable capability would need explicit ordering/depth ownership. |
| Shared alpha-policy enum | No | Rejected before implementation | The callers have different validation, depth, and ordering obligations; compact syntax would hide that difference. |

`N/A` here does not waive future review. It means the ADR-0008/0009 full gate
cannot be completed honestly until there is a concrete implementation to test.

## ADR-0008: Performance And Code-Quality Ledger

### Concrete Cutout vocabulary proposal

Slice 1 will use a dedicated `CategoricalCutout` declaration containing a
checked `CutoutThreshold` and `CutoutComparison`. A
`Pipeline::textured_3d_cutout(...)` constructor will be the only admitted
entry point: it selects a dedicated textured-3D cutout pipeline, opaque color
output, and ordinary depth test/write behavior. `BlendMode` remains unchanged
and is not a parameter, variant, or fallback of this declaration.

The pipeline owns the resulting shader specialization, so the caller does not
receive a public shader authoring or backend binding interface. The threshold
is checked at declaration construction; no per-frame source-pixel scan or
per-draw threshold validation is planned.

| Gate concern | Evidence or local reason |
| --- | --- |
| Ownership / vocabulary | ADR-0013 admits Cutout only. It separates caller selection, asset pixels, renderer validation/realization, and provider execution. The implementation must not reuse this narrow vocabulary as a general alpha enum. |
| Concrete declaration | `CategoricalCutout` contains checked `CutoutThreshold` and `CutoutComparison`; `Pipeline::textured_3d_cutout` is the explicit entry point. It fixes opaque blend plus ordinary depth test/write and specializes renderer-owned WGSL. `BlendMode` remains untouched. |
| Hot-path shape | Candidate selection, WAD composition, alpha classification, and shader construction occur before frame submission in corpus preparation. The steady-state draw paths submit prebuilt meshes/materials/pipelines; they do not scan source alpha or parse source formats to choose policy. |
| Allocation / upload evidence | Shared native/browser Blend fixtures retain first-versus-warm counters. GLB fixed capture retains two warm draws with zero binding allocations, uniform writes, or mesh uploads. These are workload observations, not a universal performance claim. |
| Native and WASM | The shared fixture is observed on native AMD/Vulkan and browser/WebGPU. E1M1 cutout is observed on browser/WebGPU and native startup. GLB is native-only; its browser absence is explicit. NVIDIA remains uncovered. |
| Determinism | Fixtures lock source hashes, camera/transforms, draw identities/order, thresholds, and scene manifests. Visual images are observations, not cross-GPU pixel goldens. |
| Bounded work | Corpus input limits, fixed fixtures, retained candidate counts, and Resource Space limits bound the tested paths. No queue, cache, sorting service, or diagnostic stream was added by alpha policy work. |
| Code hygiene | `cargo fmt --all`, focused tests, WASM build/binding generation, and TypeScript compilation have passed for the touched corpus paths. The repository-wide third-party `glam` warning condition remains separately tracked by AR-0019; it is not represented as an alpha-policy lint pass. |
| Measurement exception | The pre-admission observations are not a stable-API cost claim. Cutout implementation must measure registration, pipeline variants, steady-state allocation/upload behavior, native, and sequential WASM consequences; Blend has no admission gate because it remains incubating. |

## ADR-0009: Verification, Failure, And Recovery Ledger

| Gate concern | Evidence or local reason |
| --- | --- |
| Unit / contract evidence | The alpha oracle tests threshold equality, zero/one boundaries, discarded-depth behavior, malformed RGBA8, invalid thresholds, and invalid/missing/duplicate blend order. E1M1 retains a regression proving source-classified `BROWNGRN` is selected despite fully covered pixels. GLB tests its fixed transparent entry and depth-write-disabled state. |
| Corpus composition | Shared native/browser fixture matrix, E1M1 package evidence, and the independent GLB caller retain what each target proves. Package provenance and BLAKE3 identity are recorded for the reviewed Doom archive. |
| Backend failure / recovery | The alpha fixture retains invalid custom-shader/provider rejection and an invalid depth-write request before valid setup; native and browser then present valid frames. AR-0024 separately retains the resolved clip-depth provider mapping failure. |
| Diagnostic visibility | Browser readiness/device/presentation stages remain separate. Valid runs report `diagnostic=none`; failed candidate construction is rejected rather than falling back to opaque. GLB initialization now records backend/device/adapter. |
| Containment | Opaque E1M1 presentation is separate from masked-cutout preparation/pipeline registration. Browser opaque and cutout requests are separate; a cutout failure cannot silently alter the opaque action. Missing material and invalid state failures return rather than becoming permissive substitution. |
| Recovery scope | The corpus proves recovery from its deliberately rejected setup to a following valid frame. It does **not** claim driver-loss, device-loss, process-crash, renderer readback, or general transparent-scene recovery. Those are N/A because no stable alpha contract or recovery service is proposed. |

## Separate Versus Common API Pressure

The comparison rejects a common public switch at this stage:

```text
Cutout:
  caller threshold + comparison
  discard is categorical
  retained fragments may write depth
  no transparent ordering contract

Blend:
  continuous source contribution
  caller-visible depth-write choice
  caller submission order is material to correctness
  no renderer-owned sorting
```

The only superficial commonality is consumption of RGBA source data. That is
insufficient to justify shared ownership, validation, or lifecycle vocabulary.

## Slice 6 Result

The maintainer selected bounded Cutout and ADR-0013 records it. The study
rejects a shared policy enum and leaves Blend incubating. Before Cutout is
implemented, update this ledger with the concrete type/contract, full
performance measurements, target matrix, public API/migration review, and the
exact ADR-0008/0009 answers that change from `N/A` to satisfied or blocked.

## Focused Validation Results

The following commands passed on 2026-08-09:

```powershell
cargo test -p hello-alpha-policy
cargo test -p hello-alpha-policy --features native-visual
cargo test -p tokimu-render
cargo check -p hello-alpha-policy-web --target wasm32-unknown-unknown
cargo check -p doom-ts-boundary-workbench-engine --target wasm32-unknown-unknown
cargo clippy -p hello-alpha-policy --features native-visual --all-targets -- -D warnings
cargo clippy -p tokimu-render --all-targets -- -D warnings
git diff --check
```

The headless alpha package runs 18 tests. With `native-visual`, it additionally
compiles/tests three native target binaries (25 focused tests in total). The
renderer package now runs 61 tests, including Cutout declaration, validation,
comparison, shader-specialization, and fixed-depth-state cases. The one
discovered packaging defect was that
two auto-discovered native visual binaries did not declare the optional
`native-visual` feature; their explicit `[[bin]]` declarations now make the
headless and visual validation modes honest. Strict Clippy then found and the
corpus repaired three runtime assertions over literal depth constants by making
them compile-time invariants.

## References

- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/ADR/ADR-0013-caller-declared-categorical-cutout-surfaces.md`
- `docs/Plans/categorical-cutout-capability-admission.md`
- `docs/Plans/textured-surface-alpha-policy-comparative-corpus.md`
- `docs/Plans/alpha-policy-real-caller-comparison.md`
