# Networking and Transport Spike

## Status

Complete. M6.5 documented the ownership boundary; M11 now has one concrete
replication unit, one provider-neutral transport seam, one byte-level
round-trip proof, ordered delivery policy, and a second browser-facing caller.
AR-0004 records the resulting decision: retain `network-tools` as example-side
incubation and defer first-party networking admission until a real provider or
non-example consumer supplies the missing lifecycle and authority evidence.
The completed spike also has follow-on `hello-network-client-server` evidence:
application-defined client input is validated by an authoritative example
server before it changes local simulation state.

## Purpose

Prove that Tokimu can move meaningful application observations across a
transport boundary without making sockets, wire formats, browser APIs, or
remote state mutation part of `tokimu-core`.

The first proof uses a **versioned application-owned observation snapshot** as
the replication unit:

```text
application-owned observation snapshot
                |
                v
Tokimu-owned replication envelope
  schema version
  sequence
  message kind
  payload bytes
                |
                v
provider-neutral transport seam
                |
                v
in-memory loopback mechanism
                |
                v
decode, validate, and deliver to an application-owned consumer
```

This is the first replication proof, not the final networking model.
`hello-network-client-server` adds a narrowly scoped `client_input` message to
prove local server-side validation before simulation changes. Events, state
deltas, authoritative correction, prediction, rollback, and unreliable
delivery remain future message kinds or separate review questions.

## Motivation And Existing Evidence

The roadmap names M11 as the current short-term focus:

1. name the first replication unit;
2. define a transport seam compatible with native and browser hosts;
3. serialize one message to bytes and back.

`hello-fps-web` already publishes a Rust-owned `FpsFrameSnapshot` to a
TypeScript browser shell. That is useful evidence for an observation snapshot,
but its current file/JSON bridge is example-specific and must not become a
universal networking schema.

An observation snapshot is the narrowest honest first unit because:

- a concrete caller and browser consumer already exist;
- it is read-only and does not grant remote mutation authority;
- it does not require reflection or arbitrary `World` serialization;
- it does not require a command registry that Tokimu has not yet stabilized;
- it can exercise schema versioning, sequencing, codecs, framing, transport,
  diagnostics, and native/browser constraints independently.

## Goals

- Name the first replication unit precisely.
- Keep application payload semantics application-owned.
- Define a small Tokimu-owned envelope and framed-byte transport boundary.
- Keep codec selection independent from transport selection.
- Prove exact encode/send/receive/decode behavior through loopback.
- Make unsupported versions, malformed frames, and transport failures explicit.
- Preserve a plausible implementation path for native and browser transports.
- Keep simulation truth and mutation authority local to the world/runtime
  boundary.

## Non-Goals

- Real sockets, WebSockets, WebTransport, UDP, QUIC, or HTTP clients.
- Full multiplayer gameplay.
- Raw `World` serialization.
- Replicating renderer resources or platform handles.
- State deltas, rollback, prediction, interpolation, or reconciliation.
- Lockstep determinism.
- Authentication, encryption, compression, or congestion control.
- Reliable and unreliable channel implementation.
- Remote command authority.
- Adding `tokimu-net` before the seam has independent evidence.
- Creating a universal envelope or generic movement capability from analogy
  alone.
- Unifying networking, publishing, replay, file export, clipboard, or shared
  memory before independent consumers prove the same semantic contract.
- Introducing an async runtime.

## First Replication Unit

The first replication unit is:

> A versioned, sequenced observation snapshot whose payload is defined by the
> application and whose envelope is transport- and codec-neutral.

The snapshot is an observation of selected application state. It is not:

- a complete world snapshot;
- an authoritative state transfer;
- a renderer frame;
- a serialization of ECS storage;
- permission for the receiver to mutate the sender's world.

The initial proof payload should contain a small stable subset similar to:

```text
frame or tick identity
player position/orientation
score or status
small bounded entity counts
```

The exact payload belongs to the proof application. The shared boundary owns
only the envelope, byte framing, validation, and diagnostics required to move
it.

## Candidate Contracts

The first implementation may refine these names, but it should preserve the
separation:

```rust
pub struct ReplicationEnvelope {
    pub protocol_version: u16,
    pub schema_id: String,
    pub sequence: u64,
    pub message_kind: MessageKind,
    pub payload: Vec<u8>,
}

pub enum MessageKind {
    ObservationSnapshot,
}

pub trait Transport {
    fn send(&mut self, frame: &[u8]) -> Result<(), TransportError>;
    fn try_receive(&mut self) -> Result<Option<Vec<u8>>, TransportError>;
}
```

