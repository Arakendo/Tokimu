# AR-0003: XML Document Boundary

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-07-24 |
| Last reviewed | 2026-07-24 |
| Scope | Foundational import/document capability boundary |
| Trigger | SVG migration and an independent XML inspection consumer now share bounded XML semantics |
| Related ADRs | ADR-0003, ADR-0005 |
| Related evidence | `xml-tools` tests, `hello-svg`, `hello-xml-inspect`, W3C XML selection runner |
| Admission exception | None |

## Architectural Question

Should parser-neutral, bounded XML event, document, namespace, span, and
diagnostic semantics graduate from `corpus/lib/xml-tools` into a
first-party Tokimu capability, or remain an example-side provider boundary
until stronger independent consumer evidence exists?

## Context

The SVG importer previously owned hand-written document scanning. The
incubating `xml-tools` library now provides bounded UTF-8 XML parsing,
parser-neutral events, an immutable retained document, expanded names,
source spans, and structured diagnostics. Its selected parser backend remains
private.

Two consumers now exercise the public contracts:

```text
SVG importer
    XML document -> SVG meaning -> vector geometry

hello-xml-inspect
    XML document -> terminal inspection output
```

The consumers share XML syntax and document semantics, but not SVG semantics,
filesystem ownership, terminal formatting, rendering, or engine state.

## Trigger And Evidence

- Corpus examples: `hello-svg` uses `xml-tools` through the UI SVG importer;
  `hello-xml-inspect` traverses immutable documents independently of SVG.
- Automated tests: 22 `xml-tools` unit tests cover events, document retention,
  namespace expansion, diagnostics, limits, hostile structure, and seeded
  well-formed inputs.
- Standards evidence: the reviewed W3C XML v1 selection runner distinguishes
  accepted, rejected, unsupported, and deferred cases; it is run explicitly
  rather than silently rewriting expectations.
- Portability evidence: `xml-tools` compiles for `wasm32-unknown-unknown` on
  the same Rust path.
- Repeated implementation friction: the SVG importer no longer needs to own
  XML nesting, namespace, entity, or document diagnostics.
- Missing evidence: a non-example consumer, runtime WASM execution, a reviewed
  differential-parser comparison, and an explicit decision about whether XML
  belongs under general documents, import/assets, or a provider-specific
  boundary.

## Ownership Analysis

XML syntax, bounded ingestion, source-scoped spans, expanded names, retained
document traversal, and parser-neutral diagnostics are the potential shared
meaning. Today `xml-tools` owns that meaning only inside example-side
incubation.

Consumers own their own interpretation and execution:

- SVG owns SVG elements, style, transforms, and vector lowering.
- `hello-xml-inspect` owns command-line arguments, filesystem input, and
  terminal presentation.

The XML layer must not own SVG semantics, filesystem access, browser APIs,
renderer state, simulation truth, parser-native objects, schema validation, or
XPath behavior.

## Dependency Direction

```text
Current:
SVG importer ----------> xml-tools ----------> private quick-xml adapter
hello-xml-inspect -----> xml-tools ----------> private quick-xml adapter

Possible future capability:
SVG/import services ----> tokimu-xml contracts ----> replaceable XML providers
inspection/tooling -----> tokimu-xml contracts ----> replaceable XML providers
```

Neither direction permits `tokimu-core`, renderer, platform, or filesystem
dependencies to enter the XML semantic layer.

## Alternatives Considered

### A: Promote XML Semantics Immediately

- Benefits: stable package location and an explicit shared contract.
- Costs: freezes a document boundary before a non-example consumer and runtime
  WASM execution have validated it.
- Failure mode: premature admission turns an importer implementation into a
  permanent engine subsystem.

### B: Keep XML Logic Local To SVG

- Benefits: no provisional shared abstraction.
- Costs: repeats syntax, namespace, limit, and diagnostic behavior for every
  XML consumer.
- Failure mode: future consumers fork subtly incompatible document behavior.

### C: Continue Example-Side Incubation

- Benefits: retains parser-neutral contracts and two real consumers while
  keeping package extraction reversible.
- Costs: temporary support-library location and deliberate review follow-up.
- Failure mode: the library quietly grows into general document tooling without
  renewed architectural review.

## Findings

The evidence supports a shared XML document boundary for continued incubation.
The second consumer confirms that retained traversal, expanded names, spans,
and diagnostics are useful independently of SVG meaning. The implementation
also preserves the intended dependency direction: no parser-backend types,
filesystem APIs, renderer state, or SVG concepts cross the `xml-tools` public
boundary.

The evidence does not yet support a first-party crate. Both consumers remain
examples, no runtime WASM XML corpus has executed, and neither XSD nor XPath
has a named consumer. This review does not admit a general document model.

## Disposition

Incubating. Keep `xml-tools` in `corpus/lib` and use its
parser-neutral contracts for additional concrete import or tooling consumers.
Do not extract a crate or alter accepted engine boundaries until the missing
evidence is intentionally reviewed.

## Consequences

Existing SVG and inspection work may share bounded XML behavior without
duplicating parser details. Future consumers must preserve the same boundary:
application or importer code owns source acquisition and domain interpretation;
the XML layer owns only syntax/document semantics and diagnostics. XSD and XPath
remain separately scoped extension tracks.

## Required Follow-Up

- [x] Documentation or review record
- [x] Focused implementation slice
- [x] Corpus example or automated test
- [ ] Migration, retirement, or compatibility work

## Reopening Triggers

- a non-example importer, asset pipeline, or tooling consumer requires the
  same XML document contracts;
- runtime WASM execution cannot preserve the selected profile or diagnostics;
- parser-provider details leak through a proposed public API;
- a real consumer requires XSD, XPath, mutation, non-UTF-8 input, or external
  resource resolution;
- two consumers require incompatible retained-document semantics;
- the differential-parser or hostile-input corpus invalidates a stated
  diagnostic or resource-limit guarantee.

## Review History

### Cycle 1 -- 2026-07-24

- Status entering review: Proposed
- New evidence: SVG migration, selected W3C XML runner, and the independent
  `hello-xml-inspect` retained-document consumer.
- Participants or reviewers: Codex working review
- Findings: the semantic boundary is useful and provider-neutral; promotion is
  premature because all current consumers remain example-side and runtime WASM
  behavior is unproven.
- Disposition: Incubating
- Resulting ADR or documentation change: no ADR change; Slice 6 of the XML
  tools plan is completed with an explicit incubation disposition.

### Cycle 2 -- 2026-07-24

- Status entering review: Incubating
- New evidence: the SVG importer now lowers parser-neutral `XmlEvent` values
  directly, and the Lucide, synthetic SVG, and W3C corpus artifact paths reuse
  their existing XML inspection rather than reparsing source text to recreate
  XML evidence.
- Findings: XML syntax remains isolated to `xml-tools`; SVG retains only its
  namespace/profile, presentation, numeric, transform, viewport, and path-data
  semantics. Expected SVG-profile exclusions now preserve W3C source/XML
  provenance without implying unsupported `defs` or text support.
- Disposition: Incubating, unchanged
- Resulting ADR or documentation change: no ADR change. The migration closes
  transitional XML scanning but does not add a non-example consumer or change
  the evidence needed for capability admission.

## References

- `docs/Plans/xml-tools.md`
- `corpus/lib/xml-tools/DESIGN.md`
- `corpus/lib/xml-tools/src/lib.rs`
- `corpus/lib/xml-tools/tests/w3c_selection.rs`
- `corpus/lib/ui-tools/src/svg.rs`
- `corpus/hello-xml-inspect/`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
