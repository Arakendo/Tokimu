# XML Tools Incubation Library

## Status

Implementation in progress. Slices 1 through 6 now have executable evidence:
the incubating crate, bounded parser-neutral events, immutable documents, SVG
migration, hostile-input hardening, a pinned four-case W3C smoke profile, and a
second independent consumer are implemented. AR-0003 records the current XML
document boundary. Remaining gates include runtime WASM execution, an
independent differential implementation with inspectable disagreement
artifacts, broader standards inventory, and any separately earned XPath, XSLT,
or XSD extension. Those standards remain independent graduation tracks rather
than one up-front XML stack.

### Current Progress

- Initial XML fixture baseline recorded under `tests/fixtures/xml/`.
- `corpus/lib/xml-tools` builds without rendering, platform,
  filesystem, browser, SVG, or Tokimu engine dependencies. Its selected parser
  dependency remains private behind parser-neutral `xml-tools` types.
- Source IDs, half-open spans, options, safe resource limits, and stable
  diagnostic categories/codes are implemented and tested.
- Parser-neutral bounded events now cover elements, attributes, comments,
  processing instructions, CDATA/text, predefined and numeric references, and
  namespace-expanded identities. The immutable document model retains those
  events as source-ordered nodes with document-local handles. The SVG importer
  now consumes the parser-neutral event path directly for semantic lowering and
  corpus artifact evidence.
- `quick-xml 0.39.4` is the selected first parser adapter: its pure-Rust,
  streaming namespace-aware reader is kept private behind `xml-tools` types.
  Source-span mapping, strict UTF-8 handling, element-name and resource-limit
  enforcement, and explicit unsupported-feature diagnostics are now exercised
  by Slice 2 fixtures and focused tests. Unterminated and mismatched elements
  are rejected at the parser-neutral event boundary rather than reaching a
  later consumer. Matching-end validation is deliberately adapter-owned so
  diagnostics retain the opening element span independent of parser behavior.
- The W3C XML 2013-09-23 archive is vendored intact under
  `third-party/fixtures/w3c-xml-20130923/`. Its first selection is recorded
  in `selected/selection-v1.toml` with accepted, rejected, deferred, and
  unsupported-by-profile cases. A normal `xml-tools` smoke test executes one
  accepted UTF-8 case, one malformed case, and one real UTF-16 case diagnosed
  before parser adaptation.

## Purpose

Create an incubating Rust library at:

```text
corpus/lib/xml-tools/
```

The library will provide a deterministic, bounded, native/WASM-compatible XML
ingestion boundary that can replace the current SVG importer's hand-written tag
scanning. It may later grow XPath, XSLT, and XSD support when concrete
consumers earn those modules.

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

## Why `corpus/lib/xml-tools`

The existing repository convention places shared incubating implementations
under `corpus/lib/`. `xml-tools` belongs there while its parser-
neutral API, consumers, and eventual ownership are still being discovered.

Its location records incubation rather than architectural ownership. Promotion
requires independent consumers and review; reuse by several examples does not
automatically make it a stable Tokimu capability.

## Motivation And Current Evidence

The original bounded Lucide proof used text scanning inside the SVG adapter for
element starts, comments, quoted tag endings, and attribute extraction. That
transitional scanner has now been removed. `svg.rs` consumes parser-neutral
`XmlEvent` values from `xml-tools`; XML owns syntax while the SVG importer owns
its namespace/profile, presentation, transform, viewport, and path semantics.

The W3C SVG corpus established the pressure for that real XML boundary:

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
processor, complete XPath or XSLT implementations, or a general web DOM.

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
- XSLT stylesheet, transformation, output, and supported language profiles;
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
corpus/lib/xml-tools
        |
        +--------------------+--------------------+
        |                    |                    |
        v                    v                    v
SVG importer        future XPath/XSLT       future XSD
                    modules                 module
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

Do not add placeholder schema, XPath, XSLT, mutation, visitor, serialization,
or async traits before a slice requires them.

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

