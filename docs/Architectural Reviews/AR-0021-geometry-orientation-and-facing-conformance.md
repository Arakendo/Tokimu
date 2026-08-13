# AR-0021: Geometry Orientation And Facing Conformance

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-08 |
| Last reviewed | 2026-08-08 |
| Scope | Cross-cutting rendering and corpus conformance |
| Trigger | Two independent corpus paths exposed apparently inside-out facing: the E1M1 headless wall-normal artifact and the decoded Khronos `Box.glb` workbench preview. |
| Related ADRs | ADR-0001, ADR-0004, ADR-0008, ADR-0009 |
| Related evidence | `corpus/lib/doom-geometry-provider`, `corpus/campaigns/doom/hello-wad-inspect`, `corpus/focused/data-interchange/hello-glb`, `corpus/consumers/aspnet-wasm-asset-workbench`, `tokimu-render` mesh/pipeline/WGPU boundary, E1M1 normal SVG, Box.glb workbench observation |
| Admission exception | None |

## Architectural Question

What explicit, testable geometry-orientation contract must Tokimu preserve from
authored positions through mesh lowering, transforms, pipeline construction,
and native/WASM presentation, so that front/back facing and normal direction
cannot silently invert between corpus consumers?

## Context

E1M1 initially rendered cyan `right` arrows toward the apparent exterior of
many rooms. The focused Doom review established that classic WAD sidedef slot
0 (`right_sidedef`) is the source front side, while the headless wall triangles
were wound for its inverse under the current `(doom_x, height, doom_y)`
embedding. The geometry provider and SVG were corrected together, with a
non-axis-aligned cross-product regression test.

Separately, the decoded Khronos `Box.glb` workbench preview has been observed
with apparently inside-out facing. That path starts from an indexed 3D GLB
primitive with authored positions and normals, then exposes provider-neutral
triangles to a browser-side diagnostic perspective view that applies back-face
culling. It is therefore not the same source adapter or dimensionality as
Doom. The shared symptom raises a legitimate cross-cutting question about
Tokimu's mesh convention, transform handedness, normal handling, preview
culling, pipeline front-face default, or the diagnostic interpretation.

Neither observation alone proves an engine defect. The Doom symptom included a
demonstrated adapter winding defect; the Box.glb workbench symptom may instead
involve decoded mesh order, camera/projection transform, browser preview
culling, WGPU front-face/culling defaults, or a combination. This review
preserves that distinction while treating the repeated symptom as high-value
corpus pressure.

The same workbench observation also reported inverted default orbit controls.
That symptom is related evidence rather than proof of inverted geometry: camera
motion, projection, and facing classification were implemented together in the
browser adapter, and an unintuitive camera convention can make an otherwise
correct model harder to inspect reliably.

## Trigger And Evidence

- Corpus examples:
  - `hello-wad-inspect --map-normal-svg E1M1` generated a source-facing
    diagnostic that exposed inverted right/front wall winding. The corrected
    artifact is
    [`E1M1-wall-normals.svg`](../../corpus/lib/doom-wad-package/results/E1M1-wall-normals.svg).
  - The Asset Workbench's decoded `Box.glb` preview has an independently
    observed inside-out-facing symptom after Tokimu GLB decoding and browser
    diagnostic projection/back-face culling.
  - The same preview's default orbit moved camera-facing features opposite the
    pointer on both axes despite claiming a model-following drag convention.
- Automated tests:
  - `doom-geometry-provider` now verifies both side-0/front and side-1/back
    normals for an oblique wall with the triangle cross product.
  - Current mesh and pipeline tests exercise construction and validation, but
    do not yet prove a cross-layer front-face result under culling.
- Audits or diagnostics:
  - The Doom source-side review documents slot 0 as front and slot 1 as back.
  - `tokimu-render` maps `CullMode` to WGPU culling but currently inherits
    WGPU's primitive front-face default rather than exposing a Tokimu-facing
    orientation contract.
  - `hello-glb` explicitly selects `CullMode::None`; it can compare decoded
    shape and shading, but cannot currently act as a front/back-culling oracle.
- Repeated implementation friction:
  - One map adapter required a winding correction, while an imported 3D model
    preview reports the same visual class of symptom.
