# Doom Viewer-Relative Presentation Conformance

This corpus target is the small, source-first companion to E1M1 for the Doom
viewer-relative presentation study. It builds bounded `DoomMapCore` fixtures,
calls the production Doom geometry provider, and retains structural manifests.

It is not a second renderer, WAD decoder, or generic visibility API. A passing
synthetic fixture proves only its named Doom source invariant. Candidates still
require rendered native/browser evidence and E1M1 falsification before any
claim about the wider campaign.

Run the fast structural tests with:

```powershell
cargo test -p hello-doom-visibility-conformance
```

Run the bounded ordered reference-planner synthetic gate with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin ordered_reference_planner_report
```

This report composes the production Doom provider's near-first BSP order,
solid/pass ranges, vertical coverage transitions, wall tiers, plane
marks/instances, sky intervals, deferred masked-middle work, and explicit
fail-open observations into one deterministic Doom-private manifest. It runs
the retained sky, aperture, shared-plane, runtime-snapshot, projection-edge,
and cutout controls through that one planner. It is a campaign oracle, not
historical pixel parity, a renderer API, or application movement policy.

Run the Doom-private authoritative sky-region model report with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin authoritative_sky_region_report
```

This report consumes the corrected ordered ledger and models only retained
`F_SKY1` ceiling intervals as bounded, normalized view-local regions. It keeps
source plane/SEG order, prepared-view and runtime-snapshot identity, while the
diagnostic columns remain conservation evidence rather than presentation
primitives. Paired-sky boundary events without a retained sky-plane interval
are reported but do not fabricate coverage. One-sky and ordinary-aperture
controls likewise remain empty; ambiguous authority fails open in the unit
fixture with an explained omission.

Run the headless AR-0030 G2 lifetime report with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin submission_local_geometry_report
```

Run the authorized unstable native WGPU intake and exit after its retained
three-frame evidence sequence with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin submission_local_geometry_presentation -- --exit-after-evidence
```

The presentation fixture reuses local slot numbers under three distinct
submission identities, retains two ordinary persistent control meshes, and
proves bounded rejection followed by recovery. It is a feature-gated corpus
experiment, not stable renderer vocabulary.

Run the source-topology admission matrix with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin topology_admission_report
```

Run the Doom-private prepared source-occurrence model report with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin source_occurrence_model_report
```

This headless Slice 1 control proves that one stable Doom source contribution
can produce two disjoint, correlated occurrences without assigning renderer
resource identity early. It also exercises whole retain, positively authorized
reject, unresolved fail-open, and one shared prepared boundary consumed by wall,
floor, ceiling, and sky preparation. The model is campaign-private; only the
bounded observation line is exported by this crate.

Run the headless partial-survival reconstruction with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin partial_survival_reconstruction_report
```

This Slice 2 report replays one source SEG at baseline, jittered, and nearer
viewer poses. It constructs continuous source-relative survivor intervals,
validates their private occurrence identities, and reconstructs wall geometry
and UVs from the source SEG. Diagnostic columns are compared only afterward;
they are never inverse-projected into geometry. The report also exercises
near-plane and unsupported-role fail-open controls, a positively authorized
empty result, and a thin valid interval.

This report classifies original source-labelled occurrences as `admitted`,
`rejected`, or `unresolved-fail-open`. A rejection requires positive terminal
source provenance. Mere absence, back-facing projection, near-plane ambiguity,
or unsupported semantics never becomes rejection. The report also compares
declared door and platform height snapshots without implementing activation,
timing, or movement policy. It is Doom-study evidence, not a public Tokimu
visibility or topology API.

Run the shared wall/plane boundary conservation audit with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin shared_boundary_conservation_report
```

This Slice 3 report consumes the production provider's ordered transition,
wall-cell, plane-instance, and paired-sky observations for five bounded source
fixtures. It verifies that retained wall and plane contributions share one
ordered boundary account, that paired-sky events do not acquire independent
occlusion authority, and that a two-sided masked middle does not close source
coverage. Bounded ray-depth uncertainty remains explicit fail-open evidence;
it is not silently counted as rejection or repaired by a second coverage
algorithm.

Run the whole-contribution partial-survival falsifier with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin whole_contribution_falsifier_report
```

It keeps the partial-paired-sky fixture's source geometry unchanged and compares
the whole-contribution control with the conservative topology admission result
at baseline, jittered, and nearer viewer poses. The report is expected to say
`status=falsified`: valid side intervals and a source-invalid central overlap
belong to one source SEG, and no ordinary depth authority exists to repair a
whole-source keep. This is retained AR-0030 evidence, not a fragment API or an
invitation to patch the fixture.

Run the ordinary occurrence-lowering report with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin occurrence_presentation_lowering_report
```