These are candidate shapes, not pre-approved stable APIs.

Important constraints:

- transport sees framed bytes, not application structs;
- codecs encode/decode envelopes and payloads but do not perform I/O;
- application code converts decoded payloads into application meaning;
- receiving bytes never grants direct `&mut World` access;
- sequence numbers identify order but do not silently promise reliable
  delivery;
- schema and protocol versions are distinct so payload evolution does not
  require redefining transport.

## Ownership And Dependency Boundaries

### `tokimu-core`

Must not gain socket, browser, codec, wire-format, or protocol dependencies.
No networking implementation belongs here during this plan.

If later evidence proves a tiny universal message identity or command meaning,
that requires separate admission review. The first example-side proof does not
make that decision.

### `tokimu-runtime`

May eventually coordinate inbound delivery, outbound observation cadence,
authority, and ordered application at lifecycle boundaries. The first proof
does not require runtime integration.

### Future `tokimu-net`

Would own Tokimu networking and replication semantics such as sessions,
channels, framing, replication policy, authority, and delivery diagnostics.
It is not created until this plan proves a stable seam and a second caller
justifies extraction.

### Transport providers

Own concrete mechanisms such as loopback queues, WebSockets, native sockets,
or browser APIs. Providers do not define replication meaning.

### Codecs

Own conversion between typed envelopes/payloads and bytes. JSON may be used for
the first browser-readable proof, but JSON must not become inseparable from the
transport contract.

### Applications

Own observation payload schemas and the decision about how decoded
observations are presented or consumed.

## Dependency Direction

```text
application snapshot schema
          |
          v
replication envelope + codec
          |
          v
framed bytes
          |
          v
transport contract
          |
          v
loopback / native / browser provider
```

No dependency may point from transport providers back into application
semantics or from `tokimu-core` into networking.

## Emerging Movement Pattern

This plan may expose a broader pattern already suggested by publishing and
diagnostic artifact work:

```text
semantic object
      |
      v
canonical representation
      |
      v
envelope or manifest
      |
      v
byte representation
      |
      v
movement provider
      |
      v
destination
```

Possible destinations include loopback queues, sockets, browser transports,
files, packages, replay logs, clipboards, and shared memory. The resemblance is
architecturally interesting, but it is not yet evidence for a universal
`Envelope`, `Movement`, or distribution capability.

For this networking proof:

- the application owns observation meaning;
- the replication layer owns protocol framing, version, sequence, and
  bounded-message diagnostics;
- the codec owns conversion to and from bytes;
- the transport provider owns movement mechanics;
- the destination application owns interpretation and application of the
  received observation.

Publishing, replay, and diagnostics may require different guarantees for
identity, ordering, durability, authority, retries, atomicity, provenance, and
failure recovery. They should remain separate implementations until at least
two domains independently require the same provider-neutral contract.

If that convergence occurs, open a dedicated Architectural Review. The review
should ask whether the shared result is:

1. only a reusable design pattern;
2. a small canonical envelope contract;
3. a provider-neutral movement capability; or
4. separate sibling capabilities with deliberately parallel shapes.

## Implementation Location

Incubate the first reusable seam under:

```text
examples/lib-example/network-tools/
```

Exercise it with:

```text
examples/hello-network-loopback/
```

After the loopback proof is stable, adapt `hello-fps-web` as the second caller.
Do not create `tokimu-net` solely to complete this plan.

## Implementation Slices

### Slice 0: Record The Boundary And Baseline

Deliverables:

- [x] Record the existing `hello-fps-web` snapshot fields, serialization path,
  cadence, and browser consumption behavior.
- [x] Identify which fields are simulation observations and which are
  presentation-only.
- [x] Record native and browser constraints without selecting a real network
  library.
- [x] Confirm the first unit as an application-owned observation snapshot.

Acceptance criteria:

- [x] The baseline identifies one bounded payload with no renderer or platform
  handles.
- [x] The document distinguishes observation, replication, transport, codec,
  and application behavior.
- [x] No real socket or browser mechanism is needed to complete the baseline.

### Slice 1: Add The Envelope And Codec

Deliverables:

- [x] Create `examples/lib-example/network-tools`.
- [x] Define protocol version, schema identity, sequence, message kind, and
  bounded payload.
