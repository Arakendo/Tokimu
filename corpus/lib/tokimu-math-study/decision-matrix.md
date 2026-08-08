# Provisional Decision Matrix

| Field | Value |
| --- | --- |
| Status | Interim decision aid; no stable API selection |
| Date | 2026-08-08 |
| Scope | Frozen 0.1 operation inventory and current corpus-local fixtures |
| Deferred pressure | Re-run after `docs/Plans/DOOM/DOOM WAD Checklist.md` completes |

The matrix separates ownership claims from implementation and measurement facts.
“Favorable” means only that the candidate has cleared the named evidence in
this bounded study; it does not mean it is generally better.

| Category | A — Direct `glam` | B — Provider-backed vocabulary | C — Narrow owned implementation | D — Bounded derivation |
| --- | --- | --- | --- | --- |
| Semantic/public-vocabulary independence | No: public names remain foreign | Yes: candidate public names; provider stays private | Yes: candidate names and mechanics are owned | Yes in the small slice only |
| Implementation independence | No | No: 41 direct private provider references | Yes: no provider reference, macro, generated source, or unsafe block; shared source also passes a dependency-free isolated crate | Yes for scalar `Vec3` only |
| Source provenance | Strong current audit/pin | Strong current audit/pin plus wrapper source | Tokimu-authored source; must establish its own correctness evidence | Explicit upstream-derived provenance and update duty |
| Bounded caller correctness | Control | Matches A for current transforms, GLB geometry, CAD ray, animated node path, stereo/orthographic cameras, and 128 finite camera cases | Matches A for the same bounded paths | Cannot cover matrix callers |
| Native/WASM build portability | Builds; isolated WASM stereo probe runs | Builds; isolated WASM stereo probe runs | Builds; isolated WASM stereo probe runs | Builds implemented slice |
| Runtime performance | Native camera 7.323 ms; isolated WASM math 9.839 ms / 100k | Native 6.663 ms; isolated WASM 9.356 ms; target repeats required | Native 9.246 ms; isolated WASM 9.352 ms; target/scope result differs | Not measured for comparable workload |
| Renderer/provider boundary | No new conversion | Private unwrap; current `tokimu::Camera` handoff adds view/projection unwraps | Column-array reconstruction; current `tokimu::Camera` handoff adds two reconstructions | No matrix boundary |
| Native representation observation | Provider layout: `Vec4`/`Mat4` align 16 | Matches A for retained types | Same `Vec4`/`Mat4` size but align 4; no direct compatibility claim | `Vec3` only |
| WASM representation observation | `Vec4`/`Mat4` align 16 | Matches A: align 16 | `Vec4`/`Mat4` align 4; same boundary remains | No `Mat4` |
| Migration friction observed | Existing foreign vocabulary | Accessor/setter and provider-boundary seams | Representation conversion while renderer remains provider-bound | Incomplete by design; provenance/update work begins immediately |
| Current source surface | 68 lines / 2,325 bytes | 450 lines / 13,719 bytes | 494 lines / 15,807 bytes | 139 lines / 4,064 bytes, but only `Vec3` |
| Maintenance risk not yet resolved | Provider update/toolchain surface | Provider update plus wrapper drift; isolated build compiles provider and warnings | Numeric, target, optimization, and API growth | C risks plus copied-source change tracking |
| Current study disposition | Retain as production control | Conditionally viable transitional/experimental seam | Leading independence candidate; continue incubation | Paused unless C shows a named deficit |

`Vec2` and `Quat` remain a distinct compatibility question: they are currently
public in A but have no direct in-repository caller. B's minimal probes make
that coupling visible; C intentionally omits both until concrete pressure or
downstream compatibility evidence earns them.

## Current Direction

1. Retain A in stable Tokimu code; it is the audited, compatible production
   control.
2. Keep B as a valid comparison and possible transition mechanism, not the
   assumed destination. Its concrete seams are evidence, not defects to hide.
3. Continue C as the leading test of the self-implemented Ring 0 hypothesis.
   Its present scope is small and it has cleared meaningful caller pressure,
   but it has not earned stable admission.
4. Keep D paused. A copied-source expansion needs a measured advantage over C
   or a specific compatibility reason.

## Evidence That Still Blocks Selection

- Post-DOOM operation inventory and re-run of affected caller paths.
- Repeated, host-identified native measurements and actual WASM execution.
- Binary size and clean-build-time comparisons.
- Numerical edge cases, differential/property coverage, and a deliberate
  singular/degenerate contract decision.
- Real renderer, scene, serialization/FFI, and public-API migration evidence.
- A decision on whether `Vec2` and `Quat` have real caller pressure.

No score is assigned. A weighted score would imply that the project has chosen
relative weights for semantic independence, performance, correctness, and
maintenance before the missing evidence exists.
