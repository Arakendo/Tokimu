# Alternative C Real-Provider Staging Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Status | Implemented; live browser execution pending |
| Scope | Corpus-private WGPU resource-set staging |
| Stable API admission | None |

## Question

Can one WGPU-backed scene-resource set B be allocated alongside a still-live
set A, fail without disturbing A, then commit as the new live set while A is
retired predictably?

This experiment does not decide public handles, a general provider lifecycle
API, physical reclamation timing, overlap-memory budgets, or repeated
replacement policy.

## Implemented Boundary

The feature-gated WGPU experiment now separates two kinds of ownership:

```text
provider session
    instance + device + queue + surface layouts
    shared by live and candidate sets

scene resource set
    meshes + textures + materials + pipelines
    + cameras + commands
    isolated until candidate commit
```

Native uses thread-safe shared provider ownership; browser WASM uses
single-thread shared ownership. A provider-session token prevents committing a
candidate into a different backend. The live surface itself remains owned by
the live backend and is not duplicated.

The browser corpus exposes one bounded sequence:

```text
create provider session
    -> allocate and present A
    -> allocate most of B beside A
    -> fail B on a missing staged texture reference
    -> drop failed B
    -> present A again
    -> allocate complete B beside A
    -> validate B draw references
    -> commit B by backend-local resource-set swap
    -> present B
```

The failure is late: eight textures, eight materials, eight meshes, one
pipeline, and one camera are staged before the intentionally invalid material
is attempted. No reset or mutation of A precedes that failure.

## Structural Validation

- `cargo clippy -p tokimu-render --features experimental-scene-resource-staging --all-targets -- -D warnings`
- `cargo test -p tokimu-render --features experimental-scene-resource-staging`
  - 64 passed
- `cargo test -p hello-render-resource-identity`
  - 24 passed
- `cargo check -p hello-render-resource-identity-web --target wasm32-unknown-unknown`
- `cargo build -p hello-render-resource-identity-web --target wasm32-unknown-unknown --release`
- `cargo check --workspace`

The generated browser package was rebuilt successfully. Live execution was not
claimed because the managed browser connection rejected its own installed
browser runtime as outside the configured trusted code path. The fixture is
available at `http://127.0.0.1:4177/` through **Probe Alternative C
real-provider staging**.

## Authority And Limits

Successful live execution may establish that:

- A and candidate B use one instance/device/queue/surface session;
- failed B allocation leaves A presentable;
- complete B becomes live only at commit;
- the preceding logical resource set is dropped predictably after commit.

It still cannot establish:

- when WGPU or the driver physically reclaim A;
- the peak physical allocation during A/B overlap;
- whether repeated replacement remains bounded;
- that generation identity should directly govern provider objects;
- a final public handle or resource-lifecycle contract.

Those remain separate admission questions.
