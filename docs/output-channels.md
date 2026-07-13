# Tokimu Multi-Channel Output Model

| Field      | Value                                                                 |
| ---------- | --------------------------------------------------------------------- |
| Status     | Draft — grow-then-fold                                                 |
| Scope      | How run/command-line output is split into channels, throttled, and routed so useful lines are not buried. Not yet v0. |
| Relates to | `diagnostics-model.md` (diagnostic identity/codes/sinks)              |
| Relates to | SDD 10 Diagnostics, SDD 3.1 guiding principle 7 (visible diagnostics) |
| Relates to | `roadmap.md` Next Wiring Steps                                         |

## 1. Purpose

Running the native examples today floods a single stdout stream with
high-frequency, low-value lines that bury the few lines that actually matter.

Concrete evidence from a recent `cargo run -p hello-window`:

* Hundreds of repeated `move intent = (0.0, 0.0)` lines.
* A continuous `frame dt=... fixed_updates=... elapsed=...` line roughly once per
  second, forever.
* The one-shot, genuinely useful lines (backend/adapter selection in
  `hello-triangle`, any frame-overrun warning) are a needle in that haystack.
* The run produced so much text it overran the terminal tool's inline capture
  (~20 KB) and spilled to a file. Nothing failed — the signal was simply buried
  under volume.

The `diagnostics-model.md` document answers *what a diagnostic is* (stable code,
severity, class, context). This document answers a different, complementary
question: **how the stream of emitted lines is separated, paced, and routed so
that a long-running loop cannot bury a rare, important line.**

The central claim: run output should be **multi-channel**. Producers write to a
named channel with a declared cadence; a router applies per-channel pacing
policy and fans channels out to sinks. A per-frame channel being noisy must
never be able to hide a lifecycle or error channel.

Design goals:

* No high-frequency channel can bury a low-frequency channel.
* Repetition is collapsed, not reprinted (coalesce identical consecutive lines).
* Channels are cheap to mute, enable, or redirect without touching call sites.
* Presentation stays a consumer; producers only declare channel + cadence.
* Native and WASM route the same channels through platform-appropriate sinks.
* This layer carries both structured diagnostics and plain developer prints.

Design notes from review:

* Keep this document separate from `diagnostics-model.md` until the transport
  shape stabilizes. Identity and transport are adjacent but distinct concerns.
* If a future `visibility` or `importance` axis appears, add it only when a real
  use case demands it. `Channel` + `Cadence` should carry the first version.
* Keep the flow one-way: runtime emits channels, consumers subscribe. The output
  layer must not become a backchannel into runtime control.

## 2. Relationship to the Diagnostics Model

These two documents are deliberately separate and must not be merged prematurely.

| Concern                        | Owned by               |
| ------------------------------ | ---------------------- |
| Diagnostic *identity* — code, severity, class, context, provenance | `diagnostics-model.md` |
| Diagnostic *transport* — channel, cadence, throttling, routing, CLI verbosity | this document |

A diagnostic (`TKM-RUNT-0001`, frame overrun, warning) is *what* is emitted. A
channel (`frame`, sampled cadence, default-muted) is *how* that emission is
paced and where it lands. A single coded diagnostic flows over a channel to one
or more sinks. Raw developer `println!`-style prints also flow over channels,
even before they earn a diagnostic code.

This keeps the diagnostics model free of pacing policy, and keeps this model free
of identity policy.

## 3. Channels

A channel is a named, cadence-tagged output stream. Channels map to *why* a line
exists, not to which crate emitted it (that is the diagnostic `class`).

| Channel     | Purpose                                             | Typical cadence   |
| ----------- | --------------------------------------------------- | ----------------- |
| `lifecycle` | boot/shutdown, window created, backend chosen       | one-shot          |
| `env`       | adapter, backend api, device kind, feature support  | one-shot          |
| `frame`     | per-frame timing (dt, fixed updates, elapsed)       | sampled           |
| `input`     | input-to-intent changes (move intent, buttons)      | on-change         |
| `app`       | example/app gameplay events (score, target hit)     | on-change / burst |
| `warn`      | recoverable warnings (frame overrun, cap hit)       | on-event          |
| `error`     | failures and fatal conditions                       | on-event          |
| `trace`     | opt-in verbose developer detail                     | per-frame (debug) |

Rules:

* `warn` and `error` are never suppressed by volume on other channels and are
  never coalesced away to nothing (a collapsed count is still surfaced).
* `frame` and `trace` are the noisy channels and are the ones a router paces or
  mutes by default.
