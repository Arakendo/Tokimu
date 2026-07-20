# Hello UI Lucide

This is the first real Lucide provider proof. It loads a small board of real
Lucide SVGs: `minus`, `plus`, `x`, `check`, and `arrow-right`.

The first step proves provider-file discovery and line-path rendering using
rounded stroke capsules. The parser currently covers the line commands used
by these fixtures, including absolute and relative moves, horizontal and
vertical segments, and implicit line pairs.

Curves, arcs, joins, full stroke styling, and complete SVG parsing remain
separate follow-up corpus tests.