It lowers the whole-retain control and both partial-survival occurrences into
ordinary Tokimu `Mesh` declarations before any renderer call. The report
retains source order, source/occurrence correlation, continuous source
intervals, UV completeness, view-local generated-geometry scope, and a stable
structural fingerprint. It is the headless semantic counterpart to
`ordered_coverage_presentation`; neither path admits Doom occurrence vocabulary
to `tokimu-render`.

Run the first small native presentation control with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin paired_sky_presentation
```

It uses no E1M1 package or renderer-specific visibility API. The fixed scene
draws a blue sky background, then the source-owned paired-sky depth boundary,
then the far red wall. Its purpose is only to observe that declared Doom-local
ordering; it does not claim classic Doom visplane or pixel parity. The title
and terminal report the first-frame draw/material/pipeline counts. The second
unchanged frame reports warm resource counts and fails explicitly if a static
mesh was uploaded or replaced. Any drained backend diagnostic is promoted to
a terminal fixture error instead of leaving a silently black window. The third
frame applies one bounded camera translation and likewise fails if camera
movement causes static mesh upload or replacement churn.

Run the one-sky negative control against the same presentation path with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin paired_sky_presentation -- --one-sky
```

That control changes the source topology so only the higher viewer-side
ceiling is `F_SKY1`. The invisible paired-sky boundary must disappear and the
authored upper wall must be visible in green. The distant lower red wall
remains an independent draw. This is a negative authority check: merely
mentioning sky on one side must not turn an ordinary wall into a depth-only
mask.

Run the vertical-aperture presentation control with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin vertical_aperture_presentation
```

The production SEG wall lowerer supplies a green upper tier and yellow lower
tier around the source opening `24..96`. An orange far control should remain
visible through that opening while the two near tiers cover only their own
vertical intervals. The fixture fails on backend diagnostics, missing source
tiers, or unchanged warm-frame mesh churn.

Run the shared-plane-key control with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin shared_key_plane_presentation
```

It lowers bounded subsector floor surfaces from two decoded sectors that share
the same ordinary floor-key facts. Green sector 0 and orange sector 1 must
remain separate rendered instances. This guards source-plane identity only; it
does not admit a renderer-owned plane cache or candidate-selection policy.

Run the stationary dynamic-door snapshot control with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin dynamic_door_snapshot_presentation
```

The green left-side band comes from the production two-sided-wall lowerer over
an explicit closed ceiling snapshot. The orange right-side far control denotes
the corresponding open snapshot, for which the same lowerer produces no wall
band. The fixture supplies no `E` handling, ticking, waiting, or reversal
policy; those remain E1M1 application concerns.

Run the source-projection edge control with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin projection_epsilon_presentation
```

One source SEG crosses behind the viewer and therefore fails open without a
rendered wall. A one-unit-wide valid SEG and an extremely-close valid SEG are
both lowered and presented. Their per-control horizontal magnification is
diagnostic presentation only; it does not claim pixel-width parity or a
renderer projection contract.

Run the stationary moving-platform snapshot control with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin platform_snapshot_presentation
```

It lowers two displayed wall meshes from the same immutable source map with
declared current floor heights `0` and `48`. This is a source-preparation
control, not a Doom platform activation or timing simulation.

Run the headless runtime-snapshot occurrence report with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin runtime_snapshot_occurrence_report
```

It sends declared closed, opening, open, and closing door heights plus low and
raised platform heights through the same source-occurrence preparation seam.
The report retains source/occurrence/resource correlations and the bounded
create/replace/retire reconciliation implied by each immutable snapshot. It
does not simulate activation, ticks, waiting, reversal, or any other
application-owned movement policy. The resource correlations are study-local
reconciliation evidence, not a newly admitted renderer allocation or
retirement API.

Run the headless Candidate 1 synthetic conservation matrix with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin candidate1_synthetic_matrix_report
```

The matrix combines all fourteen ordered-reference planner cases with the
paired-sky positive and negative controls, vertical aperture, shared plane
identity, explicit door/platform snapshots, projection edge cases, and the
cutout non-occluder control. Candidate 1 contributes only source-authorized
submission-local sky-depth geometry. A negative authority result skips that
G2 batch; it is not represented as an empty or failed submission. Ordinary
fixture contributions remain on their existing presentation paths.
