# FPS Observation Protocol

The native `hello-fps-web` file bridge writes a JSON `network-tools`
observation envelope to `web/live-frame.json` after each frame. The browser
shell consumes it as read-only presentation data.

## Envelope

```text
protocol_version: 1
schema_id: tokimu.example.fps-frame-snapshot
schema_version: 1
sequence: application frame number
message_kind: observation_snapshot
payload: UTF-8 JSON bytes for the application-owned frame snapshot
```

The envelope is a bounded local bridge fixture. It is not a universal FPS,
browser, filesystem, or networking schema.

## Ownership

- Rust owns FPS simulation truth and produces the snapshot.
- `network-tools` owns envelope framing and validation only.
- The file provider owns local byte movement.
- TypeScript validates and presents observations; it does not mutate simulation.

The direct WASM callback remains a browser-local presentation bridge and is
not evidence for a transport contract.
