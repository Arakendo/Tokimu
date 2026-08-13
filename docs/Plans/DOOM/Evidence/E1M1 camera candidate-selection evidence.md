# E1M1 Camera Candidate-Selection Evidence

## Scope

This record retains the first AR-0025 Stage 0/1 observations from the reviewed
E1M1 static scene. It compares the existing explicit full-submission contract
with a corpus-local conservative CPU frustum/AABB filter. It does not admit a
renderer culling API, scene graph, visibility guarantee, Doom BSP policy, or
portable performance claim.

The scene contains 1,835 opaque draws and 26 categorical-cutout draws. Static
meshes are uploaded once before the frame boundary. Both policies preserve the
existing caller order; the frustum policy only removes candidates wholly
outside one Tokimu GL-style homogeneous clip plane. Missing or non-finite
bounds fail open and remain submitted.

## Reproduction Commands

Run from the repository root:

```powershell
# Deterministic CPU candidate-count report; no GPU initialization.
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-report

# Native Vulkan full-submission baseline; exits after first and warm frames.
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --spawn-observer --measure-two-frames

# Identical scene/camera with the corpus-local conservative filter.
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --spawn-observer --frustum-aabb --measure-two-frames
```

The retained measurements below were made in the development profile on
2026-08-10 with the Vulkan backend and an AMD Radeon RX 7900 XTX discrete GPU.
They are individual observations, not a statistically useful benchmark.
`frame_cpu_us` covers corpus frame preparation through the provider's present
call; it does not measure GPU completion.

## Fixed-Pose Candidate Counts

| Pose | Policy | Candidates | Rejected | Submitted | Uncertain bounds |
| --- | --- | ---: | ---: | ---: | ---: |
| Overview | Full submission | 1,861 | 0 | 1,861 | 0 |
| Overview | Frustum/AABB | 1,861 | 0 | 1,861 | 0 |
| Source spawn | Full submission | 1,861 | 0 | 1,861 | 0 |
| Source spawn, source heading | Frustum/AABB | 1,861 | 1,366 | 495 | 0 |
| Source spawn, yaw +90 degrees | Frustum/AABB | 1,861 | 810 | 1,051 | 0 |
| Source spawn, yaw +180 degrees | Frustum/AABB | 1,861 | 1,817 | 44 | 0 |
| Source spawn, yaw -90 degrees | Frustum/AABB | 1,861 | 1,411 | 450 | 0 |

The overview camera is intentionally constructed to show the complete map, so
zero rejection is a useful control rather than a failed optimization. At the
source spawn, the filter rejected 970 candidates at the left plane, 390 at the
right, two at the bottom, and four at the top. No candidate was classified
outside the near or far plane. The retained 495 draws were all opaque; all 26
cutout candidates were outside this fixed view. The yaw-plus-90 pose retains
1,025 opaque and all 26 cutout draws, so it is the fixed visual target for
checking selected cutout presentation.

## Paired Native Frame Observations

| Frame | Policy | Submitted | Selection CPU | Command build CPU | Frame CPU | Material resolutions | Pipeline switches | Frame mesh uploads/replacements |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| First | Full submission | 1,861 | 74 us | 125 us | 1,066,483 us | 1,861 | 2 | 0 / 0 |
| Warm | Full submission | 1,861 | 67 us | 79 us | 46,281 us | 1,861 | 2 | 0 / 0 |
| First | Frustum/AABB | 495 | 2,220 us | 52 us | 323,598 us | 495 | 1 | 0 / 0 |
| Warm | Frustum/AABB | 495 | 2,588 us | 43 us | 28,257 us | 495 | 1 | 0 / 0 |

Both policies reported 1,861 lifetime mesh uploads and zero lifetime mesh
replacements. This confirms the earlier static-upload repair remained intact:
camera candidate selection did not reintroduce steady-state geometry upload.

The source-spawn filter removed about 73.4% of submitted draws. In this single
paired observation the warm CPU frame was about 38.9% shorter despite roughly
2.6 ms of CPU selection work. That is evidence worth pursuing, not a portable
performance conclusion. Repeated traces, release-profile sampling, browser
execution, and independent scenes remain required.

## Theory Trial: AABB Versus Enclosing Sphere

### Hypothesis and containment

The trial asks whether a cheaper, source-neutral enclosing sphere preserves
enough of the CPU frustum-selection benefit to be preferable to a derived AABB
for this retained workload. Both shapes derive solely from prepared mesh
positions; both reject only when the entire bound lies outside one homogeneous
clip plane; neither changes draw order or becomes a renderer type. A missing or
non-finite shape fails open. The trial remains corpus-local because it has one
scene, no actual visual false-negative review for the sphere, no second caller,
and no stable capability proposal.

