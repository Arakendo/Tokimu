# Doom Ordered Source-Occurrence Reference Trace

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Study slice | Ordered Source-Occurrence Preparation, Slice 0 |
| Inspection date | 2026-08-16 |
| Classic reference | Linux Doom 1.10 `r_bsp.c` and `r_segs.c` |
| Faithful control | Chocolate Doom `src/doom/r_bsp.c` and `r_segs.c` |
| Corpus specimen | `partial-paired-sky-far-control` |
| Corpus fingerprint | `e79bb365ef3c1d8bb77dcce721cef1d5a08c1394a1370ffe4a6d35aef8ba94db` |

## Question

Can one Doom source contribution produce more than one disjoint surviving
presentation occurrence, and if so, what is the smallest invariant Tokimu must
preserve without adopting Doom's screen-column rasterizer?

This record distinguishes three kinds of evidence:

- **Direct reference observation** describes control flow and retained values
  visible in the inspected Doom implementations.
- **Corpus observation** describes the retained Tokimu falsifier.
- **Tokimu inference** proposes the smaller provider-owned representation to be
  tested in later slices. An inference is not attributed to Doom source.

## Inspected Primary Sources

Classic Doom:

- [`R_ClipSolidWallSegment`, `R_ClipPassWallSegment`, and `R_AddLine`](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/r_bsp.c)
- [`R_RenderBSPNode`](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/r_bsp.c)
- [`R_RenderSegLoop` and `R_StoreWallRange`](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/r_segs.c)

Faithful implementation control:

- [Chocolate Doom `r_bsp.c`](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/doom/r_bsp.c)
- [Chocolate Doom `r_segs.c`](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/doom/r_segs.c)

The comparison intentionally uses a port whose purpose includes preserving
vanilla behavior. It is not evidence that every modern source port uses this
storage or traversal.

## Retained Corpus Specimen

The corpus fixture contains:

```text
viewer at (0, -96), facing north
        ↓
near two-sided paired-sky boundary, x=-24..24 at y=0
        ↓
far one-sided wall SEG, x=-48..48 at y=64
```

The complete decoded source geometry remains unchanged across the baseline,
`x + 2` jitter, and `y + 16` nearer control. The whole-contribution report
retains:

| Pose | Far source result | Forbidden overlap | Required survivors | Partial survival required |
| --- | --- | ---: | ---: | --- |
| Baseline | admitted whole by the Boolean candidate | 81 columns | 15 columns | yes |
| Jitter `x + 2` | admitted whole by the Boolean candidate | 81 columns | 15 columns | yes |
| Nearer `y + 16` | admitted whole by the Boolean candidate | 97 columns | 9 columns | yes |

This is not claimed as pixel-identical reproduction of one historical Doom
frame. It is a corpus expressiveness specimen: a nearer source-authorized
middle interval and the farther source contribution's two required outer
intervals cannot be represented by one whole retain/reject decision.

The closest Classic Doom horizontal arrangement is a projected far SEG whose
inclusive range overlaps one or more already accumulated `solidsegs` ranges.
That arrangement directly answers whether Doom permits one source SEG to have
multiple disjoint horizontal survivors. Paired-sky vertical behavior is then
resolved inside the retained wall range through the shared upper/lower clip
state; it must not be re-described as an ordinary invisible solid wall.

## Stage Trace

