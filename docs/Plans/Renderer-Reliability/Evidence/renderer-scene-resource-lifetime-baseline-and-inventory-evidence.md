# Renderer Scene-Resource Lifetime Baseline And Inventory Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Plan | [Renderer Scene-Resource Lifetime And Replacement](../renderer-scene-resource-lifetime-and-replacement.md) |
| Status | Slice 1 accepted; Slice 2 inventory retained |
| Callers | Doom TypeScript boundary workbench and `hello-render-resource-identity-web` |

## Evidence Boundary

The browser log has not reported an explicit out-of-memory error, WGPU device
loss, or validation failure. Edge closed after cumulative map switching, first
around E1M3 and—after eliminating simultaneous surfaces—later around E1M5 or
E1M6. This record therefore describes observable lifetime pressure and current
ownership. It does not claim that memory exhaustion is the cause.

The added estimates cover only facts available before provider submission:

- WGPU vertex payload shape: position, normal, and texture coordinate, or 32
  bytes per lowered vertex;
- complete source RGBA8 payload bytes supplied during texture creation; and
- logical mesh, texture, material, pipeline, camera, and command counts.

They exclude surface images, depth/stencil storage, buffer alignment, staging,
bind groups, samplers, shader/pipeline allocations, driver overhead, internal
caches, and physical residency. Logical retirement is not physical
reclamation.

## Alternative-A Reproduction Harnesses

### Doom Working-Map Rotation

The browser workbench now exposes **Run 3x map rotation**. One retained
Rust/WASM intake session performs the deterministic sequence E1M1 through E1M9
three times. It yields one browser animation frame before and after every map,
but does not interpret that yield as a reclamation fence.

Each successful Rust observation now reports:

```text
lifetime-baseline=whole-backend-replacement
replacement-attempts
replacements-presented
backend-creations
device-creations
surface-creations
current-logical-resources
current-estimated-bytes
retired-logical-sets
retired-logical-resources
retired-estimated-bytes
physical-gpu-reclamation=unobserved
```

The TypeScript harness retains a bounded 27-entry record containing sequence,
round, map, replacement elapsed time, and the Rust observation. A returned
Rust/WASM error becomes `map-rotation-rejected`. If the browser renderer, GPU
process, or whole Edge window disappears, the absent completion record must be
joined with the external Edge/Crashpad log; page JavaScript cannot honestly
report after its own host disappears.

The historical manual walkabout remains separate. Automation proves repeated
replacement pressure; it does not reproduce every movement, camera, or
pointer-lock condition.

### Independent Non-Doom Control

`hello-render-resource-identity-web` now retains a
`BrowserReplacementPressure` session and exposes **Run 27 whole-backend
replacements**. Each iteration drops the previous backend, creates a fresh
backend/device/surface on the same canvas, and uploads:

- 64 meshes;
- 64 independent 16x16 RGBA8 textures;
- 64 textured materials;
- one pipeline;
- one camera; and
- one clear plus 64 draw commands.

It uses no WAD, Doom geometry, sky parity, sector trim, or map preparation.
Matching cumulative failure here would implicate provider-session churn more
strongly; survival here would not acquit the larger Doom workload.

## Current WGPU Ownership Inventory

| Storage | Current lifetime | References / dependencies | Reset consequence |
| --- | --- | --- | --- |
| `meshes` | Backend | Draw commands resolve `MeshHandle` at presentation | Can clear only when no current queued command may resolve the old set |
| `textures` | Backend | Materials retain `Arc<TextureView>`; render targets may also retain depth storage | Removing the map entry alone does not retire views captured by materials |
| `materials` | Backend | Bind groups retain texture views and samplers | Must retire before or with their referenced textures |
| `derived_materials` | Backend cache | Derived key points to source material identity; value clones its view/sampler references | Must retire with source materials; current code invalidates only affected entries on material upload or target replacement |
| `pipelines` | Backend | Queued draws resolve pipeline handles | Must remain until no current draw references them |
| `pipeline_registry` | Backend | Label-to-handle mapping and monotonic next handle correspond to compiled pipelines | Clearing only compiled pipelines leaves stale label resolution; both sides require one policy |
| `renderables` | Backend | Expand to mesh/material/pipeline handles during `submit` | Must not survive if any referenced set is retired |
| `cameras` | Backend | Queued draws select camera handles | Replacement may reuse logical handles only after old draws are unreachable |
| `camera_bindings` | Backend cache | GPU bindings keyed by camera handle | Retains prior handle generations unless explicitly reset or reused |
| `instance_bindings` | Backend high-water cache | Presentation indexes bindings by draw position | Grows to the largest submitted draw count and does not shrink at `begin_frame` |
| `queued_draws` | Frame | Holds logical resource handles | Cleared by `begin_frame`; this is the existing quiescent logical boundary |
| `submission_local_meshes` | Frame when feature-enabled | Queued submission-local slots | Cleared by `begin_frame`; this precedent does not cover persistent textures/materials/pipelines |
| `SurfaceState` | Backend/provider session | Surface config, depth/stencil texture/view, and bind-group layouts | Must survive ordinary resource-set replacement under B; recreated only for surface/session lifecycle |
| WGPU instance/device/queue | Backend/provider session | Owns or schedules every concrete provider resource | Must survive ordinary replacement under B; drop does not prove immediate physical reclamation |
| backend diagnostic sink | Device callback lifetime | Retains messages until caller drains them | Not scene state; must remain observable across replacement and bounded/drained by the caller |
| render statistics | Backend | Frame and lifetime counters | A retained backend needs explicit per-set observations without erasing session totals |

