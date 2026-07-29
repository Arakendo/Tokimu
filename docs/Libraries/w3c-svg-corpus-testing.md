# W3C SVG Corpus Testing

## Status

Active and structurally validated as of 2026-07-28. The pinned archive,
selection manifest, derived fixtures, focused tests, and all 60 presentation
geometry goldens verify locally. The manifest contains 62 entries representing
40 unique upstream conformance documents; 50 SVG cases are currently
registered in the runner. This remains a bounded geometry profile, not SVG 1.1
conformance.

## Purpose

Tokimu uses a deliberately selected subset of the W3C SVG 1.1 Second Edition
test suite as a presentation-geometry corpus. The corpus exercises the SVG
pipeline at stable ownership boundaries:

```text
W3C SVG fixture
    -> XML parsing and SVG lowering
    -> renderer-neutral vector geometry
    -> fill or stroke evidence
    -> mesh and diagnostic artifacts
```

This is an engineering corpus, not a claim of complete W3C conformance. Its
purpose is to expose parser, lowering, topology, tessellation, and renderer
boundary problems using independent, standards-based input.

## Goals

- Preserve a pinned, verifiable standards corpus.
- Add high-return geometry cases in small, diagnosable batches.
- Keep XML, SVG semantics, vector geometry, and rendering failures distinct.
- Validate structural artifacts before relying on pixels.
- Report source coverage, selected cases, and runner status separately.

## Non-Goals

- Complete SVG 1.1 browser or W3C conformance.
- Making SVG Tokimu's canonical vector model.
- Treating reference images as structural truth.
- Admitting DOM, scripting, animation, text, filters, or every paint feature
  through geometry work.
- Bulk-registering fixtures whose failures cannot be localized.

## Upstream Fixtures

The verbatim upstream fixture set is stored at:

```text
third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/
```

The preserved archive contains **1,139 SVG files** across its conformance
documents, harness, resources, and support material. The coverage denominator
is the **525 conformance SVG documents** under `upstream/svg`.

The selected cases and their provenance are recorded in:

```text
third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/selection-v1.toml
```

The feature-level scope is summarized in:

```text
third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/feature-matrix.md
```

The upstream copy remains intact. Derived fixtures are intentionally reduced
to the geometry needed to test a capability, so unsupported document-level
features do not get mistaken for geometry failures.

Archive identity and W3C copyright material are recorded in
`provenance.json` and `LICENSES/`. Redistribution must preserve that material;
the existence of a public test URL is not treated as a license substitute.

## Current Coverage

As of 2026-07-28:

| Measure | Count | Percentage | Meaning |
| --- | ---: | ---: | --- |
| Unique upstream conformance documents represented by the manifest | 40 / 525 | **7.62%** | The clean upstream fixture-coverage metric |
| Selection manifest entries | 62 | Not a coverage percentage | 16 source entries and 46 derived entries |
| Registered SVG runner cases | 50 | Not a coverage percentage | 47 cases retain upstream provenance and 3 are local derived diagnostics |
| Registered structural goldens, all producers | 60 / 60 | **100% of reviewed goldens** | Includes glyph, synthetic, Lucide, SVG, and UI cases |

The **7.62% figure is the official progress number** for upstream fixture
coverage. Manifest entries, runner cases, and passing goldens are useful
operational measures, but none may be substituted for unique upstream
coverage.

The current registry is defined in:

```text
corpus/lib/presentation-geometry-corpus/src/lib.rs
```

## What We Test Today

The current selection prioritizes high-return geometry and pipeline features:

- move, line, horizontal, and vertical path commands;
- relative and repeated path commands;
- quadratic, cubic, and smooth curves;
- elliptical arcs as focused diagnostic evidence;
- close paths and multiple contours;
- polygons, polylines, lines, rectangles, rounded rectangles, circles, and ellipses;
- even-odd and non-zero fill evidence;
- nested groups and presentation inheritance;
- inherited fill, stroke, and stroke-width overrides through nested groups;
- `currentColor` fill resolution through inherited and local `color` values;
- in-range stroke-opacity resolution as renderer-independent paint intent;
- transform composition and transform order;
- open-path stroke intent, without claiming complete stroke expansion;
- parser-neutral XML, vector, mesh, and mesh-fingerprint artifacts where the
  selected case reaches those stages.

The feature matrix is the authoritative description of current capability
status. A case that reaches a stage with a recorded limitation is evidence of
that boundary, not automatically a passing implementation of the full SVG
feature.

## Case Categories

The corpus keeps several kinds of evidence distinct:

### Verbatim source exclusions

These preserve complete upstream XML when a document includes semantics that
are not currently admitted, such as broader test-suite behavior. They are
useful for provenance and parser inspection but do not claim a complete
conformance result.

### Derived geometry fixtures

These retain only the source geometry needed to exercise a named capability.
They are the primary way to test vector lowering and mesh production while
keeping unsupported document semantics out of the result.

### Local diagnostic fixtures

These are small fixtures created to isolate a geometry behavior discovered by
the corpus. They count as registered cases but do not increase upstream W3C
document coverage.