* A channel is a producer-side grouping; a sink (see `diagnostics-model.md` §6)
  is the consumer-side destination. Channels and sinks are orthogonal.

## 4. Cadence Classes

Cadence declares how often a channel is *expected* to emit, so the router can
apply the right anti-burial policy without knowing the call site.

| Cadence      | Meaning                                    | Router policy                        |
| ------------ | ------------------------------------------ | ------------------------------------ |
| `one-shot`   | emitted once per run/lifecycle transition  | pass through, always visible         |
| `on-change`  | emit only when the value actually changes  | dedupe by last value                 |
| `on-event`   | emitted when a discrete event occurs       | pass through, count if bursty        |
| `sampled`    | high-frequency, only a sample is useful    | rate-limit (e.g. ≤1/sec or 1/N)      |
| `per-frame`  | genuinely every frame (debug only)         | default-muted; ring buffer if on     |

The two example problems map directly:

* `move intent = (0.0, 0.0)` is `input` / `on-change`. It already only prints on
  change, but under a channel model it is trivially mutable and coalescable.
* `frame dt=...` is `frame` / `sampled`. It should be rate-limited and
  coalesced, not streamed once per second forever.

## 5. Anti-Burial Behaviors

The router applies these behaviors so a long run stays readable:

* **Coalesce consecutive duplicates.** Identical consecutive lines on a channel
  collapse to a single line plus a repeat count, flushed on change or on a timer
  (`… (repeated 240×)`), rather than reprinted.
* **Rate-limit sampled channels.** A `sampled` channel emits at most once per
  window; intermediate values update a rolling summary instead of printing.
* **Bound high-frequency channels.** `per-frame`/`trace` write into a bounded
  ring buffer that is only drained on request (inspector, on error, on exit).
* **Promote rare over frequent.** When sinks are shared, `lifecycle`, `env`,
  `warn`, and `error` are ordered ahead of `frame`/`trace` so the important
  lines are never visually lost between spam.
* **Summaries on exit.** On shutdown, the router can flush a compact summary
  (total frames, cap hits, last/max dt) from counters instead of the stream —
  this is exactly what `RunLoopDiagnostics` already tracks.

## 6. Routing Channels to Sinks

Channels are produced at the call site; a policy table at the runtime/platform
edge maps each channel to sinks and a pacing policy. Sinks are the same ones
described in `diagnostics-model.md` §6 (log sink, WASM console sink, inspector
sink, startup gate, test/capture sink).

```text
Producer writes  →  (channel, cadence, payload)
                        ↓
                 Output Router  (per-channel pacing + coalescing)
                        ↓ fan-out by policy table
      ┌───────────────┼───────────────┬───────────────┐
   Log sink      WASM console     Inspector        Capture sink
 (native tracing) (browser)     (timing/log panel)  (tests)
```

* The policy table — `channel → {sinks, pacing, default_enabled}` — is owned by
  the runtime/platform edge, not by the emitting subsystem.
* Native default: `frame`/`trace` muted, everything else on, `warn`/`error`
  always on. WASM default: same identities, routed through the console/panic
  bridge with the same muting.

## 7. Command-Line Ergonomics

The run surface should let a developer choose channels without editing code.

* **Verbosity levels** map to channel sets, e.g. `quiet` = `lifecycle`+`warn`+
  `error`; `normal` adds `env`+`app`; `verbose` adds `frame`; `trace` adds
  everything.
* **Explicit channel toggles** (enable/disable a named channel) override the
  level, so `--output-channels=+frame,-input` style control is available.
* **Default is not silent and not a firehose.** The default should surface
  lifecycle, env, warnings, and errors, and keep `frame`/`trace` off.
* Native and WASM expose the same channel vocabulary; only the transport differs.

Current prototype syntax:

* `--output-verbosity=quiet|normal|verbose|trace`
* `--output-channels=+channel,-channel,...`
* Environment fallbacks remain `TOKIMU_OUTPUT_VERBOSITY` and
  `TOKIMU_OUTPUT_CHANNELS`.

## 8. What This Fixes, Concretely

Applied to today's code:

* `hello-triangle` backend line → `env` / `one-shot`: always visible, never
  buried, printed once.
* `hello-window` move intent → `input` / `on-change`: coalesced and mutable.
* `hello-window` per-second frame line → `frame` / `sampled`: rate-limited and
  default-muted, with an exit summary from `RunLoopDiagnostics` instead.
* `hello-window` close request → `lifecycle` / `one-shot`: compact shutdown
  summary instead of losing the final state in the stream.
