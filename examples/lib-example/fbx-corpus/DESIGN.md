# FBX Corpus

## Purpose

`fbx-corpus` makes bounded FBX source interpretation observable without
admitting FBX as a Tokimu engine capability.

The current binary profile, plus the bounded ASCII source-graph, static
geometry, source-transform, and selected skinning bridge, provides deterministic, provider-local
evidence for:

```text
binary FBX bytes
    -> source records and properties
    -> objects, connections, and hierarchy
    -> static geometry, source transforms, and material bindings
    -> animation stacks, layers, curves, and keys
    -> skin clusters, `LimbNode` skeleton, and static blend-shape source evidence
    -> deterministic structural artifacts
```

Paired source encodings can also emit a versioned static-observation comparison
report. Selected paired deformation evidence compares source influence and
skeleton-hierarchy observations plus source `Link_Mode` presence and values,
without treating encoding-local labels, IDs, offsets, record order, or
f32-versus-decimal storage as shared semantics. Retaining `Link_Mode` does not
evaluate it or define a Tokimu deformation contract.

All evidence remains source-format-local. It does not define Tokimu model,
mesh, material, animation, or renderer contracts. Pivots, interpolation,
runtime clips, skeleton semantics, runtime deformation beyond static mesh
lowering, and renderer submission remain separate corpus slices so a failure
can be assigned to the first boundary whose evidence diverges.

## Ownership

- This crate owns corpus-only FBX syntax, source interpretation, and
  diagnostics.
- Tokimu engine crates do not depend on this crate.
- FBX source records and source evidence do not define Tokimu model semantics.
- Rendering is outside this crate.
