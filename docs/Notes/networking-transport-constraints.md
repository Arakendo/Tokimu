# Networking Transport Constraints

## Status

Recorded 2026-07-27 for Slice 6 of
`docs/Plans/Standalone/networking-and-transport.md`. This is a contract comparison, not a
provider selection and not a dependency decision.

## Result

The incubating `Transport` contract remains viable for native and browser
providers when providers adapt their own callback, stream, or socket mechanisms
to bounded frame queues:

```text
provider callback / socket read
             |
             v
provider-owned bounded receive queue
             |
             v
Transport::try_receive()
```

The contract does not promise that all providers are reliable, ordered, or
backpressure-aware. Those are provider capabilities that later replication
policy must select and diagnose explicitly.

## Contract Comparison

| Mechanism | Delivery shape | Ordering / reliability | Backpressure | Integration finding |
| --- | --- | --- | --- | --- |
| In-memory loopback | immediate bounded queue | FIFO within this provider | explicit queue capacity | current proof provider |
| Native stream or datagram adapter | read readiness or worker callback | selected by concrete socket/channel | adapter must bound its queue | can expose frames through the existing polling seam |
| Browser WebSocket | event callback with text or binary messages | ordered, reliable stream semantics | browser `WebSocket` API has no receive backpressure control | adapter needs an explicit bounded queue and overflow diagnostic |
| Browser WebTransport | promise/stream/callback-oriented streams and datagrams | reliable streams; datagrams may be unordered and unreliable | stream APIs provide flow-control mechanisms; datagram support must be queried | adapter needs readiness, capability, and queue diagnostics |

## Browser Findings

The standard `WebSocket` API is widely available, supports two-way browser
communication, and can carry binary frames. It does not expose backpressure;
an application receiving faster than it can consume can accumulate buffered
work. A future Tokimu WebSocket provider must therefore declare a bounded local
queue and surface overflow rather than silently claiming flow control.

`WebTransport` provides reliable streams plus datagrams. Its `reliability`
capability is initially pending and can resolve to `reliable-only` or
`supports-unreliable`; a Tokimu provider must inspect and report that result.
WebTransport is a secure-context API with incomplete cross-browser availability,
so it cannot be assumed as the first browser provider.

References:

- [MDN WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API)
- [MDN WebTransport API](https://developer.mozilla.org/en-US/docs/Web/API/WebTransport_API)
- [MDN WebTransport reliability](https://developer.mozilla.org/en-US/docs/Web/API/WebTransport/reliability)
- [W3C WebTransport](https://www.w3.org/TR/webtransport/)

## Native Finding

Native sockets, datagram channels, and worker-driven adapters have different
readiness mechanisms, but each can own a bounded `VecDeque`-style receive queue
and expose completed frames through `try_receive`. The native adapter owns
threading or readiness integration; neither application payloads nor
`tokimu-core` need socket types.

No native networking crate is selected by this slice. The loopback proof is
sufficient to validate frame boundaries, bounded queue behavior, and error
ownership before a runnable socket proof is required.

## Explicit Non-Guarantees

- The current `Transport` trait does not promise reliable delivery.
- It does not promise global ordering, only whatever a selected provider and
  replication policy document.
- It does not promise native/browser parity for datagrams.
- It does not select an async runtime or imply one is required.
- It does not make a polling consumer responsible for browser callbacks.

## Follow-Up Trigger

Before selecting WebSocket, WebTransport, or a native socket dependency, add a
runnable provider proof that states its delivery, queue, lifecycle, and
security guarantees. Revisit the trait if that proof cannot adapt to bounded
frame queues without target-specific types leaking into the contract.
