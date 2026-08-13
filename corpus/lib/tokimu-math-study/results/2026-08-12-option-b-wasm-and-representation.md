# Option B Native, WASM, Target, And Representation Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |
| Native host | `x86_64-pc-windows-msvc` |
| WASM engine | Node.js `22.23.2` through `wasm-bindgen-test-runner` |
| Private providers | exact pinned `glam` 0.29.3 and isolated 0.33.3 candidate |
| Production migration | none |

## Executed Semantic Evidence

The shared A/B/C conformance module executed in the Node WebAssembly engine
under both the default WASM feature set and explicit `+simd128`. Both runs
passed all 14 exported tests. The suite includes direct A controls,
provider-backed B controls, C controls, independent scalar projection checks,
fixed-seed affine/camera sweeps, and retained degenerate behavior.

Narrow B executed unchanged under each exact provider pin and each WASM feature
set:

- four checked camera/projection contract tests passed;
- four representative-caller tests passed; and
- values, GL-depth landmarks, and bounded rejection categories agreed with the
  native suite within `1e-5 + 1e-5 * abs(expected)`.

Full B executed unchanged under each exact provider pin and each WASM feature
set. All five external contract tests passed, including fixed-seed
normalization/inverse properties, independent camera/projection arithmetic,
bounded failures, scalar column transport, and the deliberately bounded
compatibility surface.

This is actual WebAssembly engine execution. It is not actual-browser or
browser-WebGPU evidence.

## Other Target Evidence

Both candidates and all their test/example targets compile for
`aarch64-pc-windows-msvc` under both provider pins. `rustc --print cfg` reports
`target_feature="neon"` for that target. No ARM64 machine was available, so
this is NEON-capable compile evidence only, not NEON execution.

No `wasm64` target, NVIDIA host, or NVIDIA browser adapter was available. Those
paths remain unobserved; AMD/Vulkan evidence elsewhere in the repository does
not substitute for them.

## Actual-Browser Gate

The local A/B/C chart fixture was built and a browser attachment was attempted.
The browser-control runtime reported no attachable browser surfaces. Therefore:

- the earlier A/C browser chart observation is retained as inherited evidence;
- the new B chart path has native and Node/WASM evidence only; and
- no new Option B actual-browser, browser-WebGPU, or browser-adapter claim is
  made.

This is an environmental gap, not a failed browser semantic result.

## Representation Observations

The native representation observer and the isolated Full-B observer produced
the following facts on this compiler/host. Full B produced the same values
under both exact private provider pins.

| Type | Full-B size | Full-B alignment | `Copy` observed |
| --- | ---: | ---: | --- |
| `Vec2` | 8 | 4 | yes |
| `Vec3` | 12 | 4 | yes |
| `Vec4` | 16 | 16 | yes |
| `Quat` | 16 | 16 | yes |
| `Mat4` | 64 | 16 | yes |

`Vec3::to_array` preserved `[1, 2, 3]`, and a `Mat4` constructed from the
ordered scalar columns `1..=16` returned those exact columns under both pins.
Narrow B uses the provider value types directly, so its value representation
is provider representation by identity; only its semantic constructors are
Tokimu-owned.

These are observations, not promises. Neither B contract admits provider
layout, SIMD identity, ABI/POD compatibility, serialization representation,
public fields, unsafe conversion, or direct GPU-buffer compatibility. Renderer
crossings remain explicit scalar-column copies.

## Disposition

Slice 6 passes for the available native, Node/WASM, `simd128`, and ARM64
compile-only paths. Actual-browser, NVIDIA, wasm64, and NEON execution remain
explicit gaps. No observed cross-target result requires changing either B
contract, and Full B has not accidentally acquired a representation contract.
