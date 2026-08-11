# Browser Camera/Input Observation — 2026-08-10

| Field | Observation |
| --- | --- |
| Target | Browser/WASM WebGPU in Microsoft Edge |
| Fixture | `camera.html` from `hello-render-orientation-web` |
| Shared model | `render-orientation-conformance` camera command replay |
| Acquisition | Browser pointer lock and DOM keyboard/mouse events |
| Status | Manually observed; accepted as browser Slice 3 input evidence |

The maintainer reported that the browser fixture behaved correctly across the
same interaction cases as the native fixture:

- free pointer motion did not rotate the camera;
- clicking the canvas acquired pointer lock;
- captured horizontal and vertical motion produced expected right/up look;
- W/S, A/D, and Q/E applied the same movement command signs;
- arrow keys applied the same deterministic yaw/pitch command signs;
- Escape released pointer lock.

The browser page displayed pose, basis, capture state, raw pointer observation,
and mapped commands. Both targets executed the same corpus-local pose, basis,
command, and first-person-policy implementation while retaining distinct
native-grab and browser-pointer-lock acquisition mechanisms.

This agreement is conformance evidence for the fixture. It neither admits the
corpus command types as stable Tokimu input vocabulary nor claims that touch,
gamepad, orbit, or editor interaction should reuse first-person policy.
