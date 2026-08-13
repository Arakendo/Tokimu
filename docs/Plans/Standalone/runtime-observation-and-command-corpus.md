# Runtime Observation And Command Corpus

## Status

Proposed.

This plan defines corpus work for bounded runtime observation and semantic
commands. It does not admit a universal world-export API, reflection system,
animation capability, or new crate. Architectural conclusions remain subject
to corpus evidence and review.

## Purpose

Tokimu consumers increasingly need to inspect what the engine believes exists,
present that information through native or TypeScript tools, and request
changes without receiving raw `World` access.

The first corpus should answer:

> Can an external consumer inspect bounded Tokimu-owned state and request
> deterministic object mutations, animation playback, and presentation changes
> through observations and commands without owning simulation truth, importer
> objects, or renderer state?

The intended interaction is:

```text
Tokimu-owned state
        |
        v
bounded immutable observation
        |
        v
Rust or TypeScript consumer
        |
        v
semantic command request
        |
        v
validation and lifecycle boundary
        |
        v
Tokimu-owned mutation
        |
        v
new observation revision
```

The corpus is an API-design tool. It should discover repeated contracts before
those contracts are promoted into engine crates or stabilized for WASM.

## Motivation And Existing Evidence

Tokimu already contains several partial proofs:

- `World::snapshot()` produces an immutable structural `WorldSnapshot`.
- `World::query_component()` and `World::query_relationships()` support
  read-only typed queries without exposing mutable storage.
- the SDD calls for a text snapshot of entities, component/resource summaries,
  relationships, and named component inspection;
- `corpus/lib/presentation-control` provides stable presentation-target IDs,
  target enumeration, layered overrides, and deterministic diagnostics;
- the ASP.NET/WASM presentation workbench proves that TypeScript can issue
  bounded presentation commands without parsing source assets or owning
  rendering semantics;
- `hello-hole-punch` decodes five named GLB translation clips (`step1` through
  `step5`), samples them deterministically, and preserves completed assembly
  steps while later clips play;
- networking work already uses versioned, sequenced application-owned
  observation snapshots without serializing an arbitrary `World`;
- performance diagnostics already prove that observations can be bounded and
  structured without turning diagnostics into a universal world object.

These proofs do not yet form one shared runtime observation and command
boundary. Each consumer currently assembles the state it needs locally.

## Architectural Thesis

> Consumers observe immutable, owner-labeled state and issue semantic commands.
> Tokimu validates and applies those commands at explicit lifecycle boundaries.

Queries read. Commands request mutation. Neither grants ambient access to the
world.

## Observation Families

The corpus may aggregate several observation families into one workbench, but
it must not collapse their ownership into one universal state object.

### World observation

Owned by simulation and runtime semantics:

- entity identity and lifetime;
- registered component summaries or selected values;
- resources selected for observation;
- relationships;
- simulation tick and world revision.

### Application observation

Owned by the application:

- score, lives, tool mode, document state, or selected domain object;
- application-specific labels and commands;
- domain policy not implied by the ECS.

### Asset observation

Owned by importers and asset services:

- source nodes, meshes, clips, materials, metadata, and diagnostics;
- provider-neutral identities produced by bounded import profiles;
- no provider-native parser object or source-format mutation authority.

### Presentation observation

Owned by presentation semantics:

- stable presentation target IDs;
- source and resolved color, opacity, visibility, and emphasis roles;
- selected, hovered, warning, and hotspot overrides;
- no implication that a presentation target is an ECS entity.

### Diagnostic observation

Owned by diagnostic producers and policy:

- bounded warnings and failures;
- performance-budget transitions;
- command rejection and unsupported-capability evidence;
- no unbounded log or universal state dump.

## Candidate Interaction Vocabulary

The first implementation may refine these names. They are planning vocabulary,
not accepted public APIs.

### Observation envelope

```rust
pub struct ObservationEnvelope<T> {
    pub schema: String,
    pub version: u16,
    pub sequence: u64,
    pub tick: u64,
    pub revision: u64,
    pub kind: ObservationKind,
    pub payload: T,
}
```

Constraints:

