# Doom E1M1 Ordered Source Protocol Ledger Evidence

## Scope

This record moves the released-Doom clip-state audit from synthetic fixtures
to canonical E1M1 at the fixed source spawn. It observes the Doom-owned
near-to-far BSP, wall-tier, vertical-clip, and plane-instance ledger before any
mesh reconstruction or `tokimu-render` submission.

It does not claim that the existing fixed-view cell reconstruction or the
continuous occurrence-wedge lowering is an acceptable realization.

## Commands

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-seg-classic-vertical-clip-trace

cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --doom-seg-classic-plane-span-trace
```

## Fixed source-spawn ledger

```text
admitted SEGs                         37
wall tiers
  upper                               8
  lower                               7
  middle                             23
plane marks
  floor                              36
  ceiling                            37
  paired sky                          0
clip mutations
  ceiling                           746
  floor                             885

source plane keys
  floor                               5
  ceiling                             2
plane instances                       9
collision splits                      2
horizontal spans                     17
populated columns                  1,205
populated cells                   50,679
overlapping writes                    0
empty after clip                    596
resolved instances                    9
unresolved instances                  0
sky instances                         0
```

The retained source instances comprise five floor keys and two ceiling keys;
equal keys may still produce separate instances. Flat resolution accounts for
all nine instances. No overlapping plane write or unresolved flat destination
was observed.

## Representation comparison

The same canonical observation now exposes three materially different facts:

| Stage | Retained result | Meaning |
| --- | ---: | --- |
| Doom source ledger | 9 plane instances, 17 horizontal spans, 1,205 populated columns | Source-faithful viewer-relative coverage evidence |
| Historical fixed-view cell lowering | 1,205 quads / 2,410 triangles | Preserves the raster-shaped ledger by inverse-projecting a fixed observation; not reusable world geometry |
| Continuous occurrence-wedge candidate | 457 total prepared declarations | Structurally balanced, but visually falsified by missing required walls/planes and exposed background |

The exact clip-state repairs changed which inclusive rows survive but did not
remove this representational mismatch. The source ledger is coherent before
lowering; the remaining false negatives therefore cannot honestly be
classified as missing source observation or unresolved material identity.

## Finding

The current evidence establishes a boundary rather than an implementation
victory:

- released Doom semantics retain viewer-relative plane coverage at
  column/row-span granularity;
- the historical Tokimu reconstruction can preserve that evidence only by
  creating fixed-view, raster-derived geometry;
- the continuous ordinary-world-geometry approximation loses source-authorized
  contributions even while its own conservation ledger balances; and
- a generic AABB/frustum filter cannot restore work removed before renderer
  handoff.

The next realization must either demonstrate a non-raster Doom-private
fragment representation that conserves this vertical coverage, or return the
need for bounded view-local/screen-local realization to AR-0030. No stable
renderer contract is admitted by this evidence.

## Validation

The source-parity repair and this escalation were validated with:

```text
cargo test -p doom-geometry-provider                 39 passed
cargo check -p hello-doom-e1m1 --bin static_scene    passed
cargo clippy -p doom-geometry-provider \
  -p hello-doom-visibility-conformance \
  -p hello-doom-e1m1 --all-targets -- -D warnings    passed
```

Windows incremental-cache hard-link fallback messages are tooling noise, not
test or compiler failures.
