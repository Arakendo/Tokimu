# ADR-0018 Stable Native WGPU Conformance Evidence

| Field | Value |
| --- | --- |
| Status | Lifecycle mechanics pass; stable candidate falsified by unscoped submission bypass |
| Date | 2026-08-20 |
| Target | Native WGPU, Vulkan, AMD Radeon RX 7900 XTX |
| Contract | Stable `RenderResourceSetLifecycle` |
| Authority | Executable provider-backed evidence; not physical GPU reclamation evidence |

## Question

Does the provider-neutral lifecycle candidate preserve a complete current resource
set through a late candidate failure, commit a successor atomically, and reject
a retained predecessor command before reused local handles resolve through every
public submission path?

## Fixture

`staged_resource_set_native` creates one native WGPU provider session and runs:

```text
present A
    -> stage B mesh + texture + material + pipeline + camera + commands
    -> inject a late missing-texture material failure
    -> discard B and present A again
    -> populate complete B through replace_resource_set
    -> commit B
    -> reject retained A command batch
    -> present committed B
    -> submit retained ordinary A commands through Renderer::submit
    -> observe them resolve successfully against B's reused local keys
    -> submit and present scoped B command batch
    -> terminate normally
```

Both sets deliberately reuse local handle values. The failed candidate consumes
set identity 2, so the successful successor advances from set 1 to set 3.

## Result

```text
status=falsified
target=native-wgpu
A-draws=1
A-after-failure-draws=1
B-draws=1
scoped-B-draws=1
unscoped-A-after-B-draws=1
set-A=1
set-B=3
forced-failure=MissingTexture(2)
retired=[draws:1,materials:1,textures:1,meshes:1,pipelines:1,cameras:1]
committed=[draws:1,materials:1,textures:1,meshes:1,pipelines:1,cameras:1]
stale-A=StaleResourceSet(requested:1,current:3)
provider-diagnostics=0
unscoped-submit-bypass=true
backend=vulkan
device=discrete-gpu
adapter=AMD Radeon RX 7900 XTX
process-exit=0
```

The lifecycle mechanics passed: current A remained complete after an all-family
candidate failed, and commit replaced every family in one observable operation.
The scoped A batch rejected before B handle resolution. The same retained
ordinary A commands then passed through `Renderer::submit` and produced one draw
against B's reused local keys. Therefore the candidate does not make stale-set
rejection authoritative across the renderer's public submission surface. The
process still terminated normally, so this is a semantic falsifier rather than
a provider crash or diagnostic failure.

## Validation

- `cargo clippy -p hello-render-resource-identity --bin staged_resource_set_native -- -D warnings`
- `cargo run -p hello-render-resource-identity --bin staged_resource_set_native`
- `cargo test --workspace`

## Non-claims

This does not establish physical GPU reclamation timing, bounded VRAM overlap,
device-loss recovery, individual resource-handle encoding, or browser WGPU
conformance of the stable surface.

## References

- `docs/ADR/ADR-0018-atomic-staged-render-resource-set-replacement.md`
- `docs/Architectural Reviews/AR-0032-atomic-staged-render-resource-set-replacement.md`
- `crates/tokimu-render/src/resource_set.rs`
- `crates/tokimu-render/src/wgpu_backend/resource_set_staging.rs`
- `corpus/campaigns/renderer-reliability/hello-render-resource-identity/src/bin/staged_resource_set_native.rs`
