# XML Tools Incubation Library

## Status

Implementation in progress. Slice 0 has an initial local fixture baseline and
Slice 1 has established the incubating crate, parser-neutral diagnostic core,
and bounded input contract. The first consumer remains the SVG corpus/import
path. XML parsing, document modeling, XSD, and XPath are separate graduation
steps rather than one up-front standards implementation.

### Current Progress

- Initial XML fixture baseline recorded under `tests/fixtures/xml/`.
- `examples/lib-example/xml-tools` builds without rendering, platform,
  filesystem, browser, SVG, or parser dependencies.
- Source IDs, half-open spans, options, safe resource limits, and stable
  diagnostic categories/codes are implemented and tested.
- Parser-neutral bounded events now cover elements, attributes, comments,
  processing instructions, CDATA/text, predefined and numeric references, and
  namespace-expanded identities. The immutable document model and SVG
  migration remain open slices.
- `quick-xml 0.39.4` is the selected first parser adapter: its pure-Rust,
  streaming namespace-aware reader is kept private behind `xml-tools` types.
  Source-span mapping, strict UTF-8 handling, and explicit unsupported-feature
  diagnostics are now exercised by Slice 2 fixtures and focused tests.
- The W3C XML 2013-09-23 archive is vendored intact under
  `third-party/fixtures/w3c-xml-20130923/`. Its first selection is recorded
  in `selected/selection-v1.toml` with accepted, rejected, deferred, and
  unsupported-by-profile cases. A normal `xml-tools` smoke test executes one
  accepted UTF-8 case, one malformed case, and one real UTF-16 case diagnosed
  before parser adaptation.

## Purpose

Create an incubating Rust library at:

```text
examples/lib-example/xml-tools/
```

The library will provide a deterministic, bounded, native/WASM-compatible XML
ingestion boundary that can replace the current SVG importer's hand-written tag
scanning. It may later grow XSD and XPath support when concrete consumers earn
those modules.

The primary architectural claim is:

> Tokimu importers can consume XML through one parser-neutral document and
> diagnostic boundary without making XML a kernel concern or coupling document
> semantics to browser APIs.

The initial stage flow is:

```text
source bytes or UTF-8 text
        |
        v
xml-tools parser adapter
        |
        v
XML events / immutable document
        |
        v
SVG semantic importer
        |
        v
SvgVectorRecord
        |
        v
VectorPath -> mesh
```

## Why `examples/lib-example/xml-tools`

The existing repository convention places shared incubating implementations
under `examples/lib-example/`. `xml-tools` belongs there while its parser-
neutral API, consumers, and eventual ownership are still being discovered.

Its location records incubation rather than architectural ownership. Promotion
requires independent consumers and review; reuse by several examples does not
automatically make it a stable Tokimu capability.

## Motivation And Current Evidence

The current SVG adapter in `examples/lib-example/ui-tools/src/svg.rs` scans text
for element starts, tracks comments, searches for tag endings outside quotes,
and extracts attribute text directly.

That implementation was appropriate for a bounded Lucide proof, but the W3C SVG
corpus now creates pressure for a real XML boundary:

- source order must remain stable;
- malformed structure needs precise diagnostics;
- namespace-qualified names must not be confused with plain text matching;
- comments, processing instructions, character references, and declarations
  must be handled deliberately;
- nested SVG groups and inherited state require a reliable element stack;
- parser failure should be distinguishable from SVG semantic, vector, and mesh
  failure;
- native and browser builds should interpret the same source consistently.

The evidence supports an XML ingestion library and an XML stage in the SVG
corpus. It does not yet support a first-party engine crate, a complete XSD
processor, a complete XPath implementation, or a general web DOM.

## Architectural Position

`xml-tools` is example-side document infrastructure.

It is not:

- a `tokimu-core` primitive;
- an alternative runtime;
- a browser DOM wrapper;
- an SVG semantic model;
- a filesystem or networking service;
- a commitment to every XML-family standard.

