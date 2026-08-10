# Alpha-Policy Real-Caller Comparison

Date: 2026-08-09  
Scope: Slice 5 of `textured-surface-alpha-policy-comparative-corpus.md` and
AR-0023.  
Status: real-caller structural comparison; no renderer contract is admitted.

## Callers

| Caller | Real need | Declared experiment | Shared-fixture analogue | Current evidence |
| --- | --- | --- | --- | --- |
| `hello-doom-e1m1` | E1M1 source-classified two-sided masked middles | Binary coverage: discard RGBA8 alpha at or below `0`; depth write | Cutout over opaque | Canonical shareware package: 13 observations / 26 non-degenerate candidates; three source textures have uncovered pixels and one is fully covered. Native AMD/Vulkan initializes with 1,861 draws; browser/WebGPU visibly presents the same package at 1,835 opaque or 1,861 cutout-enabled draws. |
| `hello-glb` | Interactive inspection opacity on the retained Box mesh | Opacity `0.35`; alpha blend; depth test; no depth writes; submission-order limitation | Blend over opaque and ordering/depth-off panels | Six focused tests pass. The fixed native AMD/Vulkan capture visibly presents the translucent Box over its opaque floor and reaches two-draw warm frames with no new bindings, uniform writes, or mesh uploads. A browser GLB variant is not part of this corpus entry. |

The callers are intentionally not symmetric. Doom asks for categorical
visibility based on retained source classification; Box inspection asks for
continuous contribution from application-owned opacity. This is the pressure
the synthetic shared fixture was designed to separate.

## Boundary check

| Question | E1M1 cutout | GLB blend |
| --- | --- | --- |
| Who selects intent? | Doom consumer, from retained two-sided-middle source observation | GLB application presentation override |
| What reaches the renderer? | No candidate reaches it yet; eventual input must be only generic declared cutout intent | Existing generic pipeline state plus material override |
| Format-specific renderer input? | No WAD name, marker, linedef, or palette type crosses | No GLB/glTF type crosses |
| Is policy inferred from texture alpha? | No; fully covered `BROWNGRN` remains source-classified masked, proving classification is not a coverage heuristic | No; the application explicitly changes resolved opacity to `0.35` |

## Present API pressure

1. Cutout needs only a declared categorical visibility threshold and depth
   behavior. E1M1's exact `0` threshold is a binary-raster corpus choice, not
   evidence for a universal default.
2. Blend needs a declared depth-write policy and visible caller ordering
   responsibility. `hello-glb` deliberately states its limit: intersecting
   transparent geometry has no guarantee beyond submission order.
3. Neither caller needs a Doom-aware renderer material, a GLB-aware material,
   a general shader-authoring API, renderer-owned sorting, PBR, or automatic
   alpha classification.

## Outstanding comparison evidence

- Retain a fixed-camera native E1M1 cutout observation using the same
  corpus-local shader/pipeline machinery. Browser/WebGPU already visibly
  presented the selected package at 1,861 draws, versus 1,835 for its opaque
  companion action. Neither presentation admits a renderer interface.
- Native E1M1 visual capture is useful corroboration but is not required to
  repeat the browser/WASM real-caller observation already retained.
- The GLB entry intentionally has no browser variant. The shared alpha fixture
  supplies browser/WASM Blend evidence; the real GLB caller supplies native
  continuous-alpha pressure. This asymmetric evidence must not be reported as
  cross-target GLB conformance.

## Slice 5 Comparison Result

The two real callers agree with their matching shared fixture roles without
forcing common public vocabulary:

| Question | E1M1 cutout | GLB blend |
| --- | --- | --- |
| Fragment meaning | categorical discard at a consumer-declared binary threshold | continuous straight-alpha contribution from application-owned opacity |
| Depth behavior | ordinary depth writes after retained fragments | explicit no-depth-write policy |
| Ordering pressure | no general transparent ordering problem claimed | submission order remains the only stated guarantee for intersections |
| Target evidence | native startup plus browser/WebGPU visual observation | native AMD/Vulkan fixed visual and warm-frame observation |

The comparison supports separate capability review rather than a prematurely
shared `AlphaPolicy` shape. It found no forced new renderer vocabulary, shader
resource API, renderer-owned queue, PBR contract, or source-format term.

## Retained Native Structural Observations

### E1M1 categorical cutout

The native E1M1 opt-in path initializes on AMD Radeon RX 7900 XTX/Vulkan with
1,835 opaque draws and 26 source-selected cutout draws. Browser/WebGPU
visually presents the companion states with the same `1835 -> 1861` draw delta.
The candidate pipeline has explicit opaque blend state, `LessEqual` depth test,
and depth writes; this is a local binary-cutout experiment, not generic alpha
policy.

### GLB continuous alpha

`cargo run -p hello-glb -- --transparent` selects the existing application
opacity override (`0.35`) and freezes the normal orbit/mesh transforms at the
initial frame. On AMD Radeon RX 7900 XTX/Vulkan it reports:

```text
first: 2 draws, 1 submit, 4 binding allocations, 0 uniform writes, 0 mesh uploads
warm:  2 draws, 1 submit, 0 binding allocations, 0 uniform writes, 0 mesh uploads
```

The pipeline is explicitly `AlphaBlend`, `LessEqual`, depth-write disabled,
and two-sided. The source remains the retained Khronos Box GLB; its source
material is not modified. No diagnostic was emitted during the observed valid
run. A reviewer visibly observed the frozen translucent Box over the opaque
floor with the window title reporting `presentation=transparent`. This
establishes an independent continuous-alpha caller and confirms that fixed
capture does not create per-frame presentation churn. It is not browser/WASM
GLB evidence, a general transparent sorting claim, or public Blend admission.
