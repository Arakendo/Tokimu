# Option C Slice 11: Chart Cross-Review Control

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Complete bounded Slice 11 chart-control evidence |
| Scope | Corpus-local AR-0026 three-chart transition trace under A and C0 |
| Non-claim | No chart API, portal, renderer recursion, or math migration |

## Fixed Semantic Layer

The control owns three local identities (`Entry`, `Junction`, and `Exit`), two
declared rigid transitions, and a separate reflected-transform control. Its
authored semantics remain identical for both numerical alternatives:

```text
Entry -> Junction -> Exit
point transport + direction transport + composed inverse

rigid composition     -> orientation preserving
negative X reflection -> orientation reversing
```

`ChartId`, traversal path, and orientation declarations are corpus-local
semantic wrappers. They are not counted as low-level math-library growth.

## Native Result

`cargo run -q -p tokimu-math-study --bin observe_chart_junction --locked --offline`
reported the same bounded trace for both alternatives:

```text
A:  endpoint=[0.9999995, 0.0, 20.0]
C0: endpoint=[0.9999995, 0.0, 20.0]
restored point=[2.0, 0.0, -1.0]
transported direction=[0.0, 0.0, 1.0]
composed orientation=Preserving
reflection orientation=Reversing
fingerprint=2520c9de
```

The 63-test math-study suite includes exact A/C trace comparison, explicit
inverse round-trip, and the independent orientation result. This reproduces
AR-0028's lesson: invertibility and orientation behavior are distinct facts.

## Target Evidence

The DOM-hosted `hello-bulk-compute-web` fixture exposes **Run AR-0026 chart A/C
control**. It performs no WGPU acquisition and returns the same compact
fingerprint only if both alternatives agree. The fixture compiles and generated
bindings were refreshed for `wasm32-unknown-unknown`.

The maintainer ran the DOM control on 2026-08-12 and retained:

```text
status=completed; workload=ar-0026-chart-control; alternatives=A,C0;
fingerprint=2520c9de; host=DOM; provider=none
```

This is actual browser/WASM execution of the bounded chart mechanics. It is
not WebGPU, renderer, portal, or general chart-system evidence.

## Ordinary Math Surface Requested

The control uses only already-inventoried operations:

- `Vec3`: construction, add/subtract, dot, cross, normalize, array observation;
- `Mat4`: translation, Y rotation, scale (reflection control), composition,
  inverse, point transport, and direction transport.

No quaternion, new vector type, determinant API, special numerical solver,
unsafe path, SIMD path, or provider dependency was requested. The
orientation-preserving/reversing classification is derived in the semantic
fixture rather than inferred or stored by `Mat4`.

## Cross-Review Clamp

- **AR-0025:** ordered AABB/point filtering remains a caller-owned generic
  numerical control. Doom BSP/SEG/screen-clip semantics remain source-specific
  and cannot be promoted by this result.
- **AR-0026 / AR-0028:** framed/chart meaning belongs above raw mechanics.
  The trace supports a bounded owned numerical core only provisionally; it does
  not admit chart or frame vocabulary.
- **CAD:** its independently useful point-cloud control confirms that stable
  candidate IDs, query domains, rejection reasons, and final interpretation
  stay with the caller whether the numerical filter is CPU or WGPU.
- **Provider boundary:** WGPU remains an Outer Ring experimental mechanism.
  Nothing in this trace creates a Ring 0 provider boundary or selects GPU work.

## Next Evidence

Expand the AR-0026 corpus only if the chart review authorizes its planned
synthetic junction work; do not enlarge C0 merely to make the chart example
more exotic.

## Option B Cross-Review Update -- 2026-08-12

The later Option B Slice 11 replay added provider-backed Full B and reviewed
Narrow B, stereo/multiple-camera pressure, and the renderer clip boundary. It
requested **zero new C0 operations**. A, Full B, and C0 again produced
`2520c9de`; Narrow B remains A's exact ordinary value mechanics plus only the
three checked camera constructors. Portal-derived and recursive views remain
future AR-0026 semantic pressure and do not enlarge this operation ledger.