- payloads are owned immutable values;
- every payload identifies its semantic owner;
- sequence, tick, and revision have distinct meanings;
- absence and unsupported data are explicit;
- observation budgets bound item counts and payload size;
- this local shape is not automatically the networking replication envelope.

### Command request and result

```rust
pub struct CommandRequest<T> {
    pub command_id: String,
    pub expected_revision: Option<u64>,
    pub target: TargetId,
    pub command: T,
}

pub struct CommandResult {
    pub command_id: String,
    pub status: CommandStatus,
    pub applied_tick: Option<u64>,
    pub resulting_revision: Option<u64>,
    pub diagnostic: Option<CommandDiagnostic>,
}
```

Commands must be validated before mutation and applied through an explicit
runtime/application phase. A successful request is observable through both its
result and a later state revision. Unknown, stale, unauthorized, conflicting,
or unsupported requests fail deterministically.

## First Corpus Scenario

The first headless scenario should contain a deliberately small world:

- two addressable objects with position and enabled state;
- one parent/child relationship;
- one application-owned selection value;
- one presentation target linked by explicit mapping rather than identity
  equivalence;
- one bounded command queue;
- deterministic fixed-step time.

The scenario exercises:

1. observe the initial structure and selected values;
2. request one object translation;
3. request one enabled-state change;
4. reject a stale-revision mutation;
5. apply accepted commands at the documented phase boundary;
6. observe the resulting revision and command results;
7. replay the same scripted sequence and compare evidence hashes.

The object commands are application-owned semantic commands. The corpus should
not begin with arbitrary `set_component(type_name, bytes)` mutation.

## Animation Corpus Scenario

Use `corpus/assets/GLB/hole_punch1.glb` and the existing
`corpus/campaigns/textured-presentation/hello-hole-punch` evidence. The source exposes five named translation
clips representing sequential assembly steps.

### Required animation observations

- stable imported clip identity and source name;
- duration and admitted channel profile;
- playback state: stopped, playing, paused, or completed;
- active clip identity;
- local playback time;
- speed and loop policy;
- held/composed prior-step state where application policy requires it;
- affected provider-neutral target identities;
- unsupported channel/interpolation diagnostics.

### Required animation commands

- select clip;
- play;
- pause and resume;
- stop;
- seek to a bounded time;
- set playback speed within a documented range;
- set loop or one-shot policy;
- advance to the next authored assembly step;
- reset the sequence.

The corpus must distinguish:

```text
imported clip data       importer-owned evidence
playback state           runtime/application-owned state
object transforms        simulation or scene state
camera orbit             presentation-only inspection state
```

The corpus should not promote `hello-hole-punch`'s current clip-cycling helper
as the engine animation model. It should use that behavior as evidence to
discover the minimum reusable command and observation contract.

## Presentation And Hotspot Scenario

Selection and hotspot changes are presentation commands unless an application
explicitly owns a separate simulation meaning.

```text
application says "target is a hotspot"
        |
        v
presentation command
        |
        v
override composition
        |
        v
resolved presentation observation
        |
        v
renderer execution
```

The scenario must prove that:

- selecting an object does not mutate imported material data;
- hotspot, warning, hover, and selection layers compose deterministically;
- clearing one layer restores the remaining resolved state;
- a presentation target may map to an entity or imported node without being
  identical to either;
- renderer resources and GPU handles never appear in observations or commands.

## TypeScript Consumer Scenario

After the headless contract stabilizes, build a bounded WASM consumer that can:

- request a summary observation;
- inspect one selected object in detail;
- select an object from a viewport or list;
- request translation or an application-owned property mutation;
- inspect available animation clips;
- play, pause, seek, and reset playback;
- apply and clear selection/hotspot presentation roles;
- display accepted/rejected command results and the resulting revision.

TypeScript owns interaction and presentation. It must not:

- parse GLB/FBX/CGM/SVG source formats;
- receive `World`, ECS storage, Rust references, or provider-native objects;
- mutate observation JSON and treat that as engine state;
- update a browser shadow scene graph as the authority;
- invoke renderer-specific resource APIs.

## Evidence Artifacts

Each deterministic run should emit bounded artifacts such as:

