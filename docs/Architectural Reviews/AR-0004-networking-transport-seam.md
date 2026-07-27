# AR-0004: Networking Transport Seam

| Field | Value |
| --- | --- |
| Status | Deferred |
| Opened | 2026-07-27 |
| Last reviewed | 2026-07-27 |
| Scope | Capability boundary and transport-provider incubation |
| Trigger | M11 loopback proof, FPS browser bridge, and authoritative client/server corpus now share bounded envelope and framed-byte transport semantics |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005 |
| Related evidence | `network-tools` tests, `hello-network-loopback`, `hello-network-client-server`, `hello-fps-web`, transport constraints note |
| Admission exception | None |

## Architectural Question

Should the incubating observation envelope, codec, sequence policy, and
framed-byte transport seam graduate from `examples/lib-example/network-tools`
into a first-party `tokimu-net` capability, or remain example-side until a
real application or provider proves the missing lifecycle and authority
semantics?

## Context

M11 required a narrow networking proof without admitting sockets, browser APIs,
or remote world mutation into `tokimu-core`. The completed proof now has two
callers:

```text
hello-network-loopback
    application observation -> envelope -> codec -> bounded loopback -> decode

hello-fps-web
    application frame snapshot -> envelope -> native file bridge -> browser validation
```

The shared layer owns protocol version, schema identity, message kind, bounded
payload/frame validation, byte encoding, sequence diagnostics, and a bounded
loopback provider. Applications retain payload meaning. The native FPS file
bridge remains an explicit local example provider, not a socket implementation
or general networking contract.

## Trigger And Evidence

- Corpus examples: `hello-network-loopback` proves read-only observations,
  malformed-frame diagnostics, and duplicate/stale/gap policy without a window,
  filesystem, or network. `hello-fps-web` proves the same envelope can carry an
  independent browser-facing application snapshot through a local bridge.
- Automated tests: `network-tools` covers exact codec round trips, malformed
  bytes, protocol/schema/message-kind rejection, bounds, FIFO behavior, queue
  closure, injected provider failures, and deterministic sequence decisions.
  `hello-fps-web` round-trips its own application snapshot through the neutral
  envelope and loopback provider.
- Target evidence: `hello-fps-web` and `network-tools` compile for native and
  `wasm32-unknown-unknown`; strict TypeScript checking validates the browser
  envelope and snapshot contract.
- Constraint evidence: WebSocket, WebTransport, and native queue adaptation
  were compared at the contract level in
  `docs/Notes/networking-transport-constraints.md`.
- Independent consumers: two example callers share the same envelope/codec
  contract while owning different application schemas and destinations.
- Missing evidence: a real socket or browser transport provider, session
  lifecycle, reconnect behavior, authentication/security policy, authority,
  inbound application integration, and a non-example application consumer.

## Ownership Analysis

The shared meaning is a bounded, versioned observation envelope and movement of
framed bytes through a provider boundary. It is currently example-side
incubation support.

- Applications own snapshot fields and their interpretation.
- `network-tools` owns envelope validation, codec behavior, ordered-observation
  diagnostics, and provider-neutral transport mechanics for the proof.
- Providers own concrete queues, files, browser callbacks, sockets, or other
  byte movement mechanisms.
- Runtime/application code would own any future authority decision and explicit
  lifecycle point where an inbound observation is consumed.

The seam must not own `World` serialization, remote mutable access, session
authority, renderer resources, platform handles, a concrete async runtime, or
browser/native transport objects.

The observed resemblance to publishing, replay, and diagnostic artifact
pipelines is not evidence for a generic envelope or movement capability.
Those domains have unproven differences in durability, retry, authority,
atomicity, provenance, and failure recovery.

## Dependency Direction

```text
Current:
application snapshots --> network-tools contracts --> loopback/file providers
browser TypeScript ----> validated envelope ----------> presentation only

Deferred future capability:
application/runtime policy --> tokimu-net semantics --> replaceable providers
```

Neither shape permits `tokimu-core` to depend on codecs, transports, browser
types, socket types, filesystem providers, or application payload schemas.

## Alternatives Considered

### A: Extract `tokimu-net` Now

