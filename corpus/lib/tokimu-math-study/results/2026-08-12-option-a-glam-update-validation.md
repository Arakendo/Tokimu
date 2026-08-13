# Option A `glam` 0.33.3 Candidate Validation

| Field | Value |
| --- | --- |
| Status | Complete Pause outcome; AR-0029 seam remains under review; candidate is not admitted |
| Candidate | 0.33.3 at `9928729066db87d97fa779e129469721a289beae` |
| Execution location | ignored detached worktree `target/ar0019-glam-update-worktree` |
| Production worktree | unchanged; remains at audited 0.29.3 |

## Isolated mechanical result

The candidate submodule, workspace requirement, and lockfile were changed only
inside a detached worktree based on parent `c84108c`. Cargo updated exactly one
package from local `glam` 0.29.3 to local 0.33.3. The resolved core closure was:

```text
tokimu-core
└── glam feature "std"
    └── glam v0.33.3 (local third-party/ring-0/glam path)
```

No optional `glam` dependency or second source/version was selected.

## Passing evidence

| Command/control | Result | Wall time | Observation |
| --- | --- | ---: | --- |
| isolated lock update | pass | 848 ms | only local `glam` changed from 0.29.3 to 0.33.3 |
| strict `tokimu-core` Clippy | pass | 1,972 ms | zero old generated `unused_attributes` diagnostics |
| `tokimu-core` tests | pass | 2,652 ms | 29 passed; no failures |
| wasm32 `tokimu-core` build | pass | 4,413 ms | zero old generated diagnostics |
| existing `tokimu-math-study` suite | pass | 39,182 ms | includes retained differential and pinned GLB controls available at the isolated parent revision |
| representative render/CAD/DOOM `cargo check` | compile succeeds with warnings | 61,750 ms | deprecated camera/projection constructors exposed a stable-vocabulary finding |

The Ring 0 audit, run with `-AllowViolations`, reported the intended local
0.33.3 `std` closure and one expected enforcement violation: the parent does
not yet pin the candidate gitlink. This is correct behavior for an unadmitted
isolated experiment.

The later staged clean-control audit passed without violations. A
machine-readable RustSec scan was also completed against both locks. It found
the same two vulnerabilities and four warnings in unrelated corpus/provider
dependencies and no `glam` advisory or candidate-specific delta. Accordingly,
the `glam` revision security comparison passes, while the whole-workspace
security baseline remains non-green and is not claimed as resolved here.

## Architectural finding: camera vocabulary pressure

`glam` 0.33.3 deprecates the `Mat4` associated constructors Tokimu and its
corpus currently use:

```text
Mat4::look_at_rh
Mat4::perspective_rh_gl
Mat4::orthographic_rh_gl
```

The deprecation advice redirects callers to functions beneath new
`glam::camera::*` modules. A repository search found 54, 28, and 4 textual call
sites respectively across current Rust crates, corpus, tests, and retained
alternatives. Some are experimental duplicates, but `tokimu-render` itself
uses all three constructor families.

Strict representative Clippy fails immediately in `tokimu-render` with three
deprecated-API errors. The broader DOOM check also observed the corresponding
warnings in its caller paths.

This cannot be repaired as an ordinary mechanical update under the current
plan:

- adopting `glam::camera::*` would expose or depend on foreign camera and
  projection vocabulary beyond AR-0019's admitted five types;
- applying `#[allow(deprecated)]` would suppress a new warning specifically
  forbidden by the update plan; and
- creating Tokimu-owned compatibility constructors would raise a new ownership
  and stable-contract question rather than merely updating the pin.

The candidate continues to compile because the methods have not yet been
removed, but strict-warning compliance and the planned narrow vocabulary cannot
both be claimed without maintainer judgment.

## AR-0029 isolated constructor prototype

Maintainer review authorized investigation of a Tokimu-owned seam rather than
public adoption of `glam::camera`. AR-0029 records that decision separately so
the provider update does not silently change stable ownership.

The disposable candidate worktree now contains exactly three checked public
functions in `tokimu_core::math`:

```text
try_view_look_at_rh
try_projection_perspective_rh_gl
try_projection_orthographic_rh_gl
```

