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
