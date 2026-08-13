# Renderer Failure Observation Boundary Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-11 |
| Plan | [Renderer Resource Identity And Failure Presentation](../renderer-resource-identity-and-failure-presentation.md) |
| Reviews | AR-0024, AR-0027 |
| Status | Complete scoped comparison; shared terminal owner dormant pending reopening pressure |

## Provisional Corpus Envelope

`corpus/campaigns/renderer-reliability/hello-render-resource-identity` contains a fixed-capacity, corpus-only
`FailureObservation` record:

```text
sequence
phase
operation
provider-neutral category
optional typed mesh identity
caller correlation
continuation result
```

It deliberately excludes WGPU objects, native-window objects, JavaScript
objects, formatted diagnostic text, source-format terms, recovery policy, and
visual-presentation choice. The fixture holds at most eight records and
retains its monotonic total separately. It is a comparison model, not a public
diagnostic contract.

## Cross-Layer Matrix

| Layer | Retained corpus evidence | Provisional category / continuation | Where the detailed evidence lives |
| --- | --- | --- | --- |
| Source preparation | E1M1 door re-lowering initially lacked a retained `BRNBIGL` extent | `SourceUnavailable` / reject operation and continue | E1M1 console/stderr plus AR-0024/AR-0027 history |
| Renderer resource resolution | Mutable-handle fixture resolves a never-uploaded handle after aliasing | `ResourceUnresolved` / reject operation and continue | `hello-render-resource-identity` tests |
| Provider validation | `hello-shader --backend-diagnostic-fixture` submits intentionally invalid WGSL | `ProviderRejected` / active fixture ends after retaining evidence | WGPU diagnostic sink; fixture output retains module and entry-point identities |
| Surface presentation | Browser/WASM orientation fixture historically passed browser adapter/device preflight but timed out before Tokimu readiness | `SurfaceUnavailable` / end active composition | Browser status surface and AR-0021 history |
| Application frame handler | Before E1M1 containment, door-refresh error returned from `on_frame` | `HandlerReturnedError` / end active composition | native platform `pending_error`; historical AR-0024 Cycle 6 |
| Platform termination | Native platform records handler error, exits the loop, and returns the error after `run_app` | `EventLoopTerminated` / no continuation claim | `tokimu-platform` native runner; terminal caller receives error after window closes |

The matrix is intentionally not a claim that every provider or surface
failure has the same recovery. It preserves the fact that an error can be
explicitly observed at one layer yet poorly presented after a composition ends.

## Live Provider Validation Observation

Command:

```powershell
cargo run -p hello-shader -- --backend-diagnostic-fixture
```

Native AMD Radeon RX 7900 XTX / Vulkan output retained on 2026-08-11:

```text
hello-shader backend diagnostic fixture passed:
  module=hello-shader-intentional-invalid
  vertex=vs_fixture
  fragment=fs_fixture

WebGPU backend validation failed:
  unresolved_fixture_symbol
```

The detailed WGPU parsing and pipeline-validation messages remain in the
provider sink. The corpus envelope records only that a provider validation
operation was rejected and what the application chose to do next.

## Native Terminal Delivery Fixture

`hello-render-resource-identity` now includes the native-only
`native_terminal_error` binary. It returns one intentional error from
`PlatformEventHandler::on_frame`, lets `tokimu-platform` end the active event
loop, then reports the returned error from the terminal caller. This is a
negative presentation result as well as a delivery result:

```text
active native window closes
    -> platform returns application error to caller
    -> terminal caller can retain/present it
```

The fixture deliberately does **not** claim an in-window terminal record. Its
purpose is to distinguish "the error was lost" from "the platform ended the
composition before the application selected a presentation surface." It is
native-only; its browser counterpart remains the fixture-owned DOM status
surface rather than a shared platform contract.

Command and observed output on 2026-08-11:

```powershell
cargo run -q -p hello-render-resource-identity --bin native_terminal_error
```

