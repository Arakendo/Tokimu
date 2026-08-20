# Alternative C Semantic Generation Prototype Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Status | Corpus-private semantic prototype; no admission |
| Native subject | `hello-render-resource-identity` |
| WASM subject | `hello-render-resource-identity-web` |
| Renderer/provider resources | Not exercised |

## Why C Was Earned

Alternative B was rejected by two retained provider-path falsifiers:

1. adapter-private reset retires the current scene before successor staging,
   so staging failure cannot preserve the last-known-good scene;
2. bare numeric renderer handles reused by the successor make an old command
   indistinguishable from a current command, so stale identity aliases rather
   than rejects.

The C prototype introduces only the two distinctions needed to test those
failures: a staged candidate separate from current state and a corpus-local
generation attached to resource references.

## Fixed Sequence

The pure Rust fixture executes this sequence:

```text
stage and commit E1M1 generation 0
    -> retain E1M1 mesh key 1 handle
stage E1M2 generation 1
    -> inject failure at material key 1 after three resources
    -> E1M1 remains current
    -> retained E1M1 handle still resolves E1M1 mesh 1
stage complete E1M2 generation 1
    -> validate every draw reference against the candidate
    -> commit in one current-state replacement
    -> retire E1M1 logically
    -> retained E1M1 generation-0 handle rejects as stale
    -> E1M2 generation-1 handle with the same mesh key resolves E1M2 mesh 1
```

An independently staged competing candidate also rejects if another commit
changes its expected predecessor. Missing draw dependencies and generation
counter exhaustion reject without mutating current state.

## Retained Observation

Native and generated WASM execution report the same essential result:

```text
status=complete
lifetime-alternative=C-corpus-private-generation
generation-a=0
failed-generation-b=InjectedStagingFailure(E1M2, Material 1, staged=3)
map-after-failed-stage=E1M1
generation-b=1
retired-map=E1M1
map-after-commit=E1M2
generation-a-after-commit=StaleGeneration(0, current=1)
generation-b-after-commit=E1M2 mesh 1
renderer-resources=not-exercised
provider-session=not-exercised
physical-gpu-reclamation=not-applicable
admission=none
```

## Validation

- `cargo test -p hello-render-resource-identity`: 22 tests passed, including
  failed-stage preservation, successful commit, stale rejection, same-key
  generation reuse, competing-candidate rejection, incomplete-candidate
  rejection, and generation exhaustion;
- strict package clippy with warnings denied;
- native executable observation;
- `cargo check -p hello-render-resource-identity-web --target wasm32-unknown-unknown`;
- release WASM build and regenerated browser binding;
- direct Node instantiation of the generated WASM and execution of
  `run_scene_generation_prototype()`;
- browser JavaScript syntax validation.

## Nonclaims And Next Gate

This prototype does not choose Tokimu's public handle shape, stage WGPU
resources, retain a device/surface through C, reclaim provider allocations, or
prove repeated replacement. Its E1M1/E1M2 resources are a deliberately minimal
logical fixture named for the intended Doom sequence, not decoded WAD
inventories.

The next C slice must correlate these semantics with real Doom-prepared
resource inventories and an independent resource-rich caller before deciding
whether any adapter-private mechanism is required. No Native Ring contract has
been changed.

## Heterogeneous Inventory Correlation Increment

The corpus-private model now accepts an observed resource inventory containing
mesh, texture, material, pipeline, camera, and command counts. It executes the
same stateful sequence over that inventory shape: commit A, inject a material-
stage failure for B, resolve A after failure, commit complete B, reject A's
retained mesh key as stale, and resolve the same local mesh key in B. Required
command resource families are validated before staging.

Two real caller paths supply those counts without changing renderer contracts:

- the independent browser pressure caller supplies the exact 64 meshes, 64
  textures, 64 materials, one pipeline, one camera, and generated command count
  used by each presented scene; and
- the Doom browser workbench projects `WorkingLogicalResources`, computed from
  each map's actual prepared draws, texture uploads, sky boundaries, sky
  planes, pipelines, camera, and command accounting.

Both paths label the result
`alternative-c-authority=semantic-shadow-not-provider-lifetime`. The
correlation happens after successful presentation and cannot make the current
Alternative A or B provider replacement atomic.

The regenerated independent WASM executed headlessly and reported 194
resources for both scenes, failure after 129 staged resources, A retained after
failure, A stale after commit, and B resolving reused mesh key 1. The Doom path
compiled for WASM and its native projection test preserved every counted
resource family.

## Live Doom E1M1 To E1M2 Correlation

The maintainer supplied the missing live browser record. E1M1 presented first
with this prepared inventory:

```text
meshes=1117 textures=55 materials=56 pipelines=7 cameras=1 commands=2068
logical-resource-total=1236
alternative-c-inventory-correlation=None
```

E1M2 then presented in the same browser session with:

```text
meshes=2184 textures=80 materials=81 pipelines=7 cameras=1 commands=4106
logical-resource-total=2353
retired-E1M1-resource-total=1236
```

The E1M2 record contained the complete correlation result:

```text
generation-a=0
generation-b=1
failed-stage-family=Material
failed-stage-resources=2265
source-after-failed-stage=DOOM E1M1 prepared inventory
retired-source=DOOM E1M1 prepared inventory
source-after-commit=DOOM E1M2 prepared inventory
generation-a-after-commit=StaleGeneration(requested=0,current=1)
generation-b-reused-mesh-key-resolves=true
authority=semantic-shadow-not-provider-lifetime
```

The arithmetic is consistent with the caller's accounting: `1117 + 55 + 56 +
7 + 1 = 1236`, `2184 + 80 + 81 + 7 + 1 = 2353`, and injected material
failure occurred after `2184 + 80 + 1 = 2265` staged resources. This completes
the heterogeneous real-caller correlation gate. It does not prove atomic WGPU
staging, retained-provider reclamation, or a public generation contract.

Additional validation:

- `cargo test -p hello-render-resource-identity` (24 passed);
- `cargo test -p doom-ts-boundary-workbench-engine` (6 passed);
- strict `hello-render-resource-identity` clippy with warnings denied;
- both browser consumers checked for `wasm32-unknown-unknown`;
- both generated binding/build paths completed; and
- direct execution of `run_scene_generation_prototype()` from regenerated
  WASM retained the independent inventory result above.
