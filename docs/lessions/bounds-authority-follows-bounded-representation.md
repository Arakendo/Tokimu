# Bounds Authority Follows The Bounded Representation

A spatial bound has rejection authority only over the representation it
actually bounds. Do not promote a valid bound over source records into a bound
over larger geometry derived from those records without proving containment.

This matters whenever an importer reconstructs presentation geometry from
topology, partitions, implicit surfaces, procedural rules, hierarchy, or other
source facts. Every individual calculation can be correct while their
composition is wrong:

```text
valid source bound
    -> valid source traversal decision
    -> invalid authority promotion
    -> required derived geometry rejected
```

## Separate The Representations

Name the bound, its members, and the contribution being rejected separately:

```text
source hierarchy bound
    bounds explicit source members
    -> may support traversal decisions about those members

derived presentation geometry
    may include implicit or reconstructed support
    -> needs its own proven bound and participation rule
```

A hierarchy node being skipped does not automatically prove that every larger
object correlated with its descendants is invisible. Conversely, a derived
object intersecting the camera frustum does not prove visibility; it only
vetoes an unsafe claim that the object is definitely outside.

## Audit Before Repair

When a bound appears to reject visible geometry:

1. Compare raw source fields with decoded fields.
2. Recompute an envelope from the source members the bound claims to cover.
3. Verify that the decoded bound contains that envelope.
4. Compute the derived contribution's support independently.
5. Compare the derived support with the source bound without assuming they are
   equivalent.
6. Fail open when rejection authority is unresolved.

Keep these questions distinct:

```text
Was the source decoded faithfully?
Does the source bound contain its declared members?
Does a derived contribution extend beyond those members?
Which representation does the proposed rejection actually target?
```

Do not rebake, repartition, inflate bounds, or change tolerances merely because
a different source build makes the visible symptom move. Such changes can hide
an authority error without correcting it. Use alternate builders later as
diagnostic perturbations after the canonical data has been audited.

## Doom Evidence

The canonical E1M1 audit independently compared raw NODES records, decoded BSP
child boxes, descendant SEG endpoints, and Tokimu's inferred convex plane
regions:

```text
raw NODES records                    236/236 exact decoded matches
child boxes                          472
descendant SEG envelopes contained  472/472
descendant SEG underbounds           0
inferred plane regions contained     149/472
inferred plane-region overruns       323/472
```

The BSP boxes correctly bound the descendant SEG representation. Approximately
68% do not bound the larger plane regions inferred from BSP partition paths.
Classic Doom can validly use those boxes while incrementally constructing
wall and plane presentation; Tokimu cannot therefore reject an entire
independently reconstructed floor or ceiling mesh.

The safe diagnostic policy is asymmetric:

```text
derived geometry definitely outside its actual frustum
    -> safe geometric rejection

source proxy negative but derived geometry intersects
    -> retain and diagnose disagreement

missing or ambiguous evidence
    -> retain fail-open
```

This lesson does not make camera intersection a visibility oracle and does not
promote source hierarchy semantics into Tokimu's renderer.

## Evidence

- [Doom BSP presentation-domain resolver study](../Plans/DOOM/Studies/Doom%20BSP%20presentation-domain%20resolver%20study.md)
- [AR-0030 source-owned presentation preparation boundary](../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md)
- [AR-0025 camera candidate selection and visibility culling](../Architectural%20Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md)

