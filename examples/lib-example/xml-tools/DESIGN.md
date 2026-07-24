# XML Tools

## Purpose

`xml-tools` is an incubating, parser-neutral XML ingestion boundary for Tokimu
examples and importers. Its first intended consumer is the SVG corpus/import
path.

The crate owns source identity, source spans, resource limits, and structured
XML diagnostics. It does not own SVG semantics, rendering, assets, filesystem
loading, browser APIs, XSD, XPath, or a browser-shaped mutable DOM.

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
future parser adapter
    |
    v
parser-neutral XML events
    |
    v
SVG semantic importer
```

## Incubation Status

This crate is example-side infrastructure under `examples/lib-example/`. It
has one named consumer and is not a Tokimu kernel or native capability.
Promotion requires independent consumers and architectural review.

## Current Profile

The current slice establishes only contracts. A later parser adapter must use
UTF-8 input, disable external entity/resource resolution, enforce `XmlLimits`,
and return `XmlDiagnostic` values at the XML boundary.

## Non-Goals

- XML parsing in the diagnostic-core slice.
- TTF, SVG, XSD, XPath, DOM mutation, or browser compatibility APIs.
- Filesystem, network, rendering, platform, or engine dependencies.