They expose only Tokimu's existing `Mat4`/`Vec3` vocabulary and scalar
parameters. The private implementation delegates to the 0.33.3 camera module.
The selected contract rejects non-finite or degenerate view bases, invalid FOV
or aspect/depth ranges, and unordered orthographic bounds with `None`. Valid
projections retain right-handed Y-up input and GL `[-1, 1]` clip depth. WGPU's
private depth conversion was not changed.

Nine new core tests retain:

- eye-to-origin and `-Z` forward view behavior;
- coincident, parallel, and non-finite view rejection;
- perspective near/far mapping to `-1`/`+1`;
- invalid perspective rejection;
- orthographic near/far mapping to `-1`/`+1`; and
- invalid orthographic rejection;
- operand-ordered vector `min`/`max` NaN behavior introduced by the candidate;
- corrected scalar-over-matrix division; and
- corrected small-angle quaternion `rotate_towards` behavior.

Representative migration covered `tokimu-render`, CAD, mono 3D, GLB, native
and browser textured-box callers, the shared orientation fixture, E1M1
static/observer paths, and the Doom sidedef conformance fixture. It removed 28
of the 86 direct deprecated constructor calls in the isolated worktree; 58
remain, predominantly retained A/B/C study controls and additional corpus
consumers. No caller imports `glam::camera`.

| Command/control | Result | Observation |
| --- | --- | --- |
| `cargo test -p tokimu-core --locked --offline` | pass | 38 tests; six constructor-contract and three candidate-semantic regressions |
| strict core/render Clippy | pass | no deprecated camera constructor in the migrated engine boundary |
| strict CAD/mono/orientation Clippy | pass | representative callers use checked Tokimu constructors |
| strict E1M1 package-local Clippy | pass | all E1M1 bins migrate without suppression |
| `cargo check -p hello-render-orientation-web --target wasm32-unknown-unknown` | pass | compile-only WASM evidence; actual browser execution remains open |
| `cargo test -p render-orientation-conformance` | pass | 13 directional/camera/picking controls |
| `cargo test -p hello-doom-e1m1 --lib` | pass | 41 Doom geometry/orientation/collision controls |
| strict `hello-glb` Clippy | pass | representative GLB caller compiles without deprecated or foreign camera vocabulary |
| native/browser textured-box checks | pass with unrelated existing warnings | both caller shapes compile; actual presentation observations remain open |

A later locked/offline representative batch re-executed the available library
tests after all candidate repairs. It passed 41 E1M1 geometry, embedding,
collision, door, sky, UV, and diagnostic controls plus 13 orientation,
camera-basis, projection, picking, cull, and directional-atlas controls. The
migrated mono, stereo, CAD, GLB, textured-box, renderer, native Doom, and
browser-oriented callers had already passed strict focused compilation. The
pinned Khronos Box and transform-hierarchy paths remain covered by the retained
math-study suite. These are real caller-shaped checks, not upstream `glam`
tests; actual browser presentation remains separately open.

The first combined strict representative run also reached an unrelated
pre-existing `doom-geometry-provider` `clippy::type_complexity` failure. The
same targets compile, and package-local E1M1 strict Clippy passes with
dependencies excluded. That existing workspace lint is retained as a separate
validation prerequisite; it is not attributed to `glam` or suppressed here.

The isolated-worktree filesystem does not support incremental-cache hard
links, so Cargo repeatedly reports fallback-to-copy warnings. They are
environmental worktree evidence rather than Rust source diagnostics.

## Representation observation

The existing native A/B/C layout observer was executed once against the
production 0.29.3 worktree and once against the isolated 0.33.3 candidate.
Tokimu's five A types produced the same size/alignment observations on both:

| Type | 0.29.3 size/alignment | 0.33.3 size/alignment |
| --- | ---: | ---: |
| `Vec2` | 8 / 4 | 8 / 4 |
| `Vec3` | 12 / 4 | 12 / 4 |
| `Vec4` | 16 / 16 | 16 / 16 |
| `Quat` | 16 / 16 | 16 / 16 |
| `Mat4` | 64 / 16 | 64 / 16 |

These are host observations, not newly admitted ABI, serialization, or GPU
layout promises.

