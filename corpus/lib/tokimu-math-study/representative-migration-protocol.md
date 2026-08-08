# Representative Migration Protocol

| Field | Value |
| --- | --- |
| Status | Planned next evidence; no stable source migration authorized |
| Related plan slice | Slice 6 |
| Rule | Original callers remain unchanged; ports live only in corpus-local fixtures |

## Selected Pressure Set

| Role | Existing pressure source | Migration evidence required |
| --- | --- | --- |
| Renderer camera | `crates/tokimu-render` camera use of `Mat4`/`Vec3` | View/projection, position/direction transform, provider upload boundary, explicit conversion count |
| Basic 3D corpus | `corpus/hello-3d-mono` | Camera/object transform and visible deterministic transform result |
| Repeated-motion corpus | `corpus/hello-fps-web` | Direction construction, zero-safe normalization, in-place movement, component mutation, and distance observation |
| CAD interaction corpus | `corpus/hello-cad` | Cursor-to-world ray, homogeneous `Mat4 * Vec4`, perspective divide, and degenerate-ray rejection |
| Imported-scene corpus | `corpus/hello-glb` | Transform composition and `Vec4`/matrix result handling; record any API absent from a candidate |
| Animated imported-scene corpus | `corpus/hello-hole-punch` | glTF column-array node input, translation override, writable final matrix column, and parent-child composition |

The selected set is intentionally small. It represents renderer, basic object,
and imported-scene pressure without claiming that it covers all present or
future callers.

## Port Rules

1. Copy only the bounded math-facing path into a new corpus-local fixture; do
   not edit the original caller or stable math API.
2. Build the same behavior for every candidate that implements the required
   operations. If a candidate cannot implement it, retain that failure and do
   not substitute a weaker workload.
3. Count source lines changed, candidate/provider conversions, compatibility
   helpers, and any provider type leaking into a candidate-facing signature.
4. Run native tests, `wasm32-unknown-unknown` build, and the shared allocation
   observation where applicable.
5. Retain a rollback note: deleting the corpus fixture is sufficient because
   no stable crate or original caller is modified.

## Required Result Table

Each port report must include:

| Candidate | Compiles | Same behavior | Source edits | Explicit conversions | Helpers | Provider signature leak | Rollback |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A | Yes | Control | 0 stable source edits | 0 | None | Existing A vocabulary | No experiment change |
| B | Yes | Bounded fixtures match A | 9 corpus-local modules; 0 stable | 9 | One private upload helper | 0 candidate-facing leaks | Delete corpus modules |
| C | Yes | Bounded fixtures match A | 9 corpus-local modules; 0 stable | 9 | One private upload helper | 0 candidate-facing leaks | Delete corpus modules |
| D | Vec3 slice only | Vector cases only | 1 corpus-local module | 0 | None | N/A | Delete derivation |

No blank cell is interpreted as success; use `not implemented` or `not
applicable` with a reason.

The retained source/module and conversion-count breakdown, actual boundary
round-trip evidence, and rollback details are in `migration-accounting.md`.

## Duplicate-Case Rule

For the selected `hello-3d-mono` path, use the separate candidate copies
reserved in `corpus-cases/hello-3d-mono/`. A shared preflight helper is useful
for semantic comparison, but it cannot replace independently compileable copies
when measuring source edits, conversion boundaries, and application integration.

## Initial Port Status

`src/migration_hello_3d_mono.rs` ports `hello-3d-mono`'s rotating-cube
position/normal transform path for B and C without changing the original app.
D is explicitly not implemented for this path because its paused `Vec3` slice
does not include `Mat4`; that retained gap is evidence, not a skipped result.

Independent integration cases run the same bounded path separately for A, B,
and C under `tests/hello_3d_mono_*.rs`. Full B and C window/render-shell copies
now live under `corpus-cases/hello-3d-mono/`; the original
`corpus/hello-3d-mono` is the unchanged A control. Offline native compilation
of both candidate copies passed. Visual runtime observation, deterministic
frame capture, allocation measurement at the camera upload boundary, and any
WASM result remain required before claiming a complete application-level port.

`src/migration_hello_glb.rs` now adds the bounded imported-scene transform
path: model and floor composition, non-uniform scaling, inverse-transpose
normal handling, and `normalize_or_zero`. The independent
`tests/hello_glb_khronos_box.rs` comparison decodes the pinned Khronos
`Box.glb` fixture and applies that path to its real positions and normals; A,
B, and C agree within the retained floating-point tolerance. This is
caller-operation evidence only; it is not a full GLB application copy,
asset-loader migration, or renderer migration.

`src/migration_hello_cad.rs` adds the bounded cursor-ray path from
`hello-cad`: camera view/projection construction, inverse view-projection,
homogeneous `Mat4 * Vec4`, perspective divide, `normalize_or_zero`, and the
zero-length rejection. A/B/C agree within the retained floating-point
tolerance. This is not a copy of the CAD application, its picking UI, or its
renderer integration; it is retained evidence for the previously unexercised
`Vec4` caller pressure.

`src/migration_hello_fps.rs` adds the bounded `hello-fps-web` camera-motion
path: directional construction, zero-safe normalization, in-place motion,
eye-height restoration, and distance to a target. A/B/C agree within the
retained floating-point tolerance. B must reconstruct the vector after the
baseline's mutable component assignment; this is retained wrapper migration
cost, not a hidden compatibility helper. The fixture excludes application
state, input, renderer, and browser lifecycle behavior.

`src/migration_b.rs` and `src/migration_c.rs` now also construct the current
public `tokimu::Camera` from candidate-facing camera state. The current facade
stores provider `Mat4` values for both `view` and `projection`, so B makes two
private unwrap conversions and C makes two private column-array
reconstructions at that handoff. The exact composed provider matrix agrees
with the candidate view-projection result in both fixtures. This is real
public renderer vocabulary migration pressure, but not a stable renderer API
migration: the candidate camera and all conversions remain corpus-local.

`src/migration_hello_3d_stereo.rs` adds the distinct `hello-3d-stereo`
two-camera shape. Each eye forms an orbit-derived view and a half-width
perspective projection before crossing into the current renderer `Camera`.
The A/B/C composed matrices agree within the retained tolerance and the left
and right results remain distinct. B and C each make four explicit crossings
in this bounded path (view and projection for each eye). It adds no candidate
operation beyond the existing `Vec3` subtraction, normalization, cross
product, scalar multiply, and `Mat4` view/projection inventory.

`src/migration_hello_asteroids.rs` adds one representative orthographic-camera
control. It matches the current renderer's world-height bounds calculation and
its zero-height aspect fallback, then crosses candidate identity and projection
matrices into the provider-valued `Camera`. A/B/C agree for both cases. This
demonstrates that B/C can reproduce the current renderer policy without new
candidate operations; it does not settle whether orthographic projection is
universal Native math meaning or a renderer-owned camera concern.

`src/migration_hello_hole_punch.rs` adds the bounded node-resolution path from
`hello-hole-punch`: glTF column-array input, an animation translation override
that replaces the final matrix column, and parent-child composition. A/B/C
agree for synthetic two-node data, and the independent test uses a decoded node
from the pinned Khronos `Box.glb` fixture. This is not scene traversal,
animation scheduling, mesh lowering, or renderer integration.