The library owns:

- XML well-formedness processing for its declared profile;
- namespace-aware names;
- source spans and source-order identity;
- bounded events and, when earned, an immutable document representation;
- parser options and resource limits;
- structured XML diagnostics;
- stable parser-neutral types consumed by importers.

Consumers own:

- SVG elements, paint, transforms, inheritance, and references;
- XSD schema meaning and validation profiles;
- XPath query semantics and supported language profiles;
- conversion from XML diagnostics into broader Tokimu diagnostics;
- document loading, URLs, asset identity, and external-resource policy.

Parser implementations own:

- tokenization;
- character-reference decoding;
- well-formedness checks;
- low-level input traversal.

No parser-native node, token, error, or iterator type may appear in the public
`xml-tools` API.

## Dependency Direction

```text
parser implementation
        |
        v
examples/lib-example/xml-tools
        |
        +--------------------+
        |                    |
        v                    v
SVG importer        future XSD/XPath modules
        |
        v
presentation/vector pipeline
```

`xml-tools` should initially have no dependency on rendering, windowing,
platform APIs, filesystem APIs, browser APIs, or SVG/vector semantics.

Prefer keeping the base library independent of `tokimu-core`. A narrow adapter
may translate `XmlDiagnostic` into Tokimu diagnostics at the consumer boundary.
This keeps a syntax parser from accepting the engine's entire worldview merely
to report a line and column.

The same Rust parser path should run on native and WASM. Browser `DOMParser` may
be evaluated later as a differential reference or replaceable adapter, but it
must not define canonical Tokimu behavior.

## Initial Standards Profile

The first implementation must state an explicit supported profile rather than
claiming generic XML conformance.

Initial required behavior:

- UTF-8 text input;
- XML elements and attributes;
- empty-element syntax;
- comments;
- processing instructions;
- CDATA sections if the selected parser exposes them safely;
- predefined and numeric character references;
- namespace declarations and expanded names;
- deterministic document order;
- actionable errors for malformed nesting, names, attributes, and truncated
  input.

Initially unsupported or disabled:

- external entities;
- network or filesystem resolution;
- external DTD subsets;
- validation against DTD;
- XInclude;
- arbitrary encoding transcoding;
- mutation APIs;
- browser HTML recovery behavior;
- XML signatures, canonicalization, or encryption.

Unsupported features must fail or be reported explicitly. They must not be
silently interpreted under a different grammar.

## Security And Resource Limits

XML is untrusted input unless a caller proves otherwise. The initial API should
accept explicit `XmlLimits` with safe defaults:

```text
maximum input bytes
maximum nesting depth
maximum node count
maximum attributes per element
maximum name and attribute-value length
maximum decoded text length
maximum diagnostics retained
```

External entity expansion must be disabled. If internal entity declarations are
supported later, expansion depth and total expanded bytes require independent
limits.

Limit failures are structured diagnostics, not panics or silent truncation.

## Candidate Internal Model

Exact Rust names remain provisional. The first slices should prove the smallest
useful forms of:

```rust
pub struct XmlSourceId(/* opaque */);

pub struct XmlSpan {
    pub source: XmlSourceId,
    pub start: usize,
    pub end: usize,
}

pub struct ExpandedName {
    pub namespace_uri: Option<String>,
    pub local_name: String,
}

pub struct XmlAttribute {
    pub name: ExpandedName,
    pub lexical_prefix: Option<String>,
    pub value: String,
    pub span: XmlSpan,
}

pub enum XmlEvent {
    StartElement { /* name, attributes, span */ },
    EndElement { /* name, span */ },
    Text { /* decoded text, span */ },
    Comment { /* text, span */ },
    ProcessingInstruction { /* target, data, span */ },
}

pub struct XmlDocument {
    /* immutable arena-backed nodes in source order */
}
```

Design constraints:

