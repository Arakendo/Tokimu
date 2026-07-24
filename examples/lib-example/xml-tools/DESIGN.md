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
