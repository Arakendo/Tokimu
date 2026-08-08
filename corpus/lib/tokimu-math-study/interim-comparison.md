# Interim Alternative Comparison

| Field | Value |
| --- | --- |
| Status | Interim snapshot; not a selection recommendation |
| Date | 2026-08-07 |
| Scope | Current 0.1 manifest and corpus fixtures only |
| Deferred pressure | Revisit after `docs/Plans/DOOM/DOOM WAD Checklist.md` completes |

| Dimension | A — Direct `glam` | B — Provider-backed vocabulary | C — Narrow owned implementation | D — Bounded derivation |
| --- | --- | --- | --- |
| Public vocabulary owner | `glam` | Candidate Tokimu names | Candidate Tokimu names | Candidate Tokimu names |
| Current mechanics | Pinned local `glam` | Private pinned `glam` fields | Original scalar Rust | Upstream-derived scalar Rust |
| Implemented study scope | Five re-exported types | Five names; caller-pressured vector/matrix behavior | `Vec3`, `Vec4`, `Mat4`; `Vec2`/`Quat` absent | `Vec3` only; expansion paused |
| Provider dependency in candidate source | Direct | Yes, private | None | None |
| Shared conformance | Control baseline | Vector and transform cases | Vector and non-singular transform cases | Vector cases only |
| Affine inverse sweep | Four deterministic round trips | Matches A within 3e-5 | Matches A and original points within 3e-5 | Not applicable: no `Mat4` |
| Deterministic affine differential sweep | 96 bounded non-singular cases | Matches A within 1e-3 | Matches A within 1e-3 | Not applicable: no `Mat4` |
| Finite camera/projection differential sweep | 128 bounded cases | Matches A within 1e-4 | Matches A within 1e-4 | Not applicable: no `Mat4` |
| Degenerate matrix behavior | Observed baseline non-finite masks | Matches A for degenerate view and singular inverse | Matches A for degenerate view and singular inverse | Not applicable: no `Mat4` |
| Singular inverse behavior | Provider behavior not selected as Tokimu contract | Provider behavior not selected as Tokimu contract | All-NaN provisional experiment output | Not implemented |
| Transform workload | Baseline checksum | Same checksum | Same checksum | Not applicable: no `Mat4` |
| Transform allocation observation | 0 in retained workload | 0 in retained workload | 0 in retained workload | Not measured |
| Repeated transform observation | 317,300 / 351,600 / 571,200 ns (min / median / max) | 309,700 / 357,000 / 683,600 ns | 286,300 / 290,200 / 405,500 ns | Not applicable |
| Candidate-isolated release link output | 123,904 bytes | 123,904 bytes | 124,416 bytes | Not applicable |
| Isolated fresh release build observation | Not measured | 3.518 s; compiles pinned `glam` | 0.322 s; no dependencies | Not measured |
| Native layout observation | `Vec4`/`Mat4`: 16-byte aligned | Matches A for all retained types | `Vec4`/`Mat4`: same size but 4-byte aligned | `Vec3` only |
| Isolated WASM layout observation | `Vec4`/`Mat4`: 16-byte aligned | Matches A: 16-byte aligned | `Vec4`/`Mat4`: 4-byte aligned | Not applicable: no `Mat4` |
| Renderer-boundary shape | Native provider value | Private unwrap in adapter; two more at current `tokimu::Camera` handoff | Reconstruct provider matrix from column array; two more at current `tokimu::Camera` handoff | Not implemented |
| Renderer-boundary allocation observation | N/A | 0 in fixture | 0 in fixture | Not measured |
| Imported-scene transform evidence | Control path | Matches A on pinned Khronos `Box.glb` positions/normals | Matches A on pinned Khronos `Box.glb` positions/normals | Not applicable: no `Mat4` |
| CAD cursor-ray evidence | Control path | Matches A for homogeneous `Mat4 * Vec4`, divide, and ray rejection | Matches A for homogeneous `Mat4 * Vec4`, divide, and ray rejection | Not applicable: no `Mat4` |
| Animated-node transform evidence | Control path | Matches A for decoded glTF input, final-column override, and composition | Matches A for decoded glTF input, final-column override, and composition | Not applicable: no `Mat4` |
| Stereo two-camera evidence | Control path | Matches A through four private renderer crossings | Matches A through four private renderer crossings | Not applicable: no `Mat4` |
| Orthographic camera evidence | Control path; normal and zero-height fallback | Matches A through private identity/projection crossings | Matches A through private identity/projection crossings | Not applicable: no `Mat4` |
| FPS repeated-motion evidence | Control path | Matches A; component mutation becomes getter/reconstruction seam | Matches A with direct component mutation | Not applicable: no `Mat4` |
| Short release upload observation | N/A | 1,000,600–1,000,700 ns / 1M | 1,006,700–1,016,200 ns / 1M | Not applicable |
| Stereo camera host observation | 7.323 ms median / 100k pairs; 0 allocations | 6.663 ms; 0 allocations | 9.246 ms; 0 allocations | Not applicable: no `Mat4` |
| Isolated WASM stereo math/column observation | 9.839 ms median / 100k; 11,738 raw bytes | 9.356 ms; 11,878 bytes | 9.352 ms; 12,843 bytes | Not applicable: no `Mat4` |
| WASM build evidence | Expanded study library builds; native-window app is not WASM-shaped | Expanded study library builds; copied app has same A limitation | Expanded study library builds; copied app has same A limitation | Expanded study library builds for implemented `Vec3` slice |
| Isolated WASM engine execution | Node executes actual stable re-export checksum: `292.000061...` | Node executes bounded transform/inverse checksum: `292.000061...` | Node executes bounded transform/inverse checksum: `292` | Not applicable: no `Mat4` |
| Isolated release WASM output | 2,782 bytes | 3,011 bytes | 3,953 bytes | Not applicable: no `Mat4` |
| Visible migration issue | Foreign vocabulary remains public | `w_axis` becomes accessor/setter; adapter seam | Representation conversion until renderer migrates | Provenance/update work starts immediately |
| Source/update burden | Provider audit and pin | Wrapper/API/adapter maintenance | Math correctness/target maintenance | C burden plus upstream lineage and fix tracking |
| Current candidate source observation | 68 lines / 2,325 bytes | 450 lines / 13,719 bytes; 41 provider references | 494 lines / 15,807 bytes; no provider references | 139 lines / 4,064 bytes; `Vec3` only |
| Current disposition | Retained control | Conditionally viable for further study | Under active conformance study | Do not expand without a named advantage over C |