- Missing evidence:
  - No canonical Tokimu statement currently binds mesh vertex winding, derived
    normal direction, camera/projection handedness, front-face selection,
    culling, and normal transformation into one testable contract.

## Ownership Analysis

The meaning under review is not Doom side semantics or GLB source semantics.
It is the renderer-facing truth of triangle orientation: the ordered position
triple establishes a geometric normal; transforms preserve or explicitly
account for orientation; a pipeline classifies front/back faces consistently;
and supplied vertex normals are either verified against or deliberately kept
independent from geometric facing.

`tokimu-render` should own the provider-neutral rendering contract and its
validation/diagnostics. Individual corpus adapters own translation from their
source conventions into that contract. `tokimu-platform` and the WGPU backend
own mechanism-specific realization, not a second semantic convention.

This is a rendering capability concern, not Ring 0 simulation truth. It must
not make renderers own source-map sectors, UI layout, simulation state, or a
universal importer convention.

## Dependency Direction

```text
Current:

Doom WAD / decoded GLB triangles
    -> corpus-local lowering and tessellation
    -> Mesh positions + caller-supplied normals
    -> tokimu-render pipeline render state
    -> WGPU primitive defaults and shaders
    -> native observation

Proposed evidence direction:

Tokimu renderer orientation contract + bounded diagnostics/tests
    <- corpus lowerers adapt their source-specific side/winding rules
    <- WGPU and WASM/native backends realize the same declared contract
```

No WGPU type, GPU object, or provider-specific face enum should leak into
corpus geometry or engine-neutral simulation crates.

## Alternatives Considered

### Alternative A: Treat Each Symptom As Corpus-Local

- Benefits: the known Doom issue was corrected quickly without broad engine
  work.
- Costs: the same class of mistake can recur at every adapter/mesh boundary.
- Failure mode: default front-face and transform assumptions remain implicit,
  so a later GLTF, UI, or WASM path silently disagrees.

### Alternative B: Add A Renderer Orientation Contract And Cross-Layer Proofs

- Benefits: separates source-side mapping from engine-facing semantics; gives
  every corpus a shared oracle; supports native/WASM parity.
- Costs: requires a small public-or-internal contract decision, diagnostic
  infrastructure, and real backend captures.
- Failure mode: over-generalizing into a scene/importer coordinate framework
  before the focused proofs show that one is needed.

### Alternative C: Disable Culling Or Make All Presentation Two-Sided

- Benefits: hides some visible failures immediately.
- Costs: masks wrong geometry, does not correct normal-dependent behavior, and
  undermines the evidence value of the corpus.
- Failure mode: a workaround becomes an accidental rendering policy.

## Findings

1. The E1M1 issue was a real source-adapter winding defect, now corrected and
   independently tested. It is not, by itself, evidence that the renderer
   reverses every mesh.
2. The Khronos Box decode retains all 12 source triangles, and every decoded
   triangle's geometric normal agrees with each referenced authored normal.
   The decoder therefore does not reverse this fixture's winding.
3. The workbench's Rust/WASM preview transport retains those 12 triangles after
   its scene transform, and every transported geometric normal points outward
   from the transformed Box center. That boundary does not reverse the fixture.
4. The browser projection accounts for Canvas's downward-positive Y axis. A
   `-Z` triangle facing its identity camera projects with positive screen-space
   winding and the reversed triangle is rejected. This rule now has an exact
   executable regression.
5. The workbench did contain a related camera defect: pointer deltas were
   subtracted, making a camera-facing feature move opposite a drag on both
   axes. Orbit direction is now explicit and tested independently from facing.
6. These proofs narrow the historical Box symptom away from decode, transport,
   and the browser's isolated culling sign. They do not reproduce or explain
   the original visual report under the complete deployed workbench. The
   independent shared fixture now proves native/WASM renderer agreement, while
   the deployed workbench comparison remains open.
7. The present renderer boundary has a documentation and test gap: it can
   select a cull mode but does not expose or verify the complementary
   front-face and transform-orientation assumptions.