## Dependency-isolated API and behavior fingerprint

Two temporary probes depended directly on the exact local 0.29.3 and 0.33.3
source checkouts with only `std` enabled. They exercised the five public types,
direct fields and matrix columns, constants, array/tuple conversions, vector
operators, dot/cross and `normalize_or_zero`, quaternion construction and
rotation, matrix construction and point/vector transforms, inverse behavior,
and bounded singular/non-finite observations. Results were recorded with
`f32::to_bits` rather than formatted decimal comparisons.

Both probes emitted 14 observation lines and an exact line comparison retained
zero differences. The 0.33.3 candidate also passed all 44 isolated
`tokimu-math-study` library tests and all 38 `tokimu-core` tests under
locked/offline resolution. The math-study run intentionally retains
deprecation warnings from the unmigrated camera/projection controls; those
warnings are migration-pressure evidence, not numerical failures.

## Actual plain-WASM differential observation

The independently compilable Alternative A control was release-built for
`wasm32-unknown-unknown` once against production 0.29.3 and once against the
isolated 0.33.3 candidate. Node's WebAssembly engine instantiated both modules
and called the same exported transform/inverse, stereo-camera, and alignment
probes:

| Observation | 0.29.3 | 0.33.3 |
| --- | ---: | ---: |
| transform/inverse checksum | 292.00006103515625 | 292.00006103515625 |
| stereo-camera checksum, 1 frame | 1.8112413883209229 | 1.8112413883209229 |
| stereo-camera checksum, 1,000 frames | 1724.2821044921875 | 1724.2821044921875 |
| `Vec4` alignment | 16 | 16 |
| `Mat4` alignment | 16 | 16 |

This is actual WASM execution rather than compile-only evidence. It proves an
exact result for the bounded control, not general browser presentation parity.
The 0.29.3 build reproduced the generated-source warning flood; the 0.33.3
build emitted only three deprecations from the deliberately unmigrated camera
control source.

The same two controls were then release-built with
`-C target-feature=+simd128` and executed under the workspace-local official
Node 22.22.2 runtime. Transform/inverse checksum, one-frame and 1,000-frame
stereo checksums, and both alignment observations again matched exactly. This
closes SIMD-enabled WebAssembly-engine execution for the bounded control. It
does not claim actual-browser SIMD behavior, WebGPU behavior, or wasm64 parity.

The candidate's complete WASM-applicable conformance set was then executed via
`wasm-bindgen-test-runner` under Node 22.22.2: 12/12 tests passed with default
target features, and 12/12 passed in a separate `simd128` build. These include
the deterministic affine and camera/projection differential sweeps plus the
retained singular/degenerate observations. The same intentionally unmigrated
camera deprecations remain visible.

For ARM target pressure, both exact pins compile-check for
`aarch64-pc-windows-msvc`; `rustc --print cfg` confirms that target enables
`neon`. This is compile-only evidence because the current host is x86-64. A
linked cross-target executable was not claimed. The compiler recognizes
`wasm64-unknown-unknown`, but stable Rust supplies no prebuilt standard-library
artifact for it; target installation failed explicitly and no source-built or
alternate-toolchain substitute was introduced.

A direct `tokimu-core` WASM test attempt compiled the test artifact but the
runner reported zero tests: core's ordinary Rust `#[test]` functions are not
exported through `wasm-bindgen-test`. That attempt is retained as a harness
limitation and is not counted as 38 core tests passing on WASM. The 38-test
claim remains native; the 12-test conformance claim is actual WASM execution.

## Retained conformance and performance finding

The isolated candidate passed all 12 focused retained conformance cases,
including finite camera/projection and affine differential sweeps,
non-singular inverse round trips, singular/degenerate observations, and the
initial vector/transform cases.

Same-host release observations used 15 rotated samples per candidate:

| Workload | 0.29.3 A median | 0.33.3 checked A median | 0.33.3 direct provider-backed median |
| --- | ---: | ---: | ---: |
| 1,000,000 repeated transforms | 3.737 ms | 3.354 ms | 3.412 ms |
| 100,000 stereo-camera iterations, initial | 7.384 ms | 14.581 ms | 6.764 ms |
| 100,000 stereo-camera iterations, repaired | 7.384 ms | 7.907 ms | 8.959 ms |

