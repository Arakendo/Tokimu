# ASP.NET WASM Presentation Workbench

## Purpose

`aspnet-wasm-presentation-workbench` is an independent downstream consumer
corpus for TypeScript-authored presentation controls. It validates a bounded
browser workflow where a user selects a vector or mesh target in a viewport,
changes semantic material properties in a side panel, and receives a
Rust/WASM-resolved presentation result.

It is the first real caller intended to pressure Slice 7 of
[`typescript-shader-material-presentation-control.md`](../../../docs/Plans/Standalone/typescript-shader-material-presentation-control.md).

## Primary Composition Claim

An ordinary ASP.NET-hosted browser application can author bounded presentation
intent in TypeScript without owning shader semantics, imported source truth, or
renderer state.

```text
TypeScript selection and authoring controls
        |
        v
typed local material-authoring adapter
        |
        v
bounded JSON WASM command
        |
        v
Tokimu provider-neutral presentation control
        |
        v
resolved color, opacity, visibility, and emphasis
        |
        v
canvas pixels
```

## Ownership

- ASP.NET owns static hosting and fallback routing.
- TypeScript owns DOM interaction, hit-test routing, and the authoring panel.
- The local TypeScript adapter owns only typed construction of bounded intent.
- Rust/WASM owns target registration, validation, layer precedence, and
  resolution.
- Tokimu owns provider-neutral presentation semantics.
- The canvas owns pixels and hit testing; it does not resolve presentation
  precedence or define material/shader behavior.

## Initial Scene

The fixed diagnostic scene deliberately contains unrelated target kinds:

- `vector-record:diagram/outline` for a 2D vector-like surface;
- `mesh-primitive:machine/housing` for a mesh-like solid;
- `mesh-primitive:machine/fastener` for an independently selectable component;
- `renderable:hotspot/inspection` for a semantic emphasis target.

The scene is provider-neutral fixture data. It is not imported source data and
does not claim a canonical scene format.

## Interaction Contract

Clicking a viewport target selects its stable `(kind, key)` identity. The
TypeScript panel then lowers bounded authoring state to an `application` layer
presentation override:

- replacement tint;
- opacity multiplier;
- visibility;
- selected emphasis.

The hotspot button uses the higher-priority `hotspot` layer. Reset clears only
the application layer. The browser receives the resolved value from WASM and
redraws the scene; it never calculates override precedence itself.

## Explicit Non-Goals

- author-defined WGSL or arbitrary browser JavaScript execution;
- a browser-native material system;
- WebGPU rendering parity;
- persistent scene editing;
- importing assets or duplicating the asset-workbench's format proof;
- publishing `@tokimu/shader` before this consumer exposes stable requirements.

## Acceptance Criteria

- A vector target and at least two mesh-like targets are selectable in the
  viewport.
- Selection is represented by provider-neutral stable target identity.
- TypeScript authoring controls produce bounded semantic requests only.
- Rust/WASM diagnoses invalid or unknown target requests explicitly.
- Clearing a target's application layer restores its source presentation.
- The hotspot layer overrides application tint without mutating it.
- TypeScript does not parse assets, own imported material data, or compile WGSL.
- The app builds as an ASP.NET 10 static host with a `wasm32` Rust engine.

## Evidence Expected From This Consumer

This consumer should reveal whether the proposed TypeScript material API needs
a reusable package, which type names remain stable, and whether the existing
WASM command shape is sufficient. It is not evidence that an arbitrary shader
language should be admitted.
