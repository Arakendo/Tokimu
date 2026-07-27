# glTF Corpus Feature Matrix

| Capability | Triangle glTF | Box GLB | BoxTextured glTF | Status |
| --- | --- | --- | --- |
| Pinned source provenance | Yes | Yes | Yes | Structural pass |
| glTF 2.0 JSON | Yes | Embedded JSON chunk | Yes | Structural pass |
| External binary buffer | Yes | No | Yes | Structural pass |
| GLB header and length | No | Yes | No | Structural pass |
| GLB JSON-first chunk order | No | Yes | No | Structural pass |
| GLB BIN chunk | No | Yes | No | Structural pass |
| Scene/node/mesh inventory | Yes | Yes | Yes | Structural pass |
| Accessor decoding | `FLOAT VEC3`, `UNSIGNED_SHORT` | `FLOAT VEC3`, `UNSIGNED_SHORT` | `FLOAT VEC3`, `FLOAT VEC2`, `UNSIGNED_SHORT` | Structural pass |
| Position/index extraction | 3 vertices, 3 indices | 24 vertices, 36 indices | 24 vertices, 36 indices | Structural pass |
| Normal extraction | Not supplied | 24 normals | 24 normals | Structural pass |
| `TEXCOORD_0` extraction | Not supplied | Not supplied | 24 UV pairs | Structural pass |
| External image reference | No | No | `CesiumLogoFlat.png` retained as source evidence | Structural pass, lowering deferred |
| Finite bounds | `[0,0,0]..[1,1,0]` | `[-.5,-.5,-.5]..[.5,.5,.5]` | `[-.5,-.5,-.5]..[.5,.5,.5]` | Structural pass |
| Tokimu model lowering | No | No | No | Not implemented |
| Tokimu mesh lowering | No | No | No | Not implemented |
| Renderer submission | No | No | No | Not implemented |
| Visual comparison | No | No | No | Not implemented |

The matrix describes capability evidence, not file-format conformance.
