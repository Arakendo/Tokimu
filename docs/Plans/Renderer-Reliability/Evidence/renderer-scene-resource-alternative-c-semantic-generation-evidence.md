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
