# Geometry Orientation And Orbit Controls

AR-0021 exposed several similar-looking failures with different causes. Use
this reference before changing winding, normals, culling, projection, or camera
controls together.

## Keep The Concepts Separate

```text
ordered positions
    -> geometric winding and geometric normal

supplied vertex normals
    -> lighting input

front-face convention + projection
    -> front/back classification

cull mode
    -> which classified faces are discarded

pointer deltas
    -> camera or model interaction policy
```

An inside-out appearance does not prove that all five are reversed.

Tokimu's current conformance oracle defines the geometric normal of ordered
positions `(a, b, c)` as `(b - a) x (c - a)` in a right-handed coordinate
system. Authored normals may differ intentionally, but they must not silently
redefine geometric facing.

## Projection And Culling

Canvas Y increases downward. The workbench projection accounts for that screen
axis reversal explicitly; its camera-facing GLB triangle has positive
screen-space winding. Test the projected winding rather than copying a native
API's clockwise/counter-clockwise label into Canvas code.

Cull mode and front-face selection are distinct. `CullMode::None` can make a
bad mesh look complete, but it does not prove which face is front. Retain
explicit back- and front-cull cases with visibly different results.

An orientation-reversing transform, such as a reflection or negative-
determinant scale, reverses winding. Compensate exactly once through geometry or
face policy. The shared AR-0021 fixture tests identity, rotation/translation,
reflection, and reflection with one compensation on native WGPU and browser
WASM.

## Camera Orbit Versus Dragging The Model

Name the interaction before choosing signs:

- **Model drag:** the visible model follows the pointer.
- **Camera orbit:** the virtual camera moves around a fixed model.

The workbench says “orbit,” so its pointer deltas subtract view yaw and pitch:

```typescript
yaw = yaw - deltaX * sensitivity;
pitch = clamp(pitch - deltaY * sensitivity, minimum, maximum);
```

The earlier implementation added both deltas. Its test verified that one
synthetic identity-view feature followed the pointer, accidentally proving
model-drag behavior while the UI promised camera orbit. Both axes therefore
felt inverted during real Box inspection.

Test the declared control contract directly:

- a rightward drag produces the expected signed yaw delta;
- a downward drag produces the expected signed pitch delta;
- pitch remains bounded at both poles;
- the actual default view and a transformed corpus asset remain usable.

Do not use camera-control behavior as evidence for mesh winding. In AR-0021,
the Doom winding defect, the workbench orbit defect, and the WASM timing defect
were three independent failures that happened to appear during the same
orientation investigation.

## Evidence

- [`AR-0021`](../Architectural%20Reviews/AR-0021-geometry-orientation-and-facing-conformance.md)
- [Shared orientation fixture](../../corpus/lib/render-orientation-conformance/DESIGN.md)
- [Workbench preview helper](../../corpus/consumers/aspnet-wasm-asset-workbench/Client/mesh-preview.ts)
- [Workbench interaction tests](../../corpus/consumers/aspnet-wasm-asset-workbench/tests/mesh-preview.test.mjs)

