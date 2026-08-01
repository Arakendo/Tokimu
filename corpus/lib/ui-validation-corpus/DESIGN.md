# UI Validation Corpus

## Purpose

`ui-validation-corpus` is the headless structural runner for the provider-neutral
contracts incubating in `ui-tools`. It validates semantic resolution, layout,
interaction targets, ordered lowering, diagnostics, and bounded work evidence
without opening a window or selecting a renderer.

The runner is evidence, not a UI framework and not a backend screenshot test.
For each case's canonical desktop/1.0-scale run, it also lowers the same draw
list into a deterministic CPU diagnostic image. That image makes gross visual
regressions inspectable while structural artifacts remain authoritative.

The initial selection covers a dense read-only inspector, an enabled/disabled
command toolbar, text entry with keyboard submission, a combined scroll/modal
composition, provider-neutral content stress, and a shared frame/split/stack/grid
composition. Interaction artifacts probe
each admitted target at
its resolved center and record press, release, and activation identities.
Input-sequence artifacts exercise normalized focus, activation, editing, and
pointer-capture contracts. This is deterministic routing evidence, not a
platform event recording.

## Ownership

- Cases own semantic UI intent.
- `ui-tools` owns resolution and renderer-neutral lowering.
- This runner owns artifact capture and declared selection.
- Renderers own GPU execution and native-window screenshots.

## Usage

```text
cargo run -p ui-validation-corpus -- list
cargo run -p ui-validation-corpus -- run
cargo run -p ui-validation-corpus -- run runtime-observation
```

Artifacts are written under `target/ui-validation-corpus/`. Structural JSON,
including `content.json` and `input-sequence.json`, uses schema
`tokimu-ui-structural-v1`; timing
evidence is observational and must not be treated as deterministic golden data.

A full run also writes `coverage.json` using schema `tokimu-ui-coverage-v1`.
The report names covered behaviors and reports each required matrix dimension as
`covered`, `partial`, `open`, or `manual`. A selected run writes
`coverage-<case>.json` so it cannot overwrite or impersonate full-selection
evidence. Open dimensions remain visible evidence rather than being hidden
behind an aggregate pass count.

Each case manifest records the versioned selection and input fingerprints,
generator identity, text provider, scale, viewport, resolver algorithm, and
draw-list lowering algorithm. Structural artifacts must remain identical for
identical semantic inputs. Native-window screenshots and GPU framebuffer
captures are separate backend evidence and are not produced by this runner.

Canonical runs write `cpu-image.bmp` and `cpu-image-manifest.txt`. The manifest
records the raster algorithm, pixel fingerprint, draw-list fingerprint,
built-in bitmap text provider, dimensions, and the explicit fact that the image
is not GPU-framebuffer-equivalent. The rasterizer belongs to this corpus because
it is a diagnostic observer of `UiDrawList`, not an admitted UI or renderer
contract.

Scale cases convert each selected physical viewport into logical UI units
before resolution. This exercises consumer-owned DPI/logical scaling without
adding backend DPI state to `UiTree`; manifests record physical dimensions,
logical dimensions, and scale separately.

The content-stress case records exact empty, ordinary, long, and multiline text
in `content.json`. Missing-glyph coverage remains open because glyph support is
a font-provider result; provider-neutral resolution must not invent one.

The composition-layout case uses the consumer-facing `UiFrameLayout`,
`UiHorizontalSplitLayout`, `UiVerticalStack`, and `UiUniformGridLayout` APIs.
Its cells and controls are resolved without case-local rectangle iteration, so
the structural artifacts directly pressure the reusable composition boundary.