Parser diagnostics must remain distinct from later SVG, XSD, XPath, XSLT,
vector, and mesh diagnostics.

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

The source discussion in `docs/Conversations/xml corpus.md` identifies the XML,
XPath, and XSD corpus foundations. XSLT adds a fourth independently admitted
transformation corpus:

```text
W3C XML Conformance Test Suite
        |
        v
XML parser, events, namespaces, and diagnostics
        |
        v
immutable document model
        |
        +----> W3C QT3 curated XPath subset
        |              |
        |              v
        |      query behavior
        |              |
        |              v
        |      curated XSLT standards/reference subset
        |              |
        |              v
        |      stylesheet compilation and transformation
        |
        +----> W3C XML Schema curated subset
                       |
                       v
              schema compilation and validation
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
diagnostics, and profile classification before XPath, XSLT, or XSD suites are
admitted.

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

### XSLT Standards Corpus

Open an external XSLT corpus only after a transformation consumer and bounded
XPath profile exist. Before vendoring, identify and verify a standards-derived
or reference suite for the chosen XSLT version, including its provenance,
license, expected-result format, and ability to select cases by feature.

A bounded XSLT 1.0-style profile is the first candidate for web-facing
interoperability, but the version is not accepted until a consumer and corpus
selection make that choice explicit.

Initial candidate groups:

```text
xslt/
    template-match/
    apply-templates/
    literal-result-elements/
    value-of/
    for-each/
    if-and-choose/
    attributes/
    namespaces/
    xml-output/
    text-output/