8. Supplied normals and geometric face winding are distinct inputs. A uniform
   `+Z` normal can look wrong under lighting even when a 2D triangle's winding
   is correct, and correct winding can still be culled if an orientation-
   reversing transform is not accounted for.
9. `Mesh::uniform_normal` has no current corpus caller. Its only present uses
   construct Tokimu's built-in triangle, quad, and diamond; those meshes and
   the built-in cube now prove every shading normal agrees with its triangle's
   geometric normal. The API documents intentional disagreement as a caller
   responsibility rather than treating supplied normals as facing policy.
10. The tempting shared-defect theory is not supported by the retained evidence.
    Doom lowering reversed its source front side, while the workbench reversed
    orbit interaction. Similar visual suspicion arose from independent defects.
    The shared architectural finding is narrower and still real: renderer cull
    policy exists without a complementary explicit front-face and transform-
    orientation contract.
11. A shared corpus fixture now retains the proposed matrix independently from
    Doom and GLB: paired opposite-winding triangles, deliberately identical
    shading normals, front/back fragment colors, fixed depth state, all three
    cull modes, an ordinary transform, an uncompensated reflection, and a
    once-compensated reflection. Its 12 semantic cases pass declaration,
    draw-contract, winding, and determinant tests. Native and browser/WASM
    pixel captures now agree.
12. The native WGPU consumer compiles and renders the shared shader and all 12
    cases on an AMD Radeon RX 7900 XTX. The retained matrix shows both faces
    under no culling, only green/front fragments under back-face culling, and
    only magenta/back fragments under front-face culling for all four transform
    rows. The reflection and once-compensated rows agree with their retained
    expectations. The browser/WASM capture agrees in every cell.
13. A browser/WASM consumer compiles and packages the same Rust fixture,
    shader, cull modes, transforms, and shared 12-cell layout. Edge and the
    in-app browser now reach `ready`; the retained Edge capture agrees with the
    native WGPU matrix in every cell. Canvas 2D was not substituted.
14. The earlier browser timeout was a renderer-diagnostic portability defect,
    not failed WGPU surface acquisition. `std::time::Instant::now()` aborted on
    WASM immediately before `Surface::get_current_texture()`. Renderer CPU
    timing now uses `Performance.now()` when a browser clock is available and
    otherwise omits the optional measurement; native timing retains `Instant`.
15. Browser WGPU acquisition already belongs to asynchronous provider
    construction. The successful capture did not require making presentation
    or the renderer-facing execution contract asynchronous.

## Disposition

Incubating. The repeated symptom warrants a focused renderer-orientation audit
and shared conformance fixture, but it does not yet justify an ADR or a broad
coordinate-system redesign. The corrected Doom adapter remains valid evidence
for source-side conversion; it is not a substitute for engine-facing proof.

## Consequences

- New corpus geometry should not silently choose a front-face interpretation.
- The renderer may need an explicit front-face policy or a fixed documented
  default, together with a validation rule for orientation-reversing transforms.
- Adapters retain responsibility for source convention conversion; they should
  be able to prove their output against the shared fixture rather than infer
  behavior from screenshots.
- Existing two-sided or culling-disabled rendering remains a permissible
  diagnostic posture only when explicitly selected; it must not be used to
  close this finding. `hello-glb` now uses back-face culling for its ordinary
  opaque Box path; its separate translucent diagnostic posture remains
  intentionally two-sided and cannot prove which face the native backend
  classifies as front.

## Provisional Orientation Contract

This review uses the following compact contract as a test oracle while the
binding renderer decision remains open:

1. Ordered triangle positions `(a, b, c)` define the geometric normal
   `(b - a) x (c - a)` in a right-handed coordinate system.
2. Authored vertex normals may intentionally differ for shading, but an importer
   must preserve that distinction and must not silently use them to redefine
   geometric front/back facing.
3. A source adapter either preserves its source winding or performs one explicit,
   tested conversion into renderer-facing coordinates.
4. Orientation-preserving transforms retain winding. An orientation-reversing
   transform must be accompanied by an explicit winding or face-policy change.
5. Projection and viewport transforms may reverse 2D winding; a culling adapter
   must account for that reversal exactly once.
6. Cull mode and front-face selection are separate policy inputs. Disabling
   culling does not establish or test a front-face convention.