- Benefits: visible package location and an early public seam.
- Costs: freezes session, provider, authority, and lifecycle questions before a
  real network consumer exists.
- Failure mode: example support becomes a permanent networking subsystem with
  no proof that its small polling trait is the right public API.

### B: Keep Transport Logic Inside Each Example

- Benefits: avoids a provisional shared library.
- Costs: duplicates framing, bounded diagnostics, and sequence behavior between
  otherwise unrelated examples.
- Failure mode: file, browser, and future socket paths drift into incompatible
  observation semantics.

### C: Continue Example-Side Incubation

- Benefits: preserves the two-caller seam while making extraction reversible.
- Costs: `network-tools` remains a support-library location and must not quietly
  expand into session or authority policy.
- Failure mode: a later provider starts leaking target-specific behavior into
  its public contracts without renewed review.

## Findings

The evidence supports continued shared incubation. The two callers independently
need the same bounded envelope, codec, and sequence diagnostics, while their
payload meaning remains separate. The native and browser-target compilation
evidence supports a provider-neutral framed-byte contract. The browser review
also establishes that callback-driven providers can adapt to `try_receive`
through bounded provider-owned queues.

The evidence does not support first-party capability admission. Both callers
are examples, no real socket/browser provider is runnable, no session or
authority model is defined, and the current synchronous polling seam has not
yet met real native readiness or browser callback pressure. The evidence also
does not support a generic envelope or movement abstraction.

## Disposition

Deferred. Keep `network-tools` in `examples/lib-example` as a focused,
provider-neutral incubation seam. Do not create `tokimu-net`, select a socket
library, add an async runtime, or generalize networking/publishing/replay into
a movement capability based on this evidence alone.

## Consequences

Examples may reuse the bounded observation seam without duplicating codec or
sequence behavior. New providers must expose explicit queue, delivery,
lifecycle, and capability diagnostics, and must not let concrete mechanism
types enter application payload contracts. The M11 architecture spike is
complete; networking remains a deferred capability-admission question rather
than an engine-core dependency.

## Required Follow-Up

- [x] Documentation or review record
- [x] Focused implementation slice
- [x] Corpus example or automated test
- [ ] Migration, retirement, or compatibility work

## Reopening Triggers

- a real socket, WebSocket, WebTransport, or browser-host provider needs the
  same transport contract and exposes a mismatch;
- a non-example application requires envelope, session, or authority semantics;
- inbound observations need an explicit runtime lifecycle boundary;
- a provider cannot adapt to bounded framed queues without target-specific
  types leaking upward;
- another domain proves the exact same envelope guarantees and justifies a
  separate cross-domain Architectural Review;
- corpus evidence invalidates the current sequence, bounds, or diagnostic
  contracts.

## Review History

### Cycle 2 -- 2026-07-27

- Status entering review: Deferred
- New evidence: `hello-network-client-server` carries application-defined
  `client_input` messages from two simulated clients to a fixed-step,
  authoritative server. The server validates client identity, tick, sequence,
  and bounded movement before applying any local state change, then publishes
  sequenced observations back to both clients.
- Findings: message direction and application validation are now concrete
  evidence. They do not yet establish session lifecycle, remote authority,
  reliability, or a real provider contract.
- Disposition: Remain deferred. The corpus extends example-side evidence but
  does not justify `tokimu-net` admission.

### Cycle 1 -- 2026-07-27

- Status entering review: Proposed
- New evidence: loopback envelope/codec/sequence proof, independent FPS browser
  snapshot consumer, native and WASM compilation, strict TypeScript envelope
  validation, and browser/native constraint comparison.
- Participants or reviewers: Codex working review
- Findings: a shared example-side seam is useful; extraction and generic
  movement admission are premature.
- Disposition: Deferred
- Resulting ADR or documentation change: no ADR change; M11 is recorded as a
  completed architecture spike with explicit reopening triggers.

## References

- `docs/Plans/networking-and-transport.md`
- `docs/Notes/networking-transport-baseline.md`
- `docs/Notes/networking-transport-constraints.md`
- `examples/lib-example/network-tools/`
- `examples/hello-network-loopback/`
- `examples/hello-network-client-server/`
- `examples/hello-fps-web/OBSERVATION-PROTOCOL.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
