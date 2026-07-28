# WebCGM Corpus Feature Matrix

This matrix records the intended v1 evidence boundary. A selected fixture is
not a passing conformance test until the corresponding parser and structural
artifact stages exist.

| Capability | Selected cases | Current status | Intended first evidence |
| --- | --- | --- | --- |
| Pinned source provenance | All 15 | Fixture ready | Archive and tree hashes |
| Binary encoding signature | All 15 | Inventory ready | Bounded element headers |
| Metafile and picture lifecycle | `ALLELM01` | Not implemented | `decode.json`, `cgm.json` |
| VDC extent and normalization | `VDCEXT01` | Not implemented | finite normalized bounds |
| Polyline | `POLYLN01` | Not implemented | open vector and stroke mesh |
| Polygon | `POLYGN01` | Not implemented | closed vector and fill mesh |
| Rectangle | `RCTNGL01` | Not implemented | primitive bounds and fill mesh |
| Circle | `CIRCLE01` | Not implemented | curved contour and fill mesh |
| Ellipse | `ELLIPS01` | Not implemented | oriented curved contour |
| Circular arc | `CIRARC01` | Deferred | endpoints, direction, closure |
| Elliptical arc | `ELLARC01` | Deferred | axes, endpoints, closure |
| Polygon set | `PLGSET01` | Deferred | contour and visibility topology |
| Interior style | `INTSTL01` | Deferred | resolved fill state |
| Line caps | `LINCAP01` | Deferred | provider-neutral cap intent |
| Line joins | `LNJOIN01` | Deferred | provider-neutral join intent |
| Clipping | `CLIPNG01` | Deferred | resolved clip rectangle |
| Color selection mode | `COLRMD01` | Deferred | resolved paint intent |
| Reference image identity | All 15 | Fixture ready | complementary PNG hash |
| Text | None | Explicitly excluded | future text service boundary |
| Cell arrays and raster | None | Explicitly excluded | future raster boundary |
| DOM, XCF, links, interaction | None | Explicitly excluded | future WebCGM profile work |

## Current Denominators

The generated `inventory.json` is authoritative for exact file counts. The
upstream publication describes approximately 345 tests; the archive contains
353 `.cgm` files because support/target files and test-case identity are not
one-to-one.

```text
15 selected unmodified static CGM sources
15 selected reference PNGs
0 decoded cases
0 semantic cases
0 vector cases
0 mesh cases
0 conformance claims
```