Three additional initial 0.33.3 stereo-camera runs retained checked-A medians
of 14.635, 14.682, and 14.667 ms; the direct provider-backed medians were
6.744, 6.684, and 6.726 ms. Checksums remained equal.

Investigation then found that `stereo_with_a` constructed the default checked
camera before replacing its view. Direct constructors previously allowed that
dead work to disappear; fallible semantics made it observable. The prototype
also normalized the camera basis before the provider repeated the same work.
The repaired caller constructs its final shared projection and two views once,
and the wrapper validates finite inputs/results without duplicate normalization.

Three repaired candidate runs retained checked-A medians of 7.907, 8.034, and
7.932 ms; direct provider-backed medians were 8.959, 8.960, and 9.089 ms.
Checksums again remained equal. The repaired within-candidate comparison does
not show a checked-boundary throughput penalty. The candidate checked path is
about seven percent above the separately built 0.29.3 median, but that
cross-build difference is noise-sensitive and is not attributed to the camera
seam or the provider update without stronger evidence.

This closes the initial performance blocker as an ordinary caller-shape and
duplicate-work repair while retaining the full measurement history. It does
**not** establish that all camera workloads are performance-equivalent.

## Whole-workspace gate

The isolated candidate's whole-workspace test reached unrelated retained
corpus failures after the missing Departure Mono submodule checkout was
restored:

- `hello-resource-space` omits the now-required `AssetLoader::Error` associated
  type;
- the `resource-space-assets` test loaders omit the same associated type.

`cargo check -p hello-resource-space --locked --offline` fails identically on
the unchanged production tree with `glam` 0.29.3. These are baseline workspace
defects, not candidate regressions, and were not repaired as part of the math
provider update. The complete workspace green claim therefore remains open;
the focused core/render/stereo and representative caller gates remain the
admissible candidate evidence.

## Core build and output observation

Fresh isolated release target directories on the same Windows host retained:

| Observation | 0.29.3 | 0.33.3 candidate |
| --- | ---: | ---: |
| clean `tokimu-core` release build | 4,291.5 ms | 4,238.4 ms |
| no-change incremental rebuild | 474.7 ms | 469.4 ms |
| resulting `tokimu-core` rlib | 614,226 bytes | 614,226 bytes |

These single-run build observations show no material difference for this
bounded package/host/profile. They are not whole-workspace compile-time or
final executable-size claims. The old build again emitted its large generated
warning stream; the candidate did not.

## Browser-ready candidate fixture

The candidate `hello-render-orientation-web` WASM and `wasm-bindgen` browser
package build successfully and the local server returned its HTML at
`http://127.0.0.1:4174/`. This is readiness evidence only. Automated browser
control initially could not connect because the already-running bridge
inherited system Node 22.21 while its harness requires Node 22.22 or newer. An
official portable Node 22.22.2 control was installed under the workspace and
executed successfully; Codex configuration also names its bundled Node 24.14
runtime.

The machine-wide 22.x installation was then updated to official Node 22.23.2.
The x64 MSI was downloaded from Node's `latest-v22.x` distribution, matched
published SHA-256
`ce9572ae220c345fbae2340bbb4d084e8ca5e0fe093ee7067d43094ae23be989`, and the
installed `node.exe` has a valid OpenJS Foundation Authenticode signature. The
first silent install rolled back with MSI 1730 because replacing the existing
machine installation required an administrator token; an elevated retry
completed successfully. System observations are now Node 22.23.2 and npm
10.9.8.

Windows Installer restarted the stale bridge process, but the current bridge
service subsequently reports a missing runtime path even though both the
system Node 22.23.2 executable and configured bundled Node 24.14 executable
exist. A full Codex host restart is therefore still required. No
visual/readiness status from the served candidate page is claimed; actual
browser execution remains open.

## Clean initialization, rollback, and deterministic replay

A second disposable worktree was created directly from parent `c84108c`. Its
Ring 0 source began uninitialized. With network access unavailable, the normal
submodule command failed explicitly while attempting the canonical GitHub URL;
it did not substitute crates.io, another Git source, or a local working copy.
After canonical Git access was authorized, the same command checked out the
recorded parent commit `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7`.