- expanded names compare by namespace URI and local name, not by prefix text;
- lexical prefixes remain available for diagnostics and round-trip inspection;
- node identity is document-local and opaque;
- source order is stable;
- source spans refer to original input offsets;
- consumers can traverse without depending on the parser backend;
- public nodes do not retain self-referential Rust borrows that make storage or
  WASM integration fragile.

Do not add placeholder schema, XPath, mutation, visitor, serialization, or async
traits before a slice requires them.

## Parser Implementation Policy

The first slice should evaluate an existing pure-Rust XML parser before writing
a standards parser from scratch.

Selection criteria:

- native and `wasm32-unknown-unknown` support;
- namespace and source-position behavior;
- bounded or controllable entity processing;
- streaming/event access;
- predictable allocation;
- active maintenance and acceptable licensing;
- ability to keep parser-native types private;
- diagnostics adequate for mapping back to `XmlSpan`.

Tokimu may implement missing adapter behavior, source tracking, limits, or a
small bounded tokenizer where evidence requires it. Reimplementing the entire
XML grammar requires a separate plan item that names the unsupported behavior in
available parsers and adds differential and malformed-input evidence.

## Diagnostics

Every diagnostic should identify:

- stable diagnostic code;
- severity;
- source ID and span when available;
- XML processing stage;
- concise message;
- related opening/declaration span when useful;
- whether processing can continue.

Initial diagnostic categories:

```text
syntax
well-formedness
namespace
unsupported-feature
resource-limit
encoding
internal-adapter
```

Parser diagnostics must remain distinct from later SVG, XSD, XPath, vector, and
mesh diagnostics.

## Corpus Layout

Add focused fixtures under the existing test-fixture policy:

```text
tests/fixtures/xml/
    well-formed/
    malformed/
    namespaces/
    references/
    limits/
    svg/
```

Each admitted case records:

- case ID and purpose;
- source encoding/profile;
- expected event or document summary;
- expected diagnostic code and span for rejected input;
- whether the case comes from a pinned external corpus;
- source revision, license, and provenance where applicable.

Generated artifacts belong under `target/`, not beside fixtures.

## External Standards Corpus Strategy

The source discussion in `docs/Conversations/xml corpus.md` identifies one
standards corpus for each planned layer:

```text
W3C XML Conformance Test Suite
        |
        v
XML parser, events, namespaces, and diagnostics
        |
        v
W3C QT3 curated XPath subset
        |
        v
immutable document and query behavior
        |
        v
W3C XML Schema curated subset
        |
        v
schema compilation and instance validation
```

These upstream suites are evidence sources, not dependencies that Tokimu must
run in full. Every admitted case belongs to a reviewed selection manifest.

### W3C XML Conformance Test Suite

Use the pinned W3C XML Conformance Test Suite, with the 2013-09-23 release as
the initial acquisition candidate. Before admission, verify its archive,
notices, expected-result metadata, and applicability to the declared
`xml-tools` profile.

Curate cases under:

```text
xml/
    valid/
    not-well-formed/
    namespaces/
    references/
    declarations/
    limits/
```

Classify every selected upstream case as:

```text
Accepted
Rejected
UnsupportedByProfile
```

`Accepted` means the source must parse under the declared profile. `Rejected`
means the source violates behavior the profile claims and must produce a stable
diagnostic category. `UnsupportedByProfile` means the case depends on behavior
Tokimu deliberately does not claim, such as an unsupported encoding or DTD
feature. An unsupported result is not silently counted as a parser failure or
success.

This is the first external suite to integrate. It must stabilize namespaces,
expanded names, document order, character references, malformed-input
diagnostics, and profile classification before XPath or XSD suites are admitted.

### W3C QT3 XPath Suite

Use the official W3C QT3 repository only after an XPath consumer and bounded
XPath profile exist. Pin an exact commit and select tests through QT3 dependency
metadata rather than copying arbitrary passing cases.

Initial candidate groups:

```text
xpath/
    child-axis/
    descendant-axis/
    attributes/
    expanded-names/
    predicates-basic/
    document-order/
```

Initially exclude cases requiring:

- XQuery;
- schema awareness;
- higher-order functions;
- maps and arrays;
- dates and durations;
- collation machinery;
- external collections;
- static typing.

The selection manifest must state the supported XPath version or subset and the
QT3 dependencies accepted by the runner.

### W3C XML Schema Suite

Use the official W3C `xsdtests` repository only after a concrete XSD consumer
and compiled-schema model exist. Pin an exact commit and preserve the upstream
pairing between schema documents, instance documents, expected validity, and
test metadata.

Initial candidate groups:

```text
xsd/
    simple-elements/
    attributes/
    simple-types/
    sequence/
    choice/
    occurrence/
    namespaces/
    valid-invalid-pairs/
```

Initially defer:

- identity constraints;
- substitution groups;
- wildcards;
- schema imports and includes;
- assertions;
- type alternatives;
- XSD 1.1 features that require XPath.

The bounded selection must not be labeled general XSD conformance.

### Vendoring And Provenance

Use the established third-party fixture area:

```text
third-party/fixtures/
    w3c-xml-20130923/
        xmlts20130923.tar.gz
        upstream/
        selected/
            selection-v1.toml
        provenance.json
        LICENSES/
    w3c-qt3/
        upstream/
        selection-v1.toml
        provenance.json
        LICENSES/
    w3c-xsd/
        upstream/
        selection-v1.toml
        provenance.json
        LICENSES/
```

For an archive, record:

- source URL;
- upstream release date;
- retrieval date;
- archive SHA-256;
- license and notices;
- selection policy.

For a Git-hosted suite, additionally record the exact commit. Verify the
license and notices belonging to each historical archive or repository rather
than assuming a current W3C test-suite license applies retroactively.

Keep upstream material unchanged beneath `upstream/`. Tokimu-owned expected
classification, normalization, exclusions, and case purpose belong in the
selection manifest or adjacent reviewed metadata.

### Execution Tiers

Do not put entire upstream suites in the default workspace test path.

Use three execution tiers:

```text
smoke       small reviewed cases run by normal workspace tests
selected    complete admitted manifest run explicitly or in extended CI
upstream    acquisition/audit tooling; not a conformance claim
```

Reports must distinguish unsupported-by-profile cases from unexpected failures
and must preserve the upstream case identity. Timing and total-suite pass
percentages are observational; only the admitted selection defines Tokimu's
claimed profile.

## Implementation Slices

### Slice 0: Fix The Boundary And Baseline

- Select the first SVG fixtures that currently exercise manual tag scanning.
- Record current `SvgVectorRecord`, vector, mesh, and diagnostic outputs.
- Identify XML failures separately from SVG semantic limitations.
- Decide whether the first API is event-only or needs a minimal immutable
  document for nested SVG state.
- Evaluate candidate parser implementations against the initial standards
  profile.
- Define the three-way external-case classification:
  `Accepted`, `Rejected`, and `UnsupportedByProfile`.
- Prepare the acquisition/provenance record and first deliberately small
  selection manifest for the W3C XML suite.

Acceptance:

- at least one well-formed, malformed, namespace, comment, and character-
  reference case has a written expected result;
- current SVG outputs are reproducible;
- parser selection records capabilities and gaps without leaking into the
  proposed public API.

Progress: initial well-formed, malformed, namespace, character-reference,
limit, and SVG-comment fixtures are recorded in `tests/fixtures/xml/`. Parser
selection is recorded: `quick-xml 0.39.4` provides the first private adapter
candidate because it is pure Rust, streaming, namespace-aware, and already
pinned in the workspace lockfile. The pinned W3C XML 2013-09-23 archive,
checksum, preserved upstream tree, notices, v1 selection manifest, and feature
matrix now live under `third-party/fixtures/w3c-xml-20130923/`. The default
smoke test exercises the accepted/rejected/unsupported classification without
claiming full-suite conformance. `scripts/verify-w3c-xml-fixtures.ps1`
validates a locally retained archive checksum and the selected upstream paths
without network access. Current SVG artifact comparison remains open.

