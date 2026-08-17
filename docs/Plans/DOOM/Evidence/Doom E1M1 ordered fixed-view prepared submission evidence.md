# Doom E1M1 Ordered Fixed-View Prepared Submission Evidence

## Claim

At the canonical source-spawn view, the `ordered-occurrence-prepared-full`
strategy replaces the original global E1M1 declaration domain with the entire
currently prepared Doom-owned wall and ordinary-plane declaration set, then
uses ordinary renderer full submission. No generic camera filter and no legacy
screen-column reconstruction participate.

This is integration and structural-conservation evidence, not a claim that
Slice 6 is complete or that horizontal occurrence domains are sufficient to
reconstruct source-faithful plane visibility at every canonical pose.

## Dataflow

```text
original E1M1 source + fixed source-spawn view
    -> continuous source occurrences
    -> shared wall/plane boundaries
    -> ordinary wall and plane declarations
    -> category-specific retained uploads
    -> tokimu-render full submission
```

The original 1,922-contribution scene is retained separately as the global
control. It is not merged into the candidate.

## Prepared declaration conservation

```text
wall declarations
    opaque                                      309
    cutout                                       12

ordinary plane source triangles                283
    with bounded survivors                       72
    fully rejected                              211
clipped plane triangles                        166
    ordinary plane declarations                136
    bounded degenerate omissions                30

candidate declarations
    opaque                                      445
    cutout                                       12
    total                                       457

unresolved wall failures                         0
unresolved plane failures                        0
generic camera rejections                        0
```

Every candidate draw resolves to an existing upload in its own opaque or
cutout category before replacement is allowed. Source sky destinations are
accounted as Doom background and do not become depth-writing plane geometry.
Destination, source-triangle, clipped-fragment, and declaration conservation
all balance.

## Runtime integration defect found and repaired

The predecessor two-frame integration run correctly constructed 592 declarations at
startup, but the application's legacy ordered-coverage refresh then replaced
them with 51 draws from the superseded 320-column reconstruction. That was a
pipeline-composition defect, not evidence about the new occurrence model.

The fixed-view ordered-occurrence strategy no longer supplies a runtime source
to that legacy refresh path. After occurrence-bounded plane clipping, the
corrected observation retains the 457-draw candidate through first and warm
presentation:

```text
first frame
    candidates/submitted                       457/457
    opaque/cutout                              445/12
    renderer draws (including presentation)       460
    mesh uploads/replacements                     0/0
    lifetime mesh replacements                       0

warm frame
    candidates/submitted                       457/457
    opaque/cutout                              445/12
    renderer draws                                460
    mesh uploads/replacements                     0/0
```

## Commands

```powershell
cargo test -q -p hello-doom-e1m1

cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=ordered-occurrence-prepared-full `
  --measure-two-frames
```

The crate validation passed 46 library tests, 4 auxiliary binary tests, and 60
`static_scene` tests. The known Windows incremental-cache hard-link copy
warnings do not alter the result.

## Deliberate limitations

- The camera is fixed to the source-spawn observation. Free movement would make
  stale fixed-view declarations look like valid preparation evidence.
- Ordinary plane triangles are clipped to merged camera-horizontal intervals
  carried by occurrences reaching each exact plane destination. This proves a
  bounded candidate, not that horizontal occurrence domains fully express
  Doom floor/ceiling survival.
- Door and platform runtime-height snapshots are correlated through the same
  preparation seam in a separate deterministic source-boundary-local report;
  live animated integration remains open.
- The canonical visual comparison matrix remains open.
- This admits no renderer-owned Doom vocabulary or generic visibility API.

## Canonical visual result: falsified

Manual comparison at the fixed source-spawn pose found widespread visible
false negatives despite the balanced structural manifest. Required wall,
floor, ceiling, stair-edge, and pillar-adjacent regions were absent, and the
sky/background was visible through multiple parts of the spawn room.

This is not attributed to renderer full submission: all 457 declarations
produced by preparation were submitted with zero generic rejection. It is not
attributed to resource churn or unresolved lowering: warm uploads and
replacements remained zero and all lowering failure counts were zero.

The falsified proposition is narrower and more useful:

> Exact plane destinations clipped only by merged camera-horizontal domains
> from associated SEG occurrences are sufficient to reconstruct Doom's
> viewer-relative wall/plane survival.

They are not. The candidate loses semantic coverage before renderer handoff.
The evidence points back toward Doom-owned ordered vertical wall/plane coverage
or another richer private intermediate. It does not by itself require public
screen-column vocabulary or a renderer-owned visibility algorithm.