The candidate integration diff was then replayed without the submodule working
tree and the submodule was moved explicitly to immutable commit
`9928729066db87d97fa779e129469721a289beae`. All 15 changed Tokimu files matched
the first isolated candidate byte-for-byte, and their numstat manifests were
identical. `git diff --check` passed.

The control was next rolled back to the parent manifest, lockfile, and 0.29.3
gitlink. Its worktree was clean and the workspace requirement again named
0.29.3. Replaying the same diff and immutable submodule commit reproduced the
candidate a second time. After staging that exact gitlink in the disposable
control, the Ring 0 dependency audit passed with this selected closure:

```text
tokimu-core
└── glam 0.33.3 (approved local Ring 0 submodule; feature std)
```

The replayed control also passed 38 `tokimu-core` tests and complete Cargo
metadata resolution with `--locked --offline`. This proves rollback and replay
for the bounded candidate patch. It does not claim that every unrelated corpus
fixture can be initialized or that the complete workspace suite is green.

The first clean-control `cargo fmt --all -- --check` attempt exposed one more
tooling prerequisite: Cargo traverses the upstream `glam` workspace, whose
manifest includes the nested `tools/codegen` member. Builds, metadata, the Ring
0 audit, and tests did not need that development tool, but whole-workspace
formatting could not load its absent manifest. Initializing exact nested commit
`673ed2c712d0c2db35fed00a21da7f132ab3cd7f` made the candidate and replay-control
format checks pass. This does not add the generator to Tokimu's runtime closure;
it does add a reproducible source-tooling step to Option A maintenance.

## Current disposition

The source, numerical, native, security, performance, rollback, and replay
evidence is now substantially complete while the production pin remains fixed.
The exact API fingerprint and retained tests found no numerical compatibility
defect. The constructor prototype demonstrates that the seam can remain bounded
for representative callers, but 58 of the 86 deprecated constructor calls are
deliberately unmigrated; stable admission and full migration still require the
AR-0029 maintainer decision. Before production movement, the remaining choices
are to:

1. retain 0.29.3 and reject/pause this candidate;
2. accept the bounded Tokimu-owned camera/projection seam after its remaining
   migration and actual-browser gates; or
3. explicitly review admission of the new foreign camera vocabulary.

AR-0029 currently incubates option 2; this evidence record does not promote it
to an ADR or stable production API.

## Restarted browser-gate observation

After the machine Node update and Codex host restart, the candidate fixture
server starts successfully with system Node 22.23.2 and returns HTTP 200 from
`http://127.0.0.1:4174/`. This closes the earlier runtime-version mismatch.

The restarted browser-control session reports an empty browser inventory, so
there is no attachable in-app or extension browser in which to execute WebGPU.
The server was stopped after this bounded check. Actual browser presentation,
camera interaction, and browser-SIMD remain open and are not replaced by the
already-passing Node WASM evidence.

## Final closure validation

The repository-resolvable baseline was repaired after the initial pause. On
2026-08-12, locked/offline whole-workspace tests pass against both production
0.29.3 (82.3 seconds) and the isolated 0.33.3 candidate (43.6 seconds). Both
trees pass `cargo fmt --all -- --check`.

Strict whole-workspace Clippy passes in the 0.33.3 candidate. Production 0.29.3
still reproduces the provider-owned generated-source warning flood (4,896
actionable generated diagnostics plus the summary), while the independently
found Tokimu-owned math-study lint failures were repaired and pass with the
known provider warning category isolated. This preserves the warning cleanup
as genuine update evidence rather than charging local lint debt to either
revision.

The actual-browser attempt remains unavailable because the restarted browser
controller exposes no browser instance. Under the plan's explicit Pause
completion outcome, that is a named admission resumption gate rather than a
reason to leave the study itself indefinitely active.

**Final study disposition:** complete the Option A study with a **Pause** of
0.33.3 admission. Retain audited production 0.29.3. Resume candidate admission
only after AR-0029 ownership/migration, actual-browser execution, and a fresh
maintainer admission vote.
