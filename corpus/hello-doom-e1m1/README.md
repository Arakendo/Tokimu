# E1M1 Static Presentation Corpus

This corpus consumer prepares a bounded, static E1M1 scene from the reviewed
compact Doom package. It keeps WAD interpretation and source identities at the
corpus edge, then submits only ordinary textured meshes and materials to
Tokimu's renderer.

Run these commands from the repository root:

```powershell
# Headless preparation report: source omissions, texture/material inventory,
# and renderer-neutral draw count.
cargo run -p hello-doom-e1m1 --bin hello-doom-e1m1 -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD

# Native first-frame overview of the prepared opaque static scene.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD

# ADR-0013 categorical-cutout evidence. This adds the retained masked-middle
# candidates after the unchanged opaque scene using the admitted generic path.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --masked-cutouts

# Fixed source-spawn observer for a normal native screen capture. It maps the
# reviewed player-one THING position and heading into the corpus X/Z world and
# uses the containing sector's vertical midpoint; it is not movement or player
# eye-height policy.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --spawn-observer

# Slice 6 corpus-local first walk proof. WASD moves a 16-unit disc at the
# reviewed spawn; click captures mouse look, Escape releases it, and R resets.
# The source BLOCKMAP only narrows candidates. If it has no blocking candidate,
# the proof fails safe to all known blocking linedefs. --noclip is a visible
# diagnostic control, not a gameplay mode.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --spawn-observer --walk-collision

# Renderer-free retained replay and nearest-wall contact evidence for the same
# corpus-local disc proof.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --spawn-observer --walk-collision-report

# AR-0028 bounded Doom sidedef-direction fixture. It needs no WAD package.
# The left/back BACK panel appears screen-left and the right/front FRONT panel
# appears screen-right under the fixture camera basis.
cargo run -p hello-doom-e1m1 --bin doom_sidedef_conformance

# AR-0025 headless candidate-count evidence for the fixed overview and source
# spawn poses. Full submission remains the ordinary renderer contract.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-report

# AR-0025 in-place source-spawn turn trace and source-neutral fixture.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-turn-trace
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-position-trace
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --candidate-pathological-report

# AR-0025 Stage-2 static uniform-grid evidence; no WGPU initialization.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-grid-report

# AR-0025 temporal-overlap theory; no WGPU initialization and no temporal
# culling contract. Fresh AABB selection remains authoritative on every row.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-temporal-report

# AR-0025 source-only REJECT observation. This reports the classic Doom
# monster-sight prefilter for player-one's source sector; it is not rendering
# visibility or an input to candidate selection.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-reject-report

# AR-0025 source-only SEGS-to-SSECTORS membership observation. It proves only
# source topology, including one-to-many linedef membership.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-topology-report

# AR-0025 Stage-3A conservative membership-union control; no WGPU
# initialization. It is intentionally not a Doom rendering-visibility claim.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --doom-membership-report

# AR-0025 native first/warm-frame measurement. Add --frustum-aabb for the
# corpus-local conservative selection trial; the window exits after frame two.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --spawn-observer --measure-two-frames

# AR-0025 fixed visual pose known to retain all 26 cutout candidates. Add
# --frustum-aabb for the per-draw baseline, or --frustum-grid-8x4x8 for the
# explicitly corpus-local medium-grid comparison.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --spawn-observer --spawn-yaw-plus-90 --frustum-grid-8x4x8
```

If invoked from this `corpus/hello-doom-e1m1` directory instead, replace the
package path with `../assets/DOOM/packages/doom-shareware-corpus-v1.zip`.

The default native overview intentionally does not claim original Doom
plane-span rendering, visibility, gameplay, sky rendering, masked-middle
rendering, or general alpha blending. `--masked-cutouts` is a separate
ADR-0013 path: Doom selects only retained masked-middle candidates, then
declares categorical coverage as discard alpha `<= 0` through Tokimu's generic
cutout capability. It does not change the default opaque scene or admit Blend.

`--spawn-observer` is a fixed evidence camera. For reviewed E1M1 it reports
THINGS record `0`, source position `(1056, -3616)`, angle `90`, sector `38`,
and raw floor/ceiling interval `0..72`; the camera uses the interval midpoint
(`y = 36`) rather than claiming an original-Doom player-height policy.