### Fixed-pose results

| Pose | AABB submitted | Sphere submitted | AABB selection CPU | Sphere selection CPU |
| --- | ---: | ---: | ---: | ---: |
| Overview | 1,861 | 1,861 | 2,556 us | 1,181 us |
| Source spawn, source heading | 495 | 525 | 2,176 us | 1,006 us |
| Source spawn, yaw +90 | 1,051 | 1,092 | 2,493 us | 1,159 us |
| Source spawn, yaw +180 | 44 | 84 | 2,129 us | 808 us |
| Source spawn, yaw -90 | 450 | 454 | 2,096 us | 861 us |

These are one development-profile deterministic-report observation, not a
benchmark. The sphere classifier was roughly half the measured CPU time in this
run, but it submitted 4 to 40 more draws at the non-overview poses because its
enclosing volume is less tight. The fixed source-heading case also produced two
sphere-only near-plane rejections; synthetic tests establish the conservative
plane rule, but no manual visual review has yet validated the sphere against
E1M1. The result therefore records a tradeoff, not a preferred default.

The report remains reproducible with the earlier `--candidate-report` command;
it now emits both `frustum-aabb` and `frustum-sphere` rows with selection time.

## Theory Trial: Contiguous Candidate Granularity

### Hypothesis and containment

This trial asks whether callers gain a useful CPU/selectivity tradeoff by
declaring contiguous candidate groups rather than testing every draw. The
experiment forms groups of 8 and 32 existing caller-ordered draws and derives
one enclosing AABB per group. A group survives intact when its AABB intersects
the frustum; therefore survivor order is unchanged and group members never
partially disappear. Any missing member bound makes its whole group survive.
The group is explicitly an experimental selection unit: it is not a renderer
batch, material group, source identity, or scene-ownership mechanism.

### Fixed-pose results

| Pose | Per-draw AABB submitted | Group 8 submitted | Group 32 submitted | Group 8 CPU | Group 32 CPU |
| --- | ---: | ---: | ---: | ---: | ---: |
| Overview | 1,861 | 1,861 | 1,861 | 355 us | 132 us |
| Source spawn, source heading | 495 | 760 | 1,088 | 378 us | 135 us |
| Source spawn, yaw +90 | 1,051 | 1,293 | 1,477 | 331 us | 134 us |
| Source spawn, yaw +180 | 44 | 160 | 576 | 323 us | 130 us |
| Source spawn, yaw -90 | 450 | 536 | 800 | 419 us | 128 us |

The grouped trials performed only 233 and 59 bounds tests, respectively,
instead of 1,861. They are much cheaper in this one debug report, but every
non-overview pose submits more draws than per-draw AABBs. The source-heading
case illustrates the cost particularly clearly: grouping by 8 saves roughly
1.8 ms of report-time selection while retaining 265 extra draws; groups of 32
retain 593 extra draws. This is an explicit candidate-granularity tradeoff,
not evidence that arbitrary contiguous chunks are meaningful general scene
objects.

## Retained Turn Trace and Pathological Fixture

`--candidate-turn-trace` records a deterministic in-place 360-degree turn at
the E1M1 player-one source spawn. It changes yaw only: it is neither movement,
collision, source-topology traversal, nor a claim about a playable camera.
The nine 45-degree frames retained 42 to 1,051 draws (4,466 total) from 1,861
candidates per frame, with zero uncertain bounds. The yaw-zero and yaw-360
frames both retained 495 draws, which is a useful closure check for the fixed
trace. The measured report-time selection total was 21,245 us; it excludes
renderer submission and GPU completion.

`--candidate-pathological-report` builds a separate source-neutral fixture of
128 interleaved bounds: 64 are wholly outside the identity clip volume, while
64 cross or overlap it. Per-draw AABB selection safely rejects 64. Because
each contiguous group intentionally mixes safe rejections with visible/crossing
members, group sizes 8 and 32 reject no groups and submit all 128. This is the
expected conservative failure mode and demonstrates why a low group count must
not be mistaken for effective selection.

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-turn-trace

cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --candidate-pathological-report
```

## Declared Source-Relative Position Trace

`--candidate-position-trace` uses five fixed local forward offsets from the
reviewed player-one source spawn: `-256`, `-128`, `0`, `128`, and `256`. Its
camera positions are retained in the report; no collision, map traversal,
player state, or original-Doom movement claim is made. It is simply a
source-provenanced set of camera inputs for the conservative per-draw AABB
baseline.

| Offset | Camera position | AABB submitted / CPU | Sphere submitted / CPU | Group 8 submitted / CPU | Group 32 submitted / CPU |
| ---: | --- | --- | --- | --- |
| -256 | `(1056, 36, -3872)` | 597 / 2,244 us | 627 / 946 us | 816 / 358 us | 1,184 / 140 us |
| -128 | `(1056, 36, -3744)` | 562 / 2,211 us | 583 / 944 us | 792 / 352 us | 1,152 / 144 us |
| 0 | `(1056, 36, -3616)` | 495 / 2,249 us | 525 / 1,387 us | 760 / 416 us | 1,088 / 161 us |
| +128 | `(1056, 36, -3488)` | 383 / 2,162 us | 431 / 907 us | 664 / 355 us | 992 / 140 us |
| +256 | `(1056, 36, -3360)` | 309 / 2,156 us | 347 / 887 us | 544 / 351 us | 928 / 139 us |

All five frames had 1,861 candidates and zero uncertain bounds. The same
shape/granularity tradeoff held at every offset: sphere and groups used less
CPU in this debug report, while per-draw AABBs submitted the fewest draws. This
extends the evidence beyond yaw-only selection pressure, but does not validate
a traversable path or replace a future source-topology/physics study.

## Stage 2: Static Uniform-Grid Broad Phase

The Stage-2 corpus experiment builds static 3D uniform grids over the derived
draw AABBs. A draw is inserted into every covered cell; a query skips empty
cells, rejects wholly off-frustum occupied cells, restores surviving draw
indices in their original input order, and then applies the exact existing
per-draw AABB test. Missing bounds survive outside the index. The grid is thus
a fail-open broad phase, not a replacement visibility truth.

Three resolutions were built once and queried over the overview, a nine-frame
360-degree yaw trace, and the four nonzero declared source-relative position
offsets. Every final draw count matched the Stage-1 per-draw AABB result at all
retained poses; no bound was uncertain.

| Resolution | Occupied / total cells | Memberships | Estimated storage | Build CPU |
| --- | ---: | ---: | ---: | ---: |
| `4x2x4` | 32 / 32 | 2,867 | ~35,104 B | 407 us |
| `8x4x8` | 177 / 256 | 4,503 | ~57,280 B | 462 us |
| `16x4x16` | 620 / 1,024 | 6,900 | ~101,824 B | 599 us |

| Retained pose | Per-draw final | `4x2x4`: grid candidates / CPU | `8x4x8`: grid candidates / CPU | `16x4x16`: grid candidates / CPU |
| --- | ---: | --- | --- | --- |
| Overview | 1,861 | 1,861 / 2,169 us | 1,861 / 2,434 us | 1,861 / 2,874 us |
| Spawn heading | 495 | 1,038 / 1,222 us | 722 / 1,033 us | 571 / 1,403 us |
| Spawn yaw +90 | 1,051 | 1,489 / 1,691 us | 1,259 / 1,634 us | 1,191 / 2,223 us |
| Forward offsets `-256` through `+256` | 597 to 309 | 1,066 to 656 / 1,230 to 784 us | 939 to 559 / 1,259 to 850 us | 793 to 468 / 1,608 to 1,257 us |
| Nine yaw frames | 42 to 1,051 | 128 to 1,489 / 212 to 1,691 us | 126 to 1,259 / 371 to 1,634 us | 72 to 1,191 / 817 to 2,223 us |

The grid verifies a conservative index can reduce exact tests without changing
survivor order, but it does not establish a universally preferable resolution:
finer cells improve the source-heading candidate set while increasing occupied
cell traversal, index storage, and some query costs. This is one static,
development-profile Doom-derived scene, not an index admission, dynamic-scene
solution, visual false-negative proof, or general performance claim.

### Medium-Grid Native Playback

`--frustum-grid-8x4x8` is an intentionally fixed corpus playback configuration
for visual comparison, not a selected index policy. It builds separate ordered
opaque and cutout grids, rechecks every grid survivor using the exact per-draw
AABB test, and falls back to full submission if an index cannot be formed.
At the yaw-plus-90 source-spawn pose, its native two-frame observation was:

```text
submitted: 1,051 of 1,861
opaque / cutout: 1,025 / 26
uncertain bounds: 0
mesh uploads / replacements on first and warm frames: 0 / 0
selection CPU: 2,039 us first; 2,266 us warm
```

The counts exactly match the existing per-draw AABB result. The playback CPU
observations must not be compared directly with the headless one-grid timings:
the rendered experiment maintains opaque and cutout ordering with separate
grids. A manual image observation remains required before this is described as
visually omission-free.

A subsequent native maintainer run retained the same 1,051 draws with 1,855 us
first-frame and 1,820 us warm-frame selection observations, again with zero
uncertain bounds and zero steady-state mesh uploads/replacements. This is a
corroborating development-profile observation, not a benchmark.

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-grid-report
```

