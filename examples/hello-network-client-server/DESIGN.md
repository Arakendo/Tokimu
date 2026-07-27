# Hello Network Client Server

## Purpose

`hello-network-client-server` is a headless corpus example for an
authoritative fixed-step server and two simulated clients.

It proves that applications can keep simulation truth on the server while
moving application-defined client intent and server observations through the
incubating `network-tools` boundary.

## Primary Proof

```text
client-owned input
        |
        v
bounded client_input envelope
        |
        v
loopback transport
        |
        v
server validation and fixed-step simulation
        |
        v
bounded observation_snapshot envelope
        |
        v
client presentation observation
```

## What This Proves

- Client input is application-defined and has an explicit message kind.
- The server validates input before applying it to authoritative state.
- The server advances deterministic integer state at a fixed step.
- Clients consume sequenced observations without receiving mutable server
  state.
- Inputs and observations use independent schemas and opposite transport
  directions.

## Non-Goals

- Sockets, WebSockets, or browser transport providers.
- Authentication, sessions, ownership transfer, or anti-cheat policy.
- Prediction, rollback, reconciliation, or interpolation.
- Remote `World` mutation.
- First-party `tokimu-net` admission.

This example is evidence for AR-0004, not a multiplayer framework.
