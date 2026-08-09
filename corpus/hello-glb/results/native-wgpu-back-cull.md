# Native WGPU Box GLB Back-Face-Culling Capture

| Field | Value |
| --- | --- |
| Capture | `native-wgpu-back-cull.png` |
| Example | `hello-glb` |
| Source | pinned Khronos `Models/Box/glTF-Binary/Box.glb` |
| Opaque model policy | `CullMode::Back`, opaque blend, `LessEqual` depth test, depth writes enabled |
| Translucent diagnostic policy | `CullMode::None`, alpha blend, depth writes disabled |
| Capture condition | initial opaque source presentation; window title retains `opaque-cull=back` |
| Native adapter | AMD Radeon RX 7900 XTX (WGPU/DX12 evidence environment) |

The screenshot proves that the decoded Box mesh is visibly intact from the
captured orbit viewpoint while the normal opaque model path has explicit
back-face culling. It is native-only evidence: it does not prove browser/WASM
classification or deployed workbench parity.
