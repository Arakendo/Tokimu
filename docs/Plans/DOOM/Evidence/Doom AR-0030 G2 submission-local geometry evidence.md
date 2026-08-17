# Doom AR-0030 G2 Submission-Local Geometry Evidence

## Outcome

The corpus-private G2 lifetime model accepted the authoritative Doom sky-depth
declaration without assigning durable renderer mesh identity:

```text
submission:                    1
submission-local payloads:     2
ordered draws:                 2
vertices:                     12
triangles:                     4
persistent material keys:      {doom-sky:SKY1}
persistent mesh identities:    0
```

Every draw retains a submission-local geometry identity plus bounded Doom
source/view/runtime correlation. The correlation identity may repeat when two
presentation payloads derive from the same source plane; their local payload
identities remain distinct. Neither identity is presented as a durable asset.

The same declaration now also crosses an explicitly unstable, feature-gated
`tokimu-render` intake and is realized by native WGPU without allocating a
`MeshHandle`. The experiment changes no stable `Renderer` trait or default
feature surface.

The presentation order is deliberately semantic rather than an implementation
accident:

```text
persistent background / sky colour
    -> submission-local source-authorized depth geometry
    -> persistent far-wall control
    -> present
```

Submitting the depth declaration after the far wall would not be a valid
control: a later depth-only draw cannot erase colour already written by the
far wall. Both native and browser fixtures use the order above.

## Bounded Failure Evidence

The private builder rejects before producing an immutable snapshot when it
observes:

- empty or non-triangle-list geometry;
- non-finite positions;
- per-payload, aggregate-vertex, payload-count, or draw-count overflow;
- a local identity from another submission;
- a missing local slot;
- empty durable-material or source-correlation keys;
- geometry which has no ordered draw at finalization.

A deliberately one-payload limit rejects the real two-region sky declaration
with `PayloadCapacityExceeded` and returns no partial snapshot.

## Identity And Lifetime Result

```text
doom source/view/runtime correlation
    retained for observation

submission-local geometry id
    (submission id, local slot)
    rejected outside that submission

persistent renderer identity
    one material key
    zero mesh identities
```

The snapshot fingerprint includes the submission identity. Reusing local slot
zero in a later submission cannot make the old identity resolve, even when the
geometry bytes and source correlation are otherwise identical.

## Validation

```text
cargo test -p hello-doom-visibility-conformance submission_local --no-fail-fast
    7 passed

cargo run -p hello-doom-visibility-conformance --bin submission_local_geometry_report
    completed with the retained counts above

cargo run -p hello-doom-visibility-conformance \
    --bin submission_local_geometry_presentation -- --exit-after-evidence
    submissions 41, 42 and 43 presented
    each: 2 payloads, 2 local draws, 12 vertices
    submission 41: source-x jitter 0, geometry fingerprint 8a894fff...e92bf
    submission 42: source-x jitter 8, geometry fingerprint 21843a39...de4a5
    submission 43: source-x jitter 0, geometry fingerprint 8a894fff...e92bf
    persistent controls: 2 uploads, 0 replacements across all three frames
    submission 900: missing material rejected atomically
    submission 43: presented successfully after the rejection
    provider diagnostics: none

cargo check -p hello-doom-visibility-conformance-web \
    --target wasm32-unknown-unknown
    passed

cargo build -p hello-doom-visibility-conformance-web \
    --release --target wasm32-unknown-unknown
wasm-bindgen ... --target web ...
    browser package generated

cargo fmt --all
    passed

cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
    passed

cargo clippy -p tokimu-render --all-targets \
    --features experimental-submission-local-geometry -- -D warnings
    passed

cargo test -p tokimu-render \
    --features experimental-submission-local-geometry
    64 passed

cargo test -p hello-doom-visibility-conformance
    113 library tests and all binary-target tests passed

git diff --check
    passed
```

The workspace emitted only the known hard-link fallback warnings from the
incremental compilation cache; these are environmental and not candidate
diagnostics.

Strict WASM-target Clippy remains an independent workspace baseline failure,
not a G2 regression. With the experimental feature disabled,
`tokimu-render` already reports six Rust 1.95 `arc_with_non_send_sync`
findings for WGPU `Sampler`/`TextureView` storage, while `tokimu-platform`
reports the existing `let_unit_value` pointer-lock finding. The G2-enabled web
fixture adds no distinct diagnostic before those unchanged baseline failures.

## Disposition

G2 survives its headless semantic gate and its first native Tokimu/WGPU
realization gate. The backend currently realizes each accepted payload with a
frame-private GPU mesh and clears the semantic arena at `begin_frame`; queue
completion remains a provider concern and is not exposed as submission
identity. Reusing local slot zero across submissions 41--43 therefore does not
create a durable identity, while the two persistent control meshes remain on
the unchanged handle-backed path.

The source-view jitter is real preparation pressure rather than an arbitrary
presentation offset. Submission 42 changes the fixture viewer source X by
eight units and produces a different geometry-only fingerprint. Submission 43
restores the baseline source view and exactly restores submission 41's
geometry fingerprint. Persistent controls remain at two lifetime uploads and
zero replacements throughout. This proves that ephemeral prepared geometry can
change and then recur without acquiring persistent mesh identity or mutating
the persistent controls.

Actual Browser WebGPU execution now proves the same three-submission identity,
jitter, atomic-rejection, recovery and persistent-resource counts as native.
The first visual observation also exposed an ordinary fixture defect: both
browser persistent controls used the same unit-quad footprint, so the nearer
orange control completely covered the blue background control even though both
draws executed. Native already used a large blue outer quad and a smaller
orange inner quad. The browser fixture now uses those same explicit bounds and
the regenerated package reports `persistent_controls=blue-outer-orange-inner`.
Maintainer observation of the regenerated Browser WebGPU package confirms the
blue outer control and orange inner control are both visible. The corrected
browser presentation now matches the native fixture semantically and visually;
this is not a pixel-identity claim.

No stable API follows from this result. The seam remains corpus-only,
feature-gated, hidden from generated documentation and absent from
`Renderer`. G4 persistent replacement remains a negative/control path rather
than an implementation of G2.

## Slice 2 Depth-Relationship Extension

The corrected two-control fixture established browser/native structural and
visual parity, but did not by itself prove the complete Slice 2 depth
relationship. The fixture therefore gained a third persistent control: a small
green object at depth `0.10`, in front of the submission-local authority at
depth `0.25`; the orange far wall remains at depth `0.50`, and the blue sky
colour/background remains at depth `0.90` without writing depth.

The required visual reading is now:

```text
green near control
    survives in front of authority

submission-local authority
    writes depth without colour
    suppresses farther orange where authorized

orange far control
    survives only outside authoritative coverage

blue background
    supplies sky colour where no nearer colour wins
```

Native submissions 41--43 report five draws, three persistent lifetime uploads,
zero replacements, the same two local payloads/12 vertices, stable baseline/
jitter/restored fingerprints, and recovery after the bounded invalid
submission. Actual Browser WebGPU execution reports the same five-draw
submissions, three persistent uploads, zero replacements, and
`depth_controls=near-green-wins,authority-hides-far-orange,blue-sky-remains`.
Maintainer observation confirms all three visible relationships. Together with
the native observation, this closes the Slice 2 depth-relationship gate without
claiming pixel-identical raster output.
