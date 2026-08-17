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

# Normal interactive E1M1 corpus run: PreserveNorth embedding, source-spawn
# observer, collision, masked cutouts, and the bounded SKY1 panorama.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD

# Slice 7 A/B/C render-strategy comparison. A submits the original global
# scene. B submits every declaration retained by one coherent Doom ordered
# preparation. C runs that same B preparation first, then applies the generic
# conservative frustum/AABB filter to its output. B and C now rebuild that
# Doom-owned preparation from the live observer pose as the camera moves. B
# full-submits every surviving prepared declaration; C applies the generic
# filter only afterward. Both remain experimental while their known
# missing-edge geometry is under investigation; neither is the normal E1M1
# presentation profile.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=a
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=b
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=c

# Slice 6B headless final-handoff proof. This rebuilds the literal ordered
# preparation for six retained source rays and checks terminal rejection,
# partial-plane survival, final declaration identity, and conservation without
# opening a window or applying a generic camera filter.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=ordered-occurrence-prepared-full `
  --ordered-occurrence-six-ray-report `
  --no-walk-collision

# For interactive B inspection, click the window to capture mouse look, use
# WASD to move (Shift runs), and press Escape to release the mouse. `~` opens
# the debug console; LOOK reports the source-labelled surface under the center
# ray. The live preparation currently consumes immutable decoded source state,
# so this is evidence for static floor/wall joins rather than runtime door or
# platform-height integration.

# ADR-0013 categorical-cutout evidence. This adds the retained masked-middle
# candidates after the unchanged opaque scene using the admitted generic path.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --masked-cutouts

# Corpus-local SKY1 panorama experiment. This consumes retained F_SKY1 source
# classification and the composed E1 sky raster without treating sky as an
# ordinary ceiling flat or a generic renderer capability.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --doom-sky --spawn-observer --embedding-north --noclip

# Fixed source-spawn observer for a normal native screen capture. It maps the
# reviewed player-one THING position and heading into the corpus X/Z world and
# uses the containing sector's vertical midpoint; it is not movement or player
# eye-height policy.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --spawn-observer

# Slice 6 corpus-local first walk proof. WASD moves a 16-unit disc at the
# reviewed spawn; holding either Shift key runs at twice the walk speed. E uses
# the centered source wall, click captures mouse look, Escape releases it, and
# R resets. In noclip only, Space moves vertically up and physical Left Ctrl
# moves vertically down.
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

# AR-0025 Stage 3B bounded visible-SEG comparison. This is a separate
# diagnostic scene, not the normal E1M1 renderer. Add --measure-two-frames for
# retained first/warm resource and command evidence.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-seg-clip-presentation

# AR-0025 fixed source-spawn classic-plane reconstruction control. It groups
# retained viewer-relative floor/ceiling columns into ordinary supplied-UV
# meshes while deliberately omitting walls. Add --measure-two-frames for the
# retained resource/command evidence. The camera is locked to the exact
# source-spawn reconstruction pose during manual gap inspection.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-seg-classic-plane-presentation

# The contextual companion adds only BSP-admitted whole-SEG wall tiers around
# those reconstructed planes. It makes plane gaps easier to judge, but is
# still an intermediate fixed-pose control rather than exact wall-tier clipping.
# Movement and mouse-look are intentionally disabled in both fixed-pose modes.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-seg-classic-context-presentation

# AR-0025 fixed visual pose known to retain all 26 cutout candidates. Add
# --frustum-aabb for the per-draw baseline, or --frustum-grid-8x4x8 for the
# explicitly corpus-local medium-grid comparison.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --spawn-observer --spawn-yaw-plus-90 --frustum-grid-8x4x8
```

If invoked from this `corpus/campaigns/doom/hello-doom-e1m1` directory instead, replace the
package path with `../assets/DOOM/packages/doom-shareware-corpus-v1.zip`.