```powershell
# AR-0025 medium-grid visual comparison at the fixed cutout-survivor pose.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --spawn-observer --spawn-yaw-plus-90 --frustum-grid-8x4x8
```

## Stage 2 Theory: One-Frame Temporal Candidate Carry

This corpus-only test always runs fresh per-draw AABB selection. It then reports
the union of that fresh result with the immediately preceding candidate set,
solely to measure temporal overlap; the prior frame never suppresses fresh
classification or becomes visibility truth.

| Pose transition | Fresh candidates | One-frame carry | Expanded current view | Expanded prior overlap |
| --- | ---: | ---: | ---: | ---: |
| initial -> yaw `0°` | 495 | 495 | 550 | — |
| yaw `0°` -> `5°` | 532 | 534 | 578 | 544 |
| yaw `5°` -> `10°` | 554 | 562 | 601 | 576 |
| yaw `10°` -> abrupt `190°` | 38 | 580 | 38 | 12 |
| abrupt `190°` -> declared forward teleport `+1024` | 24 | 58 | 44 | 8 |

Smooth camera motion has high candidate overlap, but the discontinuities make a
carried set substantially less selective. Because fresh selection remained
authoritative, the test establishes no CPU reduction. It is negative evidence
against treating a naive temporal carry as a culling answer. A 72-degree
expanded current frustum retained every 60-degree fresh candidate but added
work and still lost overlap at discontinuities. Neither approach reduces the
authoritative fresh classification, so neither is a temporal culling answer.

```powershell
# AR-0025 temporal-overlap theory report; it does not initialize WGPU.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --candidate-temporal-report
```

## Stage 3 Preflight: Doom Source-Topology Boundary

The current static preparation can locate the player-one point in a source BSP
subsector and retains subsector provenance for flat surfaces. Its wall and
cutout draw plans intentionally retain linedef/sidedef provenance instead. A
single linedef may bound multiple subsectors, so there is no existing safe
wall-to-leaf attribution for a BSP candidate filter.

The decoded map now retains a bounded `REJECT` matrix with its original source
meaning: LSB-first row-major bits where the row is a monster sector and the
column is a player sector. `--doom-reject-report` observed 85 sectors, 904
bytes, and 9 monster sectors forbidden to sight player-one sector 38 (76 not
forbidden). This is not rendering visibility and is not fed into candidate
selection. A too-short matrix remains a decode failure; no partial fallback is
invented.

```powershell
# Source-only Doom REJECT observation; it does not initialize WGPU.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-reject-report
```

No Stage-3 BSP or `REJECT` selection claim is made until the remaining
comparison protocol is explicitly established. These are source-specific
research facts; they do not affect `tokimu-render` or the current
full-submission fallback.

The one-to-many attribution is now established by `SEGS`: E1M1 reports 475
linedefs, with 269 occurring in one subsector and 206 occurring in multiple
subsectors (maximum six). Existing wall/cutout meshes are whole-linedef meshes,
so a source-leaf filter can conservatively retain a wall whenever any member
leaf survives, but cannot become a leaf-granular culler. A genuinely selective
BSP comparison needs a separately authorized source-seg geometry experiment
that preserves side and texture-offset semantics.

Stage-3A's headless membership-union control derives conservative subsector
bounds from source BSP regions and sector heights. At the fixed source-spawn
yaw-plus-90 pose, 136 of 237 source subsectors survive and 1,115 of 1,861 draws
remain (including cutouts); generic per-draw AABB selection retains 1,051 at
the same pose. The 64-draw gap is expected conservative whole-linedef
coarseness, not evidence of a visual error. This is count-only evidence;
timing and visual comparison remain open.

The subsequent headless development-profile observation retained selection CPU
of 337 microseconds at overview and 311 microseconds at source-spawn yaw plus
90. It measures only source-leaf bound classification and membership union,
not renderer command construction, GPU work, or a benchmark.

