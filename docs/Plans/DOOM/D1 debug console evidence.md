# D.1 debug console evidence

This record retains the first native observation of the corpus-local embedded
debug console. It is not a renderer, Ring 0, or generic Observation Shell
acceptance record.

## Native observation

Command used:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --spawn-observer --walk-collision
```

Manual fixed-window observation:

- Window title reported `Tokimu DOOM E1M1 | 1861 draws`.
- Physical `~` opened the console and released captured mouse look.
- `COLLISION` reported `radius=16`, `blocking_linedefs=315`, `noclip=false`,
  and an empty initial contact list.
- `HELP` reported the bounded corpus-local command set.
- Repeated `LOOK` commands reported exact prepared-triangle hits with distance,
  opaque family, material handle, wall label, and compact source
  linedef/sidedef/sector identity. Flat hits retain source
  subsector/sector/plane identity through the same caller-owned path.
- The console remained readable after long `LOOK` output once font-metric
  wrapping was added. The prompt retained a visible underscore cursor.
- Closing the console returned to the scene without leaving movement or mouse
  capture stuck.

The visual result is a manual observation associated with the command and
source package above; it is not claimed as a pixel-deterministic artifact.

## Retained validation

- `cargo test -p hello-doom-e1m1 -p tokimu-input -p tokimu-platform` passed,
  including font-measured wrapping, bounded transcript, exact center-ray
  intersection, and prepared draw-provenance regressions.
- `cargo check -p doom-ts-boundary-workbench-engine
  --target wasm32-unknown-unknown` passed after the caller-side provenance
  enrichment, preserving the existing browser first-frame corpus path.
- `cargo clippy -p hello-doom-e1m1 -p tokimu-input -p tokimu-platform
  --all-targets -- -D warnings` passed. The pass exposed and repaired one
  oversized private raster-helper signature and one pre-existing
  default-then-reassign fixture construction warning; neither changed a
  public or semantic contract.
- `cargo fmt --all` and `git diff --check` passed.

## Boundary and remaining work

The exact hit is a prepared-triangle result. Wall hits retain a source linedef,
sidedef, and sector; flat hits retain a source subsector, sector, and plane.
Thing inspection is not yet exposed because Things have not entered prepared
caller data; D.1 does not invent radius, height, or billboard semantics solely
to make a source point selectable. The browser intake still presents one frame
and does not own a persistent input/frame lifecycle, so browser console parity
remains open.

The console remains corpus-local composition under AR-0013. No reusable shell,
picking, or Doom source-inspection contract is admitted by this evidence.

## Reuse and admission review

D.1 adds another real consumer of bounded transcript presentation, explicit
focus transfer, normalized keyboard input, and font-measured wrapping. Those
mechanics overlap the console-command-window and runtime-observation corpora,
but the semantic seams do not yet converge:

- Doom owns `CAMERA`, `COLLISION`, `LOOK`, `NOCLIP`, and the source identities
  returned by those commands.
- The console-command-window retains owner-routed Tosumu command/session
  evidence and separately studies Ratatui projection.
- The runtime-observation workbench owns a bounded Rust/WASM observation
  facade rather than a persistent command transcript or picking session.
- D.1's exact ray/triangle query operates on caller-prepared Doom draw data;
  there is no second non-Doom caller demonstrating the same picking contract.

The repeated pressure therefore supports reuse of existing low-level input,
font, UI, and rendering seams, but it does not yet justify extracting a stable
embedded-console or picking capability. AR-0013 remains Incubating. Reopen that
review before extraction if two independent persistent hosts preserve the same
session semantics, or open a separate picking review if a non-Doom caller
requires the same query identity, authority, and failure contract.