`--walk-collision` is a separate Slice 6 corpus proof layered on that observer:
it moves on the X/Z plane using a 16-unit disc, treats one-sided and explicitly
blocking source linedefs as walls, and uses validated source `BLOCKMAP` cells
solely as a candidate accelerator. After horizontal resolution it consults
retained BSP/subsector sector ownership: descents and up-to-24-unit steps move
the camera by the accepted floor delta; taller upward steps, insufficient
56-unit clearance, and ambiguous ownership remain explicit rejections. It
does not yet claim complete player movement, two-sided opening traversal,
doors, lifts, gamepad input, or a reusable Tokimu collision API.
`--noclip` makes this omission visible by bypassing the corpus collision check;
`R` restores the exact reviewed source-spawn pose.

`--frustum-aabb` is an AR-0025 corpus experiment, not a renderer capability.
It derives world-space bounds from already prepared meshes, rejects only a
candidate wholly outside one homogeneous clip plane, preserves survivor order,
and fails open when bounds are unavailable. `--candidate-report` compares the
derived AABB and enclosing-sphere corpus bounds without GPU initialization.
`--measure-two-frames` exists only to retain comparable
first/warm native renderer statistics without leaving a measurement window
running.
`--candidate-turn-trace` rotates the source-spawn observer in fixed 45-degree
steps without advancing application state; `--candidate-pathological-report`
uses a synthetic source-neutral bounds fixture to expose conservative grouping
behavior. Neither flag initializes a renderer or extends the public contract.
`--candidate-position-trace` uses declared local offsets from the reviewed
source spawn, not collision-validated game movement.
`--candidate-grid-report` is a static corpus-only uniform-grid comparison; it
does not create an engine scene/index capability.
`--candidate-temporal-report` compares one-frame candidate overlap over smooth
yaw, an abrupt turn, and a declared teleport, plus a wider current-view
superset. It always performs fresh AABB selection first; prior candidates are
observation-only and never cull a draw.
`--doom-reject-report` retains a bounded source-format observation of the
classic `REJECT` matrix for the player-one source sector. Its rows describe
monster sectors and its column describes the player sector, so it remains a
monster-sight prefilter rather than a generic camera or render-visibility
claim.
`--doom-topology-report` reports how source `LINEDEFS` occur in source
`SSECTORS` through `SEGS`, retaining a one-to-many relation. It neither creates
renderer scene membership nor selects a render candidate.
`--doom-membership-report` compares a source-topology control: flats retain
their source subsector, while whole-linedef walls survive when any of their
source subsectors survives. It is intentionally conservative and headless.
`--doom-membership-union` renders that same control for fixed-pose visual and
two-frame command-build comparison; it remains a corpus-only source-topology
experiment, not a renderer candidate-selection capability.
`--frustum-grid-8x4x8` renders the retained medium grid experiment so it can be
visually compared with full submission or `--frustum-aabb`. It preserves input
order, rechecks grid survivors with the per-draw AABB test, and falls back to
full submission if no finite grid can be built. It is not an admitted default.
`--spawn-yaw-plus-90` is a fixed corpus observation pose rather than a player
or input policy.

For native inspection, click the scene to capture the mouse; press `Escape` to
release it. `W`/`A`/`S`/`D` move; accepted source-sector floor transitions
adjust observer height. These controls are presentation-only corpus navigation:
they do not add a complete Doom player, physics, or a Tokimu input contract.

Press the physical backquote/tilde key (`~`) to open the bounded Doom debug
console. Opening it releases mouse capture and suppresses movement input. The
native proof currently accepts `HELP`, `CLEAR`, `STATUS`, `CAMERA`, `COLLISION`, `LOOK`,
`USE <linedef>`, and `NOCLIP [ON|OFF|TOGGLE]`. `USE 151` starts the observed
E1M1 manual-door visual proof; it is currently an explicit source request, not
a physical reach/side interaction claim. The center crosshair identifies the ray used by
`LOOK`; a hit is an exact prepared-triangle intersection and reports distance,
opaque/cutout family, material handle, source label, and retained draw source.
With the console closed, `E` performs that same exact center-wall lookup and
submits the resulting corpus-local `Use` request automatically; aim at a
manual door such as `BIGDOOR2` and press `E`. Its retained response includes
the source linedef, target sector, and number of prepared meshes initially
eligible for the narrow visual lowering.
`STATUS` retains each active manual door as
`sector:current-height/closed-height/open-height:phase` for native inspection.
This is corpus-local Observation Shell evidence under AR-0013, not an admitted
engine console, command language, or generic picking contract.