The default interactive run is the current corpus baseline, not a claim of
complete original-Doom behavior: PreserveNorth source embedding, fixed source
spawn, collision proof, admitted masked-middle cutouts, and the bounded SKY1
panorama are enabled. Use `--overview-camera`, `--no-walk-collision`,
`--no-masked-cutouts`, `--no-doom-sky`, or `--embedding-current-reflected` for
explicit comparison controls. `--diagnostic-sky-omissions` replaces the normal
sky panorama with the purple AR-0027 omission presentation.

The SKY1 panorama remains corpus-local: it uses the real `SKY1` raster, but
does not claim the original view-dependent sky projection or admit a generic
sky contract. Masked cutouts use the admitted generic categorical-coverage
path and do not admit Blend.

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
`--doom-seg-report` is Stage 3B's headless representation control. It reports
the separately lowered, source-labelled SEG wall triangles before they are
uploaded or selected; it is not a renderer SEG feature or a visibility claim.
`--doom-seg-clip-report` and `--doom-hut-clip-report` add the bounded
near-first screen-span diagnostic controls for the source-spawn and
source-derived hut poses respectively. They lower only currently uncovered
source subintervals into separate labelled meshes and retain the result as a
comparison representation. The fixed diagnostic columns are not renderer
pixels or historic-Doom projection, and the resulting meshes are not uploaded
by those report modes.
`--doom-seg-clip-presentation` uploads that same bounded source-spawn
comparison representation as a separate native scene. It deliberately omits
flats and masked middles, retains ordinary wall materials and source labels,
and must be compared manually with the full static shell. The current
horizontal-only control is intentionally retained as a **falsified** variant:
it visibly removes valid spawn-room walls. It is diagnostic presentation
evidence only, not an original-Doom renderer or a Tokimu visibility feature.
`--doom-seg-classic-plane-presentation` uploads the separate source-spawn
plane-column reconstruction as grouped ordinary flat meshes with continuous
source-spatial UVs. It deliberately omits walls and cutouts so visual review
can identify floor/ceiling gaps without confusing the control with a complete
Doom presentation path. The meshes are fixed-pose corpus evidence, not
visplanes, renderer pixels, or a reusable visibility contract. The camera is
structurally locked to that source-spawn pose so movement cannot manufacture
out-of-domain omissions.
`--doom-seg-classic-context-presentation` adds the BSP-admitted opaque
whole-SEG wall tiers to that plane reconstruction. It exists solely to frame
manual plane-gap inspection. The walls have not yet undergone exact projected
tier clipping, so this mode remains neither classic-Doom parity nor a proposed
normal presentation path. Projection alignment removed its broad plane losses,
but retained thin wall/plane edge openings; the mode is therefore retained as
a falsified representation control rather than usable presentation selection.
`--doom-seg-clip-2d-report` is the next headless Stage 3B control. It adds a
bounded source-space vertical-span grid to the same near-first SEG order and
Doom-owned occluder authority. Its rectangular projected spans are still only
comparison evidence; it intentionally has no presentation mode yet.
`--doom-seg-clip-per-column-report` refines that headless grid by intersecting
each diagnostic source ray with its finite source SEG before deriving the
vertical span. It is a more conservative comparison control, not a visual
filter or renderer feature.
`--doom-seg-per-column-turn-trace` repeats the per-column control over four
declared source headings without initializing WGPU. It exists to show that the
candidate set is viewer-dependent before any dynamic source-selection design
is considered.
`--doom-seg-per-column-position-trace` repeats the same headless control at
six declared source offsets around player one. The offsets are test inputs,
not collision-valid movement or a Doom player simulation.
`--doom-seg-per-column-failure-trace` replays the retained close-wall and
courtyard false-negative poses. It is evidence that the dynamic control is
unsound, not an alternate presentation mode. The trace also reports its
non-mutating local-depth audit: a positive `depth_order_inversions` count means
a later SEG attempted to close a cell closer than the SEG that current
near-first-subsector/source-record order had already allowed to close it. This
falsifies that ordering as a direct per-cell occluder order; it does not change
selection or establish a replacement visibility model.
`--doom-seg-per-column-order-trace` compares those same poses with a
diagnostic global nearest-SEG ordering. It exists to separate ordering defects
from boolean-coverage defects; it neither establishes Doom traversal nor
enables a presentation mode.
`--doom-seg-classic-admission-trace` is the next, headless Stage 3B control.
For the retained false-negative poses it applies the directed source-SEG
right-side/front rule, a bounded horizontal FOV preflight, and the existing
Doom-owned solid-versus-pass classification. Solid intervals are unioned in
near-first BSP leaf order, while pass intervals deliberately remain out of that
union. It deliberately stops before BSP bbox pruning, vertical wall-tier
clipping, or any draw selection, so its counts are source-protocol evidence
rather than a presentation claim.
`--doom-seg-classic-bsp-trace` continues that same source-only control through
viewer-side BSP recursion. It visits the near child first, admits only
front-facing/in-FOV SEGs, unions solid intervals, and skips a far child only
when its decoded Doom bbox is safely projected inside an already closed
interval. A bbox that is definitely outside the source horizontal FOV is
separately rejected; unprojectable/ambiguous bboxes fail open. The trace also
retains whether the known exterior suspect at linedef `247` was reached or
admitted, including the source node that rejected its subtree when applicable.
It also inventories existing source-labelled static floor and ceiling meshes
whose owning subsectors were reached. Those numbers are explicitly not Doom
plane spans or selected draws; they make the unimplemented source plane stage
visible rather than permitting the wall protocol to masquerade as a complete
presentation result. The trace separately reports upper/lower/middle source
wall-tier counts for the admitted SEG triangles; that inventory does not infer
opaque/cutout policy from a horizontal solid range.
It additionally reports the first source wall-stage floor/ceiling eligibility
marks at the source eye height, including paired-`F_SKY1` ceiling adjustments.
Those observations precede per-column clip arrays and visplane construction;
they are neither plane spans nor selected flat draws.
This is a diagnostic comparison with the original protocol, not a renderer
candidate API, historic Doom parity claim, or presentation mode.
`--doom-seg-classic-vertical-clip-trace` follows the same recursive source
admission through a deliberately bounded wall-tier checkpoint. It records
upper/lower/middle tier spans, source plane-mark facts, and diagnostic
ceiling/floor clip-boundary changes for a small set of source poses. Middle
tiers remain source presentation facts but do not automatically close a
ceiling/floor boundary. This command constructs neither visplanes nor flat
draws and must not be used as a visibility/presentation selector.
`--doom-seg-classic-plane-identity-trace` records the decoded plane grouping
facts that a later provider-local span experiment would need: admitted floor
and ceiling contributors, distinct `(height, flat, light)` keys, and the
normalization of `F_SKY1` ceilings to a common sky key. It does not allocate
visplanes, construct spans, select flats, or create a presentation mode.
`--doom-seg-classic-plane-span-trace` continues that headless control by
recording bounded per-column floor/ceiling cells before each admitted wall
range changes the clip state. Conflicting writes to the same source plane key
are split into separate diagnostic instances instead of being merged into
fabricated coverage. The result remains a source-protocol observation: it does
not claim classic visplane parity, select flat meshes, upload geometry, or
change presentation visibility.
`--doom-seg-per-column-presentation` is a fixed-source-spawn visual comparison
which retains normal flats/cutouts but substitutes selected whole SEG walls. It
does not update after a camera turn and is not an interactive culling mode.
`--doom-seg-per-column-dynamic` is the separate interactive Stage 3B control:
it uploads lowerable SEG walls once and recomputes only a corpus-local
source-owned draw-enable mask as the observer moves or turns. It deliberately
does not establish generic renderer culling, historic Doom visibility, or a
stable public contract. Current missing source wall materials are reported at
startup rather than synthesized. It is now a retained **falsified** control:
the per-column approximation removes valid nearby walls at reproduced poses.
`--doom-seg-classic-dynamic` is its distinct Doom-owned successor. It uploads
SEG-derived walls once, recalculates the recursive BSP/solid-range source
protocol from the live observer every frame, enables only admitted SEG walls,
and retains whole flat draws only for reached subsectors. Draws without that
retained identity fail open, and unsupported SEG materials retain the original
whole-linedef wall rather than disappearing. This remains an opt-in corpus
control: it preserves caller order but does not claim historic pixel parity,
visplane reconstruction, or renderer-owned visibility. Run it from the repo
root with:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-seg-classic-dynamic
```

`--frustum-grid-8x4x8` renders the retained medium grid experiment so it can be
visually compared with full submission or `--frustum-aabb`. It preserves input
order, rechecks grid survivors with the per-draw AABB test, and falls back to
full submission if no finite grid can be built. It is not an admitted default.
`--spawn-yaw-plus-90` is a fixed corpus observation pose rather than a player
or input policy.

AR-0030 Candidate 1 compares unchanged global-full geometry against the same
geometry plus Doom-owned, view-local authoritative-sky depth. Run the retained
exterior-hut-east pose from the repository root with collision disabled because
the diagnostic pose does not claim player-sector state:

```powershell
# Unchanged global-full control.
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --exterior-hut-east-view --no-walk-collision --measure-two-frames

