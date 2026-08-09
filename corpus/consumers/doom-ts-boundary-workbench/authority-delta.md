# Authority Delta: Browser Asset Intake

## Slice 1 Baseline

| Field | Evidence requirement |
| --- | --- |
| Requested authority | One user-selected local file's bytes, name, and browser media hint. |
| Granted authority | Gesture-bound file read only; no filesystem path, directory enumeration, network, storage, timer, or ambient DOM authority beyond intake controls. |
| Actually exercised | 2026-08-09 local browser selection of `doom-shareware-corpus-v1.zip`: 1,810,639 bytes transferred once to Rust/WASM; schema-v1 observation reported one retained resource and BLAKE3 `58146f5aa0e14ef38047a79878307344aec821b9f312da6a9208ec08e399660c`. |
| Denied attempts | Rust tests reject empty labels, empty byte arrays, and over-limit selections without retaining a resource. Browser exercise on 2026-08-09 confirmed picker cancellation remains a cancelled presentation outcome; no request is issued. Unsupported-format evidence remains pending. |
| Authority surviving disposal | Rust regression tests prove replacement and disposal release all retained resources and bytes. Browser observation on 2026-08-09: the explicit Clear intake action returned `retainedResources: 0` and `retainedBytes: 0`. `pagehide` invokes the same generated-binding disposal path. |

## Non-Authority Statement

This mechanism cannot parse WAD/ZIP bytes, define Doom namespaces, create
world state, select renderer policy, or persist state beyond the explicit
Rust/WASM session. A successful transfer is not TTSDD semantic authoring or a
runtime TypeScript host.

## Slice 5B Browser Presentation Bridge (Unexercised)

| Field | Evidence requirement |
| --- | --- |
| Requested authority | A browser canvas passed to one explicit Rust/WASM static-frame request after local package selection. |
| Granted authority | Presentation-surface use only; TypeScript provides neither Doom geometry nor texture/material policy. |
| Actually exercised | 2026-08-09: after explicit local selection, the supplied canvas request returned `browser first frame presented: 1835 draws` and visibly presented the static E1M1 overview. Rust/WASM retained/derived package bytes and executed the rendering path; TypeScript supplied only the user-owned canvas and presentation state. |
| Denied attempts | No rendering request is enabled until the Rust/WASM session reports a retained package; clearing selection disables it again. |
| Authority surviving disposal | The canvas is not retained by the session; disposal releases selected bytes. The existing browser clear/disposal observation remains applicable; no canvas identity enters the Rust session state. |