## Dependency And Retirement Order

The observed persistent dependency shape is:

```text
queued command
    -> renderable (optional expansion)
    -> mesh
    -> material
        -> texture view
        -> sampler
    -> pipeline
    -> camera
        -> camera binding cache

surface/provider session
    -> bind-group layouts
    -> depth/stencil surface storage
    -> device + queue
```

The minimum safe logical quiescence point currently begins after
`begin_frame()` has made old queued draws unreachable. A whole-set prototype
must then retire dependent objects before their sources, conceptually:

```text
queued draws / renderables
    -> derived materials / materials
    -> textures

queued draws
    -> meshes / pipelines / cameras

camera and instance binding caches
    -> reset or retain only under an explicit provider-session cache rule
```

This is a logical ordering statement, not a claim that WGPU completes or frees
physical allocations synchronously in that order.

## Baseline Findings

1. Alternative A conflates composition replacement with provider-session
   replacement. Every successful map produces one new backend, device, and
   surface.
2. `begin_frame()` already supplies a narrow quiescent boundary for queued
   draws and submission-local meshes, but no equivalent boundary covers the
   persistent resource graph.
3. A texture map entry is not the sole owner of a sampleable texture view;
   material and derived-material bind groups can keep it alive.
4. Pipeline lifetime comprises both compiled provider objects and the
   provider-neutral registry. Resetting only one side would create stale
   identity.
5. Instance bindings are a provider-session high-water cache rather than
   obvious scene membership. Camera bindings are keyed by application handles
   and therefore need an explicit reuse/invalidation rule.
6. The current consumer estimates are adequate for comparing logical growth
   and submitted payloads, but cannot establish GPU residency or reclamation.

## Live Doom Alternative-A Observation

The maintainer ran the three-round control in Microsoft Edge on 2026-08-19.
The page returned:

```text
kind=map-rotation-complete
requested-replacements=27
completed-replacements=27
elapsed-ms=19657.4
physical-gpu-reclamation=unobserved
```

All nine maps completed three times. Replacement timing was:

| Map | Runs | Minimum ms | Mean ms | Maximum ms |
| --- | ---: | ---: | ---: | ---: |
| E1M1 | 3 | 422.0 | 493.7 | 632.1 |
| E1M2 | 3 | 741.5 | 765.1 | 801.9 |
| E1M3 | 3 | 787.3 | 918.8 | 1,143.8 |
| E1M4 | 3 | 597.5 | 624.5 | 673.6 |
| E1M5 | 3 | 651.9 | 743.3 | 892.7 |
| E1M6 | 3 | 1,072.6 | 1,178.9 | 1,357.1 |
| E1M7 | 3 | 778.6 | 794.7 | 827.0 |
| E1M8 | 3 | 369.4 | 454.9 | 536.7 |
| E1M9 | 3 | 464.1 | 561.8 | 754.2 |

The final E1M9 record reported 27 backend/device/surface creations and 26
logically retired sets comprising 46,762 meshes, 1,644 textures, 1,670
materials, 182 pipelines, 26 cameras, and 88,199 commands. The submitted
payload estimates accumulated to 12,252,864 mesh-vertex bytes and 54,255,104
source-texture bytes across those retired sets. These are cumulative logical
facts, not simultaneously live counts.

The page and Edge window survived. The retained Edge log contains no new GPU
process start during this run and no device-loss, OOM, WGPU validation, fatal,
or Crashpad record. Its repeated fallback-task-provider warning remains an
Edge task-manager warning and does not identify a Tokimu or WebGPU failure.

This successful run falsifies a deterministic E1M3/E1M5/E1M6 replacement
crash under the automated no-movement workload. It does not falsify movement-
conditioned pressure, timing-sensitive reclamation, or the need for a retained
provider session. The separate manual walkabout and non-Doom 27-replacement
control remain required.

## Live Independent Alternative-A Observation

The maintainer then ran the corrected port-4177 non-Doom control in the same
Edge session. It returned:

```text
status=complete
replacements=27
elapsed-ms=1644.4
minimum-replacement-ms=15.0
mean-replacement-ms=46.92
maximum-replacement-ms=381.7
physical-gpu-reclamation=unobserved
```

Every replacement presented 64 meshes, 64 textures, 64 materials, one
pipeline, one camera, and 64 draws. The final record retained 27 fresh
backend/device/surface creations and 26 logically retired sets. No diagnostic
was returned and the page/window survived.