| Stage | Classic Doom direct observation | Chocolate Doom control | Current Tokimu Boolean candidate |
| --- | --- | --- | --- |
| Source input | `R_AddLine` receives one `seg_t *curline`; the SEG retains linedef, sidedef, front/back sector, source vertices, direction, and offset relationships. | Same source unit and relationships are retained. | One source-labelled SEG contribution is observed. |
| BSP order | `R_RenderBSPNode` visits the viewer-side child first, then tests the far child's bounding box against accumulated solid coverage. | Same near-first recursion and far-child test. | Ordered topology traversal exists, but the final contribution disposition is Boolean. |
| Initial projection | `R_AddLine` clips endpoint angles to the view and maps them to `x1` and `x2`; a zero-width result is rejected. The clip routines receive inclusive `first=x1`, `last=x2-1`. | Same projected-range behavior. | Diagnostic coverage is observed on a bounded `320 x 200` oracle. |
| Solid/pass classification | One-sided and vertically closed boundaries use the solid clipper. Open/tier-changing two-sided boundaries use the pass clipper. Empty trigger-only lines can be rejected. | Same classification. | Source facts can admit or reject only the complete contribution. |
| Horizontal split | Both clippers scan sorted `solidsegs`. Each visible prefix, internal gap, and suffix causes a distinct `R_StoreWallRange(first,last)` call. The solid variant merges the new range afterward; the pass variant does not. | Same multiple-call behavior. | No occurrence multiplicity exists; the far contribution remains whole. |
| Retained occurrence | Each `R_StoreWallRange` call owns one contiguous inclusive horizontal range and still refers to the same `curline`. | Same contiguous range per call. | The required left and right survivors cannot be named separately. |
| Wall setup | `R_StoreWallRange` derives distance, scale endpoints/step, texture offset/mids, sector heights, wall tiers, silhouette, and plane-mark decisions for that one range. | Same semantic setup; minor implementation maintenance does not alter the range contract. | Whole source mesh data and provenance exist, but the bounded retained domain does not. |
| Per-column boundary | `R_RenderSegLoop` derives wall upper/lower bounds from `ceilingclip` and `floorclip`, marks floor/ceiling plane intervals from those same bounds, then updates the clip arrays for solid or upper/lower tiers. | Same shared mutable boundary process. | Prior experiments reconstructed walls and planes through separate paths, permitting cracks or overreach. |
| Masked middle | A masked two-sided middle records texture columns for later masked rendering. It does not by itself close the horizontal solid range; geometric solid/pass classification remains separate. | Same deferred behavior. | Cutout is already distinguished from an ordinary occluder, but Boolean source admission cannot express the retained partial interval. |
| Final result | One source SEG may result in zero calls, one whole-range call, or multiple disjoint `R_StoreWallRange` calls. Every individual call remains horizontally contiguous. | Preserved. | Zero or one whole-source result only. |

## Explicit Answers

### 1. What exact source unit enters clipping?

**Direct observation:** one `seg_t` selected as `curline` enters `R_AddLine`.
Its source vertices define the line projected to the view; its linedef,
sidedef, front/back sector, direction, and source offset provide wall-role and
texture semantics.

### 2. At which stage may it split?

**Direct observation:** after angular view clipping and integer horizontal
projection, `R_ClipSolidWallSegment` or `R_ClipPassWallSegment` compares the
projected SEG range with accumulated `solidsegs`. Splitting occurs when the
clipper calls `R_StoreWallRange` separately for more than one uncovered gap.

### 3. Can one processing unit yield multiple disjoint survivors?

**Direct observation: yes.** One `curline` can cause multiple
`R_StoreWallRange` calls. A previously covered middle interval can leave both
a prefix and suffix, each emitted under the same source SEG identity.

The solid clipper then merges coverage; the pass clipper preserves the existing
solid coverage unchanged. This distinction is behavioral, not merely a storage
detail.

### 4. Which continuous or projective values exist before quantization?

**Direct observation:** source vertex coordinates, viewer coordinates and
angle, SEG endpoint angles, view-angle clipping limits, and source wall offset
relationships exist before endpoint angles become integer screen columns.
During range setup Doom also retains fixed/projective values such as wall
distance, texture offset, scale at the range ends, scale step, and source
sector heights.

Classic Doom does **not** retain an explicit normalized source interval for
each occurrence. Deriving a continuous source interval from the clipped view
boundary is a **Tokimu inference** intended to avoid making integer columns
semantic geometry.

### 5. Which wall and plane boundaries derive from the same mutable state?

