# Runtime Observation Workbench

## Purpose

This consumer corpus tests whether a TypeScript-facing WASM adapter can inspect
and control a bounded Tokimu runtime scenario without receiving simulation
storage, importer-native values, or renderer resources.

## Contract

```text
TypeScript interaction
        |
        v
JSON command or query
        |
        v
Rust/WASM observation facade
        |
        v
runtime observation adapter
        |
        v
Tokimu-owned scenario state
```

The browser owns selection controls, labels, and presentation. Rust owns
command validation, lifecycle application, revision changes, animation state,
and provider-neutral observations.

The browser must never:

- parse a source asset;
- receive `World` or ECS storage;
- treat edited observation JSON as authoritative state;
- receive GLB parser objects or renderer/GPU handles.

## Fixture Status

Native evidence derives the five-step hole-punch catalog from
`corpus/assets/CheckLicense/hole_punch1.glb`, which requires the optional
meshopt decoder. The current
WASM consumer uses a native-test-verified provider-neutral catalog fixture so
that it can validate observation and playback semantics without claiming WASM
meshopt import support. Admitting a WASM meshopt provider remains separate
corpus work.
