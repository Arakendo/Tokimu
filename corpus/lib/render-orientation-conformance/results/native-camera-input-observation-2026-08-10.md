# Native Camera/Input Observation — 2026-08-10

| Field | Observation |
| --- | --- |
| Target | Native Windows WGPU |
| Fixture | `camera_direction_conformance` |
| Shared model | `render-orientation-conformance` camera command replay |
| Status | Manually observed; accepted as native Slice 3 live-input evidence |

The maintainer reported that the complete native interaction behaved correctly:

- free cursor motion did not rotate the camera;
- clicking captured the pointer;
- captured horizontal and vertical motion produced the expected right/up look;
- W/S moved forward/backward;
- A/D moved left/right along the displayed local-right basis;
- Q/E moved down/up;
- arrow keys applied deterministic yaw/pitch commands;
- Escape released pointer capture.

The window title simultaneously displayed camera position, yaw, pitch,
forward/up/right basis, capture state, raw pointer observation, and mapped
camera commands. This allowed the physical observation and interaction policy
to be inspected separately from the resulting camera basis.

This result supports the corpus-local native policy only. Browser pointer-lock
parity and any stable Tokimu input/camera vocabulary remain separate questions.
