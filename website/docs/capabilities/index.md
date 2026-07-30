---
title: Capabilities
description: A bounded map of Tokimu's current engine capabilities and maturity language.
---

<p class="page-kicker">Capabilities / current map</p>

# Current capability map

Tokimu is actively evolving. The table below describes broad ownership and
direction, not a release compatibility promise.

| Capability | Current role | Public posture |
| --- | --- | --- |
| World and runtime | State, resources, scheduling, fixed-step execution | Active |
| Rules | Conditions, actions, diagnostics, and execution semantics | Active |
| Rendering | Provider-neutral meshes, materials, pipelines, and draw submission | Active |
| Input | Normalized engine-facing input state | Active |
| Assets | IDs, handles, byte-oriented loading boundaries | Active |
| Presentation | Text, vector, material, and override semantics under evidence | Experimental |
| WASM | Browser entry surfaces and consumer corpus integration | Experimental |
| Networking | Envelope and loopback corpus evidence | Deferred |
| VR/XR | Architectural concern with implementation pending | Deferred |

## Format vocabulary

Format pages use evidence-specific states:

- **Renderable** means admitted semantics can be lowered and presented through
  the declared Tokimu path.
- **Previewable** means a bounded diagnostic preview exists while important
  source semantics remain deferred.
- **Inspected** means Tokimu can decode and report bounded structure without
  claiming canonical rendering.
- **Experimental** means the behavior currently runs through an incubating
  boundary.
- **Deferred** means the behavior is intentionally unsupported or awaiting
  more evidence.

These labels are scoped. A renderable geometric subset does not imply complete
format compatibility.