* Frame overrun / cap hit → `warn` / `on-event` (and the future coded
  `TKM-RUNT-0001`): promoted above frame spam, never suppressed by volume.
* Net effect: the recent overflow run would emit a handful of durable lines plus
  a compact exit summary, instead of thousands of near-identical lines.

## 9. Non-Goals

* Not a general logging-framework rewrite. It is a thin channel + pacing layer in
  front of whatever sink (`tracing`, console) is chosen.
* Not diagnostic identity. Codes, severity, and classes stay in
  `diagnostics-model.md`.
* Not a change to simulation. Emitting to a channel is a side effect only; it
  must not mutate world state, advance time, or alter scheduling.
* Not core-owned policy. `tokimu-core` stays neutral; the router lives at the
  runtime/platform edge (or in an example first, then folded up).

## 10. Open Questions

1. Does the router live in `tokimu-runtime`, at the `tokimu-platform` edge, or
   start as an example-local helper and fold up once shape stabilizes?
2. Is channel writing push-based (producer calls a router handle) or is it a
   thin macro/facade over the eventual diagnostic emission API?
3. How do channels compose with the diagnostic `class`/`code` — is `channel`
   derived from `(class, severity)` by default, with explicit override?
4. What is the default coalescing flush policy (on change, on timer, on N)?
5. Where do exit summaries belong — router responsibility, or a runtime
   shutdown hook reading `RunLoopDiagnostics`?
6. Minimum viable channel set for M1/M4: probably `lifecycle`, `env`, `frame`,
   `warn`, `error` only.

## 11. Current Prototype

The first code slice has started in `examples/hello-window`:

* A local `Channel`/`Cadence` vocabulary exists for the example.
* `move intent` output now routes through an `input` channel helper instead of
  raw ad hoc printing.
* The per-second frame line now routes through a `frame` channel helper and is
  muted by default in the example-local router.
* `hello-window` can now explicitly enable or disable a channel, which is the
  first small step toward later verbosity controls.
* `hello-window` now has named verbosity presets (`quiet`, `normal`,
  `verbose`, `trace`) built on the same channel policy table.
* The current prototype now reuses shared startup helpers from
  `tokimu-runtime` for `--output-verbosity` / `TOKIMU_OUTPUT_VERBOSITY` and
  `--output-channels` / `TOKIMU_OUTPUT_CHANNELS`.
* The repeat-count coalescer now lives in `tokimu-runtime`, and `hello-window`
  uses it to collapse consecutive identical lines and flush a repeat count when
  the value changes or on shutdown.
* Sampled `hello-window` output now also emits a periodic repeat summary when
  the sample window advances, so a stable line still produces a bounded recap.
* The shared router policy now also lives in `tokimu-runtime`; the example
  routers are the remaining thin wrapper surface to fold over that runtime-owned
  implementation.
* The close-request path now emits a compact shutdown summary through the
  `lifecycle` channel.
* Frame overrun output now routes through a `warn` channel helper.
* The example now has a compact exit line, which can later be folded into a
  runtime-owned summary sink.

`examples/hello-triangle` has started the same pattern for startup visibility:

* `hello-triangle` now emits a lifecycle line when the native window is created.
* The backend/adapter/device summary now routes through an `env` one-shot line
  instead of a raw `println!`.
* `hello-triangle` now emits a sampled `app` status line with score, target,
  motion phase, and draw calls once the sample window opens.
* The close-request path now emits a compact lifecycle shutdown summary.
* `hello-triangle` now shares the same named verbosity preset vocabulary as
  `hello-window`, and normal verbosity now includes the `app` status channel.
* `hello-triangle` now also uses the shared runtime startup helpers for the
  same flag/env surface.

## 12. Suggested First Increment

Kept small and concrete, matching Tokimu's "one concrete path first" habit:

1. Define a `Channel` enum and a `Cadence` tag in the chosen home (example-local
   helper first is acceptable, then fold into runtime).
2. Add a minimal router that supports three behaviors: pass-through, on-change
   dedupe, and sampled rate-limit + consecutive-duplicate coalescing.
3. Convert `hello-window` prints to channels: `input`/on-change for move intent,
   `frame`/sampled for timing, `warn` for overruns.
4. Default-mute the `frame` channel and print a single compact exit summary from
   `RunLoopDiagnostics` on shutdown.
5. Convert `hello-triangle`'s backend line to `env`/one-shot so it is always the
   first durable, un-buried line.
6. Only after this reads well in practice, fold the channel vocabulary toward the
   diagnostic emission API in `diagnostics-model.md` rather than keeping two
   parallel surfaces.
