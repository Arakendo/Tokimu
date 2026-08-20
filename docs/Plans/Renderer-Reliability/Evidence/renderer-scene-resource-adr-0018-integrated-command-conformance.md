# ADR-0018 Integrated Command Conformance Evidence

| Field | Value |
| --- | --- |
| Status | Integrated experimental-candidate gate complete |
| Date | 2026-08-20 |
| Scope | Provisional provider-neutral set-scoped command validation integrated with the corpus-private WGPU staging path |
| Authority | Structural and executable evidence; not final API admission or physical reclamation evidence |

## Question

Can a real command retained from resource set A reject after resource set B
commits with the same local resource keys, before those keys can resolve as B's
resources?

This is the integrated Finding 5 gate retained by AR-0032 and ADR-0018.

## Implemented Seam

The feature-gated experiment originally added a provider-neutral command batch with:

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

At the time of this retained run, the seam remained named `Experimental*` and
feature-gated by `experimental-scene-resource-staging`. It has since been
realized as the stable `RenderResourceSetLifecycle` and `RenderCommandSet`
surface without changing the evidence recorded here. Individual resource-handle
encoding remains undecided.

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

## Live Browser WGPU Observation

The maintainer ran **Probe staged replacement + stale command rejection** on
2026-08-20. The retained result was:

```text
status=complete
backend=browser-webgpu
backend-creations=1
device-creations=1
surface-creations=1
retained-provider-session=true
staged-before-failure=26
forced-stage-failure=MissingTexture(9)
A-draws-initial=8
A-draws-after-failed-B=8
last-known-good-preserved=true
resource-set-A=1
resource-set-B=3
retained-A-command-after-B=StaleResourceSet(requested=1,current=3)
reused-local-resource-keys=true
stale-rejected-before-resource-resolution=true
B-draws-after-commit=8
scoped-B-draws=8
retired-A-predictable=true
provider-diagnostics=0
overlap-physical-bytes=unmeasured
retired-physical-reclamation=unobserved
```

The resource-set jump from 1 to 3 is expected: the deliberately failed B stage
consumed set identity 2 without changing current authority, and the complete B
stage received identity 3. A remained presentable across that failure. After B
committed with A's local keys, the retained A command rejected with the exact
requested/current set identities before ordinary command resolution. B then
presented once from committed queued commands and again from its scoped batch,
both with eight draws and no delivered provider diagnostics.

This closes AR-0032 Finding 5 for the feature-gated experimental candidate. It
does not promote that candidate unchanged or settle the non-claims below.

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
- `crates/tokimu-render/src/resource_set.rs`
- `crates/tokimu-render/src/wgpu_backend/resource_set_staging.rs`
- `corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/src/main.rs`
