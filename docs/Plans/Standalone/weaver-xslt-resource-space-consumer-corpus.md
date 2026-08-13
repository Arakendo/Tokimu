# Weaver XSLT Resource Space Consumer Corpus

## Status

In progress. Weaver is pinned as `third-party/weaver-xslt` at
`e7472c6ae2894345f59ed38da38816092af34fea`. The initial source-buffer
baseline passes through Weaver's interpreter and auto/native path without
making Tokimu own TypeScript, XSLT, URI syntax, or host I/O.

## Purpose

Prove, in small reversible slices, whether a TypeScript XSLT host can consume
Tokimu Resource Space selections through a bounded adapter.

This is a consumer corpus, not an engine dependency. `tokimu-core`,
`tokimu-runtime`, and the Resource Space semantic contract must remain free of
Weaver, Node, npm, XSLT, XPath, DOM/XDM, and URI-policy dependencies.

## Ownership

```text
Tokimu Resource Space
    selected resource identity, folders, visibility, retained bytes

Weaver
    XSLT/XPath semantics, base URI, relative URI resolution, source identity

Consumer adapter
    selected-session mapping and structured error translation

TypeScript host
    fixture loading, allowed schemes, controls, and publication destination
```

The consumer adapter may map a Weaver-resolved identity to a selected Tokimu
resource. It must not reinterpret display names as identity, make Resource
Space parse arbitrary URIs, or grant access outside the selected session.

## Fixture Contract

`corpus/consumers/weaver-xslt-resource-space/fixtures/` contains:

- `source.xml`: the selected XML input;
- `stylesheet.xsl`: a source-buffer stylesheet transform;
- `related.xml`: a selected sibling reserved for future resolver evidence;
- `expected.xml`: the expected transform result.

The initial fixture intentionally has no `xsl:include`, `doc()`, or
`xsl:result-document`. Those operations require Weaver's documented public
resolver seam and must not be simulated with direct filesystem reads.

## Slices

### Slice 1: Pin And Baseline

- [x] Add Weaver as the independently pinned `third-party/weaver-xslt`
      submodule.
- [x] Record the exact source revision in AR-0010.
- [x] Add selected XML/XSLT fixture bytes and expected output.

Acceptance criteria:

- Weaver remains an external submodule, not a Cargo dependency.
- A reviewer can identify the fixture input, stylesheet, related resource, and
  expected output without host paths.

### Slice 2: Source-Buffer Consumer Exercise

- [x] Add a TypeScript runner that loads only the selected fixture bytes and
      compares Weaver output with `expected.xml`.
- [x] Run the same fixture through Weaver's interpreter and generated/native
      execution paths where the selected stylesheet supports both.
- [x] Record source identity and execution mode in a concise result artifact.

Acceptance criteria:

- The runner does not parse XML/XSLT itself.
- A failed transform identifies the selected fixture and Weaver execution mode.
- The baseline compares semantic XML output after fixture-edge whitespace
  normalization and records raw interpreter/native output separately.
- No Tokimu crate gains an npm or TypeScript dependency.

Observed 2026-08-03: the interpreter preserved one leading literal newline
from the stylesheet while auto/native did not. The semantic XML result matched;
both raw outputs remain in `target/weaver-xslt-resource-space/baseline.json`.

### Slice 3: Resource Space Bridge

- [ ] Wait for Weaver to expose its documented resolver contract, or an
      equivalent public extension point.
- [ ] Implement a replaceable bridge over only public Resource Space lookup.
- [ ] Test one admitted same-folder reference.
- [ ] Test missing, parent-directory, unknown-scheme, and denied-resource
      failures.

Acceptance criteria:

- Resolution remains pure; loading remains a separate host decision.
- Parent traversal and unselected resources cannot escape the selected session.
- Diagnostics preserve lexical reference, operation, and selected resource
  identity where available.

### Slice 4: Admission Evidence

- [ ] Compare bridge behavior with `resource-space-xml` conformance tests.
- [ ] Record divergences as Weaver-specific policy, Resource Space contract
      refinement, or rejected semantics.
- [ ] Reopen AR-0010 and AR-0009 with the results.

Acceptance criteria:

- The review distinguishes successful composition from speculative API
  extraction.
- No shared capability is promoted without a second independent XML/XSLT
  consumer or a documented ADR-0005 exception.

## Current Constraint

Weaver documents a `ResourceResolver` design, but its current public
`XsltProcessor` accepts source text and transform options rather than a
resolver. This corpus records that gap as evidence; it does not bypass it with
ambient filesystem access.
