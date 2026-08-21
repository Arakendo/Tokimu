# Renderer Reliability Campaign

| Field | Value |
| --- | --- |
| Status | In progress -- ADR-0018 and ADR-0019 pass native/browser WGPU conformance |
| Controlling plan | [Renderer Scene-Resource Lifetime And Replacement](renderer-scene-resource-lifetime-and-replacement.md) |
| Related reviews | AR-0024, AR-0027, AR-0030, AR-0032, and AR-0033 |
| Current disposition | ADR-0018 admits atomic staged resource-set replacement; ADR-0019 admits only fixed-descriptor texture-content replacement in the current set |
| Next action | Apply ADR-0019 to the persistent browser Doom console without generalizing to meshes/descriptors or claiming physical reclamation |

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

The
[AR-0033 provider and pressure evidence](Evidence/AR-0033%20Slice%202%20provider%20and%20pressure.md)
records the formerly feature-gated fixed-descriptor texture transaction on
native WGPU and browser WebGPU, including externally observed repeated
console-sized updates. ADR-0019 admits its narrow provider-neutral semantics.