Together, the two automated controls show that repeated whole-backend creation
is not deterministically fatal either for the nine Doom maps or for a smaller
independent resource-rich caller. The earlier adversarial movement/map-switch
walkabout remains the retained negative baseline. The evidence therefore
separates a deterministic map defect from timing/movement/provider-lifetime
pressure without identifying physical memory exhaustion as the cause.

## Validation Retained So Far

- `cargo check -p doom-ts-boundary-workbench-engine --target wasm32-unknown-unknown`
- Doom workbench TypeScript strict typecheck and complete build
- `cargo check -p hello-render-resource-identity-web --target wasm32-unknown-unknown`
- `cargo test -p hello-render-resource-identity` (18 passed)
- `cargo fmt --all`

Slice 1 is accepted with both automated controls plus the earlier adverse
manual Doom walkabout. No shared renderer contract is admitted by this
evidence. The plan may now advance to the feature-gated, corpus-private
Alternative B prototype.

## External Terminal-Outcome Observer

ADR-0017 follow-up added the corpus-private
`hello-browser-terminal-observer` supervisor. It launches one isolated browser
profile that it owns, hosts a bounded loopback observation endpoint outside the
page, and correlates every record with both a run identity and a page-subject
identity. The independent renderer fixture and Doom browser workbench now emit:

```text
subject-started
operation-started
heartbeat
page-error
operation-completed | operator-completed | structured-rejection
```

The separate page-subject identity is required because a renderer/page crash
followed by automatic reload could otherwise resume heartbeats under the same
run identity and conceal the lost operation. A changed subject before a
terminal record is therefore classified as `unresolved-disappearance`, not as
recovery or a fresh success.

The observer bounds request and field sizes, uses a unique profile and browser
log per run, terminates only the browser process it launched, and reports an
unknown physical cause unless the host explicitly supplied one. It currently
observes the owned browser process and page liveness. It does not independently
enumerate renderer or GPU processes, and browser-log text is not promoted into
that missing process identity evidence.

Focused validation retained on 2026-08-19:

- `cargo test -p hello-browser-terminal-observer` (7 passed);
- `cargo clippy -p hello-browser-terminal-observer --all-targets -- -D warnings`;
- Doom workbench TypeScript build and strict typecheck;
- JavaScript syntax checks for both instrumented fixtures; and
- a deliberately terminating external subject, classified by the supervisor
  as `externally-terminated`, exit code 3, with `cause=unknown`, and retained
  as a schema-versioned terminal JSON artifact.

The actual Edge/WebGPU rotation and adversarial Doom walkabout have not yet run
under this observer. This section is implementation and controlled-harness
evidence, not a browser survival result and not a resolution of the earlier
Edge disappearance.

## Alternative-B Prototype Boundary

The feature-gated `experimental-scene-resource-reset` seam now clears the
complete logical scene graph in dependency order while retaining the WGPU
instance/device/queue/surface, diagnostics, statistics, and the draw-indexed
instance-binding high-water cache:

```text
queued draws / renderables
    -> derived materials / materials
    -> textures

meshes / compiled pipelines / pipeline-label registry
cameras / camera bindings / active camera
submission-local meshes when enabled
    -> cleared

instance / device / queue / surface
diagnostic sink / statistics / instance-binding high-water cache
    -> retained
```

The returned observation reports the logical counts removed and the retained
instance-binding count. It explicitly reports no physical GPU reclamation.
Both the Doom workbench and the independent fixture can run 27 replacements
through this retained session without changing their preparation or draw
declarations.

Static inspection already identifies two exact Alternative-B sufficiency-gate
falsifiers, both exposed as live browser probes:

1. Reset retires the preceding logical set before successor GPU staging. If
   staging fails, the previous scene is no longer addressable; B therefore
   cannot provide atomic last-known-good replacement.
2. Mesh, material, pipeline, and camera commands contain bare numeric handles.
   Once a successor set reuses those values, an old command is
   indistinguishable from a current command and resolves successor resources;
   B therefore cannot provide cross-set stale-handle rejection.

Immediate post-reset missing-handle errors remain deterministic, but they are
not generation safety. The browser probes are retained to confirm the actual
provider path and to distinguish these semantic failures from a reset crash.
Alternative C is earned only by these two concrete failures, not by a general
preference for arena vocabulary.

Validation retained for this increment:

- `cargo test -p tokimu-render --features experimental-scene-resource-reset`
  (64 passed)
- native `tokimu-render` clippy with the experimental feature and warnings
  denied
- independent fixture release WASM build and generated-binding refresh
- Doom workbench WASM build, generated-binding refresh, TypeScript build, and
  strict typecheck
- JavaScript syntax checks for both browser fixtures

WASM-target clippy remains blocked by the pre-existing
`arc_with_non_send_sync` findings for WGPU texture views and samplers. The
feature compiles for WASM; this increment neither introduced nor suppresses
those findings.
