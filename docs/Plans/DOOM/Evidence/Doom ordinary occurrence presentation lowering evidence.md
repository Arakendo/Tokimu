# Doom Ordinary Occurrence Presentation Lowering Evidence

## Scope

This record retains Slice 4 evidence for
[Doom Ordered Source-Occurrence Preparation](../Studies/Doom%20ordered%20source%20occurrence%20preparation.md).
It asks whether the private Doom occurrence result can cross into ordinary
Tokimu presentation without transferring Doom vocabulary or authority to
`tokimu-render`.

The tested path is:

```text
Doom source contribution
    + ordered partial-survival result
        ↓
private presentation occurrence
        ↓ continuous source-domain interpolation
ordinary Mesh with positions, normals, UVs, indices
        ↓
tokimu-render
```

The resulting `Mesh` contains no BSP node, subsector, SEG, screen-column, sky,
or occurrence vocabulary. Source and occurrence identities remain bounded
diagnostic correlation owned by the corpus-side preparation result.

## Reproduction

Headless lowering and conservation report:

```powershell
cargo run -p hello-doom-visibility-conformance --bin occurrence_presentation_lowering_report
```

Native WGPU presentation:

```powershell
cargo run -p hello-doom-visibility-conformance --bin ordered_coverage_presentation
```

Focused and strict validation:

```powershell
cargo test -p hello-doom-visibility-conformance source_occurrence::tests --lib
cargo test -p hello-doom-visibility-conformance --bin ordered_coverage_presentation
cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
```

## Retained Headless Result

The same source correlation was exercised through a whole-retain control and
two partial occurrences:

```text
source=subsector:1/seg:1/linedef:1/sidedef:2

whole:
  occurrence=None
  source interval=[0.0, 1.0]
  vertices=6
  UV stream present=true
  generated view-local=false

left partial:
  occurrence=17
  source interval=[0.0, 0.08333333333333333]
  vertices=9
  UV stream present=true
  generated view-local=true

right partial:
  occurrence=18
  source interval=[0.9166666666666666, 1.0]
  vertices=9
  UV stream present=true
  generated view-local=true
```

Conservation and structural result:

```text
retained occurrences=2
lowered occurrences=2
source order preserved=true
source correlation preserved=true
endpoints from continuous source domains=true
UV streams complete=true
generated geometry view-local=true
fingerprint=c707513fb367f3184bf32699a661bf4e71f078d4efa5dd0987e27d1c0e0fc94c
```

Two consecutive report executions produced the same fingerprint. The focused
library suite passed 16 tests, the presentation binary test passed, and strict
crate-wide Clippy completed successfully. Rust emitted only the already-known
filesystem incremental-cache hard-link warnings and copied cache entries
instead; those warnings do not describe a fixture or contract failure.

## Native Presentation Observation

The rewritten native fixture consumes the ordinary `Mesh` declarations
reported above; it no longer independently reconstructs Doom SEG triangles.
Native Vulkan/WGPU presented the first, warm, and camera-jitter frames with no
backend diagnostic and retained the same structural fingerprint:

```text
first:
  occurrences=2
  draws=4; materials=4; pipelines=2
  binding allocations=5
  frame mesh uploads=0; replacements=0
  lifetime mesh uploads=4; replacements=0

warm:
  occurrences=2
  draws=4; materials=4; pipelines=2
  binding allocations=0
  frame mesh uploads=0; replacements=0
  lifetime mesh uploads=4; replacements=0

camera jitter (offset x=0.08):
  occurrences=2
  draws=4; materials=4; pipelines=2
  binding allocations=0
  frame mesh uploads=0; replacements=0
  lifetime mesh uploads=4; replacements=0

all frames:
  fingerprint=c707513fb367f3184bf32699a661bf4e71f078d4efa5dd0987e27d1c0e0fc94c
  diagnostic=none
```

This proves successful native presentation and the absence of warm/jitter
resource churn. Maintainer visual review confirmed the semantic image and the
bounded-jitter stability described below.

The retained observation is deliberately semantic rather than pixel-golden:

- a blue background/field;
- a central green near authority;
- narrow orange left and right survivors from the partially retained far
  source contribution;
- no orange participation in the forbidden middle interval;
- no revealed finite preparation box or shared-seam crack after the bounded
  camera jitter.

Maintainer review of the native WGPU fixture confirmed that exact semantic
image. The observation is not a pixel-golden rendering contract.

## Disposition So Far

Slice 4 passes. A private Doom occurrence can lower to
ordinary Tokimu mesh data while preserving source order, correlation,
continuous endpoint/UV derivation, and exact retained-to-lowered conservation.
Generated partial geometry remains explicitly view-local; the unchanged whole
control is not relabeled as generated global truth.

No renderer API, stable shared contract, or engine ownership change is implied.
The maintainer-observed native frame retained both left/right source fragments,
kept the forbidden middle absent, and showed neither a shared-seam crack nor a
finite preparation box after bounded camera jitter.
