# Hello Particles

## Purpose

`hello-particles` is the first visual corpus consumer of the incubating
`particle-tools` library. It proves that deterministic particle state can feed
Tokimu presentation without making the renderer understand emitters or effect
meaning.

## Primary Proof

```text
application effect policy
        |
        v
particle-tools simulation
        |
        v
provider-neutral particle state
        |
        v
local role-to-presentation mapping
        |
        v
Tokimu mesh commands
```

The example presents three independent cases:

- a fixed-rate upward stream;
- a fixed-rate directional spray; and
- a periodic or manually triggered radial burst.

## Ownership

`particle-tools` owns bounded spawning, deterministic variation, integration,
drag, lifetime expiration, and structural reports.

This example owns effect timing, presentation-role meaning, colors, meshes, and
the decision to pause or reset.

Tokimu rendering owns mesh upload, material upload, draw submission, and
pixels.

## Structural Evidence

Before opening a native window, the corpus writes deterministic artifacts for
the burst, stream, and spray cases under:

```text
target/particle-corpus/hello-particles/
```

Each JSON artifact records the validated request, spawn report, particle-system
snapshot, bounded visible-instance batch, fixed-step sequence, and seed.
Lowering duration is reported separately to stderr because wall-clock timing is
measurement evidence, not deterministic state.

## Controls

- `Space`: trigger a radial burst.
- `Q`: pause or resume emission and simulation.
- `R`: reset with the original seed.
- `Escape`: close the example.

## Non-Goals

- textures or sprite atlases;
- GPU particle simulation;
- blend-mode or shader admission;
- ECS integration;
- 3D billboarding;
- gameplay collision; and
- extraction into a permanent engine crate.

## Success Criteria

- Equal seeds and fixed steps retain deterministic particle behavior.
- All three effects use the shared particle state and emitter contracts.
- Presentation roles remain application-defined.
- Capacity exhaustion remains bounded and visible in the window title.
- Pause and reset behavior do not depend on renderer state.
- Repeated artifact generation produces identical structural JSON.
