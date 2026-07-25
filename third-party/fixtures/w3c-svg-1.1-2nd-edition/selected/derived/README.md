# W3C-Derived Geometry Fixtures

These documents are reduced geometry fixtures derived from the verbatim W3C
SVG 1.1 2nd Edition sources under `../../upstream/svg/`.

They retain only the named test's path data and directly relevant fill
properties. They deliberately omit test-suite metadata, `defs`, text, frames,
and stroke styling that sit outside Tokimu's currently admitted SVG profile.

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
| `paths-data-03-arcs-geometry.svg` | `paths-data-03-f.svg` | closed arc diagnostic geometry; upstream negative-test provenance |
| `shapes-polygon-01-geometry.svg` | `shapes-polygon-01-t.svg` | filled polygon geometry |
| `shapes-polyline-01-geometry.svg` | `shapes-polyline-01-t.svg` | open polyline geometry; no fill mesh claimed |
| `shapes-rect-01-geometry.svg` | `shapes-rect-01-t.svg` | rectangle and rounded-rectangle fill geometry |
| `shapes-circle-01-geometry.svg` | `shapes-circle-01-t.svg` | circle fill geometry |
| `shapes-ellipse-01-geometry.svg` | `shapes-ellipse-01-t.svg` | ellipse fill geometry |
| `shapes-line-01-geometry.svg` | `shapes-line-01-t.svg` | open line geometry; no fill mesh claimed |
| `paths-data-04-geometry.svg` | `paths-data-04-t.svg` | explicit line and close path geometry |
| `paths-data-06-geometry.svg` | `paths-data-06-t.svg` | absolute and relative horizontal/vertical path geometry |
| `paths-data-08-geometry.svg` | `paths-data-08-t.svg` | implicit line pairs after move commands and nested fill contours |
| `paths-data-13-geometry.svg` | `paths-data-13-t.svg` | repeated arguments after horizontal and vertical commands; open geometry |
| `paths-data-14-geometry.svg` | `paths-data-14-t.svg` | relative implicit line pairs and nested subpaths |
| `shapes-rect-02-geometry.svg` | `shapes-rect-02-t.svg` | rounded rectangle radius coupling when one radius is omitted |
