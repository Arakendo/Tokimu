# Hello Network Loopback

## Purpose

`hello-network-loopback` is a headless corpus example proving that an
application-owned observation can cross a provider-neutral framed-byte
boundary without granting remote mutation authority.

## Primary Proof

```text
application observation
        |
        v
bounded JSON payload
        |
        v
versioned replication envelope
        |
        v
bounded in-memory loopback
        |
        v
decode and application comparison
```

The example prints the selected provider, protocol version, schema identity,
sequence, byte count, and sequence-policy decision. It also demonstrates a
malformed-frame diagnostic.

## Sequence Policy

The receiver owns a tracker for one observation stream:

- first and immediately next observations are accepted;
- gaps are accepted and diagnosed;
- duplicates are ignored and diagnosed;
- lower late arrivals are ignored as stale or out-of-order;
- no buffering, retransmission, reliability, or recovery is implied.

A monotonic sequence value alone cannot distinguish a lost frame from one that
has not arrived yet. The example records that ambiguity instead of claiming a
transport guarantee.

## Ownership

- The example owns `PlayerObservation`.
- `network-tools` owns envelope, codec, and transport proof contracts.
- The loopback provider owns byte movement only.
- The receiver validates an owned value and never receives `&mut World`.

## Non-Goals

- sockets or browser APIs;
- async execution;
- authoritative replication;
- world serialization;
- prediction, rollback, or reconciliation;
- admitting a first-party `tokimu-net` crate.
