# Render Orientation Conformance Fixture

## Purpose

This incubating corpus library supplies the deliberately small semantic fixture
required by AR-0021. Native WGPU and browser/WASM consumers must use the same
geometry, shader, render states, transforms, and expectations rather than
maintaining visually similar copies.

The fixture is not a general coordinate-system abstraction and does not decide
Tokimu's binding front-face policy. It exists to measure the current boundary
before that decision is made.

## Retained distinctions

- Ordered positions define geometric facing.
- The fixture's supplied `+Z` normals are deliberate shading evidence and do
  not classify either triangle as front.
- WGSL's `front_facing` input selects green or magenta independently from the
  normal-derived brightness.
- Cull mode independently selects which classified fragments survive.
- Identity and rotation/translation preserve orientation.
- Reflection reverses orientation; the compensated reflection reverses each
  triangle's positions exactly once before applying the reflection.

## Capture matrix

Every consumer must capture each fixture case under no culling, back-face
culling, and front-face culling with the fixed identity camera supplied here.
Native and WASM results remain open evidence until both captures exist.

## Retained results

- [`results/native-wgpu.png`](results/native-wgpu.png) — native WGPU matrix on
  the adapter identified in the window title.
- [`results/native-wgpu.txt`](results/native-wgpu.txt) — capture provenance and
  fixture layout.

The native artifact does not establish browser/WASM parity by itself.
