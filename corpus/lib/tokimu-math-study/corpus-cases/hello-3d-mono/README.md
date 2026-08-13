# Hello 3D Mono Candidate Copies

This directory reserves separate corpus-local copies of the existing
`corpus/focused/foundations/hello-3d-mono` evidence for the math vocabulary study.

```text
hello-3d-mono/
    baseline-a/             unchanged `corpus/focused/foundations/hello-3d-mono` control
    alternative-b/          provider-backed Tokimu vocabulary port (runnable)
    alternative-c/          owned Tokimu implementation port (runnable)
    alternative-d-blocked/  retained blocked record until D earns `Mat4`
```

## Copy Rules

- Each runnable copy starts from the same checked source revision of
  `corpus/focused/foundations/hello-3d-mono`; its candidate-specific edits stay within that copy.
- Do not replace the original corpus entry, edit stable crates, or hide a
  missing candidate operation behind `glam` in the copied case.
- Record per-copy source edit count, explicit conversion count, helper count,
  native/WASM result, visual/deterministic transform result, and rollback.
- `alternative-d-blocked` is a valid result until D has a reviewed `Mat4`;
  it must name the missing operation rather than silently omitting the case.

The current `src/migration_hello_3d_mono.rs` is a small A/B/C transform
preflight. The B and C app copies now add native compile evidence; the original
`corpus/focused/foundations/hello-3d-mono` remains the A control so the study does not introduce a
second unmodified app solely to duplicate source. They do not yet constitute a
visual-runtime or WASM result.

## Initial Application-Copy Evidence

| Candidate | App package | Math-facing provider type in app | Renderer crossing | Native compile |
| --- | --- | --- | --- | --- |
| A | `corpus/focused/foundations/hello-3d-mono` | direct current `tokimu_core::math` | direct camera assignment | existing control |
| B | `alternative-b` | none | one study-owned `alternative_b_camera` call | passed offline |
| C | `alternative-c` | none | one study-owned `alternative_c_camera` call | passed offline |
| D | `alternative-d-blocked` | not implemented | blocked: no candidate `Mat4` | not applicable |

The B adapter can pass its private provider representation directly because it
delegates mechanics to that provider. The C adapter performs an explicit
column-array conversion at the renderer boundary. This is the first observed
ergonomics and potential performance distinction between them; it requires
runtime measurement before it can influence a decision.

## WASM Status

`cargo build -p tokimu-math-study --target wasm32-unknown-unknown --locked
--offline` passed, and the shared platform closure-ownership compile defect was
repaired. The B/C copies still do not compile for WASM, but neither does the A
control: all three are intentionally native-window-shaped. They call
`run_window_with_app`, pass `Arc<NativeWindow>` where the browser backend needs
`HtmlCanvasElement`, and use synchronous backend construction where browser
creation is asynchronous. This is a common application-shape limitation, not
an Alternative B/C failure; native checks of both copies passed.
