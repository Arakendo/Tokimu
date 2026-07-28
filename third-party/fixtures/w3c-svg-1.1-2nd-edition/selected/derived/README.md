# W3C-Derived Geometry Fixtures

These documents are reduced geometry fixtures derived from the verbatim W3C
SVG 1.1 2nd Edition sources under `../../upstream/svg/`.

They retain only the named test's geometry and directly relevant presentation
properties. They deliberately omit test-suite metadata, text, frames, and
unrelated styling. Most omit `defs`; the focused local `<use>` fixtures retain
only the geometric definitions required to test admitted reuse behavior.

They are **not** W3C conformance passes. Corpus reports label them
`svg/w3c-derived` so structural vector and mesh evidence remains distinct from
the verbatim `svg/w3c` source exclusions.

| Derived fixture | Upstream source | Retained evidence |
| --- | --- | --- |
| `paths-data-16-geometry.svg` | `paths-data-16-t.svg` | implicit line-to and relative path coordinates |
| `painting-fill-03-geometry.svg` | `painting-fill-03-t.svg` | even-odd and non-zero fill-rule path geometry |
| `paths-data-01-curves-geometry.svg` | `paths-data-01-t.svg` | cubic and smooth-cubic closed fill geometry |
| `paths-data-02-quadratics-geometry.svg` | `paths-data-02-t.svg` | quadratic and smooth-quadratic closed fill geometry |
| `coords-trans-02-group-geometry.svg` | `coords-trans-02-t.svg` | nested group transforms, inherited fill, and rectangles |
| `coords-trans-03-elementary-geometry.svg` | `coords-trans-03-t.svg` | translation, rotation, scale, skew, matrix, and nested transform composition |
| `coords-trans-05-reflection-geometry.svg` | `coords-trans-03-t.svg` | negative-scale reflection, orientation reversal, and non-zero root viewBox origin |
| `struct-group-01-inheritance-geometry.svg` | `struct-group-01-t.svg` | group-level fill/stroke/fill-rule inheritance and child overrides |
| `painting-fill-04-inheritance-geometry.svg` | `painting-fill-04-t.svg` | nested fill, stroke, and stroke-width inheritance with child overrides |
| `painting-fill-02-current-color-geometry.svg` | `painting-fill-02-t.svg` | `currentColor` fill resolution through inherited and child-local `color` |
| `painting-stroke-08-opacity-geometry.svg` | `painting-stroke-08-t.svg` | in-range `stroke-opacity` paint intent without compositing claims |
| `paths-data-03-arcs-geometry.svg` | `paths-data-03-f.svg` | closed arc diagnostic geometry; upstream negative-test provenance |
| `shapes-polygon-01-geometry.svg` | `shapes-polygon-01-t.svg` | filled polygon geometry |
| `shapes-polyline-01-geometry.svg` | `shapes-polyline-01-t.svg` | open polyline geometry; non-enclosing fill omitted and shared stroke mesh retained |
| `shapes-rect-01-geometry.svg` | `shapes-rect-01-t.svg` | rectangle and rounded-rectangle fill geometry |
| `shapes-circle-01-geometry.svg` | `shapes-circle-01-t.svg` | circle fill geometry |
| `shapes-ellipse-01-geometry.svg` | `shapes-ellipse-01-t.svg` | ellipse fill geometry |
| `shapes-line-01-geometry.svg` | `shapes-line-01-t.svg` | open line geometry; non-enclosing fill omitted and shared stroke mesh retained |
| `paths-data-10-stroke-geometry.svg` | `paths-data-10-t.svg` | open/closed triangular paths across cap and join styles |
| `shapes-polyline-02-geometry.svg` | `shapes-polyline-02-t.svg` | polyline/path equivalence with open strokes and filled open contours |
| `struct-use-01-geometry.svg` | `struct-use-01-t.svg` | bounded local `href` and `xlink:href` geometric reuse without rendering `defs` storage |
| `struct-use-01-placement-geometry.svg` | `struct-use-01-t.svg` | bounded local `href` and `xlink:href` reuse with use-site `x`/`y` placement; style overrides and transforms remain excluded |
| `masking-path-01-circle-clip-geometry.svg` | `masking-path-01-b.svg` | one user-space circular clip intersecting a rectangular fill through the shared convex-clip mesh path |
| `masking-path-02-curve-clip-geometry.svg` | `masking-path-02-b.svg` | one user-space rectangular clip intersecting a cubic closed fill after curve flattening |
| `paths-data-04-geometry.svg` | `paths-data-04-t.svg` | explicit line and close path geometry |
| `paths-data-06-geometry.svg` | `paths-data-06-t.svg` | absolute and relative horizontal/vertical path geometry |
| `paths-data-08-geometry.svg` | `paths-data-08-t.svg` | implicit line pairs after move commands and nested fill contours |
| `paths-data-13-geometry.svg` | `paths-data-13-t.svg` | repeated arguments after horizontal and vertical commands; open geometry |
| `paths-data-14-geometry.svg` | `paths-data-14-t.svg` | relative implicit line pairs and nested subpaths |
| `shapes-rect-02-geometry.svg` | `shapes-rect-02-t.svg` | rounded rectangle radius coupling when one radius is omitted |