## What This Does Not Decide

- No real Tokimu renderer, full scene consumer, asset boundary, serialization
  path, or public API has been migrated. The pinned Khronos `Box.glb`
  comparison supplies real decoded geometry to the bounded transform fixture;
  it is not a full `hello-glb` application migration.
- The CAD fixture is likewise a bounded caller-operation port, not a migration
  of CAD interaction, picking UI, or renderer behavior.
- The animated-node fixture is a bounded transform port, not scene traversal,
  animation scheduling, mesh lowering, or renderer migration.
- The retained transform result now has nine warmed, rotating-order samples per
  A/B/C candidate, but its host CPU/OS metadata was unavailable to the
  sandbox. Its overlapping ranges are descriptive only, not comparative
  performance findings.
- The candidate-isolated release executables differ by at most 512 bytes for
  the shared transform workload. They do not model renderer/application
  dependency closure, LTO, distribution packaging, or WASM output and are not
  a binary-size selection input.
- The isolated B/C fresh builds have one retained result each and intentionally
  model only the minimal candidate closure. They expose the current provider
  compilation and warning surface but do not model workspace or application
  build cost and are not a compile-time selection threshold.
- The study library builds for `wasm32-unknown-unknown`; its browser/WASM
  measurements have not run. The platform closure-ownership compile defect was
  repaired during this study. The original A `hello-3d-mono` application and
  the B/C copies still cannot compile for WASM because they are deliberately
  native-window-shaped: `run_window_with_app` is unavailable, their window
  value is not an `HtmlCanvasElement`, and browser `WgpuBackend` creation is
  asynchronous. This is common application-shape evidence, not a candidate
  portability difference.