**Direct observation:** `R_RenderSegLoop` bounds wall columns using
`ceilingclip[x]` and `floorclip[x]`; it also writes the floor and ceiling plane
top/bottom extents from those bounds. One-sided walls close both limits.
Upper and lower wall tiers update the corresponding limit. The wall/opening
result and plane marking therefore share one causal per-column boundary state.

### 6. How are masked middles deferred without gaining occlusion authority?

**Direct observation:** a masked texture records its texture column for later
masked-segment rendering. The two-sided opening remains governed by the
solid/pass classification and ordinary upper/lower clip changes. Merely having
a masked middle does not turn its complete supporting quad into a solid
horizontal occluder.

### 7. Which facts are required to rebuild source occurrences and UVs?

**Tokimu inference, justified by the trace:** the private Doom occurrence path
needs:

- stable SEG, linedef, sidedef, sector, wall-side, and wall-role provenance;
- original source endpoints and source wall/SEG offset;
- prepared view identity and the exact current runtime-height snapshot;
- one contiguous normalized source interval per horizontal occurrence;
- upper/lower domains produced from the same prepared boundary consumed by
  the related floor/ceiling occurrence;
- texture identity, extent, pegging/offset facts, and continuous source `u`;
- ordering and the positive reason for retain, reject, fragment, or fail-open;
- source-to-occurrence correlation independent of renderer resource identity.

The source interval must be obtained from source/view intersection facts, not
by inverse-projecting a diagnostic column after the fact.

### 8. Which facts exist only because Doom rasterizes columns?

**Direct observation:** integer `x1/x2` ranges, `solidsegs` entries,
`ceilingclip[]`, `floorclip[]`, visplane `top[]/bottom[]`, masked texture-column
arrays, lookup tables, and the fixed screen width are implementation-shaped
facts of the historical renderer.

They are useful oracles for coverage, ordering, and shared-boundary behavior.
They are not admitted as persistent identities, mesh endpoints, renderer
scissors, or public Tokimu vocabulary.

## Contiguity Result

Slice 0 supports the following bounded private invariant:

```text
one source contribution
    → 0..N presentation occurrences

one presentation occurrence
    = one contiguous horizontal source interval
    + bounded upper/lower prepared domains
    + shared source/view/snapshot provenance
```

One source contribution may therefore have disjoint survivors, but the private
model does not need arbitrary-region geometry. Disjoint survival is represented
as multiple simple occurrences correlated to one source identity.

This is the key narrowing requested by Slice 0. It simplifies validation,
triangulation, continuous UV interpolation, jitter comparison, and conservation
accounting without denying the directly observed multiplicity.

## Exact Divergence

The current whole-contribution candidate first diverges at the horizontal
solid/pass clipping result:

```text
Classic Doom:
    one projected SEG
        → each uncovered gap calls R_StoreWallRange

current candidate:
    one source SEG
        → admitted or rejected as one indivisible unit
```

It diverges again if wall and plane presentation independently reconstruct the
upper/lower boundary that Classic Doom derives from the same mutable clip
state. The successor must therefore test both occurrence multiplicity and a
shared prepared boundary.

## Faithful-Port Differences

Chocolate Doom preserves the source unit, near-first traversal, solid/pass
classification, multiple `R_StoreWallRange` calls, contiguous range contract,
shared clip arrays, and masked-middle deferral.

The inspected implementation contains maintenance differences such as a
larger clip-range capacity tied to screen width and ordinary portability or
compiler-cleanliness changes. Those differences protect the historical
behavior; they do not change the semantic result above. They are recorded
separately so Tokimu does not mistake a faithful port's safer storage capacity
for new source semantics.

## Slice 0 Disposition

Slice 0 passes.

- Multiple occurrences per source contribution are directly justified.
- Each occurrence can remain one contiguous horizontal source interval.
- Doom's integer column structures remain diagnostic/reference machinery.
- The proposed private representation is narrower than the historical
  rasterizer and remains Doom-owned.
- Slice 1 may define the private occurrence model; no visual E1M1 candidate or
  stable Tokimu renderer contract is admitted by this result.