```text
runtime-observation-command/
    manifest.json
    initial-observation.json
    command-trace.json
    animation-trace.json
    presentation-trace.json
    final-observation.json
    diagnostics.json
```

Artifacts should record:

- schema and artifact version;
- producer and algorithm identity;
- corpus fixture identity and input hash where relevant;
- fixed-step policy;
- observation sequence, tick, and revision;
- command IDs, validation outcomes, application ticks, and resulting revisions;
- bounded counts and truncation diagnostics;
- deterministic content hashes;
- target platform when comparing native and WASM behavior.

Screenshots may provide complementary visual evidence. Structural observations
and command traces remain authoritative for state and lifecycle validation.

## Implementation Location

Incubate reusable support under:

```text
corpus/lib/runtime-observation-command/
```

Exercise it first through:

```text
corpus/focused/observation/hello-runtime-observation/
```

Then adapt independent consumers rather than growing one universal demo:

```text
corpus/campaigns/textured-presentation/hello-hole-punch/
corpus/consumers/aspnet-wasm-presentation-workbench/
corpus/consumers/<runtime-observation-workbench>/
```

The exact directory names may be refined during implementation. Do not create
`tokimu-observation`, `tokimu-command`, or `tokimu-animation` solely to satisfy
this plan.

## Implementation Progress

### 2026-07-31: Read-Only Structural Baseline

`corpus/focused/observation/hello-runtime-observation` now exercises the first read-only boundary:

- the headless scenario contains two entities with application-owned position
  and enabled state, one parent/child relationship, and one registered resource;
- `WorldSnapshot` remains the source for entity, component/resource type, and
  relationship summaries;
- a corpus-owned adapter emits bounded summary or selected-detail JSON without
  exposing `World`, component storage, or borrowed component values;
- selected component detail is deliberately registered for only `Position`
  and `Enabled`; unavailable detail is diagnostic rather than reflective;
- sequence, tick, and revision are explicit scenario context supplied to the
  adapter and are not represented as hidden `World` fields;
- unchanged observations produce identical bytes, and generated evidence is
  written beneath `target/runtime-observation-command/`;
- relationship target ordering is now deterministic in `WorldSnapshot`, not
  repaired only by the corpus adapter.

No shared `corpus/lib/runtime-observation-command` library has been extracted.
The first caller remains local while the semantic contract is still being
interrogated. Entity destruction and stale-generation evidence are not yet
available in `World`, so stale identity behavior remains a later slice rather
than a simulated claim.

### 2026-07-31: Application-Owned Command Boundary

The same corpus now proves a bounded command path without admitting generic
world mutation:

- `MoveBy` and `SetEnabled` are scenario-owned commands; no command can name
  arbitrary component storage or obtain a `World` reference;
- requests enter a fixed-capacity FIFO queue and are processed only at the
  application-owned `apply_commands` phase via `apply_pending_at_tick`;
- expected revision mismatches, unknown targets, unsupported target/component
  combinations, and queue overflow each return a distinct result and
  diagnostic;
- accepted commands advance the scenario revision exactly once and identify
  their applied tick and resulting revision;
- rejected commands are side-effect free, including structurally: validation
  checks immutable component presence before calling the mutating world query;
- a fixed script writes `command-trace.json` and a final selected-detail
  observation alongside the read-only evidence, and replays to identical
  results and JSON bytes.

This remains application-local evidence. It does not establish that every
Tokimu application should share a command queue, revision policy, or lifecycle
phase name.

### 2026-07-31: Animation Catalog And Playback Baseline

`hello-runtime-observation` now adapts the existing `hole_punch1.glb` corpus
asset without exposing GLB decode objects:

- a provider-neutral catalog records five named translation clips, their
  durations, and animated node identities;
- playback state carries local time, speed, looping, selected clip, mode, and
  an application-selected completed-step hold policy separately from catalog
  data;
- bounded play, pause, resume, stop, seek, speed, loop, next-step, and reset
  commands retain state on rejection and report explicit outcomes;
- playback advances only under a deterministic 60 Hz step, while samples are
  emitted as provider-neutral node-to-translation evidence;