- [x] Add one explicit codec for the proof.
- [x] Add maximum frame and payload limits.
- [x] Add structured errors for malformed data, unsupported versions, unknown
  message kinds, and exceeded limits.

Acceptance criteria:

- [x] One observation payload encodes and decodes exactly.
- [x] Unsupported protocol and schema versions fail explicitly.
- [x] Truncated, oversized, and malformed frames cannot panic.
- [x] Codec types do not perform I/O or depend on platform APIs.
- [x] Application payload fields do not enter the transport trait.

### Slice 2: Add In-Memory Loopback Transport

Deliverables:

- [x] Implement a bounded in-memory loopback provider.
- [x] Preserve frame boundaries.
- [x] Define empty receive behavior.
- [x] Define queue-full, closed, and injected-failure behavior.
- [x] Keep loopback selection explicit in diagnostics.

Acceptance criteria:

- [x] A framed envelope survives send and receive byte-for-byte.
- [x] Multiple frames preserve documented queue order.
- [x] Queue limits and closure produce structured transport errors.
- [x] The provider knows nothing about snapshots or application schemas.
- [x] Repeated setup and shutdown leave no hidden global state.

### Slice 3: Build `hello-network-loopback`

Deliverables:

- [x] Create a focused example with one sender and one receiver.
- [x] Produce one observation snapshot from application-owned state.
- [x] Encode, send, receive, decode, and compare it.
- [x] Print protocol, schema, sequence, byte count, and selected provider.
- [x] Demonstrate one malformed or unsupported frame diagnostic.

Acceptance criteria:

- [x] The example completes one end-to-end round trip.
- [x] The received snapshot exactly matches the sent semantic values.
- [x] No renderer, window, filesystem, or live network is required.
- [x] The receiver does not mutate a `World`.
- [x] Failure output identifies the owning stage: encode, transport, decode, or
  application validation.

### Slice 4: Prove Ordered Observation Delivery

Deliverables:

- [x] Send a bounded sequence of snapshots.
- [x] Detect duplicate, stale, skipped, and out-of-order sequence numbers.
- [x] Keep sequence policy separate from transport mechanics.
- [x] Record whether each condition is accepted, ignored, or diagnosed.

Acceptance criteria:

- [x] Ordered input produces ordered application observations.
- [x] Every abnormal sequence condition has deterministic behavior.
- [x] Sequence handling does not imply reliable delivery guarantees that the
  transport contract does not make.
- [x] Diagnostics identify schema and sequence without dumping unbounded
  payload data.

### Slice 5: Adapt `hello-fps-web`

Deliverables:

- [x] Replace duplicate envelope/framing logic with `network-tools`.
- [x] Keep `FpsFrameSnapshot` application-owned.
- [x] Keep the current file/browser bridge as an explicit example provider or
  replace it with an equally bounded local bridge.
- [x] Generate or validate the TypeScript snapshot contract from one recorded
  schema boundary, without moving simulation to TypeScript.
- [x] Preserve the existing browser presentation behavior.

Acceptance criteria:

- [x] Rust remains the owner of FPS simulation truth.
- [x] Browser TypeScript consumes observations and does not become an
  authoritative simulation.
- [x] The same encoded envelope can pass through loopback and the browser bridge.
- [x] The provider can change without changing `FpsFrameSnapshot`.
- [x] Browser unavailability does not alter native simulation behavior.

### Slice 6: Review Native And Browser Transport Constraints

Deliverables:

- [x] Compare WebSocket, WebTransport, and one native-capable mechanism at the
  contract level.
- [x] Record push/callback versus poll/queue adaptation.
- [x] Record reliable/unreliable, binary/text, backpressure, lifecycle, and
  security-context differences.
- [x] Decide whether the candidate `Transport` trait survives both target
  shapes.
- [x] Avoid adding a real dependency unless one mechanism is needed by a
  runnable proof.

Acceptance criteria:

- [x] The semantic contract can be implemented by both native and browser
  adapters without target-specific types leaking upward.
- [x] Unsupported target features are discoverable and diagnostic.
- [x] The comparison does not claim unreliable delivery where the selected
  browser mechanism cannot provide it.
- [x] No browser-specific runtime fork is introduced.

### Slice 7: Admission Review

Deliverables:

- [x] Record whether envelope, codec, sequence, and transport meanings are
  genuinely shared by two callers.
- [x] Record what remains application-specific.
- [x] Decide whether `network-tools` remains example-side or justifies a
  first-party networking capability.
