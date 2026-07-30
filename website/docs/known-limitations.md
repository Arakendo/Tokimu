---
title: Known Limitations
description: Current Tokimu limitations and deliberately deferred capabilities.
---

<p class="page-kicker">Status / honest boundaries</p>

# Known limitations

Tokimu is an early engine project. Its architecture and corpus are substantial,
but many capabilities remain provisional, incomplete, or intentionally
deferred.

## Distribution

- There is no stable end-user SDK release.
- Public APIs may change while corpus evidence continues to shape ownership.
- The website scaffold is not yet a deployed documentation service.

## Browser support

- WASM entry surfaces exist, but browser execution is still under active corpus
  pressure.
- Not every native presentation path has browser visual parity.
- WebGPU, lifecycle, payload, and startup guarantees are not yet published.

## Asset formats

- Format maturity varies by admitted subset.
- SVG has the deepest geometry corpus but does not imply complete SVG 1.1
  presentation support.
- CGM currently includes bounded inspection and diagnostic preview work; many
  semantics remain deferred.
- GLB and glTF have decoded geometry evidence, with broader scene, material,
  and animation behavior still evolving.
- FBX support is bounded and should not be read as general compatibility.

## Presentation

- Text, font providers, vectors, icons, materials, and shader controls are at
  different stages of admission.
- Transparency does not currently claim order-independent composition.
- Browser workbenches are corpus consumers, not polished production tools.

## Networking and XR

- Networking has bounded envelope and loopback evidence, not a production
  multiplayer stack.
- VR/XR remains planned architecture rather than an implemented capability.

Limitations are expected to change. A public status should move only when
implementation and corpus evidence justify the stronger claim.
