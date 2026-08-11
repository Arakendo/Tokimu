# Doom Sidedef Visual Observation — 2026-08-10

## Claim

The bounded native AR-0028 fixture presented complete readable asymmetric art
through both currently selected Doom sidedef texture-axis mappings:

- screen-left: left/back `BACK`;
- screen-right: right/front `FRONT`.

Both walls passed through `lower_doom_textured_wall_triangles` and the ordinary
static supplied-UV mesh lowering. The renderer received no WAD, sidedef, or
Doom texture-direction vocabulary.

## Observation

The maintainer visually confirmed both panels looked correct. Corner labels
`1` through `4`, horizontal `U- LEFT` / `RIGHT U+`, vertical `V- TOP UP` /
`V+ BOTTOM`, and the face labels remained complete and readable.

The apparent screen ordering is intentional: the fixture camera's local right
is world `-X`, so the positive-X left/back source wall appears screen-left.
This is retained basis evidence rather than a claim that screen direction and
world X share a universal Tokimu sign.

## Failed Fixture Attempt

An earlier version used two cameras and half-surface `ViewportRect` values.
Only one horizontal half of each source panel remained visible. Repository
inspection confirmed that the current renderer field lowers to a pixel-space
scissor; it does not remap NDC into an independent viewport. The test removed
that mistaken assumption and used one camera with spatially separated walls.
No renderer contract was changed.

## Reproduction

```powershell
cargo run -p hello-doom-e1m1 --bin doom_sidedef_conformance
```
