# Browser Slice 2 Cutout Observation — 2026-08-09

## Capture Identity

| Field | Value |
| --- | --- |
| Target | browser/WASM WebGPU canvas |
| Backend | browser WebGPU |
| Device kind | other (browser-reported) |
| Adapter | unavailable on this browser path |
| Viewport | 960 × 600 |
| Host readiness | browser adapter/device preflight passed; Tokimu first frame presented |
| Fixture source | shared manifest-locked `mixed-alpha` and `binary-mask` exact RGBA8 arrays |
| Candidate threshold | `128/255` (`0.5019608`) |
| Declared depth state | `LessEqual`, depth write enabled |

This is a retained visual observation, not a pixel-golden artifact or a public
renderer contract. Browser WebGPU did not expose a useful adapter name, so the
record preserves that absence rather than inventing an identity.

## Observed Result

The browser top row matches the native observation:

1. explicit opaque control shows all five mixed-alpha texels;
2. `discard below 128/255` removes the 0 and 64 alpha texels while retaining
   the exact 128 alpha texel;
3. `discard at or below 128/255` also removes the exact 128 alpha texel.

The binary-mask depth panel exposes the opaque background through discarded
pixels. The status reports `ready` only after the first `present()` succeeds.

## Interpretation Boundary

This establishes categorical agreement between the currently tested native
AMD/Vulkan and browser/WebGPU realizations for the interior threshold cases.
It does not establish NVIDIA behavior, choose a canonical threshold/comparison,
or admit a cutout renderer contract.
