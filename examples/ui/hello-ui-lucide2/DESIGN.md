# Hello UI Lucide 2

`hello-ui-lucide2` is the second Lucide corpus proof. It focuses on shapes
that are more complex than independent straight line segments:

- star-like closed geometry;
- heart-like curved silhouettes;
- activity-style connected paths.
- diamond and zap-like compound silhouettes;

The first implementation intentionally keeps the geometry source explicit so
the mesh proof is visible. The next step is to replace those source shapes
with flattened `C`, `Q`, and `A` commands from the real Lucide SVG files.

## Current proof

Each shape is converted into a filled triangle fan or rounded stroke and
rendered in its own grid slot. This tests closed-path composition, winding,
stroke joins, and independent placement without changing the line-stroke
implementation in `hello-ui-lucide`.

## Follow-up

- flatten cubic and quadratic curves;
- flatten SVG arcs;
- preserve `Z` closure and fill winding;
- compare generated geometry against `star.svg`, `heart.svg`, and
  `activity.svg` from the prepared Lucide corpus.