### Slice 1: Create The Library And Diagnostic Core

- Create `examples/lib-example/xml-tools` as a Rust library.
- Add it explicitly to the workspace.
- Add `DESIGN.md` describing its primary proof and incubation status.
- Implement source IDs, spans, options, limits, and structured diagnostics.
- Add native unit tests and a WASM compilation check.

Acceptance:

- the crate builds without rendering, platform, filesystem, or browser
  dependencies;
- invalid options and limit violations are diagnostic;
- no public parser-backend types exist.

Progress: complete. `xml-tools` is a zero-dependency workspace member with
opaque source IDs, half-open source spans, validated `XmlLimits`,
`XmlOptions`, `XmlDiagnostic`, and pre-parse input-size validation. Native unit
tests cover defaults, invalid limits, source-scoped limit failures, and spans.

### Slice 2: Add Bounded XML Events

- Adapt the selected parser into namespace-aware `XmlEvent` values.
- Preserve deterministic source order and original spans.
- Decode the admitted character-reference profile.
- Diagnose malformed nesting, attributes, namespace declarations, truncated
  input, and unsupported declarations.
- Enforce depth, node/event, attribute, and decoded-text limits.

Acceptance:

- the initial XML corpus passes on native;
- the same accepted sources compile and execute through the same Rust path on
  WASM where the test harness permits;
- malformed and resource-limit cases fail with stable diagnostic categories;
- external resources are never resolved.

Progress: native implementation complete for the first bounded profile.
`xml-tools` exposes only owned parser-neutral events, expanded names,
attributes, source spans, limits, and diagnostics. Fixture-backed tests cover
well-formed order, namespaced elements and attributes, allowed character
references, malformed nesting, disabled DTD processing, unsupported encodings,
depth limits, and W3C source bytes. `parse_xml_bytes` diagnoses non-UTF-8
inputs before parser adaptation, allowing external UTF-16 cases to remain
explicitly unsupported rather than being silently transcoded. WASM execution
and broader malformed/limit coverage remain part of hardening rather than
silently claimed complete.

### Slice 3: Add The Minimal Immutable Document

- Introduce an arena-backed immutable document only if the SVG importer or first
  XPath/XSD consumer needs retained traversal.
- Preserve parent/child relationships, attributes, namespaces, source spans,
  and document order.
- Provide narrow traversal methods rather than a browser-shaped mutable DOM.
- Keep event parsing usable for streaming consumers.

Acceptance:

- document construction respects the same resource limits;
- traversal order is deterministic;
- namespace scope and expanded-name comparison are tested;
- node handles cannot cross documents undiagnosed.

### Slice 4: Migrate SVG Document Syntax

- Replace manual SVG tag, comment, quote, and attribute scanning with
  `xml-tools`.
- Keep SVG path-data tokenization and SVG semantic interpretation in the SVG
  importer.
- Maintain an explicit SVG state stack for inherited paint and transforms only
  when corresponding corpus cases require it.
- Preserve `SvgVectorRecord` source order and all existing structural outputs.
- Add an XML stage to presentation-geometry corpus artifacts or stage graphs.

Acceptance:

- existing Lucide and admitted W3C SVG cases preserve intended vector/mesh
  results;
- comments and similarly named elements/attributes cannot be misread as
  geometry;
- XML errors stop at the XML stage with source spans;
- unsupported SVG features remain SVG diagnostics rather than XML errors;
- the old manual document scanner is removed after equivalent coverage passes.

