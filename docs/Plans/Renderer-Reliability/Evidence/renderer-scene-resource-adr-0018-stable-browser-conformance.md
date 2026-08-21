# ADR-0018 Stable Browser WGPU Conformance Evidence

| Field | Value |
| --- | --- |
| Status | Complete |
| Date | 2026-08-20 |
| Target | Browser WASM / WebGPU, 640x360 canvas |
| Contract | ADR-0018 provider-neutral resource-set session |
| Authority | Live page result plus terminal observer outside the page failure domain |

## Sequence

```text
populate A
    -> consume backend into resource-set session
    -> present scoped A
    -> stage all B resource families
    -> force late MissingTexture(9)
    -> present scoped A unchanged
    -> stage complete B
    -> commit B atomically
    -> reject retained scoped A before local resolution
    -> present B
    -> submit current scoped B
    -> present B
```

A and B deliberately reuse local resource keys. The failed candidate consumes
set identity 2, so the committed successor advances from set 1 to set 3.

## Result

```text
status=complete
lifetime-alternative=C-corpus-private-real-provider-staging
backend-creations=1
device-creations=1
surface-creations=1
retained-provider-session=true
staged-before-failure=26
staged-families-before-failure=meshes+textures+materials+pipelines+cameras+commands
forced-stage-failure=MissingTexture(9)
A-draws-initial=8
A-draws-after-failed-B=8
last-known-good-preserved=true
resource-set-A=1
resource-set-B=3
retained-A-command-after-B=StaleResourceSet(requested:1,current:3)
reused-local-resource-keys=true
stale-rejected-before-resource-resolution=true
unscoped-submit-surface=absent
B-draws-after-commit=8
scoped-B-draws=8
retired-A-predictable=true
provider-diagnostics=0
resource-set-contract=ADR-0018-provider-neutral-resource-set-session
individual-handle-encoding=undecided
backend=browser-webgpu
```

The complete commit observation reported exact retirement/installation
symmetry for eight queued draws, materials, textures, and meshes, plus one
pipeline and camera on each side. Eight retained instance bindings remained.

The external terminal observer independently recorded:

```text
classification=completed
subject-started=true
reason=resource-lifetime-C-real-provider-staging
physical-cause=unknown-unless-explicitly-reported
```

The browser launcher handed off before page acknowledgement. The observer
retained page identity, heartbeat, and terminal events as authority rather than
misclassifying the launcher exit as browser termination.

## Non-claims

This does not establish physical GPU reclamation timing, bounded physical VRAM
overlap, device-loss recovery, repeated replacement sustainability beyond the
separate corpus-private pressure record, or individual handle encoding.

## References

- `docs/ADR/ADR-0018-atomic-staged-render-resource-set-replacement.md`
- `docs/Architectural Reviews/AR-0032-atomic-staged-render-resource-set-replacement.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-adr-0018-stable-native-conformance.md`
- `corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/`
- `corpus/campaigns/renderer-reliability/hello-browser-terminal-observer/`
