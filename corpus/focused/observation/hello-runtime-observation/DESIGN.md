# Hello Runtime Observation

## Purpose

`hello-runtime-observation` is a headless corpus test for bounded, immutable
inspection of Tokimu-owned runtime state.

It asks:

> Can a consumer receive deterministic structural world evidence and selected
> application-approved detail without receiving a `World`, component storage,
> or mutation authority?

## Primary Proof

```text
World
  |
  v
WorldSnapshot
  |
  v
application-owned observation adapter
  |
  v
bounded owned observation
  |
  v
deterministic JSON evidence
```

`WorldSnapshot` remains the structural source. The corpus adapter adds a
versioned envelope, explicit scenario tick/revision metadata, budgets, and a
small selected-detail registry for the scenario's `Position` and `Enabled`
components.

## Ownership

- `tokimu-core` owns entity identity, component/resource registration,
  relationships, and immutable structural snapshots.
- The corpus scenario owns the meaning of tick, revision, `Position`,
  `Enabled`, and which component values may be inspected.
- Serialization owns representation only; JSON is not a kernel contract.
- A consumer owns no mutation authority through an observation.

## Command Phase

The corpus also proves one application-owned mutation boundary:

```text
Command request
  |
  v
bounded application queue
  |
  v
validate at `apply_commands`
  |
  v
accepted scenario mutation
  |
  v
revisioned observation evidence
```

- `MoveBy` and `SetEnabled` are scenario commands, not generic component
  mutation APIs.
- Commands are processed FIFO only when `apply_pending_at_tick` is called.
- Accepted commands advance the application revision exactly once; rejected
  commands leave the `World` and revision unchanged.
- Revision expectations reject conflicting writes explicitly. Unknown targets
  and targets without the command's required component are also explicit.
- Requests name corpus-local `observer` or `operator` authority. Only an
  operator request can enter the bounded mutation queue; this is evidence for
  admission order, not a promoted authorization service.

## Animation Observation

`corpus/assets/CheckLicense/hole_punch1.glb` provides a second, independent
producer of observation
pressure. Its GLB decoder remains a provider implementation; the corpus only
exposes a bounded catalog of named translation clips and application playback
state.

```text
GLB provider
  |
  v
clip catalog (name, duration, animated nodes)
  |
  v
application playback state and policy
  |
  v
sampled translation evidence
```

- `step1` through `step5` are catalog evidence, not public GLB objects.
- Play, pause, resume, stop, seek, speed, looping, next-step, and reset are
  application-owned semantic commands.
- Playback advances only under the fixed 60 Hz corpus step.
- Retaining completed assembly steps is an explicit `PlaybackPolicy`, never a
  hidden interpretation of the imported clips.
- The provider currently admits finite linear translation channels only;
  unsupported GLB animation data remains a decoder diagnostic.

## Presentation Identity Observation

The scenario keeps its application entity, imported node, and presentation
target identities as separate values. `presentation-control` resolves transient
selection and hotspot overrides; it never changes source asset data, node
identity, or the ECS entity.

```text
application arm entity 1
  |
  +-- explicit map --> imported node 21
  |
  +-- explicit map --> mesh-primitive:hole-punch/node/21/mesh-primitive
                              |
                              v
                    resolved presentation override
```

Unknown presentation targets produce a bounded application-owned diagnostic.
This corpus does not yet claim expiry behavior because `World` destruction and
source-asset lifetime invalidation are not part of this scenario.

## Determinism And Bounds

- Entities, type summaries, relationship families, edges, and targets are
  emitted in stable order.
- Entity and relationship-edge counts are bounded by explicit limits.
- Truncation, unknown identities, and unavailable selected detail are explicit
  diagnostics.
- Repeating an observation over unchanged state produces identical bytes.

## Non-Goals

- Generic reflection or arbitrary component serialization.
- Generic commands or mutable remote world access.
- Animation playback control.
- Network transport or replication semantics.
- A stable `tokimu-observation` crate.

Those concerns remain later slices of
`docs/Plans/Standalone/runtime-observation-and-command-corpus.md`.

## Success Criteria

- Observation leaves the world unchanged.
- Summary and selected-detail modes are both bounded.
- Unknown entities and unavailable detail are diagnosed.
- No borrowed component value, raw storage, or `World` reference escapes.
- Generated JSON is deterministic across unchanged observations.
