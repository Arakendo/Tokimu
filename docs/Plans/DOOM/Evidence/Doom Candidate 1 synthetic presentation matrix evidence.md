# Doom Candidate 1 Synthetic Presentation Matrix Evidence

## Outcome

Slice 3 combines the authoritative-sky Candidate 1 delta with the established
synthetic Doom presentation controls without turning Candidate 1 into a
replacement scene-preparation pipeline.

```text
ordered reference cases:       14 / 14 balanced
focused controls:              10
sky intervals:                 66 / 66 conserved
sky oracle cells:           2,046 / 2,046 conserved
positive G2 declarations:       2
positive G2 triangles:           4
negative authority controls:     2, both zero declarations
removed ordinary contribution:   0
persistent mesh identities:      0
unexplained contribution:        0
matrix fingerprint:              b451da57bf315fd9191c6a7687b324b3e72cbfcbdf2aef3920168eb49599685f
```

Candidate 1 owns only the source-authorized sky-depth delta. Ordinary walls,
planes, vertical apertures, shared plane instances, runtime-height snapshots,
projection controls and cutout presentation retain their existing paths. The
matrix therefore tests conservation at the composition boundary rather than
claiming that G2 prepares all Doom presentation.

## Negative Authority Meaning

The paired-sky-without-plane and one-sky controls produce no authoritative sky
declaration. Their correct realization is:

```text
no source authority
    -> no submission-local sky-depth batch
    -> ordinary presentation continues unchanged
```

They are not represented by an empty or rejected G2 batch. This keeps absence
of Doom authority distinct from renderer-boundary failure.

## Runtime And Material Controls

Door and platform cases use explicit immutable height snapshots only. They do
not simulate activation, ticking, waiting, reversal or movement policy. The
cutout fixture remains a non-occluder control: transparent texels reveal the
far contribution, and Candidate 1 does not infer occlusion from alpha bytes.

## Cross-Target Observation

Native WGPU and actual Browser WebGPU both presented the extended G2 depth
relationship:

```text
near green geometry wins
source-authorized local depth hides farther orange geometry
blue sky/background remains outside the authority region
```

Submissions 41, 42 and 43 each retain five draws. Camera/source jitter changes
the ephemeral geometry fingerprint and restoring the baseline restores the
baseline fingerprint. Both targets retain three persistent uploads and zero
replacements, and a bounded invalid submission is rejected before a later
valid submission recovers.

The remaining synthetic fixtures already have native and Browser WebGPU visual
observations with bounded camera jitter and no warm-frame mesh churn. These are
semantic observations only; no pixel-identical cross-adapter claim follows.

## Validation

```text
cargo run -p hello-doom-visibility-conformance \
    --bin candidate1_synthetic_matrix_report

candidate1 synthetic matrix:
    ordered=14/14
    controls=10
    sky=66/66 intervals,2046/2046 cells
    declarations=2
    local=2/2/4 triangles
    negative-authority=2:declarations=0
    removed-non-sky=0
    persistent-mesh-identities=0
    runtime-snapshots=true
    cutout-deferred=true
    continuous=true
    no-generic-filter=true
    unexplained=0
    semantic-comparison-only=true
```

The `no-generic-filter=true` field confirms that no generic filter participated;
it does not authorize or exercise AABB/frustum selection.

## Disposition

Candidate 1 passes its synthetic conservation gate. This result authorizes the
plan's E1M1 falsification slice, not a stable renderer API. G2 remains private,
feature-gated and corpus-only; Doom semantics remain above the renderer.
