# XML Tools

## Purpose

`xml-tools` is an incubating, parser-neutral XML ingestion boundary for Tokimu
examples and importers. Its first intended consumer is the SVG corpus/import
path.

The crate owns source identity, source spans, resource limits, structured XML
diagnostics, parser-neutral events, and the smallest immutable retained
document needed by importer traversal. It does not own SVG semantics,
rendering, assets, filesystem loading, browser APIs, XSD, XPath, or a
browser-shaped mutable DOM.

## Primary Proof

The initial proof is that importers can receive stable XML diagnostics and
bounded parsing options without depending on a parser implementation's public
types.

```text
source text
    |
    v
xml-tools options and diagnostics
    |
    v
xml-tools parser adapter
    |
    v
parser-neutral XML events
    |
    v
optional immutable XmlDocument
    |
    v
SVG semantic importer
```

## Incubation Status

This crate is example-side infrastructure under `examples/lib-example/`. It
has one named consumer and is not a Tokimu kernel or native capability.
Promotion requires independent consumers and architectural review.

## Current Profile

The current bounded profile uses a private pure-Rust adapter for UTF-8 XML.
It disables DTD processing and external resource resolution, enforces
`XmlLimits`, and returns parser-neutral `XmlDiagnostic` values at the XML
boundary. Element-name, nesting, node, attribute, attribute-value, and decoded
text limits are enforced before consumer adaptation; unterminated and
mismatched elements are XML-boundary diagnostics. Mismatched-end diagnostics
retain their opening-element span as parser-neutral related context.
`XmlDocument` is an immutable event adapter with document-local node handles;
it is not a general DOM.

## Module Ownership

The crate root preserves the importer-facing API while implementation concerns
remain separate:

```text
contracts.rs   source identity, spans, options, and bounded input validation
diagnostic.rs  stable parser-neutral diagnostic contracts
model.rs       expanded names, attributes, and event values
document.rs    immutable retained traversal and document-local handles
parser_support.rs
               private decoding, buffering, event, and diagnostic helpers
parser_names.rs private namespace/name resolution and attribute adaptation
parser_state.rs private nesting, text, event-order, and EOF invariants
parser.rs      private quick-xml event-loop policy
parser_adapters.rs
               byte-source decoding and retained-document entry points
lib.rs         module declarations and public re-exports
```

Parser implementation types remain private to the adapter. The retained
document and public contracts do not depend on `quick-xml`.

The local unit tests follow the same ownership boundaries:

```text
tests/contracts.rs  limits, source identity, and span contracts
tests/parser.rs     event ordering, namespaces, decoding, and hostile input
tests/document.rs   retained traversal, handles, and document construction
```

## Validation Tiers

Normal crate tests cover the local smoke fixtures and parser-neutral document
contracts. The reviewed W3C XML v1 selection remains an explicit test tier:

```text
cargo test -p xml-tools --test w3c_selection -- --ignored --nocapture
```

Its report distinguishes accepted, rejected, unsupported-by-profile, and
deferred cases. It is evidence for the declared profile, not a claim of full
W3C XML conformance.

## Non-Goals

- TTF, SVG, XSD, XPath, DOM mutation, or browser compatibility APIs.
- Filesystem, network, rendering, platform, or engine dependencies.