```

Initially defer:

- `document()`, collections, and external URI access;
- extension elements and extension functions;
- script execution or host-language callbacks;
- multiple result documents;
- schema-aware processing;
- streaming transformations;
- packages, dynamic evaluation, and implementation-specific extensions;
- HTML recovery or browser DOM mutation as transformation semantics.

Every selected case must identify the stylesheet, source document, parameters,
expected principal result, namespace/output normalization policy, and supported
feature dependencies. Unsupported cases are classified
`UnsupportedByProfile`, not counted as transformation failures.

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
    w3c-xslt/
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

Deliverables:

- [x] Select the first SVG fixtures that exercise manual tag scanning.
- [ ] Record current `SvgVectorRecord`, vector, mesh, and diagnostic outputs as
  reviewed comparison artifacts.
- [x] Identify XML failures separately from SVG semantic limitations.
- [x] Decide the initial event API and whether retained traversal needs a
  minimal immutable document.
- [x] Evaluate candidate parser implementations against the initial standards
  profile.
- [x] Define the three-way external-case classification: `Accepted`,
  `Rejected`, and `UnsupportedByProfile`.
- [x] Prepare the acquisition/provenance record and first deliberately small
  W3C XML selection manifest.

Acceptance criteria:

- [x] At least one well-formed, malformed, namespace, comment, and character-
  reference case has a written expected result.
- [x] Current SVG structural and diagnostic outputs are captured as
  reproducible reviewed evidence.
- [x] Parser selection records capabilities and gaps without leaking backend
  types into the proposed public API.

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
without network access. The admitted Lucide and W3C SVG cases now have explicit
reviewed report snapshots and deterministic mesh fingerprints; SVG artifacts
also preserve the XML stage that precedes semantic lowering.

### Slice 1: Create The Library And Diagnostic Core

Deliverables:

- [x] Create `corpus/lib/xml-tools` as a Rust library.
- [x] Add it explicitly to the workspace.
- [x] Add `DESIGN.md` describing its primary proof and incubation status.
- [x] Implement source IDs, spans, options, limits, and structured diagnostics.
- [x] Add native unit tests.
- [x] Add and run a WASM compilation check.

Acceptance criteria:

- [x] The crate builds without rendering, platform, filesystem, or browser
  dependencies.
- [x] Invalid options and limit violations produce structured diagnostics.
- [x] No public parser-backend types exist.

Progress: complete. `xml-tools` has no Tokimu engine, rendering, platform,
filesystem, or browser dependencies; its selected parser remains a private
adapter. The crate owns opaque source IDs, half-open source spans, validated
`XmlLimits`, `XmlOptions`, `XmlDiagnostic`, and pre-parse input-size
validation. Native unit tests cover defaults, invalid limits, source-scoped
limit failures, and spans.

### Slice 2: Add Bounded XML Events

Deliverables:

- [x] Adapt the selected parser into namespace-aware `XmlEvent` values.
- [x] Preserve deterministic source order and original spans.
- [x] Decode the admitted character-reference profile.
- [x] Diagnose malformed nesting, attributes, namespace declarations,
  truncated input, and unsupported declarations.
- [x] Enforce depth, node/event, element-name, attribute,
  attribute-value, and decoded-text limits.
- [x] Keep matching-end validation at the parser-neutral boundary with the
  opening element available as related diagnostic context.

Acceptance criteria:

- [x] The initial XML corpus passes on native.
- [x] The same Rust parser path compiles for `wasm32-unknown-unknown`.
- [ ] Accepted sources execute through the same Rust path on WASM when a test
  harness is available.
- [x] Malformed and resource-limit cases fail with stable diagnostic
  categories and source spans.
- [x] DTD processing and external resource resolution remain disabled.

Progress: native implementation complete for the first bounded profile.
`xml-tools` exposes only owned parser-neutral events, expanded names,
attributes, source spans, limits, and diagnostics. Fixture-backed tests cover
well-formed order, namespaced elements and attributes, allowed character
references, malformed nesting, disabled DTD processing, unsupported encodings,
depth limits, and W3C source bytes. `parse_xml_bytes` diagnoses non-UTF-8
inputs before parser adaptation, allowing external UTF-16 cases to remain
explicitly unsupported rather than being silently transcoded. Focused tests
also cover element-name, attribute, value, decoded-text, node, nesting,
unbound-prefix, truncated-input, and related-span mismatched-end boundaries.
WASM execution and broader hostile-input coverage remain part of hardening
rather than silently claimed complete.

### Slice 3: Add The Minimal Immutable Document

Deliverables:

- [x] Introduce an arena-backed immutable document for retained traversal.
- [x] Preserve parent/child relationships, attributes, namespaces, source
  spans, and document order.
- [x] Provide narrow traversal methods rather than a browser-shaped mutable
  DOM.
- [x] Keep event parsing independently usable by streaming consumers.

Acceptance criteria:

- [x] Document construction through the parse boundary respects the same
  resource limits as event parsing.
- [x] Traversal order is deterministic.
- [x] Namespace expansion and expanded-name retention are tested.
- [x] Node handles from another document are rejected.
- [x] Accepted standards-derived cases can retain a document element.

Progress: complete for the first retained traversal contract. `XmlDocument`
builds immutable nodes from parser-neutral events, preserving source order,
parent/child relationships, attributes, expanded names, and spans. Consumers
provide an opaque `XmlDocumentId`; every `XmlNodeId` carries that identity and
is rejected by another document's traversal methods. Top-level whitespace,
comments, and processing instructions remain visible through `roots()` for
inspection, while `document_element()` supplies the first top-level element for
ordinary importer traversal. The model intentionally provides no mutation,
selector, schema, or SVG-specific API.

### Slice 4: Migrate SVG Document Syntax

Detailed implementation tracking:
`.workbench/Todos/svg-xml-tools-pipeline.md`.

Deliverables:

- [x] Route the public SVG document parser through `xml-tools` for tag,
  comment, quote, attribute, and namespace handling.
- [x] Keep SVG path-data tokenization and SVG semantic interpretation in the
  SVG importer.
- [ ] Add an explicit SVG state stack for inherited paint and transforms when
  admitted corpus cases require it.
- [x] Preserve focused `SvgVectorRecord` source order and structural outputs.
- [x] Add an XML stage to presentation-geometry corpus artifacts or stage
  graphs.
- [x] Remove the legacy test-only manual document scanner after focused
  replacement coverage is complete.

Acceptance criteria:

- [x] Existing Lucide and admitted W3C SVG cases have reviewed equivalence for
  intended vector/mesh results.
- [x] Comments and similarly named elements or attributes cannot be misread as
  geometry.
- [x] XML errors stop at the XML stage with source spans.
- [x] Unsupported SVG semantics remain SVG diagnostics rather than XML errors.
- [x] No manual SVG document-syntax scanner remains after focused equivalent
  coverage passes.

Progress: in progress. The public `parse_svg_document_vector_records` path now
consumes owned `xml-tools` start-element events for source ordering, expanded
names, attribute decoding, comments, quoted text, and malformed-markup
diagnostics. SVG retains ownership of `d` parsing, primitive lowering,
coordinate normalization, and paint interpretation. Existing focused SVG tests
preserve record order and paint results; truncated markup now stops with an
explicit XML-stage error. Focused primitive, quoted-attribute, malformed-input,
and geometry-normalization tests now exercise the parser-neutral public path,
so the test-only manual scanner and its duplicate tag/attribute logic have been
removed. The presentation-geometry corpus
now records SVG and admitted W3C SVG evidence as `source -> xml -> vector ->
mesh`; W3C artifact output includes a parser-neutral `xml.json` summary and
the same stage graph. The direct corpus dependency observes the XML boundary
for diagnostics only and does not constitute a second semantic consumer or a
promotion trigger. The registered Lucide archive case and both admitted W3C
cases now compare explicit report snapshots plus deterministic mesh
fingerprints. SVG artifact generation uses the same `SvgVectorRecord` fill and
fill-rule semantics as normal execution, so the reviewed artifacts no longer
substitute an even-odd-only corpus path. This is reviewed structural
equivalence for the declared Tokimu SVG subset, not a claim of full W3C visual
conformance.

### Slice 5: Harden And Compare

Deliverables:

- [x] Add constrained malformed-input and resource-limit stress cases.
- [x] Compare accepted event and document behavior against a
  standards-derived corpus.
- [x] Run the selected W3C XML manifest and report accepted, rejected,
  unsupported, deferred, and unexpected outcomes separately.
- [x] Keep a small smoke subset in normal workspace tests and the complete
  admitted selection behind an explicit extended command.
- [x] Add bounded seeded generation after stable event/document invariants
  exist.
- [x] Detect drift between the reviewed selection manifest and its executable
  runner.
- [ ] Add an independent implementation comparison with inspectable summaries.
- [x] Expand focused hostile-input coverage after the first bounded generated
  tier.

Acceptance criteria:

- [x] Parser panics, excessive work, and unbounded expansion are covered as
  explicit failures across the admitted hostile-input set.
- [ ] Differential disagreement produces inspectable evidence rather than
  silently changing reviewed expectations.
- [x] Native execution and WASM compilation preserve the declared dependency
  and parser profile.
- [ ] The declared semantic profile is exercised at runtime on WASM.
- [x] Normal smoke and selected runs do not rewrite reviewed expectations.

Progress: the first explicit selected-manifest runner now lives in
`corpus/lib/xml-tools/tests/w3c_selection.rs`. It is intentionally
ignored by the default test path and runs with:

```text
cargo test -p xml-tools --test w3c_selection -- --ignored --nocapture
```

The v1 report currently records one accepted case, one rejected case, one
unsupported-by-profile UTF-16 case, and one deferred namespace-diagnostic
case. Accepted cases are validated as both parser-neutral event streams and
immutable documents. This preserves the reviewed manifest's distinctions
instead of reducing the selection to a single pass/fail percentage.
`xml-tools` also compiles for `wasm32-unknown-unknown`. A normal seeded tier
generates 32 bounded, well-formed documents and verifies repeatable event order
plus retained roots. Focused hostile-input tests now cover multiple/no document
elements, non-whitespace outside the root, duplicate attributes, malformed
comments, unsupported entity references, and disabled DTD payloads with stable
source-scoped diagnostics. Broad fuzz-style hostile-input coverage and
differential parser comparison remain open hardening work.

The admitted hostile tier now also runs 64 deterministic malformed inputs
through panic containment. It covers mismatched and truncated nesting, multiple
roots, disabled DTD entity declarations, unsupported entity references, and
trailing non-whitespace text. Every generated case must produce a
non-continuable, source-scoped diagnostic rather than panic or silently parse.
Existing explicit resource-limit tests remain the evidence for bounded input,
depth, node, attribute, name, value, and decoded-text work; DTD rejection keeps
entity expansion disabled in the initial profile. This is intentionally a
bounded hostile corpus, not a claim of broad fuzzing or differential-parser
coverage.

### Slice 6: Add A Second XML Consumer

Deliverables:

- [x] Select and name a consumer independent of SVG, such as an XML-backed tool
  document, interchange fixture, or inspection utility.
- [x] Reuse the existing event/document and diagnostic contracts in that
  consumer.
- [x] Record which behavior is genuinely shared and which remains
  consumer-owned.
- [x] Open an Architectural Review after the second consumer has produced
  implementation evidence.

Acceptance criteria:

- [x] Shared XML types acquire no SVG-specific concepts.
- [x] The second consumer requires no filesystem, browser, rendering, or engine
  state in the base library.
- [x] The second consumer does not require parser-backend types to cross the
  public boundary.
- [x] Architectural Review records whether `xml-tools` remains example-side or
  graduates and names the evidence supporting that disposition.

Progress: `corpus/hello-xml-inspect` is a base-example command-line
inspection utility independent of SVG. It reads a checked-in fixture or an
explicit application-owned file path, then consumes only `XmlDocument`,
`XmlNodeKind`, expanded names, attributes, source spans, and `XmlDiagnostic`.
It deliberately owns filesystem access and terminal formatting itself;
`xml-tools` remains bounded parser/document/diagnostic infrastructure with no
filesystem, browser, renderer, or engine dependency. The second consumer is
evidence for review, not automatic promotion. `AR-0003-xml-document-boundary`
records the resulting incubation disposition: two example-side consumers are
enough to preserve the boundary, but not enough to extract a first-party crate.

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

## XSLT Extension Track

XSLT is not part of the initial XML parser, immutable document, or XPath
surface. It is an executable document transformation language and therefore has
stronger authority, resource, and determinism requirements than ordinary
traversal.

Open an XSLT slice when an asset pipeline, authoring tool, SVG/XML normalization
step, report generator, or other concrete consumer needs declarative XML
transformation. Before implementation, record:

- the exact XSLT version or bounded profile;
- the required XPath profile;
- stylesheet compilation and caching expectations;
- parameter and variable types;
- template matching, priority, mode, and import-precedence behavior;
- output methods and normalization rules;
- result and diagnostic types;
- extension-function policy;
- external document and URI policy;
- evaluation, recursion, and output budgets;
- native/WASM equivalence expectations.

The transformation boundary should be:

```text
immutable stylesheet document
        +
