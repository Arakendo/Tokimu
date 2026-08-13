# Renderer Resource Identity Alternatives Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-11 |
| Plan | [Renderer Resource Identity And Failure Presentation](../renderer-resource-identity-and-failure-presentation.md) |
| Reviews | AR-0024, AR-0027 |
| Fixture | `corpus/campaigns/renderer-reliability/hello-render-resource-identity` |
| Scope | Slice 2 corpus-local comparison; no public API or ownership decision |
| Status | Complete corpus comparison; no shared identity capability admitted |

## Alternatives Exercised

| Alternative | Create / intentional replace | Collision / missing / stale behavior | Retire | Dynamic addition preserves a live resource | Boundary retained |
| --- | --- | --- | --- | --- |
| A: E1M1 disjoint ranges | Caller-local numeric range | Cannot detect an independently chosen conflicting range | No shared operation | Yes, when every local range is coordinated | Application/corpus only |
| B: application registry | Allocates monotonically; replacement requires same logical owner | Wrong logical owner and missing handle reject | Yes; later lookup is missing | Yes | Application/tooling candidate |
| D: generational registry | Allocates slot plus generation; replacement requires same logical owner | Wrong owner rejects; retired slot reference is stale after reuse | Yes; slot may reuse with a new generation | Yes | Corpus experiment; no current renderer-handle adapter |
| E: explicit lifecycle ledger | Caller selects handle; create, replace, and retire are distinct | Already-live, wrong owner, and missing handle reject | Yes | Yes | Renderer-validation candidate, not admitted |
| F: validation only | Preserves existing upload mechanics | Unrelated replacement remains an upload but emits a bounded observation | No lifecycle claim | Not prevented; observation retained | Diagnostic-only candidate |

Alternative C (renderer-owned allocation) is deliberately not prototyped. The
results so far do not establish that application-side stable identity plus
validation cannot satisfy the observed E1M1 pressure.

## Deterministic Validation

```powershell
cargo test -p hello-render-resource-identity
cargo check -p hello-render-resource-identity --target wasm32-unknown-unknown
```

Result:

```text
11 passed, 0 failed
wasm32-unknown-unknown check passed
```

The tests establish that:

- a deliberate same-logical-resource replacement remains valid;
- an unrelated dynamic resource does not change a live cutout identity in B,
  D, or E;
- B and E classify post-retirement lookup as missing;
- D distinguishes a stale generation after slot reuse;
- E distinguishes create, replace, retire, collision, and unresolved lookup;
  and
- F retains at most four mismatch observations while preserving its total
  mismatch count.

The WASM result is compile feasibility only. This fixture does not execute a
browser/WGPU path and therefore does not claim browser timing or provider
behavior.

## Independent Native Renderer Caller: Hello GLB

`corpus/focused/data-interchange/hello-glb` is independent of Doom source/topology and deliberately
replaces its stable `MODEL_MESH = 1` and `FLOOR_MESH = 2` identities each
frame as the presentation mesh is rebuilt. A corpus-only
`--measure-two-frames` switch now exits after two native frames. It moves
`begin_frame()` before its replacement uploads so the existing renderer's
defined per-frame counter boundary includes the work being observed.

Command:

```powershell
cargo run -p hello-glb -- --measure-two-frames
```

Retained AMD Radeon RX 7900 XTX / Vulkan observation:

```text
frame 1: draws=2, mesh_uploads=2, mesh_replacements=0
frame 2: draws=2, mesh_uploads=2, mesh_replacements=2
```

The first frame creates the model and floor mesh resources. The second frame
replaces exactly those two existing identities. This is independent evidence
that intentional same-handle replacement is useful renderer behavior; it does
not test dynamic allocation, retirement, stale references, or browser/WASM
execution. The initial result before reordering `begin_frame()` retained zero
per-frame uploads despite successful presentation, which was a corpus-local
measurement-order defect rather than a renderer replacement failure.

## Representation And Churn Observation

Command:

```powershell
cargo run -p hello-render-resource-identity
```

Retained native debug-profile observation from this workspace on 2026-08-11:

```text
MeshHandle: 8 bytes
GenerationalMeshHandle: 8 bytes
LogicalMesh label: 8 bytes
LifecycleCounts: 32 bytes

10,000 create / resolve / replace / retire / rejected-lookup cycles
application registry: 3.1825 ms
generational registry: 848.9 µs
explicit lifecycle ledger: 3.3161 ms
each: 10,000 creates, replacements, retires, and rejections
```

The loop deliberately includes a failing lookup after every retirement. These
durations are debug-profile, one-host corpus observations over simple
`BTreeMap`/slot implementations. They are useful for detecting gross
regressions and showing that the candidate models have different costs; they
are not an ADR-0008 performance gate, a portable benchmark, or evidence that
the fastest fixture implementation is the correct Tokimu owner.

The same command under `--release` retained this bounded result on the same
host:

```text
10,000 complete lifecycle cycles
application registry: 201.7 µs
generational registry: 92.9 µs
explicit lifecycle ledger: 200.4 µs
```

The release observation confirms the fixture has no obvious per-operation cost
failure. It deliberately does not establish a cross-machine budget, measure
GPU resource creation, or convert local time into a Native Ring admission
claim.

## Bounded Observation Detail

Alternative F uses a fixed four-slot `Option<IdentityError>` ring. Every
unrelated replacement increments the total diagnostic count; only the most
recent four payloads are retained. It allocates no growing failure history in
the fixture. This establishes a narrow candidate for Slice 3, not a chosen
diagnostic envelope or a renderer feature.

## Provisional Reading

The evidence rules out a simplistic answer: prohibiting repeated handles would
break deliberate mesh replacement already used by the renderer. It leaves at
least three credible corpus-local shapes:

1. application-owned allocation (B);
2. caller-owned identity plus explicit renderer-side lifecycle validation (E);
3. current mechanics plus bounded replacement observations (F).

Generational identity (D) gives the strongest stale-reference distinction with
no larger handle representation, but it requires an explicit translation or
new renderer identity model. That additional boundary has not yet earned
implementation work.

No alternative is admitted. Slice 2 now has a real native renderer workload
and a second dynamic caller, but it still lacks comparative cross-target
runtime measurements, a resource-rich workload, and an ADR-0008-grade
performance budget before any shared ownership claim is considered.

## Browser/WASM Fixture Prepared

`corpus/campaigns/renderer-reliability/hello-render-resource-identity-web` reuses the same Rust-owned B, D,
and E alternatives in a focused browser composition. It also uploads
`Mesh::triangle()` and then `Mesh::diamond()` to the same WGPU mesh handle,
submits the replacement, and returns the provider's lifetime upload and
replacement counts to a caller-owned DOM status surface after `present()`.

The fixture packages successfully for `wasm32-unknown-unknown`; the 18 native
control tests remain green. This is implementation evidence only until an
actual browser run supplies the returned record. It does not admit an identity
registry, make the DOM a shared terminal-record owner, or alter renderer
replacement behavior.

The actual browser run subsequently retained all B/D/E rejection cases and
presented the replacement diamond. The WGPU provider reported two lifetime
mesh uploads, one lifetime replacement, and one draw. The bounded unresolved
record retained `ResourceUnresolved`, `MeshHandle(44)`, and caller
`identity-fixture`. This closes the browser-execution comparison while leaving
the blank adapter name and post-page-disposal lifetime explicit. It confirms
mechanical parity; it does not select B, D, or E or admit shared vocabulary.

## Slice 2 Disposition

Slice 2 is complete as a bounded corpus comparison:

- B, D, and E survive the exercised lifecycle and release-profile pressure;
- F survives only as a bounded observation mechanism because it deliberately
  does not prevent aliasing or claim retirement semantics;
- C has no demonstrated need and remains unimplemented; and
- no result establishes Native Ring ownership, a stable renderer API, or a
  browser runtime claim.

The next question is therefore not “which handle model wins?” It is whether
the failure observations around a rejected resource operation have a shared
renderer/platform boundary at all. That begins Slice 3.
