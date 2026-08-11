# Native Directional-Label Observation — 2026-08-10

| Field | Observation |
| --- | --- |
| Target | Native Windows WGPU |
| Backend | Vulkan |
| Adapter | AMD Radeon RX 7900 XTX |
| Fixture | Shared AR-0028 directional atlas and chamfered panel matrix |
| Rows | Identity, rotate/translate, reflect-X, reflect-X plus caller compensation |
| Columns | Cull none, back, front |
| Status | Manually observed; accepted as native Slice 2 visual evidence |

The maintainer inspected the running native fixture after the shared source,
generated atlas, and shader changes.

- Identity and rotation/translation showed the left source panel as `FRONT`
  and the right source panel as `BACK` with culling disabled.
- Back-face culling retained only `FRONT`; front-face culling retained only
  `BACK`.
- The uncompensated X reflection exchanged screen positions and reversed
  geometric facing. Its readable labels and U/V axes were visibly mirrored.
- Caller-side triangle reversal compensated facing in the final row without
  silently repairing the mirrored UV presentation.
- The chamfer, readable directional labels, supplied `N +Z` declaration, and
  distinct UV corners remained independently visible.

This is a manual presentation observation of the deterministic structural
manifest. It does not claim pixel identity with browser/WebGPU, a stable global
coordinate convention, or automatic reflection compensation.
