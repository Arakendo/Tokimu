# W3C XML Corpus Testing

## Status

Active bounded-profile evidence as of 2026-07-28. The W3C XML 2013-09-23
archive is checksum-pinned, its 3,078 XML files verify locally, and a four-case
v1 selection exercises accepted, rejected, deferred, and explicitly
unsupported behavior. `xml-tools` currently passes 22 focused library tests.
`hello-xml-inspect` provides a second example-side consumer, and AR-0003
retains the parser/document boundary in incubation rather than promoting it.

This is not a claim of complete XML, namespace, DTD, XSD, XPath, or XSLT
conformance.

## Purpose

Use independent W3C fixtures to validate the parser-neutral XML boundary used
by SVG and future document consumers:

```text
source bytes
    -> bounded parser adapter
    -> parser-neutral events
    -> immutable document
    -> consumer-owned semantics
```

XML owns syntax, well-formedness, expanded names, source spans, bounded
resource handling, and XML diagnostics. SVG and other consumers own their
document semantics.

## Goals

- Preserve a pinned W3C source and explicit profile selection.
- Validate deterministic events, immutable documents, diagnostics, and limits.
- Keep XML syntax independent from SVG and other consumer semantics.
- Expand standards evidence in small requirement-oriented batches.
- Keep ordinary validation offline, headless, and bounded.

## Non-Goals

- Complete XML or XML-family conformance.
- DTD validation or external entity resolution in the initial profile.
- Treating XPath, XSD, or XSLT as automatic consequences of XML parsing.
- Building a browser DOM or mutable document framework.
- Promoting `xml-tools` solely because one standards archive is available.

## Current Coverage

| Measure | Count | Percentage | Meaning |
| --- | ---: | ---: | --- |
| Upstream XML files | 3,078 | Denominator | Pinned archive inventory |
| Selected profile cases | 4 / 3,078 | **0.13%** | Source-file coverage, not conformance |
| Accepted parser cases | 1 | Not a coverage percentage | UTF-8 event/document proof |
| Rejected cases | 2 | Not a coverage percentage | Malformed end tag and deferred QName diagnostic |
| Unsupported-by-profile cases | 1 | Not a coverage percentage | UTF-16 is diagnosed before parser adaptation |
| Focused `xml-tools` tests | 22 / 22 | **100% of current tests** | Contract and regression status |

The selection is intentionally tiny. Its purpose is to prove stable
classification and diagnostics before broad standards coverage.

## Fixtures

```text
third-party/fixtures/w3c-xml-20130923/
    provenance.json
    upstream/
    selected/
        selection-v1.toml
        feature-matrix.md
```

The archive and upstream tree remain authoritative. Selection metadata
references upstream paths rather than duplicating them.

## Ownership And Scope

`xml-tools` owns:

- bounded source ingestion;
- parser-neutral event and document contracts;
- namespace-expanded names;
- source identity and half-open spans;
- deterministic resource-limit and syntax diagnostics.

Consumers own:

- SVG, XSD, XPath, XSLT, or application semantics;
- external-resource policy;
- semantic validation and lowering.

The initial profile deliberately excludes DTD processing, external entities,
external resource resolution, and non-UTF-8 parser adaptation. Those
boundaries must remain explicit rather than silently recovered.

## Validation

```powershell
pwsh -NoProfile -File .\scripts\verify-w3c-xml-fixtures.ps1
cargo test -p xml-tools
```

Validation succeeds when archive identity, selected paths, classifications,
expected diagnostics, and focused parser/document tests all agree.

## Implementation Slices

### Slice 0: Acquire And Classify

Deliverables:

- [x] Pin archive provenance and checksum.
- [x] Preserve upstream fixtures.
- [x] Create a feature matrix and selected profile.
- [x] Verify selected paths offline.

Acceptance criteria:

- [x] Preparation and ordinary validation are separate.
- [x] Every selected case has one expected classification.
- [x] Unsupported behavior is not reported as a parser regression.

### Slice 1: Prove The Bounded Parser Boundary

Deliverables:

- [x] Parse one accepted UTF-8 fixture.
- [x] Reject one malformed fixture with stable source identity.
- [x] Diagnose one unsupported encoding before parser adaptation.
- [x] Preserve parser-neutral events and immutable document evidence.

Acceptance criteria:

- [x] Tests require no renderer, window, GPU, or network.
- [x] Resource limits and malformed input cannot panic.
- [x] Consumer semantics do not leak into XML contracts.

### Slice 2: Expand Standards Evidence

Deliverables:

- [ ] Inventory W3C case classifications and requirement dependencies.
- [ ] Add small batches for namespace validity, character references,
      declarations, CDATA, processing instructions, and malformed boundaries.
- [ ] Separate cases requiring DTD validation from standalone
      well-formedness cases.
- [ ] Report accepted, rejected, unsupported, and deferred results separately.

Acceptance criteria:

- [ ] Every batch names the XML behavior and likely diagnostic boundary.
- [ ] Coverage denominators reproduce from the pinned inventory.
- [ ] Passing counts are not substituted for conformance.
- [ ] Broad runs remain bounded and filterable.

### Slice 3: Review Additional Consumers

Deliverables:

- [x] Add a second independent consumer before promoting stable XML contracts.
- [x] Evaluate XSD, XPath, and XSLT as separate extension tracks.
- [x] Record whether the parser/document boundary survives independent use.

Acceptance criteria:

- [x] No XML-family standard is admitted merely because XML parsing exists.
- [x] Extension technologies remain replaceable and consumer-driven.
- [x] Architectural Review records the current incubation disposition.

Progress: `examples/hello-xml-inspect` consumes only the parser-neutral
document and diagnostic contracts. `AR-0003-xml-document-boundary.md` records
that two example-side consumers preserve the boundary but do not yet justify a
first-party crate.

## Highest-Return Next Targets

1. Reproducible classification inventory for the pinned archive.
2. Namespace-name validity and undeclared-prefix batches.
3. Character/reference boundary and malformed UTF-8 cases.
4. Standalone well-formedness cases that do not require DTD processing.
5. Explicit DTD-dependent exclusions before any validation work.

## Completion Criteria

The bounded XML corpus is healthy when fixture identity verifies offline,
selected expectations are deterministic, parser and document artifacts remain
consumer-neutral, unsupported features are explicit, and coverage is reported
against the pinned archive rather than test pass counts.

## Graduation Criteria

Promotion beyond `examples/lib-example/xml-tools` requires:

- a non-example consumer that needs the parser-neutral contracts;
- stable lifecycle, diagnostics, limits, and source-identity semantics;
- no SVG, XPath, XSD, XSLT, filesystem, or parser-backend concepts in the base
  XML contract;
- evidence that extraction simplifies ownership compared with continued
  incubation;
- Architectural Review explicitly recommending first-party admission.

The existing two example-side consumers preserve the boundary but do not meet
that promotion threshold.

## References

- `docs/Libraries/README.md`
- `docs/Plans/xml-tools.md`
- `docs/Architectural Reviews/AR-0003-xml-document-boundary.md`
- `docs/testing-strategy.md`
- `third-party/fixtures/w3c-xml-20130923/README.md`
- `third-party/fixtures/w3c-xml-20130923/selected/selection-v1.toml`
- `third-party/fixtures/w3c-xml-20130923/selected/feature-matrix.md`
