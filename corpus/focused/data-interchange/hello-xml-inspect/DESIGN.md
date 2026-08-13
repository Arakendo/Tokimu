# Hello XML Inspect

## Purpose

`hello-xml-inspect` is an independent consumer of the incubating `xml-tools`
document and diagnostic contracts. It proves that a non-SVG caller can parse
bounded XML and inspect retained document structure without parser-backend
types leaking through the public API.

## Ownership

```text
Application
    owns command-line arguments, file input, and terminal presentation

xml-tools
    owns bounded XML parsing, retained document semantics, and diagnostics

Parser backend
    remains private to xml-tools
```

The example intentionally does not depend on rendering, browser, runtime, or
engine state. File access stays in the application; `xml-tools` only receives
UTF-8 source text and explicit source/document identities.

## Run

```text
cargo run -p hello-xml-inspect
cargo run -p hello-xml-inspect -- path/to/document.xml
```

The default command inspects the checked-in sample fixture. Output is stable
enough to make document order, expanded names, attributes, text, comments,
processing instructions, and source spans visible during review.

## Corpus Evidence

This is the second XML consumer required by Slice 6 of
`docs/Plans/Standalone/xml-tools.md`. It is not evidence that `xml-tools` should be
promoted yet. The architectural review remains responsible for deciding
whether independent usage stabilizes a capability boundary.