immutable source document
        +
explicit parameters and resource resolver
        |
        v
compiled bounded transformation
        |
        v
owned XML events/document or text result
        +
structured diagnostics
```

An XSLT transformation must not receive ambient filesystem, network, browser
DOM, application state, or host-language execution authority. External
resources, if ever admitted, use an explicit caller-supplied resolver with
scoped authority, stable identity, cycle detection, and resource budgets.

Initial limits should include:

```text
maximum stylesheet size and nodes
maximum included/imported stylesheets
maximum template applications
maximum call/recursion depth
maximum XPath evaluation work
maximum variables and parameters
maximum result nodes
maximum result bytes
maximum diagnostics retained
maximum transformation time or cooperative work budget where supported
```

Suggested progression:

1. compile literal result elements and basic template matches;
2. reuse the admitted XPath subset for selection and value extraction;
3. add `apply-templates`, built-in template behavior, and deterministic
   document-order processing;
4. add bounded variables, parameters, conditionals, and iteration required by
   the first consumer;
5. support XML and text principal-result output;
6. add imports, includes, keys, multiple outputs, or HTML output only when
   separate corpus evidence requires them;
7. compare admitted transformations against a pinned standards/reference
   corpus and independent implementation.

Stylesheets may be useful in web-facing authoring and asset preparation, but
they are not simulation rules and must not become an alternate runtime. Browser
XSLT APIs may serve as differential references or optional adapters; they do
not define canonical Tokimu semantics.

Do not label a bounded transformation profile as general XSLT conformance.

## Extension Order

Prefer this admission order:

```text
1. W3C XML selection
       proves parser/events/diagnostics

