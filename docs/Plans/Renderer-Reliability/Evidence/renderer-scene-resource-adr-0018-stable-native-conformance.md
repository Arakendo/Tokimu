# ADR-0018 Stable Native WGPU Conformance Evidence

| Field | Value |
| --- | --- |
| Status | Complete for native WGPU resource-set session; separate live browser evidence also complete |
| Date | 2026-08-20 |
| Target | Native WGPU, Vulkan, AMD Radeon RX 7900 XTX |
| Contract | Stable `RenderResourceSetLifecycle` through an opt-in resource-set session |
| Authority | Executable provider-backed evidence; not physical GPU reclamation evidence |

## Question

Does the provider-neutral lifecycle candidate preserve a complete current resource
set through a late candidate failure, commit a successor atomically, and reject
a retained predecessor command before reused local handles resolve through every
public submission path?

## Fixture

`staged_resource_set_native` creates one native WGPU provider session and runs:

```text
populate A on an ordinary backend
    -> consume the backend into a resource-set session
    -> scope and present A
    -> stage B mesh + texture + material + pipeline + camera + commands
    -> inject a late missing-texture material failure
    -> discard B and present A again
    -> populate complete B through replace_resource_set
    -> commit B
    -> reject retained A command batch
    -> present committed B
    -> submit and present scoped B command batch
    -> terminate normally
```

Both sets deliberately reuse local handle values. The failed candidate consumes
set identity 2, so the successful successor advances from set 1 to set 3.

## Result

```text
status=complete
contract=ADR-0018-provider-neutral-resource-set-session
target=native-wgpu
sequence=populate-A>enter-resource-set-session>present-A>stage-B-all-families>
  late-failure>present-A>replace-B>reject-scoped-A>present-B>
  submit-scoped-B>present-B
A-draws=1
A-after-failure-draws=1
B-draws=1
scoped-B-draws=1
set-A=1
set-B=3
forced-failure=MissingTexture(2)
retired=[draws:1,materials:1,textures:1,meshes:1,pipelines:1,cameras:1]
committed=[draws:1,materials:1,textures:1,meshes:1,pipelines:1,cameras:1]
scoped-stale-A=StaleResourceSet(requested:1,current:3)
unscoped-submit-surface=absent
provider-diagnostics=0
backend=vulkan
device=discrete-gpu
adapter=AMD Radeon RX 7900 XTX
process-exit=0
```

The lifecycle mechanics passed: current A remained complete after an all-family
candidate failed, and commit replaced every family in one observable operation.
The scoped A batch rejected before B handle resolution. The replacement-enabled
session has no raw `Renderer::submit` surface, and a compile-fail contract test
guards that fact. Ordinary backends retain raw submission but cannot perform
set replacement.

This supersedes, but does not erase, the first stable-surface run. That earlier
shape exposed replacement and unscoped submission on the same backend and was
correctly falsified when retained A declarations resolved B's reused keys. The
selected session boundary is the architectural response to that evidence.

## Validation

- `cargo clippy -p hello-render-resource-identity --bin staged_resource_set_native -- -D warnings`
- `cargo run -p hello-render-resource-identity --bin staged_resource_set_native`
- `cargo test -p tokimu-render`
- `cargo build -p hello-render-resource-identity-web --target wasm32-unknown-unknown --release`
- `cargo test --workspace`

The first workspace run reported one transient failure in
`presentation-geometry-corpus`; that package passed immediately in isolation
and the complete workspace rerun passed. Strict native Clippy passes for the
changed renderer and native fixture. Strict WASM-target Clippy remains blocked
by six pre-existing `arc_with_non_send_sync` findings in WGPU texture/material
storage; the release WASM build itself passes.

## Non-claims

This does not establish physical GPU reclamation timing, bounded VRAM overlap,
device-loss recovery, or individual resource-handle encoding. This native run
alone does not establish browser conformance; the separate stable-browser
evidence record does.

## References

- `docs/ADR/ADR-0018-atomic-staged-render-resource-set-replacement.md`
- `docs/Architectural Reviews/AR-0032-atomic-staged-render-resource-set-replacement.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-adr-0018-stable-browser-conformance.md`
- `crates/tokimu-render/src/resource_set.rs`
- `crates/tokimu-render/src/wgpu_backend/resource_set_staging.rs`
- `corpus/campaigns/renderer-reliability/hello-render-resource-identity/src/bin/staged_resource_set_native.rs`
