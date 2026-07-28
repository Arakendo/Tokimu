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
| Open polylines | focused support; derived structural evidence | `derived/shapes-polyline-01-geometry.svg` preserves open contours and reaches the shared stroke mesh; `derived/shapes-polyline-02-geometry.svg` compares polyline and path syntax with open strokes plus filled open-contour closure |
| Rectangles and rounded rectangles | supported | `derived/shapes-rect-01-geometry.svg` exercises filled primitives; `derived/shapes-rect-02-geometry.svg` exercises `rx`/`ry` coupling; `derived/shapes-rect-03-geometry.svg` exercises radius clamping |
| Circles | supported | `derived/shapes-circle-01-geometry.svg` exercises filled circle primitives; `derived/shapes-circle-02-geometry.svg` exercises default centers and zero-radius omission |
| Ellipses | supported | `derived/shapes-ellipse-01-geometry.svg` exercises circular and non-circular radii; `derived/shapes-ellipse-02-geometry.svg` exercises default centers and zero-radius omission |
| Lines | focused support; derived structural evidence | `derived/shapes-line-01-geometry.svg` preserves open line primitives, omits their non-enclosing fill views, and reaches the shared stroke mesh |
| Explicit M/L/Z paths | supported | `derived/paths-data-04-geometry.svg` exercises explicit line commands, close-path, and nested contours |
| Even-odd fill | focused support; derived structural evidence | Verbatim `painting-fill-03` remains a profile exclusion; `derived/painting-fill-03-geometry.svg` reaches mesh with its current degenerate count recorded |
| Non-zero fill | focused support; derived structural evidence | Verbatim `painting-fill-03` remains a profile exclusion; `derived/painting-fill-03-geometry.svg` reaches mesh with its current degenerate count recorded |
| Nested groups and presentation inheritance | focused support; group paint inheritance covered | `struct-group-01-inheritance-geometry.svg` covers inherited fill/stroke, child overrides, fill-rule, and nested contours; `painting-fill-04-inheritance-geometry.svg` independently covers inherited fill, stroke, and stroke-width through nested groups |
| `currentColor` paint resolution | focused support; derived structural and paint-artifact evidence | `painting-fill-02-current-color-geometry.svg` resolves inherited `color="green"` and a child `color="blue"` override through `fill="currentColor"` |
| In-range stroke opacity | focused support; paint-artifact evidence | `painting-stroke-08-opacity-geometry.svg` records a `0.2` through `0.8` stroke-opacity sequence; compositing and out-of-range clamping remain excluded |
| Local geometric reuse | focused support; derived structural evidence | `derived/struct-use-01-geometry.svg` resolves exact local `href` and `xlink:href` reuse while keeping `defs` non-rendering; `derived/struct-use-01-placement-geometry.svg` adds bounded use-site `x`/`y` placement. Style overrides and arbitrary transforms remain outside this profile |
| Transform composition | focused support; elementary, nested, and orientation-reversing transforms covered | `coords-trans-02-group-geometry.svg`, `coords-trans-03-elementary-geometry.svg`, and `coords-trans-05-reflection-geometry.svg` cover translation, rotation, positive and negative scale, skew, matrix, nesting, transform order, and a non-zero root viewBox origin |
| Stroke geometry | focused support; derived structural evidence | `derived/shapes-line-02-stroke-geometry.svg` exercises line, polyline, and curved open paths; `derived/paths-data-10-stroke-geometry.svg` distinguishes open versus closed triangular paths across butt/round/square caps and miter/bevel/round joins |
| One-level local geometric clip paths | focused support; derived structural evidence | `derived/clip-rect-geometry.svg`, `derived/clip-polygon-geometry.svg`, `derived/clip-transformed-polygon-geometry.svg`, and `derived/masking-path-01-circle-clip-geometry.svg` cover rectangular, convex polygon, transformed polygon, and circular convex intersections; nested clips remain diagnosed rather than composed |
| Gradients | deferred | Excluded from v1 |
| Masks | deferred | Excluded from v1 |
| Filters | deferred | Excluded from v1 |
| Text and font rendering | deferred | Excluded from v1 |
| Animation, DOM, and scripting | unsupported in v1 | Excluded from structural geometry run |

Structural outline, vector, and mesh artifacts are authoritative. Reference
images can be used as complementary evidence after the structural path is
understood; they do not turn unsupported SVG semantics into passing cases.
