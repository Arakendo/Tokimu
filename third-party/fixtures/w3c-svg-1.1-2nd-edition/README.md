# W3C SVG 1.1 2nd Edition Fixtures

This directory preserves the W3C SVG 1.1 2nd Edition test-suite archive as
third-party fixture data for Tokimu presentation-geometry corpus work.

The complete upstream archive is kept under `upstream/`. Tokimu does not claim
full SVG conformance by vendoring these files. The first admitted subset is
listed by `selected/selection-v1.toml` and is intentionally limited to path
data and fill/topology evidence that can be evaluated structurally.

## Layout

```text
upstream/                 Verbatim extracted archive contents
selected/                 Versioned Tokimu selection manifests
LICENSES/                 Preserved upstream copyright material
provenance.json           Source, retrieval, and checksum metadata
W3C_SVG_11_TestSuite.tar.gz
                          Original downloaded archive
```

The W3C suite also contains browser harnesses, reference images, scripts,
resources, and tests for SVG features outside the current Tokimu boundary.
Those files remain available for later, explicitly scoped review but are not
implicitly part of the v1 geometry run.

## Source

The source index and suite documentation are maintained by the W3C SVG working
group:

- <https://dev.w3.org/SVG/profiles/1.1F2/test/>
- <https://dev.w3.org/SVG/docs/SVGTestSuite-howto.html>

See `provenance.json` for the exact archive URL, retrieval date, and SHA-256
checksum.
