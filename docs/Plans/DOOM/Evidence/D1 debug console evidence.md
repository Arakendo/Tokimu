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
- `LOOK` now also reports the nearest active current Thing billboard triangle,
  its before/behind relation to the ordinary world hit, source Thing
  record/lump, kind, live source pose, sector, and combat health. This reuses
  the exact billboard already prepared for presentation rather than inventing
  a diagnostic selection volume.
- `STATUS` now retains the current map, player-start Thing record, live source
  position and heading, sector, and player health alongside its existing frame
  and presentation state.
- `INVENTORY` retains prepared draw/command counts and the application-owned
  live mesh, texture, material, pipeline, and camera handle inventory. It is
  labeled `app-owned-upload-inventory-not-physical-gpu-allocation`; it does
  not claim physical allocation size, retirement, or reclamation timing.
- `CATALOG` makes the small shell's authority visible by separating read-only
  inspection (`STATUS`, `INVENTORY`, `WARNINGS`, `TIMINGS`, `CAMERA`,
  `COLLISION`, `LOOK`, `SCAN`) from mutating corpus controls (`USE`,
  `NOCLIP`).
- `WARNINGS` separates import/preparation findings from bounded live runtime
  failures. Unsupported linedef specials and Thing kinds retain total counts
  plus at most eight source-record samples per family. Runtime audio failures
  retain the latest 16 entries. Required asset failures remain fatal before a
  scene can present, so this observation does not hide missing visible assets
  behind a successful-looking warning state.
- `TIMINGS` separates WAD-package parse, selected-map decode, remaining scene
  lowering, provider upload/pipeline setup, and the latest frame CPU interval.
  It explicitly leaves audio preparation unseparated and makes no GPU-timing
  claim.
- `LOOK` also retains the source ray as map `x/y`, vertical height, map-plane
  direction, and vertical direction alongside the Tokimu world ray. Its
  copyable `--look-ray-report=...` token replays that exact source-space probe
  headlessly against the canonical prepared scene, including explicit no-hit
  results and the source coordinate of an exact hit.
- Sky-boundary investigations additionally retain the nearest paired-sky
  depth-boundary intersection and its relation to the ordinary hit. This
  supports headless assertions about problem rays without treating a rendered
  screenshot or generic renderer occlusion as Doom source truth.
- The same observation now reports intersections with omitted source
  `F_SKY1` planes separately from paired-sky wall boundaries. Its bounded
  classic-source trace retains the viewer leaf, target leaves, target SEG
  admission, and watched BSP elisions. A wall-249 replay demonstrated a global
  prepared-shell hit whose two source SEGs were not admitted by the Doom-owned
  horizontal protocol.
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
- The focused static-scene suite passed with 31 tests after adding source-ray
  parsing and replay-format regressions. A canonical headless spawn ray also
  completed with an explicit no-hit observation while preserving source
  `(1056,-3616,36)`, source direction `(0,1,0)`, and its world-space lowering.
- `cargo check -p doom-ts-boundary-workbench-engine
  --target wasm32-unknown-unknown` passed after the caller-side provenance
  enrichment, preserving the existing browser first-frame corpus path.
- `cargo clippy -p hello-doom-e1m1 -p tokimu-input -p tokimu-platform
  --all-targets -- -D warnings` passed. The pass exposed and repaired one
  oversized private raster-helper signature and one pre-existing
  default-then-reassign fixture construction warning; neither changed a
  public or semantic contract.
- `cargo fmt --all` and `git diff --check` passed.
- The expanded static-scene suite passed all 112 tests after adding the bounded
  player-status observation and Thing-hit reporting. The focused player-status
  regression retains source identity, pose, sector, and health; a warning
  regression proves that source samples stop at eight while the total remains
  visible. Strict
  package Clippy remains blocked by pre-existing dead diagnostic-study code and
  existing manual-range/parity warnings outside this change; no clean Clippy
  result is claimed for this increment.

## Boundary and remaining work

The exact hit is a prepared-triangle result. Wall hits retain a source linedef,
sidedef, and sector; flat hits retain a source subsector, sector, and plane;
Thing hits retain identity from the active prepared billboard. The browser
working model now composes the same corpus-owned transcript and prompt
rasterizer through an always-present transparent overlay draw. Its
input/presentation path is no longer open; exact native Doom source-ray command
coverage remains a separate, unclaimed vocabulary difference.

The console remains corpus-local composition under AR-0013. No reusable shell,
picking, or Doom source-inspection contract is admitted by this evidence.

### Browser persistent-host architectural finding

The browser working model now falsifies the earlier lifecycle blocker: it
retains one input/frame loop and one ADR-0018 WGPU resource-set session across
movement and map replacement. It does not, however, make the native console a
mechanical port.

The accepted replacement session originally exposed set-scoped command
submission and a deliberate live-camera update, but no same-set texture
transaction. AR-0033 compared the resulting choices and produced ADR-0019.

1. stage and atomically replace the complete map resource set for each console
   edit;
2. admit a scoped dynamic presentation-resource update inside an authoritative
   set; or
3. present the console as a browser DOM/host overlay, which would abandon the
   D.1 requirement that console presentation use ordinary Tokimu render seams.

ADR-0019 now admits only fixed-descriptor, set-scoped atomic texture-content
replacement. The transaction preserves the current set and scoped commands,
contains failed or abandoned candidates, and retains the no-backend-bypass
shape. The browser D.1 implementation applies that operation without changing
its authoritative map set or scoped command batch.

### Browser/WASM retained observation

An isolated Edge run loaded the reviewed shareware package, presented E1M1,
opened the embedded console, typed `CAMERA`, submitted it, closed and reopened
the console, and presented the result. Tokimu's observer ran outside the page
failure domain and classified the operation `completed`:

```text
operation=doom-browser-console-adr0019
sequence=open>type-CAMERA>submit>close>reopen>present
resource-set=1 throughout
texture=TextureHandle(9100010)
descriptor=960x264 RGBA8 sRGB throughout
source-bytes=1,013,760 per update
dependent-materials=1
commands-retained=true
updates=5
provider-diagnostics=0
terminal-classification=completed
elapsed-ms=2,084.744
```

The terminal record is retained at
`target/browser-terminal-observer-34364-1787290796439710200.terminal.json`.
The operator separately typed `HELP` and `STATUS` in the live console and
confirmed that the readable transcript and prompt were composited over the
still-visible E1M1 frame. Closing the console removed the overlay while E1M1
remained presented; that update retained resource set 1, texture identity
`TextureHandle(9100010)`, and the scoped commands, with zero provider
diagnostics. The page did not claim physical GPU reclamation timing. The
browser vocabulary proves `CAMERA`, `HELP`, `CLEAR`, `STATUS`, and `NOCLIP`;
it does not claim parity for native `LOOK`, collision-world, door, or source
identity commands whose caller data is not retained by this host.

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