The renderable Stage-3A fixed-pose control submitted 1,115 draws (1,089 opaque
and all 26 cutouts), with 746 source-topology rejections, no uncertain bounds,
and no warm-frame mesh uploads/replacements. Its first/warm selection CPU was
959/665 microseconds and command construction 105/47 microseconds; frame
observation was 730080/34608 microseconds. These are development-profile
comparisons only. A manual image comparison remains open and is limited to no
additional omissions relative to the incomplete static baseline.

```powershell
# Source-only SEGS-to-SSECTORS membership observation; no WGPU initialization.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-topology-report
```

## Correctness and Evidence Limits

- Unit evidence proves an inside AABB is retained, a wholly outside AABB is
  rejected, an intersecting AABB is retained, invalid bounds fail open, and
  surviving inputs preserve caller order.
- Unit evidence also proves the enclosing-sphere counterpart retains inside and
  intersecting spheres while rejecting only a wholly outside sphere.
- Grouped-AABB unit evidence proves a group containing a crossing candidate or
  an uncertain member remains wholly submitted rather than producing a partial
  or fail-closed omission.
- The real-package report proves all 1,861 prepared E1M1 candidates yield finite
  bounds and provides bounded source-label/rejection-plane samples.
- The current static E1M1 baseline does not yet texture/materialize every
  source surface. This does not change already prepared mesh bounds or the
  comparative candidate counts, but it limits any visual conclusion to: the
  selected path introduced no additional visible omission relative to that same
  incomplete baseline. It is not evidence of complete E1M1 presentation.
- No manual side-by-side native image observation has yet established that the
  source-spawn filter introduced no visible omission.
- The deterministic yaw-plus-90 pose supplies cutout-survivor pressure, but its
  full-submission/selected visual comparison remains manual and unrecorded.
- Browser/WASM selected-cutout count/presentation evidence is retained; timing
  evidence remains intentionally open because its one-shot fixture uploads
  candidate meshes before command selection.
- The two-frame native numbers are startup/warm observations, not a benchmark;
  they cannot establish variance, GPU completion cost, or a general budget.

## Validation

```text
cargo fmt --all -- --check
cargo test -p hello-doom-e1m1 --bin static_scene
    9 passed
cargo clippy -p hello-doom-e1m1 --bin static_scene --no-deps -- -D warnings
    passed with upstream warnings suppressed for the already-recorded AR-0019 provider issue
```

The focused Clippy run also exposed and repaired a mechanical existing
`unnecessary_filter_map` warning in the E1M1 cutout-upload helper. That cleanup
does not change eligibility or upload behavior.

## Browser/WASM Selected-Cutout Build Readiness

On 2026-08-10 the `doom-ts-boundary-workbench` gained a separate
`render_static_e1m1_selected_cutouts(canvas)` request. After the existing
explicit local package selection, Rust/WASM resolves E1M1's player-one source
spawn and its sector height, selects the fixed yaw-plus-90 pose, derives the
same conservative AABBs as the native fixture, and preserves original opaque
then cutout command order while rejecting only bounds wholly outside one clip
plane. TypeScript exposes only the browser gesture/canvas and returned string.

The following checks pass:

```text
cargo check -p doom-ts-boundary-workbench-engine --target wasm32-unknown-unknown
cargo build -p doom-ts-boundary-workbench-engine --target wasm32-unknown-unknown --release
wasm-bindgen ... --target web --out-dir corpus/consumers/doom-ts-boundary-workbench/web/pkg
node corpus/consumers/aspnet-wasm-asset-workbench/node_modules/typescript/bin/tsc \
  -p corpus/consumers/doom-ts-boundary-workbench/tsconfig.json --noEmit
```

Manual browser selection/presentation was completed on 2026-08-10 after an
explicit local selection of the reviewed package. The returned observation was:

```text
browser first frame presented: 1051 draws; candidates=1861; rejected=810;
opaque=1025/1835; cutouts=26/26; frustum_aabb=true; backend=browser-webgpu;
device=other; adapter=; canvas=960x600
```

This exactly matches the native deterministic fixed-pose count: 1,025 opaque
and all 26 cutout candidates survive the yaw-plus-90 filter. It establishes
browser/WASM selection and first presentation for this bounded consumer path;
it is a manual observation, not a pixel-equivalence or performance claim. The
one-shot browser fixture uploads candidate meshes before filtering submitted
commands, so it does not measure a warm-frame performance saving.
