# glTF Corpus Feature Matrix

| Capability | Triangle glTF | Box GLB | BoxTextured glTF | MeshPrimitiveModes glTF | MultipleScenes glTF | SimpleMeshes glTF | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Pinned source provenance | Yes | Yes | Yes | Yes | Yes | Yes | Structural pass |
| glTF 2.0 JSON | Yes | Embedded JSON chunk | Yes | Yes | Yes | Yes | Structural pass |
| External binary buffer | Yes | No | Yes | Yes | Two buffers | Yes | Structural pass |
| GLB header and length | No | Yes | No | No | No | No | Structural pass |
| GLB JSON-first chunk order | No | Yes | No | No | No | No | Structural pass |
| GLB BIN chunk | No | Yes | No | No | No | No | Structural pass |
| Scene/node/mesh inventory | Yes | Yes | Yes | 1 scene, 7 nodes, 7 meshes | 2 scenes, 2 nodes, 2 meshes | 1 scene, 2 nodes sharing 1 mesh | Structural pass |
| Primitive topology inventory | `TRIANGLES` | `TRIANGLES` | `TRIANGLES` | `POINTS` through `TRIANGLE_FAN` | `TRIANGLES` | `TRIANGLES` | Source evidence pass |
| Accessor decoding | `FLOAT VEC3`, `UNSIGNED_SHORT` | `FLOAT VEC3`, `UNSIGNED_SHORT` | `FLOAT VEC3`, `FLOAT VEC2`, `UNSIGNED_SHORT` | Triangle mode only | `FLOAT VEC3`, `UNSIGNED_SHORT` | `FLOAT VEC3`, `UNSIGNED_SHORT` | Triangle decoder pass |
| Position/index extraction | 3 vertices, 3 indices | 24 vertices, 36 indices | 24 vertices, 36 indices | Inspected; non-triangle modes reject explicitly | Triangle: 3/3; square: 4/6 | 3 vertices, 3 indices | Structural pass |
| Normal extraction | Not supplied | 24 normals | 24 normals | Not decoded | 3 normals per primitive | 3 normals | Structural pass |
| `TEXCOORD_0` extraction | Not supplied | Not supplied | 24 UV pairs | Not decoded | Not supplied | Not supplied | Structural pass |
| Source material evidence | Red base-color factor | Red base-color factor | Material -> texture -> image chain | Not decoded | Not supplied | Not supplied | Structural pass, lowering deferred |
| External image reference | No | No | `CesiumLogoFlat.png` retained as source evidence | No | No | No | Structural pass, lowering deferred |
| Scene roots and node traversal | 1 root node | 1 root node | 2 nodes, parent/child traversal | 7 roots | 2 independent scene roots | 2 roots sharing mesh 0 | Structural pass |
| Local and world transforms | Identity | Identity | Column-major matrix and inherited world transform | Identity | Identity | TRS translation for node 1 | Structural pass |
| Finite bounds | `[0,0,0]..[1,1,0]` | `[-.5,-.5,-.5]..[.5,.5,.5]` | `[-.5,-.5,-.5]..[.5,.5,.5]` | Not decoded | `[0,0,0]..[1,1,0]` per primitive | `[0,0,0]..[1,1,0]` | Structural pass |
| Tokimu model lowering | No | No | No | No | No | No | Not implemented |
| Tokimu mesh lowering | No | No | No | No | No | No | Not implemented |
| Renderer submission | No | No | No | No | No | No | Not implemented |
| Visual comparison | No | No | No | No | No | No | Not implemented |

The matrix describes capability evidence, not file-format conformance.

| Measure | Current | Interpretation | Notes |
| --- | --- | --- | --- |
| Variants with geometry accessors decoded | 5 / 6 selected | **83% of v1 selection** | Triangle geometry decodes for five cases; MeshPrimitiveModes is source-inspected and confirms the decoder rejects its first unsupported `POINTS` primitive explicitly |
| Variants lowered into Tokimu model/mesh data | 0 / 6 | **0%** | Canonical model and mesh lowering remain unimplemented |
| Focused GLB boundary examples | 1 | Not a corpus-coverage metric | `hello-glb` proves source identity and renderer ownership boundaries |

The official importer progress number is therefore:

```text
6 source variants structurally inspected
5 source variants decoded into corpus-owned primitive evidence
1 source variant stopped at an explicit unsupported-topology boundary
0 source variants lowered into Tokimu model or mesh data
```
