# Browser Directional-Label Observation — 2026-08-10

| Field | Observation |
| --- | --- |
| Target | Browser/WASM WebGPU in Microsoft Edge |
| Fixture URL | Local retained fixture on `127.0.0.1:4174` |
| Fixture | Shared AR-0028 directional atlas and chamfered panel matrix |
| Rows | Identity, rotate/translate, reflect-X, reflect-X plus caller compensation |
| Columns | Cull none, back, front |
| Status | Manually observed; accepted as browser Slice 2 visual evidence |

The maintainer inspected the running browser fixture and compared it with the
native AMD/Vulkan observation.

- Identity and rotation/translation presented the same `FRONT`/`BACK` labels,
  U/V directions, chamfer, and colored/numbered corners as native.
- Back-face and front-face culling retained the same classified panels on both
  targets.
- The X-reflected labels were visibly mirrored on both targets.
- Caller-side winding compensation restored the same facing classification on
  both targets without changing the mirrored UV presentation.

The browser status reported `READY`; browser adapter identity remained blank in
the page metadata and is retained as an evidence limitation rather than filled
by inference. This observation establishes target agreement for this fixture,
not a universal Tokimu coordinate convention or pixel-identical rendering.
