# Camera Clip Depth And Provider Adaptation

Tokimu camera projection matrices use GL-style normalized clip depth:

```text
near = -1
far  =  1
```

WebGPU/WGPU accepts clip depth in `[0, 1]`. The WGPU adapter must therefore
remap depth when it constructs its private camera uniform:

```text
z_wgpu = 0.5 * z_tokimu + 0.5 * w_tokimu
```

Do this once at the provider boundary. Do not change caller-owned camera
matrices, ask applications to pre-convert them, or hide the conversion inside
a camera constructor. Surface and renderer-owned-target passes must share the
same conversion helper.

## Debugging Consequence

A missing conversion can produce all of these observations at once:

```text
draw commands accepted
resources and pipelines resolved
queue submission succeeds
present succeeds
no backend diagnostic exists
frame contains only the clear color
```

That is valid GPU clipping, not a swallowed validation error. Submission and
presentation counters describe work, not visible pixels. Use a known-good
opaque control and inspect the complete view-projection-to-provider mapping
before expanding diagnostics or adding framebuffer readback.

## Retained Regression

The WGPU unit regression requires Tokimu depths `-1`, `0`, and `1` to become
WGPU depths `0`, `0.5`, and `1` without mutating `Camera`. The AR-0023 native
blend fixture deliberately uses positive GL-space depths so losing the adapter
conversion returns a visibly empty frame.

## Evidence

- [AR-0024](../Architectural%20Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md)
- [`wgpu_backend.rs`](../../crates/tokimu-render/src/wgpu_backend.rs)
- [Alpha-policy corpus](../../corpus/campaigns/textured-presentation/hello-alpha-policy/README.md)