## Conformance Fixture Matrix

The next fixture should be deliberately small and visually unambiguous rather
than derived from Doom or an importer:

| Variable | Required evidence |
| --- | --- |
| Geometry | One CCW and one CW triangle with known, opposite geometric normals |
| Presentation | Distinct green front and magenta back results |
| Camera | Fixed view and projection with retained parameters |
| Lighting | Fixed normal-dependent result, distinct from face classification |
| Depth | Fixed depth state and non-overlapping placement |
| Cull policy | No culling, back-face culling, and front-face culling |
| Ordinary transform | Identity plus rotation and translation |
| Reversing transform | One reflection or negative-determinant scale with an explicit adjustment |
| Backends | The same semantic fixture captured through native WGPU and browser/WASM |

The fixture passes only when each backend agrees about which ordered triangle
is front under every cull mode, preserves the result under orientation-
preserving transforms, and performs exactly one declared adjustment for the
orientation-reversing transform. Front/back color, normal lighting, and culling
must remain separately observable so one cannot accidentally stand in as proof
for another.

## Required Follow-Up

- [x] Write a compact orientation contract covering ordered triangle positions,
      geometric normal, supplied normal, front face, culling, and transforms.
- [x] Add a renderer-level conformance fixture with paired CW/CCW triangles,
      distinct front/back colors, known normal lighting, depth, all three cull
      modes, ordinary transforms, and one negative-determinant transform.
- [x] Capture that fixture on native WGPU and WASM before selecting a stable
      backend-independent claim:
  - [x] Retain the native WGPU matrix with adapter identity and fixture layout.
  - [x] Retain the browser/WASM matrix from the same shared fixture.
    - [x] Build and package the shared Rust fixture for `wasm32`.
    - [x] Provide a local WebGPU host with explicit ready, failed, and
          unsupported states.
    - [x] Run and capture it in a browser exposing `navigator.gpu`; Edge and
          the in-app browser reach the shared fixture's `ready` state.
    - [x] Retain the bounded, stage-level WGPU provider initialization result
          before inferring a renderer-facing async contract change.
- [ ] Complete the Box.glb workbench audit:
  - [x] Prove decoded index order agrees with authored normals.
  - [x] Prove preview triangle transport retains outward winding.
  - [x] Extract and test camera projection and browser culling semantics.
  - [x] Correct and test the inverted model-following orbit controls.
  - [x] Retain a native `hello-glb` Box comparison with explicit opaque
        `CullMode::Back`; the translucent diagnostic pipeline remains
        intentionally two-sided.
  - [ ] Capture the deployed browser preview in a WebGPU-capable browser and
        compare it to the retained native result. The locally rebuilt consumer
        is running at `http://127.0.0.1:5188/`; select
        `third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb`
        and retain the resulting Canvas preview with its GLB observation.
- [x] Audit all current `Mesh::uniform_normal` consumers for a documented
      relationship (or intentional independence) between their normals and
      triangle order.
- [x] Feed the renderer result back into the Doom headless lowering only if a
      defined renderer contract contradicts the corrected source-side rule.
      No change is required: both backend captures agree with that rule.
- [ ] Create an ADR only if the audit needs a binding public render contract or
      changes established renderer ownership.

## Reopening Triggers

- the conformance fixture renders opposite front/back results on native and
  WASM;
- a second non-Doom consumer fails a cross-product-to-culling comparison;
- a camera or projection transform has negative orientation determinant without
  an explicit face-policy adjustment;
- the WGPU backend cannot expose the required provider-neutral policy;
- fixing the Box.glb workbench preview requires a change to `tokimu-render`'s
  public contract.

## Review History

### Cycle 1 -- 2026-08-08

- Status entering review: Proposed.
- New evidence: corrected E1M1 right/front side winding; independent decoded
  Khronos Box.glb workbench inside-out-facing observation.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: source adapter and renderer-facing orientation evidence must remain
  separate; the latter is incomplete.
- Disposition: Incubating.
- Resulting ADR or documentation change: created this AR; no binding decision.

