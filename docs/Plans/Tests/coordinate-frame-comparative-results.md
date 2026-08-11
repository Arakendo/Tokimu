# Coordinate-Frame Comparative Results — 2026-08-10

## Disposition

The retained evidence supports **AR-0028 Alternative A with no architectural
change**. Directional conversions remain explicit at the boundary that owns
their meaning. The study does not admit a universal Tokimu world handedness,
forward/right basis, yaw sign, pointer sign, or texture-origin convention.

## Evidence Comparison

| Evidence family | Observed agreement | Owner of directional meaning | What it does not prove |
| --- | --- | --- | --- |
| Doom E1M1 and synthetic sidedefs | Readable right/front and left/back art, stable winding/normals, reversible point/direction lift, deterministic spawn commands | Doom provider for source lift, side axes, headings; corpus application for observer commands | Doom's embedding is Tokimu's global world basis |
| Box/PNG | Asymmetric art remains readable under caller-supplied UVs and explicit sampler intent | Asset/corpus caller owns UV generation; renderer realizes supplied stream | One imported asset orientation defines all source formats |
| Orientation fixture | Winding, culling, transforms, reflection, and compensation agree natively and in browser/WASM | Caller owns geometry/transform intent; renderer realizes declared cull state | Supplied normals define facing, or reflection should be inferred globally |
| Camera/input fixture | Native capture and browser pointer lock reach the same declared pose/basis commands | Platform owns acquisition, normalized input carries observation, application owns gesture-to-command policy | Positive mouse delta universally means positive or negative yaw |
| Projection/picking and CAD | Shared matrices project/unproject labeled landmarks; CAD derives its own oblique-view right | Camera/CPU caller owns GL-style matrices; application owns picking policy | Initial first-person world `-X` is every camera's screen-right |
| WGPU depth regression | Tokimu GL clip depth maps exactly once to WGPU `[0,1]` | WGPU backend owns provider adaptation | Backends may normalize world, UV, or input conventions |
| Native/browser parity | Directional fixtures agree on the exercised paths | Each provider realizes the same explicit inputs | Agreement establishes universal correctness or untested GPU parity |

## Rule Classification

| Directional rule | Classification | Retained owner/location |
| --- | --- | --- |
| Doom `(x,y,height)` to corpus `(x,height,z)` | Source/provider conversion | `doom-geometry-provider` |
| Doom right/front and left/back U direction | Source/provider conversion | `doom-geometry-provider` |
| Doom player heading and source-spawn placement | Source/provider conversion | Doom corpus/provider boundary |
| Pointer delta to first-person yaw/pitch | Application control policy | Camera conformance and E1M1 consumers |
| W/A/S/D interpretation and orbit gestures | Application control policy | Individual application; intentionally not inferred from keys alone |
| Caller-supplied UVs, winding, transforms, and camera matrices | Existing Tokimu renderer input meaning | Provider-neutral renderer contract; no new admission |
| GL-style camera clip depth | Existing Tokimu camera meaning | Camera/CPU contract retained by AR-0024 evidence |
| GL clip depth to WGPU clip depth | Renderer/backend mechanism | Private WGPU upload boundary |
| Pixel rectangular clipping | Renderer/backend mechanism | `ViewportRect` currently lowers to WGPU scissor |
| Global world handedness / canonical forward / canonical right | Intentionally unspecified | Must be earned by independent semantic callers |
| Universal yaw sign or pointer-look sign | Intentionally unspecified | Gesture and camera policies remain separable |
| Universal imported texture origin or U/V flip | Intentionally unspecified | Source adapter/caller supplies conversion |
| Orientation-preserving/reversing chart transition | Candidate future Tokimu semantic convention | AR-0026 incubation; not inferable from raw matrix determinant |

## Alternatives

- **A — keep explicit corpus/source conversions:** supported. It explains all
  retained evidence without hidden compensation or new public vocabulary.
- **B — admit one named Tokimu spatial/camera basis:** not earned. The CAD
  oblique camera, application-specific gestures, and future chart-local views
  show that useful directional facts are view- or policy-relative.
- **C — provider-owned conventions without a shared contract:** rejected in
  its implicit form. Provider conversions are permitted only when named,
  tested, and lowered into existing explicit Tokimu inputs.
- **D — renderer/platform normalization:** rejected. A global UV, yaw, or
  movement flip would contradict Box/PNG, orientation, and input ownership.

## AR-0019 And AR-0026 Pressure

AR-0028 does not require raw `Vec3` or `Mat4` to carry source ownership. It does
show future pressure for semantic roles *above* raw math—source frame, view
frame, chart-local point, and declared transition—if multiple independent
callers need durable composition. Creating wrappers now would merely encode
the current fixtures prematurely, so AR-0019 remains incubating.

For AR-0026, the result is a constraint: chart transitions must declare
orientation preservation/reversal and derived views explicitly. A single
global Euclidean basis, matrix determinant, or renderer inference is
insufficient authority. No chart vocabulary or implementation is admitted by
this study.

## Failure, Performance, And Recovery

No new Native or public contract is proposed, so ADR-0008/0009 admission gates
are not activated. Existing tests reject zero/non-invertible matrices,
degenerate geometry, invalid provider data, and ambiguous camera groupings
rather than inventing directions. The conformance paths add no steady-state
engine work; they are corpus tests and retained evidence. Any future stable
frame/transition API must reopen the full performance, diagnostics, failure
containment, migration, and recovery gates.
