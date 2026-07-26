# W3C SVG Selection v1 Feature Matrix

This matrix describes the first Tokimu selection, not the coverage of the
complete W3C suite.

| Capability | v1 status | Evidence |
| --- | --- | --- |
| Move-to | supported | `paths-data-01` and later path cases |
| Line-to | supported | `derived/paths-data-04-geometry.svg` exercises explicit `L`; `derived/paths-data-05-geometry.svg` exercises relative `l` |
| Horizontal/vertical line-to | supported | `derived/paths-data-06-geometry.svg` exercises absolute and relative `H/V` and `h/v`; `derived/paths-data-07-geometry.svg` isolates relative `h/v`; `derived/paths-data-13-geometry.svg` exercises repeated arguments |
| Quadratic curves | focused support; derived structural evidence | `derived/paths-data-02-quadratics-geometry.svg` reaches mesh without degenerate triangles |
| Cubic curves | focused support; derived structural evidence | `derived/paths-data-01-curves-geometry.svg` reaches mesh with its current degenerate count recorded |
| Smooth curve commands | focused support; derived structural evidence | The admitted derived `paths-data-01` and `paths-data-12` fixtures exercise `S`; `paths-data-02` and `paths-data-15` exercise `T`, including initial smooth commands |
| Elliptical arcs | focused support; derived diagnostic evidence | `derived/paths-data-03-arcs-geometry.svg` reaches mesh with its current degenerate count recorded; upstream provenance is a negative-test case, not a conformance pass |
| Relative commands | supported | `derived/paths-data-05-geometry.svg`, `derived/paths-data-07-geometry.svg`, `derived/paths-data-09-geometry.svg`, and `derived/paths-data-14-geometry.svg` exercise direct, H/V, and implicit relative forms |
| Close path | supported | Derived `paths-data-02`, `paths-data-04` through `paths-data-09`, `paths-data-14`, and `paths-data-16` fixtures |
| Multiple contours | supported | `derived/paths-data-05-geometry.svg`, `derived/paths-data-08-geometry.svg`, and `derived/paths-data-09-geometry.svg` exercise nested closed contours across explicit and implicit forms |
| Filled polygons | supported | `derived/shapes-polygon-01-geometry.svg` reaches a finite mesh with two contours and no degenerate triangles; `derived/shapes-polygon-02-geometry.svg` exercises concave and star-shaped fills; `derived/shapes-polygon-03-geometry.svg` exercises odd-coordinate truncation |
| Open polylines | vector evidence; fill mesh out of scope | `derived/shapes-polyline-01-geometry.svg` preserves open contours without claiming stroke expansion |
| Rectangles and rounded rectangles | supported | `derived/shapes-rect-01-geometry.svg` exercises filled primitives; `derived/shapes-rect-02-geometry.svg` exercises `rx`/`ry` coupling; `derived/shapes-rect-03-geometry.svg` exercises radius clamping |
| Circles | supported | `derived/shapes-circle-01-geometry.svg` exercises filled circle primitives; `derived/shapes-circle-02-geometry.svg` exercises default centers and zero-radius omission |
| Ellipses | supported | `derived/shapes-ellipse-01-geometry.svg` exercises circular and non-circular radii; `derived/shapes-ellipse-02-geometry.svg` exercises default centers and zero-radius omission |
| Lines | vector evidence; fill mesh out of scope | `derived/shapes-line-01-geometry.svg` preserves open line primitives without claiming stroke expansion |
| Explicit M/L/Z paths | supported | `derived/paths-data-04-geometry.svg` exercises explicit line commands, close-path, and nested contours |
| Even-odd fill | focused support; derived structural evidence | Verbatim `painting-fill-03` remains a profile exclusion; `derived/painting-fill-03-geometry.svg` reaches mesh with its current degenerate count recorded |
| Non-zero fill | focused support; derived structural evidence | Verbatim `painting-fill-03` remains a profile exclusion; `derived/painting-fill-03-geometry.svg` reaches mesh with its current degenerate count recorded |
| Nested groups and presentation inheritance | focused support; group paint inheritance covered | `struct-group-01-inheritance-geometry.svg` covers inherited fill/stroke, child overrides, fill-rule, and nested contours |
| Transform composition | focused support; elementary and nested transforms covered | `coords-trans-02-group-geometry.svg`, `coords-trans-03-elementary-geometry.svg` cover translation, rotation, scale, skew, matrix, nesting, and transform order |
| Stroke geometry | vector evidence; expansion planned | `derived/shapes-line-02-stroke-geometry.svg` preserves line, polyline, and curved open-path stroke intent; cap/join expansion remains outside the current mesh profile |
| Clip paths | planned | Excluded from v1 |
| Gradients | deferred | Excluded from v1 |
| Masks | deferred | Excluded from v1 |
| Filters | deferred | Excluded from v1 |
| Text and font rendering | deferred | Excluded from v1 |
| Animation, DOM, and scripting | unsupported in v1 | Excluded from structural geometry run |

Structural outline, vector, and mesh artifacts are authoritative. Reference
images can be used as complementary evidence after the structural path is
understood; they do not turn unsupported SVG semantics into passing cases.