### Cycle 2 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: Box.glb decode normals/winding regression; workbench Rust preview
  outward-winding regression; extracted TypeScript projection, culling, and
  orbit regressions; built-in Mesh winding/normal regression and clarified
  `uniform_normal` contract.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: decode and preview transport preserve Box orientation; the isolated
  browser culling sign is coherent; orbit controls were genuinely inverted and
  have been corrected. Native culling and deployed visual captures remain open.
- Disposition: Incubating.
- Resulting ADR or documentation change: recorded the provisional contract and
  narrowed the Box audit without making a backend-independent claim.

### Cycle 3 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: independent review of the retained findings and proposed
  renderer fixture.
- Participants or reviewers: project maintainer, Monday, and Codex
  implementation review.
- Findings: the evidence disproves a single shared reversal in the audited Doom
  and Box paths, while confirming independent Doom winding and workbench orbit
  defects. The remaining cross-cutting issue is the renderer's unstated
  relationship among winding, front-face policy, culling, and transform
  orientation.
- Disposition: Incubating.
- Resulting ADR or documentation change: expanded the fixture into an explicit
  native/WASM test matrix, including front/back colors, all cull modes,
  orientation-preserving transforms, and one reflection. No ADR opened.

### Cycle 4 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: shared `render-orientation-conformance` corpus library with
  paired geometry, a `front_facing` diagnostic shader, three cull pipelines,
  and four transform/compensation cases.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: the semantic fixture covers 12 cases and proves the reflection
  expectations from winding and transform determinant without treating the
  deliberately shared shading normal as facing evidence.
- Disposition: Incubating.
- Resulting ADR or documentation change: completed the renderer-level fixture
  definition. Native WGPU and browser/WASM pixel capture remain required.

### Cycle 5 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: native `hello-render-orientation` WGPU consumer and retained
  1216-by-839 fixture capture on AMD Radeon RX 7900 XTX.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: WGPU compiled the shared `front_facing` shader and rendered the
  expected no/back/front culling result across identity, ordinary transform,
  reflection, and once-compensated reflection rows.
- Disposition: Incubating.
- Resulting ADR or documentation change: native capture complete; browser/WASM
  capture remains open, so no backend-independent convention is selected.

### Cycle 6 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: `hello-render-orientation-web`, successful wasm32 compilation,
  generated web bindings, verified local JS/WASM delivery, and an explicit
  bounded unsupported result from the available browser environment.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: both consumers now share geometry, shader, cull states, transforms,
  and layout in Rust. The available in-app browser cannot complete Tokimu
  WebGPU initialization, so no browser pixel or parity claim can be retained
  from it.
- Disposition: Incubating.
- Resulting ADR or documentation change: browser/WASM consumer and deterministic
  host complete; capture remains open for a WebGPU-capable browser.

### Cycle 7 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: `hello-glb` now selects an explicit opaque, depth-writing
  `CullMode::Back` pipeline for the decoded Khronos Box mesh; its deliberately
  translucent diagnostic pipeline retains `CullMode::None`. A native WGPU
  capture retains the full Box silhouette from the orbit viewpoint with
  `opaque-cull=back` in the window title.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: the native GLB consumer no longer relies on culling-disabled output
  for its ordinary Box path. This confirms that the inspected Box source can be
  rendered intact on the native adapter under explicit back-face culling; it
  does not establish a browser front-face convention or deployed workbench
  parity.
- Disposition: Incubating.
- Resulting ADR or documentation change: retained native capture and manifest;
  split the remaining Box workbench item into completed native evidence and
  pending deployed-browser comparison.

### Cycle 8 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: Edge successfully completed standard `navigator.gpu`
  adapter and device preflight for the shared orientation fixture, while the
  Tokimu/WGPU provider still did not complete its bounded initialization. The
  backend now uses WGPU 23's asynchronous browser-WebGPU instance detection,
  but the result remains unresolved.
- Participants or reviewers: project maintainer, Monday, and Codex
  implementation review.
- Findings: this is a blocked browser-renderer prerequisite, not orientation
  evidence and not a claim that browser WebGPU is unavailable. The existing
  website islands prove ordinary Rust/WASM bootstrap works, but they do not
  exercise this WGPU rendering path. Stage-level provider diagnostics are now
  required to distinguish instance, surface, adapter, device, and surface
  configuration before deciding whether a separate lifecycle review is needed.
