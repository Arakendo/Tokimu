---
title: W3C SVG Evidence
description: A bounded account of Tokimu's current W3C SVG presentation-geometry corpus.
---

<p class="page-kicker">Formats / SVG / corpus evidence</p>

# SVG geometry under independent pressure

Tokimu uses a deliberately selected subset of the W3C SVG 1.1 Second Edition
test suite to pressure XML parsing, SVG lowering, provider-neutral vector
geometry, tessellation, and diagnostics.

<div class="evidence-banner">
  <div>
    <span class="evidence-label">Current state</span>
    <strong>Renderable geometry profile</strong>
  </div>
  <div>
    <span class="evidence-label">Official upstream coverage</span>
    <strong data-evidence-coverage>40 / 525 · 7.62%</strong>
  </div>
  <div>
    <span class="evidence-label">Evidence date</span>
    <strong><time datetime="2026-07-28">2026-07-28</time></strong>
  </div>
</div>

This percentage means that the selection manifest represents **40 unique
upstream conformance documents** out of the **525-document conformance
denominator**. It does not mean that Tokimu is 7.62% compliant with SVG 1.1.

## Evidence ledger

| Measure | Current evidence | What it means |
| --- | ---: | --- |
| Unique upstream documents | 40 / 525 | The official coverage denominator |
| Selection manifest entries | 62 | 16 source entries and 46 derived entries |
| Registered SVG runner cases | 50 | 47 retain upstream provenance; 3 are local diagnostics |
| Reviewed structural goldens | 60 / 60 | All producers, not only SVG |

!!! note "Evidence identity"
    This page is checked against
    `docs/Libraries/w3c-svg-corpus-testing.md`. The source record was last
    updated on `2026-07-28` and is represented here from source revision
    [`7fd812e`](https://github.com/Arakendo/Tokimu/commit/7fd812e3f878bbc5cc1655304e9ff05ab9d07591).
    If these values drift, website validation fails rather than silently
    publishing an older claim.

## Artifact entry points

The public summary remains traceable to the repository evidence that produced
it:

- [authoritative W3C SVG corpus record](https://github.com/Arakendo/Tokimu/blob/main/docs/Libraries/w3c-svg-corpus-testing.md);
- [versioned fixture selection](https://github.com/Arakendo/Tokimu/blob/main/third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/selection-v1.toml);
- [registered W3C SVG corpus cases](https://github.com/Arakendo/Tokimu/blob/main/corpus/lib/presentation-geometry-corpus/src/w3c_svg_cases.rs);
- [structural golden workflow](https://github.com/Arakendo/Tokimu/blob/main/corpus/lib/presentation-geometry-corpus/src/golden_workflow.rs); and
- [fixture provenance](https://github.com/Arakendo/Tokimu/blob/main/third-party/fixtures/w3c-svg-1.1-2nd-edition/provenance.json).

These links expose inputs and structural validation machinery. Native-window
screenshots remain separately labeled manual evidence and are not substituted
for topology, mesh, bounds, or fingerprint artifacts.

## What reaches presentation geometry

The admitted profile currently exercises:

- absolute, relative, repeated, and close path commands;
- quadratic and cubic curves, smooth curves, and bounded arc evidence;
- lines, polylines, polygons, rectangles, circles, and ellipses;
- multiple contours and even-odd or non-zero fill intent;
- nested groups and selected presentation inheritance;
- transform composition and transform order;
- bounded solid and dashed stroke evidence;
- parser-neutral XML, vector, mesh, and fingerprint artifacts.

These are **WASM semantic and deterministic structural claims**. They describe
Tokimu-owned observations and geometry artifacts, not browser-native SVG
rendering.

## Evidence types

<div class="card-grid evidence-type-grid">
  <article class="feature-card">
    <span class="card-index">01</span>
    <h3>Structural</h3>
    <p>Outline, vector, mesh, topology, bounds, and fingerprint artifacts are authoritative for structural claims.</p>
  </article>
  <article class="feature-card">
    <span class="card-index">02</span>
    <h3>Deterministic CPU</h3>
    <p>Saved CPU images provide reproducible visual evidence without claiming GPU framebuffer equivalence.</p>
  </article>
  <article class="feature-card">
    <span class="card-index">03</span>
    <h3>WASM semantic</h3>
    <p>The website island invokes Tokimu's Rust/WASM importer and presents its provider-neutral observation.</p>
  </article>
  <article class="feature-card">
    <span class="card-index">04</span>
    <h3>Browser visual</h3>
    <p>Canvas output helps human inspection, but remains complementary visual evidence rather than semantic truth.</p>
  </article>
</div>

## Known exclusions

The current profile does not claim:

- complete SVG 1.1 or browser conformance;
- gradients, masks, filters, animation, DOM, or scripting;
- SVG text and font rendering;
- general nested, concave, multi-contour, or stroke clip paths;
- complete transformed or non-uniform stroke-width conformance;
- full W3C reference-image comparison.

Unsupported semantics should stop at an explicit diagnostic boundary. They are
not silently delegated to the browser and are not counted as passes.

## Reproduce the evidence

Verify the pinned fixture set:

```powershell
pwsh -NoProfile -File .\scripts\verify-w3c-svg-fixtures.ps1
```

Run structural tests:

```powershell
cargo test -p presentation-geometry-corpus --lib
```

Compare all reviewed structural goldens without rewriting them:

```powershell
cargo run -p presentation-geometry-corpus -- compare-all
```

The next highest-return pressure is transformed stroke behavior, broader
clipping, fill-rule edge cases, viewport sizing, and explicit paint-server
boundaries.
