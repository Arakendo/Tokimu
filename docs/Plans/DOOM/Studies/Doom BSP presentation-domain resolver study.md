# Doom BSP Presentation-Domain Resolver Study

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Role | Bounded R&D study and successor to the parked literal ordered-result experiment |
| Status | Parked — evidence and diagnostics retained; presentation successor is the authorized render-subsector actual-camera study |
| Parent reviews | [AR-0025](../../../Architectural%20Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md), [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Controlling plan | [DOOM WAD Checklist](../DOOM%20WAD%20Checklist.md) |
| Parked predecessor | [Doom Ordered Source-Occurrence Preparation](Doom%20ordered%20source%20occurrence%20preparation.md) |
| Primary source analysis | [Classic Doom Renderer Dataflow And Tokimu Preparation Seam](../Evidence/Classic%20Doom%20renderer%20dataflow%20and%20Tokimu%20preparation%20seam.md) |
| Initial corpus | Reviewed `DOOM1.WAD` E1M1, source spawn, neutral pitch |
| Initial execution mode | Headless shadow observation over the original complete contribution inventory |
| Stable API authority | None |
| Renderer changes authorized | None |
| Presentation successor | [Doom Render-Subsector Actual-Camera Preparation](Doom%20render-subsector%20actual-camera%20preparation.md) |

## Why This Study Exists

The parked ordered-occurrence candidate proved exhaustive accounting, source
provenance, terminal disposition conservation, live camera refresh and current
runtime-height input. Its complete source-spawn submission nevertheless shows
large peripheral and smaller interior holes through which the sky background
is visible.

The retained frame reports `458` draws: `445` opaque prepared declarations,
`12` cutout declarations and the sky pass. The failure is therefore not an
incomplete composition swap. A balanced result answered the wrong presentation
question, made incorrect dispositions, or realized correct evidence
incompletely.

This study circles back to Doom's BSP as a complete viewer-relative
presentation-domain mechanism rather than using isolated BSP facts to support
another candidate. It asks whether an original, Tokimu-owned resolver can use
decoded BSP structure, current runtime state and the actual Tokimu camera to
produce a useful source participation domain before renderer submission.

## Research Question

For geometry visible through each retained source-spawn hole:

> Which source contribution and subsector own the expected surface, how does
> the viewer-relative BSP traversal treat its root-to-leaf path, and can a
> Doom-private resolver conservatively retain the necessary source domain
> without copying Doom presentation semantics into `tokimu-render`?

The first goal is explanation, not draw-count reduction.

## Terminology And Candidate Shape

- **BSP construction** — the authored/built nodes, partition lines, SEGs and
  convex subsectors decoded from the WAD.
- **BSP traversal** — current-view near-first visitation plus any
  coverage-dependent decision about a far child.
- **Participation domain** — source-attributed subsectors, SEGs and plane
  regions that may still contribute to one prepared view.
- **Shadow resolver** — computes a manifest and disagreements but cannot alter
  renderer submission.
- **Source-authoritative resolver** — may eventually remove or partition Doom
  contributions for one explicit prepared-view identity. It is not a generic
  conservative filter.
- **Generic post-filter** — source-neutral AABB/frustum selection. It remains
  downstream and is not part of this study's initial candidate.

Candidate dataflow:

```text
decoded Doom BSP + explicit runtime-height snapshot + exact Tokimu camera
        ↓
Doom-private BSP presentation-domain resolver
        ↓
ordered source participation manifest + bounded reasons
        ↓ initially, shadow comparison only
complete original contribution inventory
        ↓ later only if earned
ordinary Tokimu declarations
        ↓
tokimu-render
```

## Architectural Clamp

- Keep BSP nodes, subsectors, SEGs, solid ranges, Doom plane rules and source
  rejection reasons inside the Doom campaign/provider.
- `tokimu-render` receives only ordinary declarations. It does not traverse a
  Doom BSP or own visibility, scene topology or runtime sector state.
- Do not reopen AR-0025 as a shared capability merely because Doom's BSP is
  useful for Doom.
- Begin with the original complete contribution inventory. Do not use the
  parked 458-draw result as unquestioned truth or as the only resolver input.
- Preserve source order and distinct source, occurrence, resource, view and
  submission identities.
- Dynamic solid/pass classification consumes an immutable snapshot of current
  heights. Activation, timing, waiting and reversal remain application policy.
- Unknown mappings, ambiguous projection, unsupported source roles and
  incomplete provenance fail open in shadow output.
- A lower draw count is not success. Any unexplained visible omission rejects
  a presentation-affecting candidate.
- Do not add sky depth, world-space closure geometry, generic filtering or a
  second screen-column patch to make an incomplete resolver appear correct.
- Primary-source behavior may be studied and cited. Do not copy GPL source
  text into Tokimu without a separate licensing decision.

## Working Hypotheses

### H1 — BSP attribution explains the large holes

Large peripheral holes correspond to source subsectors or SEGs that the parked
candidate never admitted, terminally rejected, or associated with a prepared
view different from the renderer camera.

### H2 — More than one failure family exists

Large missing regions arise from participation-domain or whole-disposition
errors, while small wedges and rectangles arise from partial SEG/plane
realization or shared-boundary loss.

### H3 — A useful BSP resolver may be conservative

A resolver over the original inventory may retain a source-attributed superset
of required work while pruning definitely irrelevant BSP subtrees. Exact Doom
wall/plane occurrence generation may remain a separate downstream operation.

H3 is deliberately weaker than historic Doom pixel parity. If safe
participation cannot be separated from the coupled wall/vertical-plane
protocol, record that result rather than disguising the complete protocol as a
prefilter.

## Slice 0 — Freeze Baselines And Failure Specimens

- [x] Park the literal ordered-result study without deleting its code,
      conservation reports or six-ray evidence.
- [x] Record that source spawn presents a complete 458-draw frame with severe
      sky leakage, falsifying visual completeness rather than atomic refresh.
- [ ] Retain the marked source-spawn screenshot at a repository evidence path.
- [ ] Capture the exact source camera, Tokimu view/projection matrices, surface
      size, aspect ratio, near/far planes and prepared-view parameters.
- [ ] Retain three baselines at the same pose: original global full submission,
      parked ordered result and the existing source-faithful diagnostic path.
- [ ] Confirm the preliminary projection comparison: neutral-pitch Tokimu view
      is approximately `85.5° x 60°`, inside the approximately `90° x 64°`
      reference preparation domain, or record the actual discrepancy.

Acceptance: every later trace names an immutable view identity and one of the
three baseline submissions.

## Slice 1 — BSP Construction Primer From E1M1 Facts

- [ ] Retain decoded counts and invariants for nodes, subsectors, SEGs,
      linedefs and sectors without introducing new engine types.
- [ ] Trace how BSP splits relate one linedef to multiple SEGs and one sector
      to multiple subsectors.
- [ ] Prove or falsify the assumptions currently made by contribution-to-SEG,
      SEG-to-subsector and subsector-to-sector correlations.
- [ ] Add focused controls for root/leaf encoding, partition-side
      classification, malformed child references and unsupported ownership.
- [x] Document what convex subsector membership guarantees and explicitly what
      it does not guarantee about visibility or complete plane coverage.

Refinement from the first live suspicious-floor replay:

- [x] Report both explicit SEG-endpoint bounds and the inferred root-to-leaf
      convex region for a hit plane subsector.
- [x] Retain whether the exact triangle hit lies inside each domain and its
      distance beyond the SEG-endpoint bounds.
- [x] Audit whether child bounding boxes may safely classify wall/SEG
      participation only, while inferred plane regions require a distinct
      visibility/occurrence rule.

### Canonical E1M1 baked-data audit

The headless control is:

```text
--doom-bsp-bounds-audit-report
```

It independently reads every 28-byte raw NODES record, compares all partition,
box and child fields with the decoded map, recursively computes each child
subtree's descendant SEG-endpoint envelope, then separately computes the
envelope of the inferred convex regions for the same descendant subsectors.
The canonical `DOOM1.WAD` E1M1 result is:

```text
raw NODES records                    236/236 exact decoded matches
child boxes                          472
descendant SEG envelopes contained  472/472
descendant SEG underbounds           0
inferred plane regions contained     149/472
inferred plane-region overruns       323/472
```

Nodes `95` and `96` reproduce the live subsector `97`/`99` family. Node `96`
right has baked box `[-3360,-3392,928,1184]` while the inferred regions for
descendant subsectors `{96,97}` span
`[928,-3552,1184,-3360]`. Node `95` left has a degenerate-y baked box
`[-3392,-3392,896,928]` while subsector `99`'s inferred region spans
`[896,-3552,928,-3360]`.

This acquits both byte decoding and the canonical bake for their demonstrated
representation: every raw field round-trips and every child box contains all
descendant SEG endpoints. The much larger `323/472` plane-region mismatch is
not malformed data; it proves that convex partition-path reconstruction adds
support outside the source structures those boxes bound. Convex membership
supports plane reconstruction and point containment, but it does not imply
that a child box bounds that reconstructed plane or that child visitation is
whole-plane visibility.

Retained rule: a bound has negative authority only over the representation it
actually bounds. Rebaking remains a later diagnostic perturbation, not a fix
or replacement for the canonical WAD. The reusable debugging guidance is
retained in [Bounds Authority Follows The Bounded Representation](../../../lessions/bounds-authority-follows-bounded-representation.md).

Acceptance: later traversal diagnostics can identify every visited or pruned
child and correlate every target SEG/subsector back to stable source records.

## Slice 2 — Four-Ray Source-Spawn Waterfall

Choose four deterministic rays from the marked image:

1. center of the large upper-left leak;
2. center of the large lower-right leak;
3. center of one interior horizontal leak; and
4. center of one small wedge or rectangular leak.

For each ray retain:

- [ ] window pixel, normalized device coordinate and world/source ray;
- [ ] whether it lies inside the exact prepared view domain;
- [ ] nearest expected contribution under global full submission;
- [ ] source linedef/SEG or plane/subsector identity;
- [ ] root-to-leaf BSP path with viewer side and near/far child at each node;
- [ ] far-child bounding-box projection and accumulated coverage at the
      decision point;
- [ ] visited, pruned, admitted, rejected or unresolved outcome with reason;
- [ ] parked Slice 6B disposition and surviving occurrence domain;
- [ ] lowered declaration identity, triangle coverage of the ray and final
      handoff membership; and
- [ ] bounded classification as view-domain, traversal/disposition, lowering,
      face-culling or renderer-realization failure.

Acceptance: no sampled hole remains described only by screenshot coordinates.
If large and small holes differ, split subsequent fixtures by failure family.

## Slice 3 — Headless Shadow BSP Resolver

- [ ] Define one Doom-private immutable input containing decoded BSP facts,
      current runtime heights and exact prepared-view projection.
- [ ] Traverse near-first and retain an ordered visit manifest before applying
      coverage-dependent subtree pruning.
- [ ] Add far-child bounding-box projection with explicit inside, outside,
      covered and unresolved outcomes.
- [ ] Correlate visited subsectors and SEGs with the original complete wall,
      floor, ceiling, sky and cutout contribution inventory.
- [ ] Emit retained, definitely irrelevant and unresolved/fail-open source sets
      while preserving original relative order.
- [ ] Run only in shadow mode and report intersections and set differences
      against global full, parked Slice 6B and reference-planner manifests.
- [ ] Retain bounded reasons for every subtree and source contribution; aggregate
      counts alone are insufficient.

Acceptance: every original source contribution is accounted for, but the
resolver has not yet changed renderer submission.

## Slice 3A — Full-Submission Classification Instrument

This diagnostic precedes any presentation-affecting resolver. It paints the
shadow resolver's beliefs onto the original complete contribution inventory;
classification never controls draw membership.

- [x] Add `--bsp-diagnostic-full` over unchanged global-full geometry and
      reject combinations with generic or SEG presentation selectors.
- [x] Use checked-in PNG families and draw-local tint overrides; do not mutate
      source materials or teach `tokimu-render` Doom vocabulary.
- [x] Keep the taxonomy source-faithful: floor plane, ceiling plane, one-sided
      wall, code-1 door, two-sided boundary, unequal-height two-sided boundary,
      masked middle and presentation-global skybox. “Window” and “stair”
      remain human interpretations, not stored source semantics.
- [x] Render accepted contributions with a bright family tint, positive
      solid-range rejections with a dark/desaturated family cast and ambiguous
      input with unmistakable purple that remains fail-open.
- [x] Add `all`, `accepted`, `rejected` and `unresolved` focus modes. Every
      mode submits the same geometry and merely dims nonmatching records;
      `Z`, `X`, `M` and `Q` switch those modes live, respectively.
- [x] Retain per-draw reason values: ordered SEG admitted, visited source
      plane, positive terminal solid range, projection/traversal ambiguity or
      presentation-global.
- [x] Refine an outside-FOV plane absence as
      `source-plane-child-seg-bounds-outside-fov`; it remains unresolved and
      cannot become a rejection merely because the child SEG bounds miss part
      of the inferred convex plane region.
- [x] Extend `LOOK` headless replay so a hit reports diagnostic family,
      classification, reason and stable source identity.
- [x] Add `LOOK PIXEL <x> <y>` and `LOOK NDC <x> <y>` so an off-crosshair
      colored contribution can be inspected without rotating the camera and
      changing the view-relative shadow classification.
- [x] Add bounded `SCAN [columns rows]` viewport sampling. It freezes the
      current camera and runtime-height manifest, casts deterministic cell-
      center rays, groups nearest rejected/unresolved prepared-triangle hits
      by source identity and emits representative `LOOK PIXEL` commands.
- [x] Keep scan acceleration and authority separate: the initial scan uses
      exact prepared triangles; AABB/frustum data may later accelerate ray
      candidates but cannot classify a contribution or remove it.
- [x] Add explicit headless viewport replay through
      `--bsp-diagnostic-scan-report=<source camera...>`. It runs the shared
      scan unit and automatically emits the complete representative LOOK
      waterfall for every reported suspicious group.
- [x] Make live `CAMERA` print a copy-ready headless scan flag containing exact
      source origin, center direction, client dimensions and grid size.
- [x] Recompute the diagnostic from the current camera and immutable current-
      height snapshot while movement and free look remain enabled.
- [x] Retain a two-frame source-spawn GPU control for both `all` and `rejected`
      focus: `1,849` candidates, `1,849` submitted, zero renderer removals and
      identical `1,852` command counts including sky/cursor presentation.
- [ ] Capture source-spawn `all`, `rejected` and `unresolved` images and verify
      visually that no original geometry disappears between focus modes.
- [ ] Identify the four marked leak targets in global-full and retain their
      classification/reason through `LOOK` replay.

Current asset legend:

```text
floor plane         -> Light/texture_04 + green
ceiling plane       -> Light/texture_07 + yellow
one-sided wall      -> Light/texture_03 + red
code-1 door         -> Light/texture_10 + orange
two-sided boundary  -> Light/texture_06 + cyan
height transition   -> Light/texture_12 + blue
masked middle       -> Light/texture_01 + magenta
skybox              -> Light/texture_13 + dark blue

accepted             -> bright category color
rejected-solid-range -> dark/desaturated category color
rejected-outside-frustum -> dark/desaturated category color
unresolved-fail-open -> unmistakable purple
```

Acceptance: the rendered draw membership remains the global-full control,
semantic family and disposition are independently legible, suspicious colors
lead to an exact reason through `LOOK`, and the visualization itself cannot
manufacture a missing surface.

### Live suspicious-floor evidence — subsector 113

The first user-selected pink floor is not a family error. Purple means
`unresolved-fail-open`. The exact replay is:

```text
--look-ray-report=883.622253418,-3036.436767578,36.000000000,0.748634815,0.177807897,-0.638694227
```

The ray starts in subsector `112` and hits the source floor triangle for
subsector `113` at `(925.819,-3026.415,0)`. Subsector `113` has one explicit
SEG, whose endpoint bounds are `x=896..928, y=-3072`; the hit is `45.585`
source units outside those degenerate bounds. Its nine-step root-to-leaf BSP
path instead infers a four-vertex convex plane region bounded by
`x=896..928, y=-3104..-2992`, and the hit is inside that region.

Node `107` correctly projects its far-child SEG bounds outside the horizontal
view (`-84.926°..-52.650°`), even though the actual hit is near the center
(`-0.451°`). This falsifies treating a child box over explicit wall/SEG
endpoints as complete negative visibility evidence for the reconstructed
whole-subsector plane. It does not show that the convex flat reconstruction is
oversized; that reconstruction follows the retained BSP partition path. The
shadow diagnostic therefore keeps the plane purple and submitted.

An additional live capture showed why center-only inspection is insufficient:
the crosshair hit accepted wall linedef `135`, while the suspicious purple
floor remained off-axis at the lower right. Pixel/NDC LOOK sampling now uses
the unchanged camera view for classification and only varies the inspection
ray. A standalone `--look-ray-report` still identifies exact geometry, but its
ray heading alone is not a complete replay identity for an off-axis
view-relative classification; retain the camera view alongside such a ray.

`SCAN` defaults to `32x20`, accepts bounded custom grids through
`SCAN <columns> <rows>`, caps work at `4,096` samples and reports at most `24`
suspicious source groups. Its result means nearest prepared-triangle shadow
classification, not rendered-pixel parity: masked texture alpha, raster sample
locations and GPU depth precision are intentionally not claimed.

Headless syntax:

```text
--bsp-diagnostic-full \
--bsp-diagnostic-scan-report=<source-x,source-y,source-z,center-dx,center-dy,center-dz,width,height[,columns,rows]>
```

The optional grid defaults to `32x20`. Headless output first retains the
frozen scan view and summary, then expands each suspicious group's
representative pixel through exact prepared-triangle hit, source geometry,
sample-centered classic trace, frozen-view shadow classification and heading
offset. Static headless replay explicitly names its runtime-height input as
the static-scene snapshot; it does not pretend to reconstruct an unrecorded
live door/platform state.

A `16x10` control at source camera
`(-29.114916,-3236.915527,140)`, center direction
`(-0.494670,-0.868813,-0.021598)`, reproduced floor subsector `64` as the sole
suspicious group and automatically expanded its representative pixel
`(1160,600)`. The exact hit was inside the inferred region and `55.046` units
beyond the leaf's explicit SEG box, matching the interactive finding.

The first live `32x20` scan at the wall/window capture conserved all `640`
samples as `570` ordinary prepared-triangle hits plus `70` misses. Of the hits,
`509` were accepted, none were positively rejected, and all `61` suspicious
samples grouped into exactly one contribution:

```text
family=floor classification=unresolved-fail-open
reason=source-plane-child-seg-bounds-outside-fov
source=flat subsector=64 sector=29 plane=Floor
pixel-bounds=(820,460)..(1260,780)
representative=LOOK PIXEL 1220 460
```

This matches the visible lower-right purple region and separates it from the
accepted wall under the crosshair and the blue height-transition boundary.
At this sampling density there is no visible positive solid-range rejection;
the complete suspicious region is another whole-plane/SEG-child-bounds
disagreement.

The representative exact hit at `LOOK PIXEL 1220 460` lands at
`(-240.941,-3291.664,104)` in source coordinates. Subsector `64` has one SEG
with endpoint bounds `x=-240..-208, y=-3264`; the hit is `27.680` units beyond
that box. Its thirteen-step inferred convex region spans
`x=-256..-128, y=-3296..-3264`, contains the hit, and therefore supports the
prepared floor geometry.

The sampled ray's own heading is `-168.884°` and a trace centered on that ray
reaches subsector `64`. The purple classification belongs to the frozen camera
view, whose different center heading classifies the child SEG box outside its
FOV. This is expected and sharpens the finding: rotating the traversal toward
the plane can visit the leaf, but the original camera already sees part of the
larger inferred plane region while the smaller SEG box lies outside. Sampled
LOOK output now prints both headings explicitly.

A subsequent live scan after the rebuilt viewer/view changed produced `543`
hits and `97` misses: `469` accepted, zero rejected and `74` unresolved. Once
again every suspicious sample grouped into floor subsector `64`, sector `29`,
with the same child-SEG-bounds reason; only its screen footprint changed to
`(660,540)..(1260,780)`. This is a useful refresh control: camera/view changes
alter the sampled footprint and counts without changing the retained source
identity or manufacturing a positive rejection.

A third view produced three suspicious groups and exposed the first distinct
camera-domain family:

- ceiling subsector `67`, sector `29`: ten samples. The exact hit is `5.667`
  units outside its sole SEG box but inside its sixteen-step inferred convex
  region, repeating the established plane-bounds mismatch;
- wall linedef `101`, sidedef `137`: two extreme-right samples. The exact hit
  is inside both SEG and inferred-region bounds and SEG `235` is admitted when
  the sample ray is treated as a centered trace; and
- wall linedef `107`, sidedef `143`: one extreme-right sample. Its hit is also
  inside both domains and SEG `232` is admitted by the centered sample trace.

The frozen camera heading is `-154.944°`. The ceiling and two wall sample-ray
headings differ from it by `-52.619°`, `-48.688°` and `-45.416°`, respectively.
They therefore lie beyond the shadow traversal's fixed classic `±45°`
horizontal source domain even though the pitched Tokimu perspective displays
them at the viewport edge. This is not evidence of incorrect wall provenance
or a failed SEG admission: it is direct free-look/camera-domain pressure.
Wall classification now reports `source-wall-seg-bounds-outside-fov` instead
of generic ambiguity, and sampled LOOK prints the normalized heading offset.

### Conservative prepared-geometry guard

The shadow classifier now separates negative authority by contribution
family. An exact wall contribution may still report positive rejection when
all of its matching finite SEGs belong to solid-range-pruned leaves. A whole
floor or ceiling contribution may not inherit that authority from a child
SEG proxy alone. For a solid-range-pruned plane, rejection is retained only
when the actual prepared draw AABB is also definitely outside the actual
Tokimu view frustum. If the prepared bounds intersect the frustum, the result
is `unresolved-fail-open` with reason
`prepared-geometry-frustum-vetoed-plane-rejection`. If camera or bounds
evidence is unavailable, it fails open as
`projection-or-traversal-ambiguous`.

This is deliberately a false-negative guard, not a second visibility oracle:

```text
prepared geometry definitely outside actual frustum + BSP negative
    -> diagnostic rejection may stand
prepared geometry intersects actual frustum + BSP plane proxy negative
    -> retain and diagnose disagreement
missing or ambiguous geometric evidence
    -> retain fail-open
```

The guard changes diagnostic disposition only. Full diagnostic submission
still contains the unchanged `1,849` ordinary declarations. It does not mark
intersecting geometry accepted, perform occlusion, or grant generic AABB
selection Doom participation authority. A focused test fixes all three plane
outcomes. The retained `16x10` subsector `64` headless replay still reports
`143` accepted hits, zero rejected hits and two unresolved hits in one floor
group; its representative remains `55.046` units beyond the explicit SEG box
and inside the inferred plane region.

A two-frame source-spawn GPU control (superseded by the positive-authority
correction below) reports `477` accepted, `262` rejected
and `1,110` unresolved declarations, including exactly `158` prepared-frustum
plane vetoes. The earlier pre-guard baseline was `477/420/952`; therefore the
complete `158`-draw change is rejected-to-unresolved reclassification, with
zero renderer removals in both frames.

A subsequent live walk/free-look control produced `631` hits and `9` misses
over a `32x20` scan. All `120` suspicious samples were prepared-frustum vetoes
across the floor and ceiling contributions of subsectors `97` and `99`; there
were no rejected nearest hits. The exact center floor hit in subsector `97`
lands at `(981.010,-3438.273,0)`, inside its inferred region
`x=928..1184, y=-3552..-3392` but `46.273` units beyond its sole SEG endpoint
box at `y=-3392`. Node `96` reports that leaf solid-range covered even though
the hit bearing is approximately `0.052°` from the view center. This is a
stronger falsifier than an outside-FOV disagreement: horizontal solid-range
coverage over the proxy still cannot reject the visible extent of the whole
plane contribution. The prepared-frustum guard correctly retains it.

The exact headless `32x20` replay reproduces the live totals, group ordering,
sample counts and pixel bounds. Its automatic representative LOOK probes for
subsector `97` ceiling/floor and subsector `99` ceiling/floor are respectively
`10.699`, `53.255`, `6.538` and `123.321` units outside their explicit SEG
boxes while remaining inside their inferred plane regions. This capture is
therefore deterministic headless evidence for both plane families, not only a
visual screenshot observation.

### Positive plane authority correction

The baked-data audit also falsifies the earlier positive classification rule.
Reaching a subsector causes Classic Doom to select possible floor and ceiling
destinations, but later wall-range and per-column clip processing determines
which plane occurrences are actually marked. It does not authorize the whole
Tokimu convex plane mesh. `visited-source-plane=accepted` therefore promoted
correlation into positive presentation authority just as whole-plane pruning
had promoted proxy bounds into negative authority.

The shadow classifier now uses these plane outcomes:

```text
actual prepared bounds definitely outside the actual Tokimu frustum
    -> rejected-outside-frustum / prepared-geometry-outside-frustum

intersects + source subsector reached
    -> unresolved-fail-open /
       source-plane-subsector-reached-occurrence-unproven

intersects + BSP proxy solid-pruned
    -> unresolved-fail-open /
       prepared-geometry-frustum-vetoed-plane-rejection

missing or ambiguous evidence
    -> unresolved-fail-open
```

The outside-frustum outcome is generic geometric non-participation and is no
longer mislabeled `rejected-solid-range`. Accepted shadow classifications are
now reserved for exact admitted wall SEG evidence; reached planes remain
submitted and purple until plane-occurrence evidence exists.

A rebuilt source-spawn GPU control conserves all `1,849` declarations with
zero renderer removals. It reports `584` plane draws rejected outside the
actual frustum, `269` unresolved plane draws, `70` accepted wall-family draws,
`208` solid-range-rejected wall-family draws and `718` unresolved wall/cutout
draws. The `269` plane uncertainties divide into `93` reached-but-unproven,
`158` prepared-frustum vetoes and `18` child-bounds-outside-FOV cases.

The retained walk/free-look `32x20` replay reports `631` hits, `9` misses,
`203` accepted wall-family hits, zero rejected hits and `428` unresolved plane
hits across fourteen source groups. This is the desired safety result: no
visible sampled plane receives either rejection class, while the report stops
pretending that reaching its subsector proves the complete mesh occurrence.

### Plane-span occurrence diagnostic

Classic's existing fixed `320x200` vertical-clip observation now participates
in the `LOOK` evidence waterfall without changing the shadow manifest. For a
flat hit, the diagnostic matches the exact source plane key
`(kind,height,texture,light)` and source sector against the frozen-view plane
instances, then reports matching instance, populated-column, populated-cell
and contributing-SEG counts.

This evidence deliberately has only these meanings:

```text
exact source key + sector appears in frozen-view Classic spans
    -> source-key view occurrence exists
    -> whole prepared mesh occurrence remains unproven

exact source key + sector absent
    -> diagnostic absence
    -> no whole-plane rejection authority
```

Interactive `LOOK`, headless `--look-ray-report` and every automatic
representative emitted by `--bsp-diagnostic-scan-report` use the same report.
They do not map arbitrary pitched Tokimu pixels onto Classic cells: the fixed
source viewport and the actual Tokimu projection are different domains.

The retained subsector `97` headless ray reports the visible hit as
source-protocol rejected at its exact occurrence while the matching sector-38
floor key is present elsewhere in one Classic plane instance spanning `249`
columns and `17,381` cells from eight SEGs. This is the desired distinction:
plane-key participation is positive source evidence, but it cannot authorize
the hit subsector's complete inferred mesh or contradict the occurrence-level
rejection by itself.

## Slice 4 — Camera-Domain And Runtime Matrix

- [ ] Replay source spawn, the four marked rays, bounded yaw, bounded movement,
      neutral/up/down pitch and near-wall controls.
- [ ] Compare actual Tokimu projection against the resolver's view domain for
      every pose; do not infer equivalence from nominal FOV alone.
- [x] Replay door and platform height snapshots through the same traversal
      input without moving runtime policy into the resolver.
- [ ] Verify terminal decisions are scoped to one prepared-view and snapshot
      identity and cannot survive camera/runtime changes accidentally.
- [ ] Distinguish projection-domain expansion from source-topology traversal
      and exact wall/plane occurrence semantics.
- [x] Add source-key/sector plane-span participation to interactive and
      headless LOOK waterfalls without claiming prepared-pixel parity.
- [ ] Correlate retained plane hits with the exact Classic occurrence cell only
      where camera/projection equivalence can be proved; otherwise retain the
      explicit domain disagreement.

Current architectural finding: pitched free look can place off-axis ground-
plane headings beyond the classic `±45°` horizontal traversal domain while
they remain inside the Tokimu viewport. Do not widen the BSP source FOV or
grant these walls rejection authority without deciding whether the Doom-
private resolver follows the actual Tokimu frustum, multiple horizontal
source views, or a deliberately constrained classic presentation domain.

Acceptance: every disagreement is attributable to camera mapping, BSP
participation, downstream occurrence preparation or explicit unresolved input.

The retained runtime-snapshot replay passes through the same shared
preparation seam with immutable current-height inputs and no activation/timing
policy. Door sector `4` changes ceiling `0 -> 68`, alters the occurrence
fingerprint and changes twelve target declarations. Platform sector `70`
changes floor `104 -> -48` and changes nineteen target declarations; the
source occurrence fingerprint remains stable while the lowered declarations
change, which is the expected separation between horizontal source occurrence
and runtime vertical realization. Both cases report zero unresolved lowering.

## Slice 5 — Presentation-Affecting Experiment, Only If Earned

This slice is not pre-authorized merely by creating the study. Enter it only
when the shadow resolver retains all four expected hole contributions and the
source-spawn/pose matrix has no unexplained false negative.

- [ ] Apply the resolver to the original complete contribution inventory while
      preserving source order.
- [ ] Fail open to the original input for unresolved resolver outcomes.
- [ ] Submit the complete resolver output with no generic post-filter.
- [ ] Validate source spawn, hut/window, exterior hut, first door, moving
      platform, green-room cutout, EXIT, free look and near-wall jitter.
- [ ] Compare visible output and contribution manifests with all three frozen
      baselines.

Acceptance: no marked hole, new omission, forbidden far geometry, crack or
stale dynamic result. Otherwise return to shadow mode.

## Stop And Escalate

Return for architectural judgment if evidence shows any of the following:

- safe BSP participation cannot be separated from exact screen/vertical
  coverage and the proposed prefilter is therefore not an honest abstraction;
- unrestricted pitch or a larger Tokimu view requires defining new Doom
  presentation semantics outside the source view model;
- correct realization requires Doom topology, coverage arrays or persistent
  presentation state in `tokimu-render` or `tokimu-platform`;
- a stable/public renderer or runtime contract is required;
- source contributions cannot be correlated to the original inventory without
  invented provenance;
- renderer-facing geometry cannot express the resolver's required surviving
  domain through the existing ordinary/G2 experimental mechanisms; or
- browser/native parity requires duplicating Doom preparation authority.

## Parking And Success Criteria

The study may succeed in more than one way:

1. a conservative Doom-private BSP participation resolver earns a later
   presentation experiment;
2. the four-ray waterfall locates ordinary disposition or lowering defects and
   the resolver remains diagnostic only;
3. evidence proves BSP traversal cannot be separated from the full ordered
   wall/plane protocol, falsifying the prefilter hypothesis cleanly; or
4. camera-domain evidence exposes an architectural free-look compatibility
   question and returns it to AR-0030 before implementation broadens.

Do not judge success by matching Classic Doom's data structures, minimizing
draw count or making one screenshot look better.