- [x] Compare the observed networking pipeline with publishing, replay, file,
  and diagnostic artifact pipelines without assuming they share an
  implementation.
- [x] Record whether any apparent movement/envelope convergence is contractual
  evidence or only structural similarity.
- [x] Open an Architectural Review before creating `tokimu-net`.
- [x] Open a separate Architectural Review before admitting a generic envelope
  or movement capability.
- [x] Update the roadmap and SDD with observed behavior only.

Acceptance criteria:

- [x] Every proposed public contract has at least two concrete callers.
- [x] No codec or transport provider is mistaken for replication semantics.
- [x] The review names authority, mutation, and delivery questions still
  deferred.
- [x] Any cross-domain movement finding names at least two independent
  consumers and the exact guarantees they share.
- [x] Crate extraction is a separate deliberate decision.

## Failure Semantics

Failures must identify their owning boundary:

| Boundary | Example failure |
| --- | --- |
| Application snapshot | invalid or unbounded field value |
| Envelope | unsupported protocol or message kind |
| Codec | malformed or truncated bytes |
| Transport | queue full, closed, unavailable, send/receive failure |
| Sequence policy | duplicate, stale, skipped, or out-of-order frame |
| Application consumer | unsupported schema or rejected observation |

No layer may silently reinterpret malformed input as a valid empty snapshot.
Diagnostics should carry bounded message identity and sequence information.

## Validation

For implementation slices, run:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Focused tests should cover:

- exact encode/decode round trips;
- malformed and oversized frames;
- version and schema rejection;
- loopback ordering and queue limits;
- duplicate, stale, skipped, and out-of-order sequences;
- provider-independent payload behavior;
- repeated initialization and shutdown;
- native/WASM compilation where dependencies permit it.

## Risks

### Observation Snapshot Becomes Universal World State

Mitigation: keep payload schemas application-owned and prohibit arbitrary
`World` serialization in the first proof.

### File Bridge Is Mistaken For Networking

Mitigation: label it as an example provider. The semantic proof is framed-byte
transport independence, not the filesystem mechanism.

### Synchronous Trait Blocks Browser Integration

Mitigation: model browser callbacks as a provider-owned bounded receive queue
and reassess the trait in Slice 6 before stabilization.

### Codec Becomes The Protocol

Mitigation: separate typed meaning, envelope metadata, and byte encoding. Use
one codec concretely without promising it as the only wire format.

### Receive Path Gains Ambient World Mutation

Mitigation: decode into owned results and require application/runtime code to
apply meaning at an explicit lifecycle boundary.

### Premature `tokimu-net`

Mitigation: incubate in example support and require a second caller plus
Architectural Review before extraction.

### Structural Symmetry Becomes Premature Unification

Networking, publishing, replay, and artifact export may all resemble
`meaning -> representation -> bytes -> provider`. Similar diagrams do not prove
identical ownership, lifecycle, authority, durability, or failure semantics.

Mitigation: record the pattern, keep implementations separate, and require
independent consumers plus a dedicated Architectural Review before introducing
a generic envelope or movement capability.

## Completion Criteria

The M11 spike is complete for this plan when:

- the first replication unit is explicitly named as a versioned
  application-owned observation snapshot;
- one payload round-trips through codec, envelope, and loopback transport;
- ordering and malformed-input behavior are explicit and tested;
- transport sees only framed bytes;
- `hello-network-loopback` provides a focused runnable proof;
- `hello-fps-web` provides a second browser-facing caller;
- native and browser constraints are reviewed against the same semantic seam;
- no networking dependency enters `tokimu-core`;
- an Architectural Review decides whether any boundary should become
  first-party.

## Graduation Trigger

Consider a first-party networking capability only when:

- two independent callers need the same envelope and transport semantics;
- provider and codec types remain absent from application payload contracts;
- authority and world-application boundaries remain explicit;
- native and browser adapters can implement the same Tokimu-owned contract;
- extracting the capability removes real duplication or enables a real
  application rather than merely improving package organization.

The networking graduation decision does not admit a generic movement
capability. That broader decision requires separate cross-domain evidence and
Architectural Review.

## References

- `docs/roadmap.md`
- `docs/Tokimu Software Design Document.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/Conversations/multithreading.md`
- `examples/hello-fps-web/DESIGN.md`
- `examples/hello-fps-web/src/main.rs`
- `examples/hello-fps-web/web/src/protocol.ts`
