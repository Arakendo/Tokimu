# Coordinate-Frame Ledger

| Field | Value |
| --- | --- |
| Status | Slice 1 retained evidence |
| Recorded | 2026-08-10 |
| Plan | `docs/Plans/Tests/coordinate-frame-directional-conformance.md` |
| Review | AR-0028 |
| Claim boundary | Current implementation and retained corpus observations; not a Tokimu-wide coordinate-system guarantee |

## Status Vocabulary

| Status | Meaning |
| --- | --- |
| Accepted | Bound by an accepted ADR or an existing narrow public contract. |
| Observed | Directly present in current code or retained native/browser evidence. |
| Provisional | Selected by a corpus/provider to satisfy bounded evidence; not admitted globally. |
| Unspecified | No stable owner or project-wide guarantee has been established. |
| Contradiction pressure | Two valid local uses differ or terminology currently implies more agreement than exists. |

Unknown values remain `?`. This ledger deliberately does not fill them from
graphics convention, `glam` behavior, or maintainer intuition.

## Cross-Fixture Ledger

| Meaning | Source | After adapter | Tokimu candidate convention | Provider realization | Status / owner |
| --- | --- | --- | --- | --- | --- |
| Numeric vector/matrix mechanics | Tokimu public math currently backed by audited `glam` | Ordinary `Vec3` / `Mat4` operations | No semantic frame encoded by the type | CPU/WASM implementation | Observed; AR-0019 remains incubating |
| World `+X` | Fixture/application-defined | Usually unchanged | `?` | Unchanged by WGPU | Unspecified globally |
| World `+Y` / up | Most current 3D corpus callers select `Vec3::Y` | Usually unchanged | `?`; broadly repeated, not yet admitted by AR-0028 | Unchanged by WGPU | Observed caller convention |
| World `+Z` | Fixture/application-defined | Usually unchanged | `?` | Unchanged by WGPU | Unspecified globally |
| Camera default forward | `Camera::perspective_3d`: eye `(0,0,3)` to origin | View matrix from `look_at_rh` | Default convenience looks toward `-Z`; no public universal-forward claim | Uploaded after clip-depth adaptation | Observed renderer convenience |
| E1M1 observer forward | Doom heading; angle `0 -> +X`, `90 -> +Z` | `doom_heading_forward`, then corpus yaw/direction helpers | Corpus yaw `0 -> +Z` | `look_at_rh(position, position + forward, +Y)` | Provisional Doom corpus convention |
| Positive E1M1 yaw | Corpus scalar | `observer_direction`: `+Z -> +X` | This is a positive mathematical yaw, but it is not the observer's screen-right turn at `+Z` | Ordinary view matrix | Provisional; interaction label must stay explicit |
| E1M1 local screen-right | Current observer forward and `+Y` | `forward.cross(+Y)`; at `+Z`, right is `-X` | `?` shared meaning | Movement only; renderer does not own it | Provisional corpus control policy |
| Native raw pointer `+X` | Winit `DeviceEvent::MouseMotion.delta.0` | Forwarded unchanged as `PlatformInputEvent::MouseMotion.delta_x` | No `tokimu-input` normalized relative-motion event currently exists | Application maps delta to intent | Observed platform mechanism; semantic sign unspecified |
| Browser raw pointer `+X` | DOM `MouseEvent.movementX` while pointer locked | Forwarded unchanged as `PlatformInputEvent::MouseMotion.delta_x` | Same event shape as native, still not normalized camera intent | Application maps delta to intent | Observed in the live browser camera fixture; agrees with native after the fixture-local first-person policy |
| E1M1 pointer-look policy | Raw relative pointer delta | `yaw -= delta_x * 0.0032`; `pitch -= delta_y * 0.0024` | Physical motion, observation, interaction policy, and camera yaw remain separate | Camera view reconstructed by caller | Provisional first-person policy |
| E1M1 keyboard strafe | `A` / `D` platform key observations | `tokimu-input` key state; A subtracts and D adds corpus local-right | `?` shared movement intent | Position changed before camera upload | Provisional first-person policy |
| AR-0021 ordered facing | Fixture triangle-list positions | Instance transform may preserve or reverse orientation | Ordered positions determine geometric facing; normals do not | WGSL `front_facing`; explicit cull mode | Observed native/browser conformance; binding contract remains under AR-0021 |
| AR-0021 authored normal | Both triangles supply `+Z` | Transmitted as ordinary vertex stream | Shading evidence only | Custom WGSL brightness | Observed; explicitly not facing truth |
| Reflection | Negative X instance scale | Negative determinant reverses observed facing unless caller reverses each triangle once | Renderer does not infer compensation | WGPU realizes declared transform/winding | Observed native/browser conformance |
| Box source axes | Khronos `Box.glb` positions/normals | Caller expands indices and supplies planar UVs | No Tokimu global model-axis claim | Ordinary textured mesh | Observed independent source fixture |
| Box texture `+U/+V` | Corpus planar mapping selected by dominant normal | Caller supplies normalized UV stream; fixture can identity/flip-U/swap-UV | Caller-owned under ADR-0012 | Shader passes UV unchanged; sampler declared by material | Accepted narrow supplied-UV contract |
| Doom map point | Source `(doom_x, doom_y)` | Lifted as world `(doom_x, height, doom_y)` | Doom-provider conversion only | Ordinary position stream | Provisional source conversion |
| Doom right/front normal | Stored linedef delta `(dx,dy)` | `(dy,0,-dx)` | Doom side/winding meaning | Ordinary triangle winding and normal | Source-backed Doom evidence |
| Doom left/back normal | Stored linedef delta `(dx,dy)` | `(-dy,0,dx)` | Doom side/winding meaning | Ordinary triangle winding and normal | Source-backed Doom evidence |
| Doom right/front texture `+U` | Sidedef X offset and linedef endpoints | U decreases from stored start to stored end after the selected 2D-to-3D lift | Doom-provider mapping; canonical `EXITSIGN` pressure | Supplied UV unchanged | Provisional; verified by synthetic native art and native/browser face 342 |
| Doom left/back texture `+U` | Sidedef X offset and linedef endpoints | U increases from stored start to stored end | Doom-provider mapping | Supplied UV unchanged | Provisional; verified by synthetic native asymmetric art |
| Doom wall texture `+V` | Source vertical anchor and top-down raster rows | `v = texture_mid_y - world_height`, then divide by texture height | Doom-provider mapping | Supplied UV unchanged | Source-backed and tested |
| Doom flat U/V | Source map axes | Static corpus mapping `u=x/64`, `v=-z/64` | Intentionally non-original plane policy | Supplied UV unchanged | Provisional static presentation policy |
| Tokimu camera clip depth | GL-style projection | `[-1,1]` camera clip Z | Retained Tokimu camera meaning from AR-0024 | WGPU private uniform maps to `[0,1]` | Accepted review disposition / provider adaptation |
| Winding after clip adaptation | Caller mesh and camera | No position-axis reflection in depth-only conversion | Must remain unchanged | WGPU changes clip Z only | Tested provider behavior |