- generated `animation-catalog.json` and `playback-evidence.json` accompany
  the world and command trace artifacts.

The only supported source animation profile remains finite linear translation
channels. Rotation, scale, weights, and non-linear interpolation continue to
be explicit GLB-provider diagnostics rather than silently approximated state.

### 2026-07-31: Identity And Presentation Mapping Baseline

The corpus now composes its existing application entity with one GLB-derived
node identity and one `presentation-control` target without treating any pair
as interchangeable:

- entity `7`, imported node `21`, and the mesh-primitive presentation target
  are recorded in a single explicit mapping artifact;
- selection and hotspot commands delegate override composition to the existing
  `presentation-control` corpus library, leaving importer/source truth and the
  ECS world unchanged;
- `presentation-mapping.json` records semantic command outcomes plus source
  and resolved target state after a hotspot override is cleared;
- an unknown presentation target produces an owner-labeled deterministic
  diagnostic.

Expiry behavior remains deferred: the current core scenario has no entity
destruction or source-asset invalidation event that would honestly establish an
expired mapping contract.

### 2026-07-31: Command Admission Authority Baseline

`hello-runtime-observation` now places one explicit, application-local
authority check before queue admission:

- requests name `observer` or `operator` authority;
- only `operator` can enter the scenario mutation queue;
- an observer mutation request returns `command_authority_denied` without
  consuming queue capacity, mutating the world, or advancing revision.

This is deliberately not a general Tokimu authorization service. It proves the
ordering requirement that authority is checked before scheduling.

### 2026-07-31: WASM Observation Facade Baseline

`corpus/consumers/runtime-observation-workbench/engine` now exposes the first
browser-facing adapter without widening the runtime contract:

- `WasmRuntimeObservationSession` emits bounded owned observation JSON, admits
  semantic command JSON, and applies the FIFO queue only through the explicit
  lifecycle call;
- TypeScript-facing contract types describe command and playback request
  records, while intentionally avoiding `World`, ECS storage, parser objects,
  and renderer handles;
- the adapter exposes presentation selection only through the scenario's
  mapped target and exposes provider-neutral animation catalog and playback
  records;
- the WASM target compiles without the meshopt C decoder by consuming a
  native-test-verified provider-neutral catalog fixture. Native decoding of
  `hole_punch1.glb` continues to verify that fixture exactly.

This establishes a WASM runtime-observation boundary, not WASM meshopt/GLB
import support. That provider capability remains an explicit later decision.

## Implementation Slices

### Slice 0: Boundary Inventory And Scenario Freeze

Deliverables:

- [x] Inventory `WorldSnapshot`, typed world queries, presentation-control,
      networking observation envelopes, performance observations, and current
      animation helpers.
- [x] Classify every proposed field by observation family and owner.
- [x] Freeze the first headless object scenario and command script.
- [x] Freeze the `hole_punch1.glb` animation scenario and expected five clips.
- [x] Record unsupported mutation, animation, and reflection behavior.

Acceptance criteria:

- [x] No proposed observation is described only as generic "world state."
- [x] Queries, commands, events, diagnostics, and replication are distinct.
- [x] Every command names its mutation owner and application phase.
- [x] No new engine crate or stable public API is required.

### Slice 1: Structural World Observation Baseline

Deliverables:

- [x] Build `hello-runtime-observation` around the existing `WorldSnapshot`.
- [x] Emit entity, registered component/resource summary, relationship, tick,
      and revision evidence.
- [x] Add bounded summary and selected-detail query modes.
- [x] Define deterministic ordering and truncation behavior.
- [x] Diagnose unavailable or unregistered detail explicitly.

Acceptance criteria:

- [x] Observation does not mutate the world.
- [x] Repeated unchanged observations serialize identically.
- [x] Entity and relationship ordering is deterministic.
- [x] Unknown selected identities fail explicitly; stale command revisions are
      rejected before mutation.
- [x] No raw component storage or `World` reference escapes.

### Slice 2: Semantic Mutation Commands

Deliverables:

