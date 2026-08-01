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

The `ui_snapshot_json` path is a second consumer of the hardened `ui-tools`
composition contract. Rust builds a semantic tree, resolves it headlessly, and
lowers it into one renderer-neutral draw list. TypeScript can inspect the owned
snapshot, including node bounds, fit status, bounded diagnostics, draw counts,
and a structural fingerprint. It does not receive mutable UI state, font or
icon providers, tessellators, or backend resources.

```text
runtime observation
        |
        v
semantic UiTree
        |
        v
resolved geometry + UiDrawList
        |
        v
bounded WASM UI evidence
        |
        v
TypeScript presentation
```

The browser must never:

- parse a source asset;
- receive `World` or ECS storage;
- treat edited observation JSON as authoritative state;
- receive GLB parser objects or renderer/GPU handles.
- treat the UI evidence artifact as a browser-owned authoritative layout.

## Fixture Status

Native evidence derives the five-step hole-punch catalog from
`corpus/assets/CheckLicense/hole_punch1.glb`, which requires the optional
meshopt decoder. The current
WASM consumer uses a native-test-verified provider-neutral catalog fixture so
that it can validate observation and playback semantics without claiming WASM
meshopt import support. Admitting a WASM meshopt provider remains separate
corpus work.