- Disposition: Incubating.
- Resulting ADR or documentation change: none. The orientation capture remains
  open; a broader renderer/provider lifecycle AR is contingent on the staged
  evidence.

### Cycle 9 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: staged browser diagnostics proved that WGPU instance, surface,
  adapter, device, and surface configuration completed. The first presentation
  then aborted in `std::time::Instant::now()` immediately before surface-frame
  acquisition. After replacing that native-only assumption with a provider-
  appropriate optional CPU timer, Edge and the in-app browser reached `ready`.
  The retained browser/WASM matrix agrees with the native WGPU capture in all
  12 cases.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: the blocker was portable diagnostic timing, not browser WebGPU
  initialization, surface acquisition, or geometry orientation. Existing async
  provider construction is sufficient; renderer presentation did not need an
  async contract change. Temporary renderer-level DOM stage probes were removed
  after isolation.
- Disposition: Incubating.
- Resulting ADR or documentation change: completed the native/WASM shared-
  fixture capture requirement and retained the timing correction. The deployed
  Box workbench comparison and any binding public orientation contract remain
  open.

### Cycle 10 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the ASP.NET workbench rebuilt successfully, its actual browser
  shell reached `Tokimu WASM consumer bridge ready`, and the local host is
  available at `http://127.0.0.1:5188/`. Its TypeScript facing/orbit suite
  passes all three checks. The Rust/WASM consumer passes 24 tests, including
  the pinned `Box.glb` preview's 12 transported outward-wound triangles.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: the local browser shell, TypeScript projection/culling logic, and
  Rust/WASM import/preview transport all have current passing evidence. The
  available browser automation can start the real app but cannot provide a
  local file-input payload, and no controllable external Edge session is
  attached. This is not a substitute for the still-required visual Canvas
  capture after selecting the pinned Box input.
- Disposition: Incubating.
- Resulting ADR or documentation change: made the remaining manual capture
  procedure and local URL explicit; no orientation or ownership decision
  changed.

### Cycle 11 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: interactive workbench review found both horizontal and vertical
  drag directions inverted for the expected camera-orbit use. The old test
  proved only that a synthetic identity-view feature followed the pointer; it
  did not prove the declared orbit semantics.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: this is a browser preview interaction defect, independent of mesh
  winding, culling, and normal direction. The orbit helper now subtracts both
  pointer deltas, its regression asserts the exact yaw/pitch deltas and bounds,
  and the workbench design now names a camera orbit around a fixed model.
- Disposition: Incubating.
- Resulting ADR or documentation change: corrected and re-tested the local
  browser interaction convention; the visual Box capture remains the final
  evidence item.

## References

- `docs/Plans/DOOM/Evidence/Classic Doom wall side and winding evidence.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `corpus/focused/data-interchange/hello-glb/DESIGN.md`
- `corpus/focused/data-interchange/hello-glb/results/native-wgpu-back-cull.png`
- `corpus/focused/data-interchange/hello-glb/results/native-wgpu-back-cull.md`
- `corpus/consumers/aspnet-wasm-asset-workbench/DESIGN.md`
- `corpus/consumers/aspnet-wasm-asset-workbench/Client/mesh-preview.ts`
- `corpus/consumers/aspnet-wasm-asset-workbench/tests/mesh-preview.test.mjs`
- `corpus/lib/render-orientation-conformance/DESIGN.md`
- `corpus/lib/render-orientation-conformance/src/lib.rs`
- `corpus/lib/render-orientation-conformance/results/native-wgpu.png`
- `corpus/lib/render-orientation-conformance/results/browser-wasm.png`
- `corpus/lib/render-orientation-conformance/results/browser-wasm.md`
- `corpus/campaigns/coordinate-conformance/hello-render-orientation/DESIGN.md`
- `corpus/campaigns/coordinate-conformance/hello-render-orientation-web/DESIGN.md`
- `crates/tokimu-render/src/mesh.rs`
- `crates/tokimu-render/src/pipeline.rs`
- `crates/tokimu-render/src/wgpu_backend/pipeline_support.rs`