## Explicit Contradictions And Unknowns

### Default camera versus E1M1 forward

`Camera::perspective_3d` is a convenience camera looking from `+Z` toward the
origin, hence toward `-Z`. The E1M1 source-spawn observer may look toward `+Z`.
This is not currently a runtime contradiction because `Camera` stores
caller-provided matrices and E1M1 replaces the default view. It does prove that
the default camera cannot be cited as a Tokimu-wide forward-axis contract.

### Positive yaw versus screen-right look

At E1M1 yaw zero, forward is `+Z`. `observer_direction(yaw + pi/2, 0)` is
`+X`, while the current `look_at_rh` screen-right/local-right vector is
`forward.cross(+Y) = -X`. Therefore positive yaw and a screen-right turn have
opposite signs in this corpus. Raw pointer motion to the right subtracts yaw.
Earlier test terminology called positive yaw a “right turn”; that label was
incorrect even though the vector arithmetic was deterministic.

This does not select a global yaw convention. It establishes a structural fact
that the camera/input fixture must name and test.

### Raw mouse motion is not normalized look intent

Native Winit delta and browser `movementX/movementY` are forwarded unchanged as
platform observations. `PlatformInputEvent::as_input_event` deliberately does
not translate relative `MouseMotion` into `tokimu-input`. Applications currently
own the conversion to first-person look or another interaction. “Mouse moved
right,” “delta X is positive,” “request look right,” and “change yaw by a
positive amount” are four separate claims.

