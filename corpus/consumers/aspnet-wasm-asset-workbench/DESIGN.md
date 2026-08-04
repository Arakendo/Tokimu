# ASP.NET WASM Asset Workbench

## Purpose

This consumer corpus entry validates Tokimu as a WebAssembly library consumed
by an ASP.NET 10 application with a TypeScript-authored browser interface.

The application accepts dropped SVG, CGM, glTF/GLB, and FBX files, transfers
their bytes into Rust/WASM, and displays importer observations without sending
the source file to the server.

## Primary Composition Claim

An ordinary web application can consume Tokimu through a bounded WASM API
without duplicating importer semantics in TypeScript or making ASP.NET own
engine state.

```text
Browser File
    |
    v
TypeScript interaction shell
    |
    v
Rust/WASM byte boundary
    |
    v
Tokimu and incubating format adapters
    |
    v
Provider-neutral observation / preview
    |
    v
Canvas and inspector presentation
```

## Composition Acceptance Checks

The consumer composition remains valid only when this boundary holds:

```text
Browser bytes
    |
    v
Rust/WASM importer
    |
    v
Provider-neutral observation
    |
    v
TypeScript presentation
```

At no point may:

- TypeScript parse a source format to complete inspection or preview;
- ASP.NET own importer state or provider-native document objects;
- browser presentation redefine importer semantics; or
- a browser-native importer or renderer silently substitute for a Tokimu
  observation that is pending or unsupported.

`Pending` is an explicit diagnostic, not a silent fallback.

## Consumer Tier

**Tier 2: incubating consumer.**

The WASM adapter consumes the public `tokimu` facade and these incubating
corpus libraries:

- `ui-tools` for SVG-to-vector lowering;
- `cgm-corpus` for bounded binary CGM inspection and vector lowering;
- `gltf-corpus` for glTF and GLB structural inspection;
- `fbx-corpus` for bounded ASCII and binary FBX record inspection.

These libraries are evidence, not stable public Tokimu importer contracts.

## Ownership

- ASP.NET owns static hosting and fallback routing.
- TypeScript owns DOM state, drag/drop, file selection, and inspector
  presentation.
- Rust/WASM owns bounded input validation, format classification, importer
  invocation, and provider-neutral result construction.
- Format adapters own source-format syntax and diagnostics.
- Tokimu owns engine and capability semantics.
- Canvas rendering owns pixels and must not redefine source-format semantics.

Application state consists of the selected file and its current observation.
Provider-native document objects do not cross the WASM boundary.

## Presentation Interaction Boundary

The workbench can apply a bounded presentation request to a selected
provider-neutral preview target. TypeScript communicates interaction intent;
the WASM presentation session resolves layered visual state and returns the
result for the canvas to draw.

```text
TypeScript target + intent
    |
    v
WASM presentation session
    |
    v
Resolved provider-neutral presentation
    |
    v
Canvas pixels
```

The ordinary tint, opacity, and visibility controls write the `application`
layer. The hotspot control writes the higher-priority `hotspot` layer so a
user can identify a mesh or vector target without replacing the ordinary
application state. Each control clears only the layer it owns. The browser
does not compute precedence, mutate imported source data, or compile shader
code.

## Initial Support

| Format | Inspection | Preview |
| --- | --- | --- |
| SVG | XML/SVG lowering and contour statistics | Provider-neutral contours |
| CGM | Binary structure, pictures, elements, diagnostics | First admitted picture contours |
| glTF | JSON structure and scene summary | Pending scene/mesh consumer boundary |
| GLB | Container chunks, scene summary, and decoded triangle primitives | Provider-neutral interactive perspective triangle preview |
| FBX | ASCII or binary bounded record graph plus static geometry lowering | Provider-neutral interactive perspective static-triangle preview |

“Pending” is an explicit diagnostic, not silent fallback to a browser-native
renderer.

## Inputs And Outputs

Inputs:

- one local file, or a same-folder multi-file selection containing a `.gltf`
  document and its declared external buffers/images;
- an admitted extension and at most 64 MiB of bytes.

Outputs:

- a versioned JSON observation;
- format, size, structural properties, and diagnostics;
- optional normalized contour preview data;
- visible failure state for malformed, unsupported, or oversized input.

## Success Criteria

- ASP.NET 10 builds and serves the static application.
- TypeScript type-checks without format-specific parsing.
- the Rust adapter builds for `wasm32-unknown-unknown`;
- SVG, CGM, glTF/GLB, and FBX bytes reach their Rust importers;
- malformed and unsupported input fails explicitly;
- SVG, one admitted GLB, and one admitted static FBX render from Tokimu-lowered provider-neutral geometry;
- no source file is uploaded to ASP.NET;
- no provider-native Rust object crosses into TypeScript.

## Failure Criteria

The proof exposes architectural friction if:

- TypeScript must parse a format to complete inspection;
- the web host must own simulation or importer state;
- WASM needs renderer-private APIs;
- provider-native records become the application's persistent model;
- native and WASM consumers require incompatible semantic contracts.

## Non-Goals

- a production asset management application;
- complete browser-level SVG compatibility;
- complete CGM, glTF, GLB, or FBX rendering;
- server-side file upload or persistence;
- editing, conversion, or export;
- a stable universal asset inspection schema;
- proving independent production adoption.

## Future Pressure

Later slices may add:

- mesh and node names;
- animation clips and playback controls;
- materials, bounds, and scene hierarchy;
- CGM picture selection;
- source diagnostics and hotspots linked to a viewport;
- saved diagnostic evidence;
- comparison of native and WASM importer observations.

## Implementation Observation

The workbench preview is clamped to a viewport-bounded stage, while the
inspector scrolls independently. Diagnostics from a verbose importer must not
silently expand the preview canvas and distort its coordinate system.

The first GLB and static-FBX previews are intentionally interactive diagnostic perspective
view of decoded triangle geometry. The browser adapter applies projection,
depth ordering, and back-face culling to provider-neutral triangles from
Tokimu; it does not parse model data or own scene semantics. Materials,
lighting, textures, animation, and production scene rendering remain pending.
