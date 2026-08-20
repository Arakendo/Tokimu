# Renderer Reliability Campaign

| Field | Value |
| --- | --- |
| Status | In progress -- repeated real-provider pressure passed; admission decision ready |
| Controlling plan | [Renderer Scene-Resource Lifetime And Replacement](renderer-scene-resource-lifetime-and-replacement.md) |
| Related reviews | AR-0024, AR-0027, AR-0030, and AR-0032 |
| Current disposition | ADR-0018 admits narrow atomic staged resource-set replacement semantics; it does not admit a shared allocator, final handle encoding, or physical reclamation policy |
| Next action | Run the live browser WGPU staged-replacement probe and retain its integrated stale-command result; implementation and WASM build are ready, while physical reclamation and final public handle shape remain unresolved |

The completed
[resource-identity and failure-presentation plan](renderer-resource-identity-and-failure-presentation.md)
retains its no-admission disposition. The new lifetime plan does not reopen
application-owned handle allocation by default; it separates logical identity
from physical renderer/provider residency.

Alternative designs live in [Studies](Studies/). Native/browser lifecycle,
containment, and diagnostic observations live in [Evidence](Evidence/).

Cross-campaign renderer-boundary evidence also lives here when its immediate
caller is another campaign. The
[AR-0030 transient geometry precedent survey](Evidence/AR-0030%20transient%20geometry%20precedent%20survey.md)
records the lifetime and ownership constraints applied to Doom's private
submission-local geometry experiment.
