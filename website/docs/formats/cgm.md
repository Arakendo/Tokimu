---
title: CGM Evidence
description: A bounded account of Tokimu's current WebCGM inspection and presentation-geometry corpus.
---

<p class="page-kicker">Formats / CGM / corpus evidence</p>

# Stateful graphics under independent pressure

Tokimu uses a 26-case selection from the WebCGM 2.1 Test Suite to pressure
binary framing, metafile and picture lifecycle, source-ordered graphics state,
primitive lowering, and explicit unsupported boundaries.

<div class="evidence-banner">
  <div>
    <span class="evidence-label">Current state</span>
    <strong>Previewable</strong>
  </div>
  <div>
    <span class="evidence-label">Selected fixtures</span>
    <strong data-evidence-cgm-selection>26 / 26 verified</strong>
  </div>
  <div>
    <span class="evidence-label">Evidence date</span>
    <strong><time datetime="2026-07-28">2026-07-28</time></strong>
  </div>
</div>

Previewable is intentionally narrower than renderable. Tokimu can inspect all
selected cases and lower a bounded set of geometry, but it does not yet claim
general CGM paint, fill, edge, palette, text, clipping, or profile conformance.

## Evidence ledger

| Measure | Current evidence | What it means |
| --- | ---: | --- |
| Selected fixtures verified | 26 / 26 | The pinned v1 selection verifies offline |
| Cases registered in the shared runner | 26 / 26 | Every selected case stops at an explicit stage |
| Successful source-to-vector cases | 13 | Finite provider-neutral path evidence exists |
| Expected vector boundary | 1 | Polygon-set topology is retained as unsupported |
| Source-only cases | 12 | Lifecycle, descriptors, text, clipping, curves, and raster evidence stop before vector lowering |
| Admitted production importer | 0 | The importer remains corpus-owned incubation |

!!! note "Evidence identity"
    This page is checked against
    `docs/Libraries/cgm-corpus-testing.md`. The authoritative record is dated
    `2026-07-28`; website validation fails if the bounded selection and stage
    counts drift.

## What the current profile proves

The selected corpus currently demonstrates:

- bounded binary CGM element framing;
- metafile, picture, and picture-body lifecycle inspection;
- integer VDC type and precision observations;
- source-ordered line, fill, edge, color, and clipping state retention;
- provider-neutral paths for selected line, polygon, rectangle, circle,
  ellipse, circular-arc, and elliptical-arc primitives;
- explicit expected boundaries for unsupported polygon-set topology; and
- deterministic source, vector, graph, and diagnostic artifacts at the stages
  that honestly produce them.

The browser asset workbench consumes the same provider-neutral observation and
lowering contracts. Its CGM canvas is diagnostic outline evidence, not a
browser-native CGM renderer and not proof of resolved source paint.

## Artifact entry points

- [authoritative CGM corpus record](https://github.com/Arakendo/Tokimu/blob/main/docs/Libraries/cgm-corpus-testing.md);
- [versioned WebCGM fixture selection](https://github.com/Arakendo/Tokimu/blob/main/third-party/fixtures/webcgm-test-suite/selected/selection-v1.toml);
- [CGM corpus cases](https://github.com/Arakendo/Tokimu/blob/main/corpus/lib/presentation-geometry-corpus/src/cgm_cases.rs);
- [CGM artifact writer](https://github.com/Arakendo/Tokimu/blob/main/corpus/lib/presentation-geometry-corpus/src/cgm_artifacts.rs); and
- [visible native CGM consumer](https://github.com/Arakendo/Tokimu/tree/main/corpus/hello-cgm).

## Known exclusions

The current profile does not claim:

- complete CGM, WebCGM, CALS, or ISO conformance;
- admitted CGM fill, edge, color, palette, bundle, or text semantics;
- resolved CGM standard defaults;
- general clipping or coordinate-normalization behavior;
- mesh or reference-image equivalence; or
- a first-party production CGM importer.

Deferred source elements remain named and counted. Unsupported semantics are
not silently omitted, guessed from a reference image, or delegated to browser
presentation.

## Reproduce the evidence

Verify the pinned fixtures:

```powershell
pwsh -NoProfile -File .\scripts\verify-webcgm-corpus.ps1
```

Run the focused corpus tests:

```powershell
cargo test -p cgm-corpus
```

Run the shared presentation-geometry cases:

```powershell
cargo test -p presentation-geometry-corpus --lib
```

The next high-value pressure is source paint resolution, text primitives,
palette and color selection, clipping semantics, and honest mesh or saved-image
evidence after those source meanings are resolved.