- [x] Define application-owned move and enabled-state commands.
- [x] Add a bounded command queue and explicit validation stage.
- [x] Apply accepted commands at one documented lifecycle phase.
- [x] Return accepted, rejected, stale, unknown-target, and unsupported results.
- [x] Advance world revision only according to documented mutation semantics.

Acceptance criteria:

- [x] A consumer cannot mutate state by editing an observation.
- [x] Stale expected revisions cannot silently overwrite newer state.
- [x] Rejected commands leave state and revision unchanged.
- [x] Accepted commands identify applied tick and resulting revision.
- [x] Replaying the same initial state and command script produces identical
      final evidence.

### Slice 3: Animation Catalog And Playback State

Deliverables:

- [x] Expose a bounded provider-neutral clip catalog for `hole_punch1.glb`.
- [x] Model playback state separately from imported clip data.
- [x] Add play, pause, resume, stop, seek, speed, loop, next-step, and reset
      commands where supported by the existing profile.
- [x] Sample playback under deterministic fixed-step time.
- [x] Retain the authored assembly-step hold behavior as explicit application
      policy rather than hidden importer behavior.

Acceptance criteria:

- [x] The catalog reports `step1` through `step5` deterministically.
- [x] Paused playback does not advance local time.
- [x] Seek and reset produce known sampled transforms.
- [x] Completed step state is retained only when the selected policy requests
      it.
- [x] Unsupported animation channels and interpolation remain diagnostic.

### Slice 4: Object, Asset, And Presentation Identity Mapping

Deliverables:

- [x] Define explicit mapping evidence among entity IDs, imported node/mesh
      IDs, and presentation target IDs.
- [ ] Preserve independent lifetime and stale-identity behavior for each kind.
- [x] Add selected and hotspot presentation commands through the existing
      presentation-control vocabulary.
- [x] Observe source and resolved presentation without exposing renderer state.

Acceptance criteria:

- [x] No ID kind is silently interchangeable with another.
- [x] Selection/hotspot changes do not mutate imported source data.
- [x] Clearing an override restores the correct composed state.
- [ ] Unknown and expired mappings produce deterministic diagnostics.

### Slice 5: Query Budgets, Authority, And Failure Semantics

Deliverables:

- [ ] Bound item counts, payload bytes, detail depth, and command queue length.
- [x] Add explicit query and command capability/authority checks.
- [ ] Define unavailable, partial, truncated, stale, conflict, and unsupported
      outcomes.
- [x] Ensure diagnostics remain bounded and identify the owning stage.

Acceptance criteria:

- [ ] Large worlds cannot cause an unbounded observation response.
- [x] Truncation is visible and deterministic.
- [x] Unauthorized commands cannot enter the mutation queue.
- [x] Malformed consumer input cannot panic or partially mutate state.
- [ ] Diagnostic payloads do not leak provider or backend internals.

### Slice 6: Native Inspector Consumer

Deliverables:

- [x] Build a simple inspector using only the bounded observation and command
      contracts.
- [ ] Show summary, selected detail, relationships, clip catalog, playback
      state, presentation state, and command results.
- [ ] Keep camera/navigation controls presentation-only.
- [ ] Capture structural evidence and optional manual screenshots.

Implementation progress:

- 2026-07-31: added `hello-runtime-inspector` as a separate native consumer
  scaffold. Its scenario facade keeps `World`, the imported animation catalog,
  playback state, and presentation-control state private; the inspector receives
  only IDs, immutable observations, and semantic command methods. Keyboard
  input queues scenario commands and the renderer consumes copied observation
  values rather than live runtime references. The fuller evidence view remains
  open.
- 2026-07-31: reorganized the native inspector into explicit world-observation,
  presentation/playback, command, and diagnostics regions. It now displays
  selected component values, relationship and edge counts, resolved
  presentation targets, current clip/playback state, and the last command
  result from copied observation data. Full relationship/catalog browsing,
  viewport navigation, and screenshot evidence remain open.
- 2026-07-31: moved the inspector's native shell from fixed normalized-card
  placement onto incubating viewport-aware frame and split layouts in
  `ui-tools`. The shared layout tests now prove non-overlap for normal and
  constrained frame bounds, and the consumer declares ellipsis rather than
  depending on text spillover. Manual native screenshot review now validates
  the wide-window frame; narrow viewport navigation and screenshot artifact
  capture remain open.