Progress: started. The public `parse_svg_document_vector_records` path now
consumes owned `xml-tools` start-element events for source ordering, expanded
names, attribute decoding, comments, quoted text, and malformed-markup
diagnostics. SVG retains ownership of `d` parsing, primitive lowering,
coordinate normalization, and paint interpretation. Existing focused SVG tests
preserve record order and paint results; truncated markup now stops with an
explicit XML-stage error. The legacy test-only scanner remains temporarily as
an equivalence reference until the new path has equivalent corpus coverage and
the old scanner can be removed deliberately.

### Slice 5: Harden And Compare

- Add constrained malformed-input and limit stress cases.
- Compare accepted events/document summaries with one independent XML
  implementation or standards-derived corpus where licensing permits.
- Run the selected W3C XML manifest and report accepted, rejected, unsupported,
  and unexpected outcomes separately.
- Keep a small smoke subset in normal workspace tests and the complete admitted
  selection behind an explicit extended command.
- Add fuzzing or seeded generation only after stable invariants exist.
- Verify that normal corpus execution never rewrites reviewed expectations.

Acceptance:

- parser panics and unbounded expansion are treated as failures;
- differential disagreement produces inspectable evidence rather than silently
  changing goldens;
- native and WASM preserve the declared semantic profile.

### Slice 6: Add A Second XML Consumer

- Select a consumer independent of SVG, such as an XML-backed tool document,
  interchange fixture, or inspection utility.
- Reuse the existing event/document and diagnostic contracts.
- Record which behavior is genuinely shared and which remains consumer-owned.

Acceptance:

- shared XML types acquire no SVG-specific concepts;
- the second consumer does not require filesystem, browser, or engine state in
  the base library;
- an Architectural Review evaluates whether `xml-tools` should remain
  example-side or graduate.

## XSD Extension Track

XSD is not part of the initial XML parser.

Open an XSD slice only when a concrete schema-backed consumer exists. Before
implementation, record:

- the exact XSD version or bounded profile;
- supported schema components and datatypes;
- namespace/import/include policy;
- identity-constraint expectations;
- validation result and diagnostic model;
- whether schema compilation can be cached;
- external-resource and recursion limits.

Suggested progression:

1. schema document parsing through the same XML layer;
2. immutable compiled schema representation;
3. simple element/attribute declarations and occurrence constraints;
4. only the datatype and composition features required by the first corpus;
5. explicit diagnostics for every unsupported schema construct;
6. differential validation against a pinned reference implementation.

Do not label a bounded profile as general XSD conformance.

When this track opens, pin the official W3C `xsdtests` repository and admit only
the manifest groups named in the External Standards Corpus Strategy. Preserve
schema/instance pairing and expected-validity metadata.

## XPath Extension Track

XPath is not part of the initial XML parser or document traversal API.

Open an XPath slice when a real tool, test, XSD feature, or importer needs
declarative selection beyond ordinary traversal. Record:

- the exact XPath version or subset;
- namespace binding rules;
- data model and document-order behavior;
- supported axes, node tests, predicates, functions, and value conversions;
- compilation/caching expectations;
- result and diagnostic types;
- evaluation budgets.

Suggested progression:

1. child and descendant location paths;
2. expanded-name tests with explicit namespace bindings;
3. attribute selection;
4. bounded predicates required by the first consumer;
5. compiled expressions only after repeated evaluation proves the need;
6. differential cases against a pinned reference.

Do not design XSD around a speculative complete XPath engine. Admit only the
query behavior the schema or tooling corpus actually uses.

When this track opens, pin the official W3C QT3 repository and filter cases by
declared feature dependencies. Do not run or vendor the full XPath/XQuery suite
as though it described Tokimu's bounded query profile.

## Extension Order

Prefer this admission order:

```text
1. W3C XML selection
       proves parser/events/diagnostics

2. QT3 XPath selection
       proves immutable document/query behavior

3. W3C XSD selection
       proves schema compilation and validation
```

XPath precedes XSD corpus work because it directly pressures document order,
expanded names, traversal, and query results without first adding schema
compilation. XSD 1.1 XPath-dependent features remain deferred. A concrete XSD
consumer may justify a small XSD profile earlier, but that exception must not
quietly require an unplanned XPath implementation.

