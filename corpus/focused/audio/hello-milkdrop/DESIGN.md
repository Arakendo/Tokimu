# Hello MilkDrop

## Purpose

`hello-milkdrop` is a headless parser, scalar-equation, and literal custom-wave
and custom-shape description corpus for a deliberately selected MilkDrop
1-style source subset.

```text
Tokimu-authored preset source
        -> milkdrop-tools parser
        -> bounded initialization / per-frame scalar evaluator
        -> selected literal custom-wave and convex custom-shape descriptions
        -> source-preserving structural artifact
```

It retains ordered sections, keys, values, construct classifications, and
source lines. The admitted evaluator supports scalar assignment equations in
source order with one or more semicolon-delimited assignments, literals,
variables, arithmetic, parentheses, and `sin`, `cos`, and `abs`. Custom-wave
and custom-shape code, texture resolution, warp shaders, composite shaders,
per-pixel equations, and unknown keys are visible deferred or
unsupported evidence.

Shader-bearing entries also receive a bounded source inspection record: their
MilkDrop pass identity, source location, byte count, and selected HLSL-like
feature markers are preserved. Inspection explicitly reports translation as
deferred; it never feeds source into WGSL compilation or a renderer.

## Non-Goals

This corpus does not render a preset, load external presets, acquire PCM, open
an audio device, or compile a shader. Per-pixel equation execution is also
deferred until it has a renderer-facing lowering contract.

## Run

```text
cargo run -p hello-milkdrop -- --write-artifacts
```

The command writes one inspection artifact per fixture under
`target/hello-milkdrop/`. The equation matrix verifies its expected scalar
state before it writes evidence, covering precedence, parentheses, selected
functions, and initialization/per-frame ordering. The selected fixture also
records one literal `[wave_0]` description, including its bounded sample count,
style flags, color, and normalized center, plus one literal `[shape_0]`
description with bounded convex-polygon properties. `milkdrop-tools` can lower
the wave into point samples when a caller provides explicit waveform or
spectrum data, and can lower the shape into normalized polygon points; this
headless corpus does not render either result. It does not execute `wavecode`
or `shapecode`, resolve textures, or claim renderer equivalence. The construct matrix verifies
that every selected parser classification is retained with an explicit deferred
or unsupported status. The fixtures are Tokimu-authored and are limited
parser/evaluator evidence, not a claim of broad MilkDrop compatibility.
