# Renderer Containment And Recovery Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-11 |
| Plan | [Renderer Resource Identity And Failure Presentation](../renderer-resource-identity-and-failure-presentation.md) |
| Reviews | AR-0024, AR-0027 |
| Status | Slice 4 corpus comparison retained; no shared recovery API admitted |

## Question

Can the existing caller, renderer upload, and platform-result seams contain the
observed failures without a generic renderer exception layer, automatic
fallback asset, or kernel-owned recovery policy?

## Corpus Comparison

`hello-render-resource-identity::observe_caller_staged_recovery` uses the
current replace-on-upload ledger only after caller-side logical identity
validation.

```text
last known-good: MeshHandle(77) -> Dynamic(7)
candidate:                           StaticCutout(7)

caller identity check
    -> rejected before upload
    -> renderer resource remains Dynamic(7)

later candidate:                     Dynamic(7)
    -> explicitly allowed replacement
    -> renderer resource remains attributable to Dynamic(7)
```

The failed candidate never reaches the replace-on-upload operation. The
renderer is therefore not asked to infer whether a replacement is accidental,
and no fallback mesh, material, texture, shader, or pipeline is substituted.

## Failure Classification

| Case | Observed containment | Claim boundary |
| --- | --- | --- |
| Missing source extent / unresolved resource | caller rejects the operation and continues with retained evidence | Does not claim a generic source fallback |
| Incompatible replacement candidate | caller retains last known-good identity by rejecting before upload | Requires caller-owned retained source/identity; renderer does not retain a rollback copy |
| Native returned frame error | platform ends active event loop and returns error to invoking caller | Terminal caller receives error; no in-window terminal surface is implied |
| Provider rejection | invalid pipeline is never retained/submitted; the `hello-shader` fixture then presents its ordinary valid scene before ending | Evidence covers rejected pipeline plus later valid frame, not provider/device-loss recovery |
| Fatal/abort category | corpus model records `FatalNoContinuationClaim` | No panic/abort continuation is attempted or claimed |

## Bounded Repeat Observation

Five repeated unresolved-resource observations were inserted into a capacity-3
corpus record. The evidence retains:

```text
total failures: 5
retained records: 3
```

The count proves that truncation cannot pretend no failures occurred, while the
fixed record capacity prevents unbounded steady-state retention. This is an
in-memory corpus comparison only; it does not claim console or browser-bridge
rate limiting.

## Validation

```powershell
cargo test -p hello-render-resource-identity
cargo clippy -p hello-render-resource-identity --all-targets -- -D warnings
cargo check -p hello-render-resource-identity --target wasm32-unknown-unknown
```

Result: 18 tests passed; strict linting and WASM compilation passed. Existing
upstream `glam` warnings remain outside this fixture's source.

## Fatal-Path Marker

`native_frame_panic` is a separately invoked native-only corpus binary. It
will panic from `PlatformEventHandler::on_frame` after recording an explicit
terminal marker. The expected outcome is process failure and destruction of
the active composition. It intentionally has no `catch_unwind`, no renderer
recovery, and no attempt to continue on the next frame.

Observed on 2026-08-11:

```text
native fatal fixture: window created
exit-code=101 (nonzero expected)
```

This retains only process/terminal evidence of the fatal path. It does not
promise a recoverable platform record after a panic.

## Current Conclusion

The smallest demonstrated containment arrangement is deliberately
compositional:

```text
caller owns source identity and replacement decision
    -> validates candidate before renderer upload
renderer owns the selected resource upload
    -> existing replace-on-upload behavior
platform owns event-loop termination and Result delivery
```

No shared recovery API, rollback buffer, fallback asset, or kernel exception
owner is earned by this result. The remaining Slice 4 gap is a real
fatal/abort lifecycle experiment; it must not be assumed from the
application-side comparison.
