# Presentation Geometry Corpus

`presentation-geometry-corpus` is an incubating diagnostic runner for the
staged transformations between presentation producers and renderer-neutral
geometry.

It is intentionally an example-side package. It observes `ui-tools`; it does
not define a new vector, font, or renderer capability.

## Current Coverage

The runner currently covers four producer groups:

- synthetic topology probes;
- prepared Inter font outline regressions;
- a prepared Lucide SVG source fixture;
- four verbatim W3C SVG source exclusions;
- eighteen explicitly labeled W3C-derived geometry fixtures;
- semantic UI surface lowering.

The font cases are:

- `glyph/inter/K`
- `glyph/inter/k`
- `glyph/inter/M`
- `glyph/inter/e`

The SVG case is:

- `svg/lucide/archive`

The verbatim W3C SVG source cases are profile exclusions that preserve XML
evidence because their complete documents include unadmitted test-suite
features:

- `svg/w3c/painting-fill-03-t`
- `svg/w3c/paths-data-16-t`
- `svg/w3c/struct-group-01-t`
- `svg/w3c/coords-trans-02-t`

The W3C-derived geometry cases retain only the upstream path data required for
the named feature and therefore reach the vector and mesh stages without
claiming W3C conformance:

- `svg/w3c-derived/paths-data-16-geometry`
- `svg/w3c-derived/painting-fill-03-geometry`
- `svg/w3c-derived/paths-data-01-curves-geometry`
- `svg/w3c-derived/paths-data-02-quadratics-geometry`
- `svg/w3c-derived/coords-trans-02-group-geometry`
- `svg/w3c-derived/paths-data-03-arcs-geometry`
- `svg/w3c-derived/shapes-polygon-01-geometry`
- `svg/w3c-derived/shapes-polyline-01-geometry`
- `svg/w3c-derived/shapes-rect-01-geometry`
- `svg/w3c-derived/shapes-circle-01-geometry`
- `svg/w3c-derived/shapes-ellipse-01-geometry`
- `svg/w3c-derived/shapes-line-01-geometry`
- `svg/w3c-derived/paths-data-04-geometry`
- `svg/w3c-derived/paths-data-06-geometry`
- `svg/w3c-derived/paths-data-08-geometry`
- `svg/w3c-derived/paths-data-13-geometry`
- `svg/w3c-derived/paths-data-14-geometry`
- `svg/w3c-derived/shapes-rect-02-geometry`

The UI case is:

- `ui/panel-surface`

It intentionally contains one closed path and two open stroke paths. The
closed path is sent through the current fill tessellator; the open paths remain
vector-stage evidence until stroke expansion is tested as its own capability.

Each case reports the ordered stages that are meaningful for its producer:

```text
source -> outline -> vector -> mesh
```

For SVG, the outline stage is omitted because SVG is already the path
producer:

```text
source -> xml -> vector -> mesh
```

The UI case also starts at source semantics and observes the public lowering
adapter before entering the same vector and mesh stages.

The first divergent stage is the owning diagnostic boundary. A valid vector
result with open paths that do not reach the fill mesh is therefore an explicit
capability boundary, not a silently accepted rendering result.

The report is structural evidence. Glyph cases emit normalized diagnostic
artifacts and CPU image evidence. SVG and W3C cases emit parser-neutral XML
and vector artifacts from the same record-aware SVG lowering used by their
report path. Fill-producing cases also emit mesh and mesh-fingerprint
artifacts. Profile-excluded and stroke-only cases intentionally stop before
the fill mesh boundary. UI artifact serialization remains producer-specific
work for a later slice.

The geometry corpus runner is the only workspace producer of the shared
diagnostic artifact schema. Visual examples may retain their own presentation
manifests or screenshots, but they do not duplicate these corpus artifacts.

The workspace-level `tests` package is a non-example validation consumer. It
uses the runner's public case registry and report API for synthetic topology
tests, proving that the diagnostic boundary can be consumed outside the
examples tree without requiring prepared font or Lucide assets.

## Running

From the repository root:

```text
cargo run -p presentation-geometry-corpus
cargo run -p presentation-geometry-corpus -- list
cargo run -p presentation-geometry-corpus -- run glyph/inter/K
cargo run -p presentation-geometry-corpus -- compare synthetic/convex-rectangle
cargo run -p presentation-geometry-corpus -- bless synthetic/convex-rectangle
cargo run -p presentation-geometry-corpus -- compare-all
cargo run -p presentation-geometry-corpus -- bless-all
cargo run -p presentation-geometry-corpus -- generate 42 20
cargo test -p tokimu-tests
```

`compare` is read-only and checks a case against its reviewed structural
snapshot under `tests/fixtures/golden/presentation-geometry/`. `bless` is the
explicit fixture-update operation; it refuses to write a snapshot for a failed
case. Fixture directories include a stable case-ID hash so case-sensitive IDs
remain distinct on case-insensitive filesystems. Normal runs do not compare or
mutate golden expectations. The `*-all` commands apply the same behavior to
every registered case; only `bless-all` mutates fixtures.

Glyph and fill-producing SVG cases emit a normalized `mesh-fingerprint.json`.
The fingerprint records mesh bounds and validation metrics plus an
order-independent triangle hash, while raw XML/vector/mesh artifacts remain
available for detailed diagnostics. This keeps reviewed evidence sensitive to
geometry changes without making tessellator emission order part of the early
contract. Profile-excluded W3C documents and stroke-only Lucide inputs retain
their source, XML, and vector evidence but intentionally do not claim fill-mesh
coverage. SVG mesh artifacts honor each record's fill and fill-rule metadata
rather than applying a corpus-only fallback rule.

Glyph cases also emit `image-fingerprint.json` for the deterministic CPU RGBA8
source buffer. Reviewed comparisons use its dimensions, source-buffer identity,
and pixel hash. This is intentionally not a GPU framebuffer or native-window
screenshot contract.

The prepared glyph corpus must exist for execution. Run
`scripts/prepare-glyph-corpus.ps1` before running the glyph cases.

`generate <seed> <count>` runs deterministic, non-golden polygon inputs for
focused topology investigation. The seed and index are reported for replay;
generated cases do not alter the reviewed case list or fixture set.

The Lucide subset prepared by `scripts/prepare-lucide-sample.ps1` also records
`provenance.json` with the provider revision, selection rule, source path, and
selected count. That subset is useful external data evidence, but is not a
claim of W3C SVG conformance.
