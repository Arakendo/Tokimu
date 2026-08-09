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
- The public website is a first-party consumer corpus, not evidence of
  independent production adoption.
- `tokimuengine.org` is canonical. DNS forwarding for `.com` and `.net` is
  configured, but public edge behavior has not yet converged on verified,
  HTTPS, path-preserving redirects for every requested documentation URL.

## Browser support

- A bounded WASM asset-observation island is published, but browser execution
  remains experimental and under active corpus pressure.
- Not every native presentation path has browser visual parity.
- The published island has lifecycle, payload, startup, and idle-work budgets;
  these are evidence for that consumer, not general WebGPU or browser-runtime
  guarantees.
- Browser heap-retention evidence and a complete cross-browser interaction
  matrix remain open launch-review work.

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

## GPU hardware coverage

- Current hands-on native renderer observations cover AMD Radeon hardware
  through Vulkan and Apple GPU hardware through Metal.
- Comparable NVIDIA hardware is not currently available to the maintainer.
  NVIDIA Vulkan and D3D12 paths are therefore unverified and may contain gaps;
  they are not known to be either conformant or defective.
- Passing AMD, Apple, or browser WebGPU evidence does not establish NVIDIA
  conformance. Adapter, backend, device, driver, target, and build identity are
  retained where available so future NVIDIA results can be assessed honestly.

## Networking and XR

- Networking has bounded envelope and loopback evidence, not a production
  multiplayer stack.
- VR/XR remains planned architecture rather than an implemented capability.

Limitations are expected to change. A public status should move only when
implementation and corpus evidence justify the stronger claim.

## Website evidence maintenance

Public evidence pages must identify their source record, evidence date, and
bounded claim. Changes to corpus measurements must update the authoritative
repository record first, then the website representation and its drift test.

Every website change is expected to pass:

- strict MkDocs generation;
- generated-site canonical metadata and internal-link validation;
- TypeScript typechecking and checked-in bundle verification;
- island lifecycle, accessibility, failure, payload, and static-fallback tests.

Native-window screenshots and supported-browser interaction reviews remain
separately labeled manual evidence. They do not replace structural or
deterministic corpus artifacts.