# Candidate 1: global full plus submission-local authoritative-sky depth.
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --global-full-plus-view-local-sky-depth `
  --exterior-hut-east-view --no-walk-collision --measure-two-frames
```

The Candidate 1 fingerprint in frame logs is submission-scoped and therefore
changes with the submission identity. Geometry-only baseline/jitter/baseline
recurrence is retained by the separate G2 conformance fixture. Neither command
enables AABB/frustum selection, and Candidate 1 adds no persistent mesh identity.

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
It also reports `source_xyz=(map-x,map-y,height)` and
`source_direction=(map-dx,map-dy,vertical)`, plus the corresponding Tokimu
world-space ray. Copy the emitted `replay=--look-ray-report=...` argument into a
headless invocation to reproduce the same ray without navigating the window:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --look-ray-report=1056,-3616,36,0,1,0
```

The headless report prepares the same canonical scene and returns the nearest
prepared triangle with its hit source coordinate and retained draw identity.
It also reports the nearest paired-sky depth boundary and omitted source
`F_SKY1` plane, including whether either falls before or behind the ordinary
hit. A bounded classic-source trace retains the viewer leaf, target leaves,
target SEG admission, and any watched BSP elision. This makes a sky leak
reproducible as a structural source-presentation relationship before
visual-regression automation exists.
It is a deterministic problem-location probe, not a claim that arbitrary free
camera state or rendered visibility has become Doom source truth.
With the console closed, `E` performs that same exact center-wall lookup and
submits the resulting corpus-local `Use` request automatically; aim at a
manual door such as `BIGDOOR2` and press `E`. Its retained response includes
the source linedef, target sector, and number of prepared meshes initially
eligible for the narrow visual lowering.
`STATUS` retains each active manual door as
`sector:current-height/closed-height/open-height:phase` for native inspection.
This is corpus-local Observation Shell evidence under AR-0013, not an admitted
engine console, command language, or generic picking contract.
`CAMERA` also reports the active AR-0028-lowered Doom source pose, so a visual
anomaly can be retained and replayed without treating free camera navigation as
source truth.

When `LOOK` identifies a potentially incorrect wall, retain its source-to-mesh
evidence without launching a renderer:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --wall-source-report=247
```

The report records both source sides, sector floor/ceiling intervals, source
textures, and generated spans. It is diagnostic only and does not classify a
source-valid span as invisible merely because a reference presentation differs.