2. QT3 XPath selection
       proves immutable document/query behavior

3a. Curated XSLT selection, when a transformation consumer exists
        proves stylesheet compilation and bounded transformation

3b. W3C XSD selection, when a schema consumer exists
        proves schema compilation and validation
```

XPath precedes XSLT because template selection, expressions, and value
extraction depend on an explicit query profile. XSLT and XSD then proceed as
independent tracks according to consumer pressure. XPath also directly
pressures document order, expanded names, traversal, and query results before
schema compilation. XSD 1.1 XPath-dependent features remain deferred. A
concrete XSD consumer may justify a small XSD profile earlier, but that
exception must not quietly require an unplanned XPath implementation.

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
- exact XPath result order and XSLT principal-result comparisons when those
  tracks open;
- XSLT recursion, output, external-resource, and extension-function denial
  tests;
- differential results labeled as observational evidence until reviewed.

## Risks

### Reimplementing A Standards Stack

Risk: XML, XSD, XPath, and XSLT become an open-ended engine project.

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

Risk: future XSD/XPath/XSLT speculation produces a large mutable browser-shaped
API.

Mitigation: start with bounded events and add the smallest immutable document
needed by a real consumer.

### XSLT Becomes Ambient Code Execution

Risk: stylesheets gain filesystem, network, browser DOM, extension-function, or
application-state authority and become an alternate runtime.

Mitigation: transform immutable inputs into owned results under explicit
budgets; disable ambient external access and host-language extensions; require
an explicit scoped resolver for any later external resource support.

## Completion Criteria

The initial XML phase is complete when:

- `corpus/lib/xml-tools` exists and has a documented standards
  profile;
- native and WASM use the same Rust parsing path;
- structured XML events and diagnostics are bounded and parser-neutral;
- an immutable document exists only if demonstrated necessary;
- the SVG importer no longer manually scans XML document syntax;
- admitted Lucide and W3C SVG cases preserve structural results;
- corpus artifacts distinguish XML, SVG, vector, and mesh stages;
- unsupported XML and SVG behavior is diagnosed at the correct boundary;
- XSD, XPath, and XSLT remain separately scoped until named consumers earn
  them.

Promotion beyond example-side incubation requires:

- at least two independent consumers;
- stable native/WASM behavior;
- no parser-backend leakage;
- a completed Architectural Review;
- an explicit decision about whether the library is general document tooling,
  asset/import infrastructure, or a provider implementation;
- an ADR if promotion changes an accepted workspace or ownership boundary.

## References

- `corpus/lib/ui-tools/src/svg.rs`
- `corpus/lib/presentation-geometry-corpus/`
- `docs/Plans/presentation-geometry-corpus-harness.md`
- `docs/Architectural Reviews/AR-0001-shared-vector-presentation-geometry.md`
- `docs/Conversations/xml corpus.md`
- `.workbench/Todos/svg-xml-tools-pipeline.md`
- `docs/testing-strategy.md`
- `docs/example-philosophy.md`
- `corpus/README.md`
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