## Validation Command

Verify archive identity, upstream references, and derived fixture presence:

```powershell
pwsh -NoProfile -File .\scripts\verify-w3c-svg-fixtures.ps1
```

Run the corpus library tests with:

```powershell
cargo test -p presentation-geometry-corpus --lib
```

The current library test result is:

```text
14 passed; 0 failed
```

Build and run the corpus example when visual or artifact output is needed:

```powershell
cargo run -p presentation-geometry-corpus
```

Compare every reviewed structural golden without rewriting it:

```powershell
cargo run -p presentation-geometry-corpus -- compare-all
```

Structural artifacts are authoritative for geometry validation. Saved CPU
images and native-window screenshots are complementary evidence and should be
labeled separately; they do not establish GPU framebuffer equivalence or W3C
conformance by themselves.

## Coverage Accounting

When updating the corpus, report all of the following separately:

1. Total upstream SVG documents available.
2. Unique upstream documents represented by the selection manifest.
3. Registered corpus cases.
4. Derived-local cases that do not map to upstream documents.
5. Feature capability status from the feature matrix.
6. Tests and cases that passed, failed, or stopped at an explicit unsupported
   boundary.

The basic calculation is:

```text
unique upstream documents represented
------------------------------------- x 100
total upstream SVG documents
```

For the current selection:

```text
40 / 525 x 100 = 7.62%
```

Do not use the number of passing cases as a substitute for coverage. A larger
case count can represent more feature evidence without representing more
upstream documents.

## Current Exclusions

The following remain outside the current structural geometry profile or are
explicitly deferred:

- nested, concave, multi-contour, and stroke clip paths; the current profile
  admits one local convex geometric clip for closed fill geometry;
- gradients;
- masks;
- filters;
- text and font rendering as SVG document features;
- animation, DOM, and scripting;
- transformed/non-uniform-viewport stroke-width conformance; the current
  profile admits bounded solid and dashed strokes with butt/round/square caps
  and miter/bevel/round joins;
- full W3C reference-image comparison.

An exclusion is intentional when the corpus records why the feature is not
admitted yet. Unsupported behavior should produce a diagnostic boundary rather
than a silent fallback or an implied pass.

## How To Expand The Corpus

Add cases in small feature-oriented batches:

1. Select independent upstream documents with high diagnostic value.
2. Record the source ID, capability, reason, and expected boundary in the
   selection manifest.
3. Preserve the upstream source and provenance.
4. Create a derived fixture only when document-level semantics would obscure
   the geometry question.
5. Register the case in the presentation geometry corpus.
6. Add or update the feature matrix.
7. Run structural tests and inspect artifacts before relying on pixels.
8. Recalculate unique upstream coverage.

The next highest-return areas are stroke expansion, clipping, fill-rule edge
cases, arc and curve stress cases, and broader transform combinations. Large
bulk imports should wait until the selected cases demonstrate that the
diagnostics remain actionable.

## Architectural Boundary

The corpus runner remains an example-side diagnostic consumer of `ui-tools`.
It does not create a new vector or renderer capability by itself.

The intended ownership remains:

```text
SVG/XML tools
    own parsing and SVG semantics

Presentation geometry
    owns provider-neutral paths, topology, and tessellation evidence

Renderer
    owns GPU execution, uploads, batching, and cache lifetime
```

If repeated independent consumers demonstrate that a boundary is stable, the
finding can proceed through architectural review before any capability is
promoted or extracted into a crate.

## Completion Criteria

The current corpus profile is considered healthy when:

- the archive hash and all manifest references verify offline;
- every admitted source, derived, and local case has an explicit classification;
- structural tests and reviewed goldens pass without rewriting artifacts;
- unsupported SVG semantics stop at a named diagnostic boundary;
- unique upstream coverage, manifest entries, and runner cases are reported
  separately;
- ordinary validation requires no network, window, or GPU.

## Next Coverage Targets

Highest-return additions remain:

1. transformed and non-uniform stroke-width behavior;
2. nested and multi-contour clipping;
3. more independent fill-rule and self-intersection documents;
4. viewport units, physical sizing, and preserve-aspect-ratio combinations;
5. paint-server boundaries such as gradients, admitted only after geometry and
   compositing ownership are explicit.

## Graduation Criteria

The SVG adapter or shared geometry extracted from this corpus may graduate
beyond example-side incubation only when:

- at least one non-example consumer needs the stable contract;
- another independent geometry producer preserves the same ownership boundary;
- SVG document semantics and XML parser types remain outside vector and
  renderer APIs;
- diagnostics, lifecycle, and unsupported-feature policy are stable;
- Architectural Review explicitly recommends admission.

The W3C corpus validates the boundary but cannot admit a capability by itself.

## References

- `docs/Libraries/README.md`
- `docs/Plans/presentation-geometry-corpus-harness.md`
- `docs/Plans/xml-tools.md`
- `docs/Architectural Reviews/AR-0001-shared-vector-presentation-geometry.md`
- `third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/selection-v1.toml`
- `third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/feature-matrix.md`
