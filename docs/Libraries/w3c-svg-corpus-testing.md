# W3C SVG Corpus Testing

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

## Upstream Fixtures

The verbatim upstream fixture set is stored at:

```text
third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/
```

The current upstream archive contains **1,139 SVG documents**.

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

## Current Coverage

As of 2026-07-27:

| Measure | Count | Percentage | Meaning |
| --- | ---: | ---: | --- |
| Unique upstream SVG documents represented | 32 / 1,139 | **2.81%** | The clean fixture-coverage metric |
| Registered W3C-labelled corpus cases | 40 / 1,139 | **3.51%** | Includes derived cases and four local diagnostic cases; not a unique-file metric |

The **2.81% figure is the official progress number** for upstream fixture
coverage. The 40 registered cases are useful for tracking the corpus runner,
but they must not be reported as 40 unique upstream documents because some
cases are derived from the same source and some are local derived fixtures.

The current registry is defined in:

```text
examples/lib-example/presentation-geometry-corpus/src/lib.rs
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
32 / 1,139 x 100 = 2.81%
```

Do not use the number of passing cases as a substitute for coverage. A larger
case count can represent more feature evidence without representing more
upstream documents.

## Current Exclusions

The following remain outside the current structural geometry profile or are
explicitly deferred:

- clip paths;
- gradients;
- masks;
- filters;
- text and font rendering as SVG document features;
- animation, DOM, and scripting;
- complete stroke expansion, cap, and join behavior;
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
