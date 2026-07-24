# W3C SVG Selection v1 Feature Matrix

This matrix describes the first Tokimu selection, not the coverage of the
complete W3C suite.

| Capability | v1 status | Evidence |
| --- | --- | --- |
| Move-to | supported | `paths-data-01` and later path cases |
| Line-to | supported | `paths-data-04`, `paths-data-05` |
| Horizontal/vertical line-to | supported | `paths-data-06`, `paths-data-07` |
| Quadratic curves | supported | `paths-data-02` |
| Cubic curves | supported | `paths-data-01` |
| Smooth curve commands | supported | `paths-data-01`, `paths-data-02` |
| Elliptical arcs | supported | `paths-data-12` through `paths-data-14` |
| Relative commands | supported | `paths-data-01`, `paths-data-02`, `paths-data-05`, `paths-data-07`, `paths-data-09` |
| Close path | supported | `paths-data-02`, `paths-data-04` through `paths-data-09`, `paths-data-16` |
| Multiple contours | supported | `paths-data-02`, `paths-data-04`, `paths-data-08`, `paths-data-09`, `painting-fill-03` |
| Even-odd fill | selected | `painting-fill-03`; runner support to be verified |
| Non-zero fill | selected | `painting-fill-03`; runner support to be verified |
| Stroke geometry | planned | W3C stroke cases are preserved upstream but not admitted in v1 |
| Clip paths | planned | Excluded from v1 |
| Gradients | deferred | Excluded from v1 |
| Masks | deferred | Excluded from v1 |
| Filters | deferred | Excluded from v1 |
| Text and font rendering | deferred | Excluded from v1 |
| Animation, DOM, and scripting | unsupported in v1 | Excluded from structural geometry run |

Structural outline, vector, and mesh artifacts are authoritative. Reference
images can be used as complementary evidence after the structural path is
understood; they do not turn unsupported SVG semantics into passing cases.
