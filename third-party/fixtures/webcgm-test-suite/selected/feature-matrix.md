# WebCGM Corpus Feature Matrix

This matrix records the intended v1 evidence boundary. A selected fixture is
not a passing conformance test until the corresponding parser and structural
artifact stages exist.

| Capability | Selected cases | Current status | Intended first evidence |
| --- | --- | --- | --- |
| Pinned source provenance | All 26 | Fixture ready | Archive and tree hashes |
| Binary encoding signature | All 26 | Lifecycle decoded | Bounded element headers |
| Metafile and picture lifecycle | All 26 | Lifecycle decoded | Inspectable elements and picture boundaries |
| Element inventory | `ALLELM01` | Source-only corpus case | broad lifecycle and element-count evidence without geometric lowering |
| VDC type, precision, scaling, and extent | `VDCEXT01` | Source-only corpus case | source-ordered VDC corners and picture-local descriptor state |
| VDC normalization | `VDCEXT01` | Helper decoded; source-only corpus case | source-order unit-square mapping; primitive consumer pending |
| Polyline | `POLYLN01` | Source-to-vector corpus case | open `VectorContour`; stroke mesh pending |
| Polygon | `POLYGN01` | Source-to-vector corpus case | closed `VectorContour`; fill mesh pending |
| Rectangle | `RCTNGL01` | Source-to-vector corpus case | closed `VectorContour`; fill mesh pending |
| Circle | `CIRCLE01` | Source-to-vector corpus case | deterministic source-VDC flattening; fill mesh pending |
| Ellipse | `ELLIPS01` | Source-to-vector corpus case | source conjugate-diameter flattening; fill mesh pending |
| Circular arc | `CIRARC01` | Source-to-vector corpus case | open counter-clockwise sweep; closure and stroke mesh pending |
| Elliptical arc | `ELLARC01` | Source-to-vector corpus case | open conjugate-diameter sweep; closure and stroke mesh pending |
| Polygon set | `PLGSET01` | Expected source-to-vector boundary | ordered 16-bit point records with visible/invisible/close-edge semantics; corpus runner reports the provider-neutral topology boundary explicitly |
| PolyBezier | `POLYBZ01`, `POLYBZ04` | Source-only corpus cases | Independent CGM continuity and integer VDC control-point records retained; shared cubic-path lowering remains unclaimed |
| Interior style | `INTSTL01` | Source-to-vector corpus case | standard solid source interior and explicit-palette solid-fill candidates retained beside finite vector paths; provider-neutral paint remains pending |
| Line caps | `LINCAP01` | Source-to-vector corpus case | active source state retained beside finite vector paths; provider-neutral cap intent pending |
| Line joins | `LNJOIN01` | Source-to-vector corpus case | active source state retained beside finite vector paths; provider-neutral join intent pending |
| Clipping | `CLIPNG01` | Source-to-vector corpus case | ordered clip controls retained beside finite vector paths; provider-neutral clipping deferred |
| Additional clipping state | `CLIPNG02` | Source-only corpus case | broadens clipped-control source evidence; provider-neutral clipping remains deferred |
| Restricted and append text | `APNTXT01` | Source-only corpus case | source positions, restriction values, final flags, active state, and strings are retained; font selection, shaping, layout, glyph synthesis, and rendering deferred |
| Character height, orientation, and alignment | `CHRHGT01`, `CHRORI01`, `TXTALN01` | Source-only corpus case | repeated integer character heights, independently witnessed up/base orientation vectors and alignment enums, plus raw continuous-alignment values, are retained in text-record state; font selection, shaping, layout, glyph synthesis, and rendering deferred |
| Character spacing | `CHRSPA01` | Source-only corpus case | encoded real-number source bytes are retained in text-record state; spacing is not interpreted as a font metric or layout adjustment |
| Text path | `TXTPTH01` | Source-only corpus case | CGM path direction enum is retained in text-record state; it is not converted into glyph placement or a transform |
| Cell array | `CELARY01` | Source-only corpus case | three corners, dimensions, local precision, representation, state, and payload span are retained; texture/raster lowering deferred |
| Color selection mode | `COLRMD01` | Source-to-vector corpus case | picture-local raw direct/indexed colours retained beside finite vector paths |
| Color value extent | `COLVAL01` | Source-to-vector corpus case | metafile-local direct RGB component range retained and narrowly normalizable; no paint conversion claimed |
| Reference image identity | All 24 | Fixture ready | complementary PNG hash |
| General text presentation | None | Explicitly excluded | CGM source records do not own font selection, shaping, layout, glyph synthesis, or rendering |
| Cell arrays and raster | None | Explicitly excluded | future raster boundary |
| DOM, XCF, links, interaction | None | Explicitly excluded | future WebCGM profile work |

## Current Denominators

The generated `inventory.json` is authoritative for exact file counts. The
upstream publication describes approximately 345 tests; the archive contains
353 `.cgm` files because support/target files and test-case identity are not
one-to-one.

```text
26 selected unmodified static CGM sources
24 selected reference PNGs
26 selected cases reach complete lifecycle evidence
25 selected cases inspect explicit source-state attributes where present
Selected primitive records snapshot active explicit presentation state
5 primitive semantic cases
7 vector-lowered primitive families
12 source-only corpus-runner cases
13 source-to-vector corpus-runner passes
1 expected source-to-vector topology boundary
0 mesh cases
0 conformance claims
```
