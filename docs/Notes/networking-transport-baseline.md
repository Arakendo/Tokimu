# Networking And Transport Baseline

## Status

Recorded 2026-07-27 for Slice 0 of
`docs/Plans/Standalone/networking-and-transport.md`.

## Existing Caller

`hello-fps-web` publishes one `NativeFrameSnapshot` after each native
application frame. Rust serializes the snapshot as camelCase JSON and writes
it to `web/live-frame.json`. The browser shell polls that file, decodes the
matching `FpsFrameSnapshot`, and presents it through the HUD.

The file bridge is example-specific. It is evidence for a read-only observation
boundary, not evidence that files, polling, JSON, or the current snapshot shape
are universal networking contracts.

## Existing Snapshot Fields

| Field | Ownership | Notes |
| --- | --- | --- |
| `frame` | simulation observation | monotonically advancing frame identity |
| `elapsedSeconds` | runtime observation | presentation timing, not authoritative simulation time |
| `player.{x,y,z}` | simulation observation | player/camera position |
| `player.{yaw,pitch}` | simulation observation | player/camera orientation |
| `hud.score` | simulation observation | game state presented by the HUD |
| `hud.wave` | simulation observation | game state presented by the HUD |
| `hud.targets` | simulation observation | bounded active-entity count |
| `hud.projectiles` | simulation observation | bounded active-entity count |
| `hud.status` | presentation-facing observation | currently an unbounded field value; the envelope enforces a payload-wide bound, while field-level limits remain deferred |

The payload contains no renderer resources, platform handles, ECS storage, or
mutable world access.

## Cadence And Consumption

- Native Rust publishes after each application frame.
- The native provider rewrites one JSON file.
- The browser polls independently and may skip intermediate frames.
- The browser treats the snapshot as read-only presentation state.
- Delivery does not grant authority to mutate Rust simulation state.
- The current provider does not promise reliable delivery, ordered observation
  of every frame, backpressure, or remote connectivity.

## First Replication Unit

The first replication unit is a versioned, sequenced, application-owned
observation snapshot carried by a transport- and codec-neutral envelope.

The first loopback proof will use a smaller bounded payload than
`FpsFrameSnapshot`. It will preserve the same ownership boundary while
isolating:

```text
application observation
        |
        v
payload codec
        |
        v
replication envelope
        |
        v
envelope codec
        |
        v
bounded loopback transport
        |
        v
application validation
```

## Native And Browser Constraints

- Native providers may use blocking or non-blocking sockets, files, or queues.
- Browser providers are callback/promise driven and commonly expose
  WebSocket, WebTransport, or host bridges.
- A provider-neutral boundary must not expose native socket or browser DOM
  types.
- A polling API may adapt browser callbacks through a bounded provider-owned
  receive queue, but that remains a later contract review.
- The baseline does not select a real network library or async runtime.

## Boundary Findings

- Applications own payload meaning.
- The replication seam owns bounded envelope metadata and diagnostics.
- Codecs own typed-data-to-bytes transformations and perform no I/O.
- Transports own framed-byte movement and know no application schema.
- Application/runtime code owns the explicit lifecycle point where a decoded
  observation is consumed.
- Receiving an observation does not imply remote authority or world mutation.
