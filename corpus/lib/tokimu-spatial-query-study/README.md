# Tokimu spatial-query study

This corpus-local crate extracts the portable mechanics demonstrated by the
conservative spatial-query study. It is incubation infrastructure, not an
admitted Tokimu capability or stable public API.

## Decomposition record

- **Subject:** deterministic conservative queries over a finite inventory of
  caller-supplied triangles.
- **Responsibility:** immutable median-split BVH construction, containment and
  conservation audit, deterministic fingerprints, AABB/frustum candidate
  queries, nearest triangle-ray queries, refit, and geometry-revision checks.
- **Authority:** none over rendering, visibility, presentation, or simulation.
  Query results are candidates and evidence only.
- **Inputs:** stable caller identities and correlation labels, finite triangle
  vertices, an explicit geometry revision, view-projection matrices, and rays.
- **Outputs:** an immutable study artifact, query identities and measurements,
  audit results, and revision mismatch diagnostics.
- **Exclusions:** source-format topology or vocabulary, source conversion,
  runtime movement/activation policy, renderer resources or submission,
  provider contracts, capability descriptors, and public-facade exports.
- **Dependency direction:** corpus consumers adapt their geometry into this
  crate. This crate depends only on engine-neutral math from `tokimu-core` and
  knows nothing about those consumers.

The native source-format campaign and the target-gated WASM conformance test
consume the same implementation. Promotion, redesign, or deletion requires a
focused architectural review after this portability evidence is complete.
