# ADR-0018 Integrated Command Conformance Evidence

| Field | Value |
| --- | --- |
| Status | Implementation and cross-target build complete; live browser WGPU observation pending |
| Date | 2026-08-20 |
| Scope | Provisional provider-neutral set-scoped command validation integrated with the corpus-private WGPU staging path |
| Authority | Structural and executable evidence; not final API admission or physical reclamation evidence |

## Question

Can a real command retained from resource set A reject after resource set B
commits with the same local resource keys, before those keys can resolve as B's
resources?

This is the integrated Finding 5 gate retained by AR-0032 and ADR-0018.

## Implemented Seam

The feature-gated experiment adds a provider-neutral command batch with:

- a private authority token that callers cannot forge from a numeric ID;
- an opaque resource-set ID;
- an owned copy of ordinary `RenderCommand` values; and
- whole-batch validation against the current authority and resource set before
  ordinary renderer submission begins.

The WGPU staging path allocates a fresh candidate set identity, carries it into
the staged backend, and changes the current identity only after candidate
validation succeeds. Failed candidates consume an identity but cannot change
the current one. Commit changes resources and set authority in the same
backend-local operation.

The seam remains named `Experimental*`, feature-gated by
`experimental-scene-resource-staging`, and hidden from generated documentation.
It does not select the final public transaction or handle representation.

## Executable Falsifiers

Native provider-neutral tests prove:

1. a command batch from A rejects when B is current;
2. B commands remain valid while reusing the same ordinary local handles; and
3. an equal numeric set ID from another authority rejects rather than aliases.

The browser WGPU probe now performs:

```text
upload and present A
    -> retain A's actual draw-command batch
    -> stage B late failure
    -> present A unchanged
    -> stage complete B using A's local keys
    -> retain B's actual draw-command batch
    -> commit B
    -> submit retained A batch: must reject stale before handle resolution
    -> present already-committed B
    -> submit scoped B batch: must succeed
    -> present B again
```

Its success record must contain:

```text
reused-local-resource-keys=true
stale-rejected-before-resource-resolution=true
B-draws-after-commit=8
scoped-B-draws=8
provider-diagnostics=0
```

## Validation Completed

- `cargo test -p tokimu-render --features experimental-scene-resource-staging`
  passed 67 tests, including all three new set-authority cases.
- Native strict Clippy passed for `tokimu-render` with the staging feature.
- Release `wasm32-unknown-unknown` compilation passed for the browser fixture.
- Strict WASM Clippy passed after allowing the pre-existing
  `arc_with_non_send_sync` findings in WGPU's WASM-only texture/view storage;
  unrelated manual parity expressions in the fixture were updated for the
  current lint set.
- `cargo test --workspace` passed.
- Full workspace strict Clippy reached an unrelated pre-existing
  `large_enum_variant` finding in Doom visibility conformance; the changed
  renderer and browser-fixture targets pass their scoped strict Clippy checks.
- The regenerated WASM package and local fixture both serve successfully over
  HTTP 200 at the documented paths.

## Pending Live Observation

The repository-side browser fixture is ready at `http://127.0.0.1:4177/` under
**Probe staged replacement + stale command rejection**. This run has not yet
been retained as live WGPU evidence. The in-app browser controller failed to
initialize because its own trusted-code-path setup rejected its browser service
dependency; that is tool availability, not a Tokimu or fixture failure.

Until the live record is captured, this evidence establishes implementation
shape, provider-neutral rejection semantics, native tests, and WASM
compatibility. It does not yet close ADR-0018's provider-backed conformance
gate.

## Non-Claims

This slice does not establish:

- the final public command-batch or handle representation;
- physical GPU reclamation or bounded VRAM overlap;
- provider fence, polling, or drop policy;
- incremental release or device-loss recovery;
- conformance of a second real renderer backend; or
- promotion of the experimental WGPU staging API.

## References

- `docs/ADR/ADR-0018-atomic-staged-render-resource-set-replacement.md`
- `docs/Architectural Reviews/AR-0032-atomic-staged-render-resource-set-replacement.md`
- `docs/Plans/Renderer-Reliability/renderer-scene-resource-lifetime-and-replacement.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-alternative-c-real-provider-staging-evidence.md`
- `crates/tokimu-render/src/experimental_render_resource_set.rs`
- `crates/tokimu-render/src/wgpu_backend/experimental_scene_resource_staging.rs`
- `corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/src/main.rs`