- 2026-07-31: `ui-tools` headless text reports now expose horizontal and
  vertical fit evidence before clip, ellipsis, or wrapping hides the excess.
  This gives future inspector layouts a deterministic way to elevate unfit
  labels into corpus diagnostics without coupling text layout to a renderer.

Acceptance criteria:

- [x] The inspector contains no duplicate world, importer, or animation state.
- [ ] Every displayed mutation is confirmed by a later observation revision.
- [ ] Rejected commands are visible and leave the viewport consistent.
- [ ] The inspector can be replaced without changing simulation semantics.

### Slice 7: WASM And TypeScript Consumer

Deliverables:

- [x] Expose bounded observation queries and command submission through WASM.
- [x] Record a TypeScript contract adjacent to the consumer facade.
- [x] Add object selection, mutation controls, animation controls, and
      presentation-role controls to a consumer corpus.
- [x] Preserve Rust ownership of validation, playback, state, and resolution.
- [x] Build the consumer into a browser WASM artifact with generated bindings
      and checked TypeScript output.

Acceptance criteria:

- [x] Browser TypeScript never receives raw `World` access.
- [x] TypeScript does not parse imported source formats.
- [x] Unknown-object and stale-command probes surface Tokimu-owned rejection
      evidence through the browser facade.
- [x] Native and WASM-facing adapters produce semantically equivalent traces
      for the fixed deterministic scenario. Browser runtime execution remains
      a separate integration check.
- [ ] Browser lifecycle failure does not alter native simulation semantics.
- [ ] Unsupported commands are explicit rather than silently emulated in the
      browser.

### Slice 8: Determinism, Replay, And Differential Evidence

Deliverables:

- [ ] Run the fixed scenario repeatedly and compare artifact hashes.
- [x] Replay the fixed semantic command script from the same initial state
      through native and WASM-facing adapters.
- [ ] Compare headless, native, and WASM observation/command traces at semantic
      boundaries.
- [ ] Separate deterministic state evidence from wall-clock and GPU evidence.

Acceptance criteria:

- [ ] Identical input, fixed time, and command order produce identical final
      observations.
- [ ] The first diverging artifact identifies the owning boundary.
- [ ] Platform-specific timing does not contaminate deterministic hashes.
- [ ] Replay does not bypass normal command validation or lifecycle phases.

### Slice 9: Architectural Review And Admission Decision

Deliverables:

- [ ] Record which observation and command contracts have at least two
      independent consumers.
- [ ] Decide whether world observation, command routing, animation playback,
      and presentation control remain sibling concerns or share a smaller
      admitted boundary.
- [ ] Decide whether an Architectural Review for Runtime Observation And
      Command Boundaries should be opened or closed with evidence.
- [ ] Update the SDD, roadmap, ADRs, and corpus docs with observed behavior.
- [ ] Keep provider, networking, renderer, and TypeScript mechanisms below
      their established ownership boundaries.

Acceptance criteria:

- [ ] No capability is promoted from diagram similarity alone.
- [ ] Every admitted semantic contract has independent callers.
- [ ] Deferred questions and unsupported behavior remain explicit.
- [ ] Crate extraction is a separate deliberate decision.

## Failure Semantics

| Boundary | Example failure |
| --- | --- |
| Observation query | unsupported schema, unknown target, budget exceeded |
| Observation production | unavailable owner, partial/truncated result |
| Command decode | malformed or unknown command kind |
| Authority | consumer lacks permission for requested operation |
| Validation | stale revision, invalid value, incompatible target |
| Scheduling | queue full, phase unavailable, target expired before apply |
| Mutation | application rejects domain transition |
| Animation | missing clip, invalid seek, unsupported channel/profile |
| Presentation | unresolved target or unsupported override |
| Consumer | unsupported observation version or command result |

No failure may be represented as a successful empty observation or silent
no-op mutation.

## Non-Goals

