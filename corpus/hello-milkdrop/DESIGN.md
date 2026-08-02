# Hello MilkDrop

## Purpose

`hello-milkdrop` is a headless parser and scalar-equation corpus for a
deliberately selected MilkDrop 1-style source subset.

```text
Tokimu-authored preset source
        -> milkdrop-tools parser
        -> bounded initialization / per-frame scalar evaluator
        -> source-preserving structural artifact
```

It retains ordered sections, keys, values, construct classifications, and
source lines. The admitted evaluator supports scalar assignment equations in
source order with one or more semicolon-delimited assignments, literals,
variables, arithmetic, parentheses, and `sin`, `cos`, and `abs`. Custom waves,
custom shapes, warp shaders, composite
shaders, per-pixel equations, and unknown keys are visible deferred or
unsupported evidence.

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
functions, and initialization/per-frame ordering. The construct matrix verifies
that every selected parser classification is retained with an explicit deferred
or unsupported status. The fixtures are Tokimu-authored and are limited
parser/evaluator evidence, not a claim of broad MilkDrop compatibility.