## Extension Admission Rule

Each XML-family module must have:

- a named consumer;
- a declared standards version or subset;
- supported and unsupported feature tables;
- bounded resource behavior;
- structured diagnostics;
- native/WASM validation where applicable;
- independent reference or standards-derived corpus evidence before claiming
  compatibility.

Adding an empty module, trait, or feature flag does not count as progress.

## Validation

For implementation slices, prefer:

- `cargo fmt --all`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`;
- explicit `wasm32-unknown-unknown` compilation for `xml-tools` and its SVG
  consumer;
- exact event/document summaries for deterministic cases;
- exact diagnostic categories and spans for malformed cases;
- existing SVG vector/mesh fingerprints and reviewed artifacts;
- limit and hostile-input tests;
- differential results labeled as observational evidence until reviewed.

## Risks

### Reimplementing A Standards Stack

Risk: XML, XSD, and XPath become an open-ended engine project.

Mitigation: separate profiles and admission tracks; use an existing parser
behind Tokimu-owned types; require a concrete consumer for every extension.

### XML Becomes Kernel Meaning

Risk: a common interchange format is mistaken for universal simulation
semantics.

Mitigation: keep the library example-side and parser-neutral; require an
Architectural Review before any first-party crate promotion.

### Browser Behavior Becomes Canonical

Risk: `DOMParser` or browser recovery behavior makes native and WASM imports
semantically different.

Mitigation: use the same Rust path on both targets; browser APIs are optional
adapters or differential references only.

### Security And Expansion Attacks

Risk: untrusted XML consumes unbounded CPU or memory or resolves external
resources.

Mitigation: disable external entities and resolution, enforce explicit limits,
and test failure boundaries.

### SVG Semantics Leak Into XML

Risk: group inheritance, paint, transforms, or geometry become generic XML
features.

Mitigation: XML owns syntax and structure; the SVG importer owns SVG meaning.

### Premature Stable DOM

Risk: future XSD/XPath speculation produces a large mutable browser-shaped API.

Mitigation: start with bounded events and add the smallest immutable document
needed by a real consumer.

## Completion Criteria

The initial XML phase is complete when:

- `examples/lib-example/xml-tools` exists and has a documented standards
  profile;
- native and WASM use the same Rust parsing path;
- structured XML events and diagnostics are bounded and parser-neutral;
- an immutable document exists only if demonstrated necessary;
- the SVG importer no longer manually scans XML document syntax;
- admitted Lucide and W3C SVG cases preserve structural results;
- corpus artifacts distinguish XML, SVG, vector, and mesh stages;
- unsupported XML and SVG behavior is diagnosed at the correct boundary;
- XSD and XPath remain separately scoped until named consumers earn them.

Promotion beyond example-side incubation requires:

- at least two independent consumers;
- stable native/WASM behavior;
- no parser-backend leakage;
- a completed Architectural Review;
- an explicit decision about whether the library is general document tooling,
  asset/import infrastructure, or a provider implementation;
- an ADR if promotion changes an accepted workspace or ownership boundary.

## References

- `examples/lib-example/ui-tools/src/svg.rs`
- `examples/lib-example/presentation-geometry-corpus/`
- `docs/Plans/presentation-geometry-corpus-harness.md`
- `docs/Architectural Reviews/AR-0001-shared-vector-presentation-geometry.md`
- `docs/Conversations/xml corpus.md`
- `docs/testing-strategy.md`
- `docs/example-philosophy.md`
- `examples/README.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- W3C XML Conformance Test Suites —
  <https://www.w3.org/XML/Test/>
- W3C QT3 test repository —
  <https://github.com/w3c/qt3tests>
- W3C QT3 runner guidance —
  <https://dev.w3.org/2011/QT3-test-suite/guide/running.html>
- W3C XML Schema test repository —
  <https://github.com/w3c/xsdtests>
