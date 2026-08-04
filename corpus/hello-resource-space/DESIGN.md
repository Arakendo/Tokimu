# Hello Resource Space

## Purpose

`hello-resource-space` is a headless consumer-corpus application for the
incubating `resource-space` contract. It validates that an ordinary consumer
can organize logical roots and folders, retain hidden resources, resolve a
small document bundle, exercise format/asset bridges, perform a bounded
recursive search, and capture bounded mutation evidence for independently
identified stores without reaching into the in-memory provider.

## Primary Claim

Two stores may have the same display name and retain equal bytes while still
remaining separate logical stores and separate resource identities. Within one
root, a document bundle retains explicit folders and requires replacement
intent rather than silently overwriting a same-name source resource.

## Boundaries Under Test

- The application supplies stable IDs and chooses provenance labels.
- `resource-space` owns logical hierarchy, address normalization, visibility,
  search, limits, and in-memory retention behavior.
- The consumer receives entries, keys, metadata, summaries, and diagnostics
  only through public APIs.
- A first-party `data.xml` / XSLT / image bundle remains navigable through
  explicit folders, including an empty document folder that has no stored
  bytes beneath it.
- Mutation observation is explicitly enabled for one store, remains disabled
  for its peer, and exposes locally ordered structured outcomes without a
  global event bus.
- No filesystem path, browser handle, importer parser, or asset-loader
  behavior is implied by this corpus.

## Evidence

The executable prints a deterministic observation report, writes a provider
conformance artifact under `target/resource-space-conformance/`, and returns
an error if any identity, search, or visibility assertion fails. The artifact
captures only public semantic facts so a future persistent provider can compare
its behavior without exposing backing collections, paths, or database records.
It is deliberately headless so it remains useful before native and WASM
adapters exist.

## Non-Goals

- filesystem import or export;
- persistence;
- complete XML/XSLT execution;
- proving kernel admission.
