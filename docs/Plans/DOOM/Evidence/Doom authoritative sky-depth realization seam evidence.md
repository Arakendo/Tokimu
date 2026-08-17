# Doom Authoritative Sky-Depth Realization Seam Evidence

## Outcome

The study produced two separate results which must not be conflated:

1. the corpus-only unstable G2 intake successfully realizes submission-local
   geometry on native WGPU and Browser WebGPU without assigning persistent mesh
   identity; and
2. Candidate 1's use of that geometry as an independent authoritative-sky
   depth surface is falsified by E1M1.

Candidate 1 clips valid far-left building geometry diagonally, masks valid
outside/hut-adjacent structure and still permits distant rooms to leak beside
the hut and above the wall. The failure is not ordinary triangle realization
error. A bounded comparison at all 320 modeled Doom column centers finds zero
coverage mismatch, no missing or extra oracle cells, no unresolved depth and a
maximum clip-depth approximation error of only `0.000000050`.

The extracted ledger subset is therefore represented faithfully, but the
authority loses essential context when detached from Doom's ordered wall/plane
protocol and introduced as a free-standing occluding surface over the global
shell. G2 remains valid experimental renderer-lifetime evidence; Candidate 1
does not remain a valid composition strategy.

No stable renderer API was changed or admitted. The private G2 seam remains an
experimental instrument.

## Headless Candidate 1 Result

The retained terminal-sky control produced:

```text
input ledger intervals:       66
input diagnostic cells:       2,046
continuous sky regions:       2
depth declarations:           2
declaration vertices:         12
declaration triangles:        4
rejected declarations:        0
persistent mesh identities:   0
diagnostic-grid identities:   0
persistent material key:      doom-sky:SKY1
structural fingerprint:       412dbc8a5b2c861fc848419a8008cca55d180e9a462a0085f6da3f5b68b7a9e7
```

The declarations are derived from continuous boundary knots. Their identity
contains source-plane provenance, prepared-view identity, runtime-snapshot
identity, material identity and the declared depth relationship. It does not
contain diagnostic cells or a persistent mesh handle.

Changing only the prepared view changes the ephemeral declaration fingerprint
while leaving the persistent sky material identity stable and the persistent
mesh-identity count at zero.

## Negative Controls

The same declaration stage retained bounded fail-open outcomes:

```text
invalid depth:
    declarations:             0
    rejected regions:         2 x InvalidDepth
    persistent mesh ids:      0
    fingerprint:              c8f54d27402efb48981c0f766cb14dfcb738522fa5a90b769907f6052b339126

near-plane depth:
    declarations:             0
    rejected regions:         2 x NearPlaneDepth
    persistent mesh ids:      0
    fingerprint:              d3bccc926734f6885457924afe041b6d5807b796afcd5f0d7ccf280dcb40aeb2

paired-sky-only control:
    paired columns observed:  161
    authoritative regions:    0
    depth declarations:       0

one-sky negative:
    authoritative regions:    0
    depth declarations:       0

ordinary aperture:
    authoritative regions:    0
    depth declarations:       0
```

These controls show that the declaration model does not infer sky authority
from adjacency, ordinary apertures or the mere presence of a sky-labelled
neighbor.

## Renderer Seam Audit

The current handoff is persistent-resource-shaped:

- `crates/tokimu-render/src/commands.rs` defines `DrawMeshCommand` with a
  `MeshHandle`; every mesh draw command reaches geometry through a handle.
- `Renderer::submit(&[RenderCommand])` accepts commands, not a bounded inline
  or ephemeral geometry payload.
- `WgpuBackend::upload_mesh(handle, mesh)` creates a GPU vertex buffer and
  inserts it into the persistent mesh map. Uploading the same handle is
  intentionally recorded as replacement.

Consequently, the existing private machinery offers only dishonest Candidate
1 realizations:

```text
view changes
    -> sky declaration geometry changes
    -> upload persistent mesh again
    -> persistent replacement churn
```

That would make ephemeral prepared-view work masquerade as a persistent
resource lifecycle. It would also obscure whether an upload represents asset
replacement, application mutation or ordinary per-view preparation.

## Minimal Pressure Returned To AR-0030

The evidence creates pressure for a provider-neutral way to keep these facts
separate:

- bounded ephemeral or view-local vertex work, or an equivalent realization;
- references to persistent material, texture and pipeline resources;
- explicit prepared-view/submission correlation;
- explicit depth/clip relationship and bounded validation failure;
- no Doom sector, SEG, visplane, sky-name or screen-column vocabulary.

This list is a pressure inventory, not an API proposal. AR-0030 must compare it
against Quake and the commissioned non-BSP campaigns before admitting shared
vocabulary.

Reviewer concurrence refined the next comparison to four private realization
candidates:

```text
G1  inline bounded geometry in a submission
G2  submission-local geometry arena and local references
G3  renderer frame-local transient staging
G4  persistent mesh replacement control
```

G2 is the leading hypothesis, not a decision. The demonstrated semantic
lifetime is a prepared submission/view occurrence; it is not yet proven to be
identical to a backend frame. The comparison must retain source correlation,
submission-local occurrence identity and persistent renderer-resource identity
as separate facts.

The cross-engine precedent survey retained under Renderer Reliability agrees
with that split: established transient buffers and staging arenas demonstrate
implementation feasibility, while completion-aware reuse remains a provider
mechanism rather than submission meaning. The next corpus step is therefore a
private G2 lifetime/identity model before any renderer API or GPU path changes.

Candidate 2 is not authorized by this result. A bounded compositing mechanism
would not remove the persistent-versus-ephemeral handoff question, and
Candidate 1 has not yet failed its depth-representation invariant.

## First Positive E1M1 Realization

Source-trace evidence found that the normal source-spawn pose contains no
authoritative sky interval and must therefore leave Candidate 1 inactive. The
retained `exterior-hut-east` pose at source `(2076, -3560)`, heading `-25.1`
degrees, produces actual authority:

```text
authoritative source regions:       6
realized G2 declarations:           6
vertices:                         264
triangles:                         88
unresolved or omitted regions:       0
persistent mesh identities:          0
persistent mesh replacements:        0
```

This run also exposed and repaired an ordinary realization defect. Doom grants
authority to inclusive raster columns, but the continuous outer edge of an
authorized column may lie fractionally beyond the finite SEG endpoint. The
realizer had intersected that edge ray with the finite segment and rejected
the whole batch. It now interpolates source depth on the SEG's supporting line
*only after* the ordered source protocol has granted the bounded column
authority. Parallel or behind-view intersections still fail open. The repair
does not extend source authority beyond the six modeled regions.

At the identical fixed camera, global-full and Candidate 1 both retain:

```text
source contribution records:       1,922
aggregate contribution hash:       30650e57ad9b3c07
ordinary opaque input:              1,823
admitted cutout input:                  14
non-owning cutout rejections:           12
```

Candidate 1 adds exactly the six G2 declarations/draws between the sky panorama
and unchanged complete ordinary geometry. This proves input conservation and
the intended layer placement. It does not yet prove that the hut survives or
that distant leaks are removed; those remain manual native/browser visual
falsification gates.

## Canonical E1M1 Falsification

The retained exterior-hut observation shows three simultaneous failure
classes:

```text
false rejection:
    far-left building clipped along a diagonal

false rejection:
    valid outside wall / hut-adjacent structure masked by sky authority

false admission:
    distant room geometry visible beside the hut and above the wall
```

The source input remains unchanged relative to global full submission. The
candidate adds six local declarations, 264 vertices, 88 triangles and six
draws, with no persistent mesh identity or replacement.

The bounded oracle/triangle diagnostic then reports on both first and warm
frames:

```text
oracle samples:                    320
coverage mismatches:                 0
coverage extra cells:                0
coverage missing cells:              0
depth samples:                      320
unresolved depth samples:             0
maximum absolute clip-depth error:   0.000000050
mean absolute clip-depth error:      0.000000017
```

This comparison is deliberately narrower than pixel parity: it samples each
modeled exact-ledger outcome at its authoritative source-column center. It
proves that the region-to-continuous-triangle translation is not losing or
adding the ledger coverage it was given. It does not prove that the extracted
sky-authority subset is sufficient to compose correctly with the unchanged
global scene; the visual falsification proves that it is not.

Further tessellation, depth bias and screenshot-specific clipping are rejected
as repairs. The successor study instead tests whether the same source authority
can classify competing contributions relationally within a finite horizontal
and vertical domain before ordinary renderer submission.

## Validation

```text
cargo test -p hello-doom-visibility-conformance authoritative_sky --no-fail-fast
    15 focused tests passed

cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
    passed

cargo fmt --all
    passed
```

The report executable retained the positive and negative counts above through
`authoritative_sky_region_report`.

## Disposition

Candidate 1 is stopped as a composition strategy. Its continuous declaration
realization accurately represents the extracted ledger subset, but an
independent sky-depth surface over complete global geometry creates both false
rejection and false admission in E1M1.

G2 separately passes headless lifetime/identity, native WGPU and Browser WebGPU
execution. See `Doom AR-0030 G2 submission-local geometry evidence.md`. That
result remains pressure for submission-local geometry, not admission of a
stable contract.

The next bounded experiment is
[Doom Source-Authorized Relational Contribution Classification](../Studies/Doom%20source-authorized%20relational%20contribution%20classification.md).
It remains Doom-private and tests keep/reject/split/fail-open relationships
before renderer submission. Candidate 2 bounded composition is still parked
unless this smaller relation proves insufficient.
