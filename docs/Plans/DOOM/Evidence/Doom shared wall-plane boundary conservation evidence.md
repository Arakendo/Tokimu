# Doom Shared Wall/Plane Boundary Conservation Evidence

## Scope

This record closes Slice 3 of the
[Doom Ordered Source-Occurrence Preparation](../Studies/Doom%20ordered%20source%20occurrence%20preparation.md)
study. It audits the Doom provider's existing ordered wall and plane facts. It
does not introduce another visibility algorithm, a renderer scissor contract,
or public Doom span vocabulary.

## Invocation

```powershell
cargo run -p hello-doom-visibility-conformance --bin shared_boundary_conservation_report
cargo test -p hello-doom-visibility-conformance source_occurrence::tests --lib
cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
```

## Result

Five controls used one ordered source-preparation observation for wall and
plane accounting:

| Fixture | Admitted SEGs | Transitions | Retained / total wall cells | Floor / ceiling / sky plane instances | Paired-sky events | Fail-open seams |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| paired-sky-far-control | 2 | 305 | 48 / 48 | 1 / 1 / 0 | 161 | 5 |
| one-sky-far-control | 2 | 304 | 208 / 208 | 1 / 1 / 0 | 0 | 6 |
| vertical-aperture | 1 | 480 | 480 / 480 | 0 / 0 / 0 | 0 | 8 |
| single-sky-plane-far-control | 1 | 640 | 0 / 0 | 1 / 1 / 1 | 0 | 2 |
| shared-key-disjoint-plane | 2 | 522 | 192 / 246 | 2 / 1 / 0 | 0 | 3 |

Every control passed these structural checks:

- each column's ordered transition begins at the boundary state left by its
  preceding transition;
- every retained wall cell lies inside both its authored raw tier and the
  then-open shared boundary;
- each retained plane interval matches the source SEG, diagnostic column, and
  interval recorded by its causal transition;
- every plane source SEG was admitted by the same near-to-far source traversal;
- paired-sky events leave upper/lower coverage unchanged and produce no plane
  interval of their own;
- plane-instance overlap writes remain zero.

The exact report fingerprint is:

```text
5b9b02259bbc7d536140710cfd918b1284a716d4dddcdda61b2e42cf0291ae81
```

The focused Slice 3 tests pass (13/13), the report completes, and strict
all-target Clippy passes. The crate-wide test run currently retains one older
red assertion:

```text
two_sided_aperture_retains_independent_upper_lower_opening_and_plane_intervals
```

That assertion expects a retained floor `plane_span` from the bounded vertical
aperture even though the same observation reports the floor source mark and
the ordered upper/lower tiers consume the visible plane interval. Slice 3 does
not silently revise that older expectation; the discrepancy remains visible
for the presentation-lowering audit.

## Explicit Fail-Open Seams

The fixtures retain 24 wall/plane and 12 masked-middle
`RaySegmentDepthUnresolved` observations at exact bounded projection seams.
They are not hidden errors and do not reject source contributions or create
coverage authority. No missing-source, missing-plane-mark, behind-viewer, or
outside-FOV failure was accepted by the conservation result.

## Cutout Control

The two-sided masked-middle fixture retains 480 middle wall cells from its
admitted source SEG. It emits no `OneSidedMiddleClosed` transition. Its alpha
policy therefore remains later fragment presentation behavior: the source
contribution exists, but it cannot close Doom coverage or act as an opaque
occluder.

## Finding

The current provider facts are sufficient to prove a shared causal boundary
between ordered wall handling and floor, ceiling, and sky-plane preparation.
Sky remains downstream paint over a source-authorized retained interval rather
than becoming world-space visibility geometry. This authorizes Slice 4's
ordinary Tokimu presentation lowering, but it does not admit any new renderer
API or claim E1M1 completeness.
