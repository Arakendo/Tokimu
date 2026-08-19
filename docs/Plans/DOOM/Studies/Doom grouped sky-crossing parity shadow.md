# Doom Grouped Sky-Crossing Parity Shadow Study

| Field | Value |
| --- | --- |
| Status | Parked at Slice 1 — two even/absent counterexamples block broadening |
| Scope | Reclassify ordered, grouped sky intersections before complete-world targets |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Predecessor | [Doom One-Way Sky-Occlusion Correlation Shadow](Doom%20one-way%20sky-occlusion%20correlation%20shadow.md) |
| Renderer changes authorized | None |
| Stable API authority | None |

## Correction And Question

The predecessor falsified only the coarse rule:

```text
any sky-related hit before target → hide target
```

It did not test whether the number of distinct, ordered hit groups has useful
parity. Its eight exact-present counterexamples each reported two groups,
while the five retained unwanted far-field controls reported one and the five
required nearby controls reported zero:

```text
even grouped crossings → World candidate
odd grouped crossings  → Sky candidate
```

This study tests that correlation without claiming that Doom's open sky
surfaces form a closed volume.

## Independent Evidence

Every retained ray records:

- the complete ordinary target and exact frozen-view source result;
- every ordered distinct sky group before the target;
- distance, semantic identity, family and raw triangle multiplicity;
- raw winding orientation, with diagnostic-only authority;
- source semantic side only where the source ceiling-plane kind supports it;
- grouped count, parity and family sequence.

`source-sky-open-plane` and `paired-sky-height-discontinuity` remain separate.
A source sky ceiling has a locally meaningful underside/topside. A paired-sky
height discontinuity has sky ceilings on both sides and remains semantically
unoriented. Neither fact proves a global World/Sky volume.

Exact-present, partial and absent remain distinct. Ordered frozen-view source
participation is a correlation axis; it is not a canonical free-look pixel
oracle and does not override the complete world.

## Slice 1 — Existing 36-Ray Reclassification

- [x] Reuse the exact deterministic eight-pose `32 × 20` scan.
- [x] Select the same 36 rays with one or more grouped sky hits.
- [x] Retain the entire ordered group sequence for every ray.
- [x] Cross-tab grouped parity against exact, partial and absent results.
- [x] Keep raw winding diagnostic-only and paired-sky semantic side unresolved.
- [x] Run twice and preserve deterministic results.

The first correlation gate inspects `odd + exact-present` and `even + absent`
counterexamples. Partial results stay separate. Any disagreement is retained
for inspection before broadening; it is not silently reclassified.

### Implemented Result

Run:

```text
cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --grouped-sky-crossing-parity-report
```

The retained corpus was reproduced exactly:

```text
rays                         36
raw/grouped hits          46/46
paired/source-plane groups 38/8

odd  + exact-present          0
odd  + absent                26
even + exact-present          8
even + absent                 2
```

The eight former one-way falsifiers do support the even/World correlation.
The matrix nevertheless contains two even/absent specimens at hut-east cells
`(115,105)` and `(125,105)`, targeting walls 279 and 290. Both have the same
family sequence as six successful hut exact-present specimens:

```text
paired-sky-height-discontinuity
    → source-sky-open-plane
```

The source-plane hit is backside in all eight such rays. Raw winding remains
diagnostic-only, and the paired boundary remains semantically unresolved.
Consequently raw parity, family sequence, and the locally proved ceiling side
cannot distinguish the two absent targets from the six exact-present targets.

Two runs produced fingerprint `26afe710bce75ebc`. Conservation is balanced
and renderer mutation is false.

### Slice 1 Disposition

Grouped parity is an interesting correlation, but it fails its authorized
exact-36 gate. Slice 2 and live A/B work are not entered. Repairing the result
would require a new semantic discriminator rather than more parity sampling.
The complete 36-row report remains available for any future hypothesis.

## Slice 2 — Conditional Full Corpus

Only if Slice 1 survives without unexplained counterexamples:

- [ ] Classify all 5,120 existing rays, including zero-crossing targets.
- [ ] Preserve canonical expected-presentation controls separately from
      ordered source-participation observations.
- [ ] Compare family sequences and exact frozen controls.
- [ ] Run twice and preserve conservation and fingerprint evidence.

Zero crossings leave the complete world available by hypothesis. Therefore a
zero-crossing ordered-source absence is not automatically a canonical parity
failure: the retained steep-pitch holes already demonstrate that frozen Doom
occurrence absence and desired free-look presentation are different claims.

Not executed: Slice 1 contains two unexplained even/absent counterexamples.

## Slice 3 — Conditional Live A/B

No live mutation is authorized by this study. A reversible walkabout requires
a separate AR-0030 review after the full headless corpus establishes which
claims are correlation and which have an independent presentation oracle.

## Binding Invariants

1. Global-full geometry remains unchanged.
2. No renderer declaration is removed, recolored or resubmitted.
3. Raw triangle winding does not become Doom semantic authority.
4. Paired-sky sheets do not acquire invented World/Sky orientation.
5. Partial source participation remains separate.
6. Every adverse specimen remains replayable.
7. Doom vocabulary remains corpus/provider-private.
8. No BSP, BVH or provider-neutral contract follows from parity correlation.

## Stop Conditions

Return to AR-0030 before live work if the retained matrix contains parity
counterexamples, if resolving them requires a new semantic side or volume
claim, or if ordered source absence would need to be treated as canonical
free-look omission authority.