- Serializing arbitrary ECS storage.
- A reflection system for every Rust component.
- Direct TypeScript or script access to `World`.
- A universal mutable scene graph.
- Equating entities, imported nodes, presentation targets, and renderer
  handles.
- A complete animation system, mixer, skinning stack, or graph editor.
- Remote authority, networking replication, rollback, or prediction.
- Event sourcing or a permanent command log.
- A general editor, shell, or inspector framework.
- Renderer resource inspection or mutation.
- Promoting new crates before repeated independent use.

## Risks

### Universal Observation Blob

A single object containing world, assets, presentation, diagnostics, and
application data would erase ownership and create accidental compatibility
promises.

Mitigation: preserve typed owner-labeled observation families and compose only
at consumer boundaries.

### Generic Set-Property Commands

An unrestricted property setter would bypass application invariants and turn
serialization shape into mutation authority.

Mitigation: begin with named semantic commands and require owner validation.

### Animation Import Becomes Runtime Ownership

Imported clip metadata, playback state, sampled transforms, and presentation
inspection controls may collapse into one helper.

Mitigation: record each stage independently and use `hello-hole-punch` only as
evidence, not as the final animation API.

### Browser Shadow State

TypeScript may start treating cached observation JSON as authoritative state.

Mitigation: observations are immutable revisions; successful commands require a
subsequent Tokimu observation before UI state is considered confirmed.

### Identity Collapse

Entity, imported node, presentation target, and renderer handle IDs may appear
equivalent in small examples.

Mitigation: use distinct types, explicit mapping records, and independent stale
identity tests.

### Observation Cost Becomes Unbounded

Convenient inspection can accidentally serialize an entire world each frame.

Mitigation: summary/detail queries, budgets, explicit cadence, truncation, and
performance diagnostics.

## Validation

Focused validation should include:

- immutable and deterministic snapshot tests;
- summary/detail query ordering and budgets;
- accepted, rejected, stale, and unauthorized command tests;
- explicit command application-phase tests;
- fixed-step animation play/pause/seek/reset tests;
- identity mapping and expiration tests;
- presentation override composition tests;
- artifact hash and replay tests;
- native/WASM contract parity where dependencies permit it.

Workspace validation remains:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Completion Criteria

This corpus effort reaches a useful pause when:

- one headless world can be observed without raw storage access;
- named semantic commands mutate it only at an explicit lifecycle boundary;
- command results and resulting observation revisions agree;
- the five-step hole-punch animation can be observed and controlled through a
  provider-neutral playback seam;
- object, asset, and presentation identities remain distinct and mappable;
- one native and one TypeScript/WASM consumer use the same semantic contracts;
- deterministic structural evidence localizes failures by ownership stage;
- query cost, authority, stale state, and unsupported behavior are bounded and
  diagnostic;
- an Architectural Review records what, if anything, deserves admission.

## Graduation Triggers

Consider promoting a runtime observation or command capability only when:

- at least two independent consumers require the same semantic contract;
- the contract remains useful headlessly;
- no provider, renderer, browser, or importer type leaks through it;
- commands preserve explicit authority and lifecycle application;
- observations remain immutable, bounded, and owner-labeled;
- extraction removes demonstrated duplication or enables a real consumer.

Animation playback requires its own admission evidence. Reusing the observation
and command seam does not automatically make animation kernel-native.

## References

- [`On World State Info.md`](../../Conversations/On%20World%20State%20Info.md)
- [`Tokimu Software Design Document.md`](../../Tokimu%20Software%20Design%20Document.md)
- [`kernel-principles.md`](../../kernel-principles.md)
- [`consumer-corpora.md`](consumer-corpora.md)
- [`networking-and-transport.md`](networking-and-transport.md)
- [`typescript-shader-material-presentation-control.md`](typescript-shader-material-presentation-control.md)
- [`performance-diagnostics-and-runtime-observation.md`](performance-diagnostics-and-runtime-observation.md)
- `crates/tokimu-core/src/world.rs`
- `corpus/lib/presentation-control/README.md`
- `corpus/campaigns/textured-presentation/hello-hole-punch/DESIGN.md`
- `corpus/consumers/aspnet-wasm-presentation-workbench/DESIGN.md`
