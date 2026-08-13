# Hello Compression

## Purpose

`hello-compression` is a focused, headless consumer of the incubating byte
compression contract. It proves that application-owned bytes can cross
provider-neutral GZip, raw Deflate, and raw Brotli boundaries without gaining
filesystem or archive semantics. A second scene explicitly composes Brotli
with Resource Space through the separate transformation bridge.

## Primary Proof

```text
application bytes + semantic goal
    -> provider-neutral encode request
    -> selected codec provider
    -> bounded provider-neutral decode request
    -> byte-identical application result + observations
```

The corpus runs every admitted codec under `Fast`, `Balanced`, and `Small`,
then deliberately rejects a high-expansion decode through the same public
contract. It also retains source, encoded, and decoded logical resources,
proving that ordinary source reads stay byte-faithful and destinations remain
explicit. It writes `target/hello-compression/report.json` as structural
evidence. The report records portable codec, goal, and byte observations; it
does not expose provider-native quality levels or stream objects.

## Non-Goals

- Archive entries, manifests, ZIP, TAR, or archive-backed views.
- Filesystem helpers, backups, or transparent Resource Space transforms.
- Benchmark claims or provider comparisons.
- Runtime cancellation or browser execution evidence.