- Two release-mode upload-boundary runs preserved the same checksum and zero
  allocations for B and C. Their short single-process timings are descriptive
  only: no host metadata, warm-up protocol, or repeated target-identified run
  exists yet, so the observed difference is not a performance conclusion.
- DOOM-driven object, transform, and scene pressure may alter the manifest and
  invalidate any premature selection.
- The affine inverse sweep is a fixed caller-shaped comparison, not randomized
  property testing, numerical fuzzing, or a selected singular/non-finite
  recovery contract.
- The 96-case differential sweep uses a fixed internal seed, excludes singular
  and near-zero-scale values, and adds no random dependency. It is expanded
  differential coverage, not fuzzing, a numerical proof, or a recovery policy.
- Node has executed one isolated A/B/C WASM transform/inverse probe. This is
  stronger than target compilation but does not execute a browser surface, the
  shared suite, or a Tokimu application, so the full WASM criterion remains
  open.
- The isolated uncompressed WASM modules are A 2,782 bytes, B 3,011 bytes,
  and C 3,953 bytes. They represent one micro-module export, not application
  packaging, compression, renderer closure, or a stable size criterion.
- C's native `Vec4` and `Mat4` presently have 4-byte rather than A/B's
  16-byte alignment. Its adapter reconstruction remains explicit, but no
  candidate may infer direct FFI/SIMD/GPU representation compatibility from
  matching math behavior or sizes alone.
- The same `Vec4`/`Mat4` alignment split executes on the isolated WASM modules.
  It is cross-target representation evidence, not a stable ABI or GPU-layout
  claim.
- The current WGPU camera path already converts `Mat4` to a renderer-owned
  `#[repr(C)] [[f32; 4]; 4]` uniform before byte upload. C's alignment therefore
  blocks direct compatibility claims, not the current explicit camera adapter.
- The public `tokimu::Camera` still stores A/provider `Mat4` values. The B and
  C candidate-camera fixtures can form that current renderer value only through
  two explicit private view/projection conversions. This is measured migration
  friction at a public vocabulary seam, not evidence that the stable renderer
  API has been ported.
- The retained orthographic control reproduces renderer-owned bounds and its
  zero-height fallback across A/B/C. It is compatibility evidence for a
  current camera policy, not proof that orthographic construction independently
  belongs in universal Native math vocabulary.
- In the rotated release-mode stereo-camera host observation, B remained in
  A's range and C's median was approximately 26% higher than A; all paths made
  zero allocations. This is a narrow native-host result, not a portable
  performance conclusion; target-native/WASM repeats remain required.
- In the isolated Node WASM stereo math/column observation, B and C were both
  about 5% below A and essentially equal to each other. It does not construct
  `tokimu::Camera` or execute WGPU, so this different target/scope result
  complements rather than overturns the native camera-facade observation.
- The full study crate, including B/C construction of the current
  `tokimu::Camera` plus stereo and orthographic fixtures, also builds for
  `wasm32-unknown-unknown`. This is compile portability evidence only; it does
  not execute the renderer facade in a browser or WGPU environment.

## Next Evidence Before Selection

1. Port bounded real corpus callers to every still-viable candidate and retain
   source-edit and conversion counts.
2. Establish repeated, target-identified native and WASM measurements.
3. Decide whether `Vec2` and `Quat` acquire real caller pressure.
4. Revisit the entire inventory after the DOOM WAD plan completes.