```text
application-frame-handler returning intentional error
terminal caller retained error after active composition ended:
  intentional corpus application-frame failure
```

The process exits successfully only because the corpus caller verified receipt
of the expected error. It does not convert the failed frame into success.

## First-Failure Preservation Regression

The opt-in E1M1 `--doom-sky` path supplied a second native terminal-lifetime
case. Startup correctly rejected `SKY1` because the composed source retained
2,048 uncovered pixels. The native platform recorded that error and requested
event-loop exit, but still entered the frame callback before shutdown. The
frame then failed because the rejected startup had never created the sky
pipeline, replacing the useful root cause with:

```text
Doom sky pipeline missing
```

This was not a missing renderer diagnostic. It was a terminal-record lifetime
defect: a later callback overwrote the already-retained failure. The native
adapter now obeys two private composition rules:

```text
first terminal callback failure wins
pending terminal failure => do not dispatch another frame callback
```

The same command now returns the original bounded source identity and count:

```text
E1M1 SKY1 retained 2048 uncovered pixels; sky coverage policy remains unresolved
```

This strengthens caller-owned terminal delivery without introducing a global
mailbox, shared terminal-record owner, renderer fallback, or new public error
type. A focused unit regression proves a secondary failure cannot replace the
first. The unresolved `SKY1` policy remains a Doom sky-presentation question,
not a platform recovery decision.

## Validation

```powershell
cargo test -p hello-render-resource-identity
cargo check -p hello-render-resource-identity --target wasm32-unknown-unknown
```

Result: 18 tests passed; the corpus fixture compiles for WASM. The fixture
proves fixed-capacity retention and all six modeled phases. It does not execute
browser/WGPU failure delivery.

## Open Slice 3 Gaps

- Native terminal delivery is still process/terminal-visible after the window
  has closed; the normal E1M1 path avoids that only by corpus-local containment
  and its console overlay.
- Browser records are presented by each fixture's DOM status surface; there is
  no common retained record after page/composition disposal.
- There is no shared agreement yet about which categories, correlation IDs, or
  continuation results belong outside corpus code.
- Native terminal delivery still carries arbitrary application error text; the
  first-error invariant preserves causality but does not by itself make every
  record structurally bounded or source-correlatable.
- No fatal panic/abort continuation is claimed.

The focused `hello-render-resource-identity-web` fixture is now packaged to
carry the same bounded `ResourceUnresolved` record into a browser-owned DOM
status beside real WGPU same-handle replacement. Its implementation does not
close the cross-target category/identity gate until the browser record is
observed, and it deliberately makes no claim after page disposal.

The actual browser run retained `ResourceUnresolved`, resource
`MeshHandle(44)`, and caller `identity-fixture` after the WGPU provider returned
and one replacement draw presented. This matches the native fixture's semantic
category and identity without pretending that DOM and terminal retention have
the same lifetime. The page-disposal owner question remains open by maintainer
decision.

These gaps prevent admission of the envelope. They are the input to the
remaining Slice 3 terminal-record and cross-target experiments.

## Current Boundary Conclusion

The study now has two real terminal-delivery mechanisms, neither of which has
earned replacement by a shared contract:

```text
native application frame error
    -> native platform ends the event loop
    -> invoking caller receives Result error after the window closes

browser/WASM fixture error
    -> fixture-owned DOM status remains visible while the page remains alive
```

Both prevent the tested failure from being silent, but their retention and
presentation lifetimes differ. Making a common record survive window/page
disposal would require a deliberate owner for storage, retrieval, and user
presentation. The current corpus evidence does not identify that owner or a
second independent caller for it, so the 2026-08-11 maintainer disposition is
to leave a shared owner open. Slice 3 must not turn this comparison model into
a platform or renderer API.

Reopen that ownership question only when a caller must retain and attribute a
terminal record after the originating composition/provider has been disposed,
when a replacement composition or supervisor must inspect that evidence, or
when independent callers converge on the same retained-record lifetime need.