### Semantic role is absent from raw math values

`Vec3(1,0,0)` does not identify its frame or whether it is a point, direction,
normal, or input intent. `Mat4` does not identify object-to-world,
world-to-view, projection, provider clip adaptation, or a future chart
transition. This is retained pressure for AR-0019 and AR-0026; it does not
authorize wrapper types or a public spatial API.

## End-To-End Trace Inventory

### Doom wall presentation

```text
WAD LINEDEF/SIDEDEF + texture offsets
    -> doom-map-provider source records
    -> doom-geometry-provider side, winding, source texel U/V
    -> hello-doom-e1m1 normalized supplied UV + ordinary Mesh
    -> Material point/repeat declaration
    -> tokimu-render Textured3d (UV unchanged)
    -> WGPU vertex/fragment realization
```

Source identity remains beside the mesh through `StaticWallMesh`; it does not
enter renderer vocabulary.

### E1M1 camera and input

```text
Winit/DOM physical event
    -> PlatformInputEvent observation
    -> corpus capture/key-state and look policy
    -> observer position + forward/up/right
    -> caller-owned look_at_rh view + perspective_rh_gl projection
    -> Camera upload
    -> private WGPU clip-depth conversion
```

The current gap is the absence of an admitted normalized relative-look or
camera-intent contract, not missing raw platform observation.

### AR-0021 orientation fixture

```text
ordered positions + separate +Z normal
    -> identity/ordinary/reflected instance transform
    -> caller-declared cull mode
    -> Camera::default identity matrices
    -> WGPU front_facing and culling
    -> retained native/browser color matrix
```

This fixture intentionally isolates facing from camera-basis and UV questions.

## Failed-Hypothesis Record

1. The first E1M1 mirrored-sign diagnosis assumed the visible sign was a
   left/back sidedef and reversed only that mapping.
2. The sign remained mirrored.
3. Canonical-package evidence showed all submitted `EXITSIGN` upper-wall
   triangles were right/front records on linedefs 342–350.
4. The provider mapping was corrected for the actual right/front case while
   leaving renderer UV behavior unchanged.
5. The rebuilt native observer displayed readable `EXIT` art.

The failed hypothesis is retained because it demonstrates why source identity
and asymmetric fixtures are required before changing a directional contract.

## Slice 1 Disposition

The tested paths are traceable without an unnamed sign flip, but several
values remain deliberately unspecified:

- Tokimu-wide world forward and right;
- a shared positive-yaw interaction meaning;
- normalized relative-look intent;
- semantic point/direction/frame/transform roles above raw math values; and
- whether orientation reversal is ever first-class spatial meaning.

These unknowns advance to later slices. Slice 1 does not justify a public API
or a change to accepted renderer contracts.

## Sources

- `crates/tokimu-render/src/camera.rs`
- `crates/tokimu-render/src/wgpu_backend.rs`
- `crates/tokimu-platform/src/input.rs`
- `crates/tokimu-platform/src/native.rs`
- `crates/tokimu-platform/src/wasm.rs`
- `corpus/lib/render-orientation-conformance/`
- `corpus/hello-textured-box/fixture-manifest.md`
- `corpus/hello-textured-box/src/main.rs`
- `corpus/lib/doom-geometry-provider/src/lib.rs`
- `corpus/hello-doom-e1m1/src/lib.rs`
- `corpus/hello-doom-e1m1/src/bin/static_scene.rs`
- `docs/Plans/DOOM/Classic Doom wall side and winding evidence.md`
- `docs/Plans/DOOM/Classic Doom wall V placement evidence.md`
- `docs/lessions/camera-clip-depth-provider-adaptation.md`
