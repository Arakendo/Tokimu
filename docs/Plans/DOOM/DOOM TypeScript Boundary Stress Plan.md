# DOOM TypeScript Boundary Stress Plan

## Status

Proposed on 2026-08-08. This is a separate corpus experiment from
[`DOOM WAD Checklist.md`](DOOM%20WAD%20Checklist.md).

The WAD checklist owns the canonical importer, Doom semantic, geometry,
gameplay, and native/WASM proof. This plan consumes those bounded observations
and deliberately places as much mechanism and authored behavior in TypeScript
as practical to discover where the TTSDD boundary remains clean, where it is
useful, and where it fails.

Nothing in this plan changes the WAD checklist's completion state. A
TypeScript experiment cannot substitute for its Rust-owned provider baseline,
and the WAD plan does not need to adopt a TypeScript finding unless AR-0020 and
an applicable ADR explicitly change the boundary.

## Purpose

Use the Doom corpus as a demanding end-to-end pressure test for
[`AR-0020`](../../Architectural%20Reviews/AR-0020-typescript-authoring-boundary-and-corpus-conformance.md)
and the
[`Tokimu TypeScript Design Document`](../../Tokimu%20TypeScript%20Design%20Document.md).

The experiment asks:

> How much asset acquisition, resource interaction, inspection, presentation,
> input, authored game behavior, and runtime orchestration can TypeScript own
> before it begins duplicating Tokimu semantics, hiding durable state, or
> becoming a second engine?

The desired result is a retained boundary map, including successful uses,
unacceptable ownership, conversion and performance costs, and mechanically
enforceable rules for a later TTSDD ADR.

## Relationship To The WAD Plan

```text
DOOM WAD Checklist
    canonical package and WAD evidence
    bounded Rust provider baseline
    Doom semantic observations
    world/presentation lowering
    native/WASM reference behavior
                 |
                 v
DOOM TypeScript Boundary Stress Plan
    browser asset acquisition
    Resource Store requests
    observation and presentation
    TTSDD-authored rules
    external runtime-host experiment
    optional TS provider comparison
    authority and durable-state negative tests
                 |
                 v
AR-0020 findings
    retain | constrain | reject | promote to ADR
```

The two plans share fixtures and observations only through explicit contracts.
They do not share hidden mutable state or duplicate checklist credit.

## Architectural Thesis

The experiment intentionally maximizes TypeScript responsibility while keeping
every responsibility classified:

```text
Browser TypeScript
    user gesture / drag-drop / fetch
    DOM, Canvas, HUD, inspector
    bounded Resource Store requests
    authored Tokimu rules
    optional runtime orchestration
                |
                v
Rust/WASM boundary
    byte and authority limits
    Resource Store identity and retained bytes
    canonical WAD/Doom semantic provider
    Tokimu durable world state
    renderer-neutral observations and requests
```

The corpus should try aggressive TypeScript placements, but a successful demo
does not prove correct ownership. Each placement must answer:

- What does TypeScript read?
- What does it emit?
- What state survives invocation or reload?
- Who owns the meaning of the output?
- What execution authority was granted?
- Can the same behavior be inspected, replayed, bounded, and recovered?

## Corpus Shape

The initial implementation should remain independently removable:

```text
corpus/consumers/doom-ts-boundary-workbench/
    DESIGN.md
    package.json
    tsconfig.json
    web/src/
    engine/                 Rust/WASM adapter over Tokimu-owned contracts
    tests/

frontends/packages/examples/src/doom/
    movement.rule.ts
    interaction.rule.ts
    pickups.rule.ts
    runtime-shell.ts
```

The exact paths may change if existing consumer infrastructure can be reused
without conflating responsibilities. Do not create `@tokimu/doom` merely to
match this sketch. A domain package requires evidence and an engine-owned
semantic target.

## TypeScript Role Matrix

| Experiment | AR-0020 class | TypeScript may own | TypeScript must not silently own |
| --- | --- | --- | --- |
| Browser asset intake | Browser/presentation mechanism | User gesture, file/fetch mechanism, source label, transfer progress | Resource identity, hashes, retained bytes, ZIP/WAD validation |
| Resource inspector | Browser/presentation mechanism | Selection, filtering, layout, visualization of observations | Resource Space rules, archive lookup, WAD namespaces |
| Doom rule source | TTSDD semantic authoring | Authored intent through admitted primitives and explicit execution mode | Engine semantic model, schedule, durable storage |
| Local Doom authoring shapes | Corpus-local precursor | Proposed typed intent and diagnostics | Stable `@tokimu/*` API claims |
| Runtime orchestration | External/runtime-provider experiment | Explicit capabilities, menus, non-authoritative flow, UI events | Ambient authority, hidden durable simulation state |
| TypeScript WAD parser comparison | External provider integration | Bounded parsing experiment and provider observations | Canonical importer truth, Resource Space semantics, trusted-core API |
| Generated WASM bindings | Generated binding | Transport mechanics | Semantic policy or hand-maintained game rules |

## Authority Delta Evidence

Every experiment in slices 1 through 9 must retain the same authority-delta
artifact. The artifact records observed authority rather than inferring trust
from a package name, language, or successful demonstration:

```text
Requested authority:
Granted authority:
Actually exercised:
Denied attempts:
Authority surviving disposal:
```

Each entry must name the applicable capability, resource, state, and lifecycle
boundary. `None` is a meaningful result only when a retained test or structural
constraint supports it. A request that was never exercised is not evidence that
the grant is safe, and a denied attempt must identify where and how rejection
occurred.

For lifecycle-bearing experiments, disposal includes the relevant combination
of cancellation, suspend, reload, revocation, exception, source replacement,
and workbench teardown. Any authority or private state that survives must be
identified explicitly and reconciled with its Tokimu-owned durable-state
contract.

The collected artifacts form an executable trust-boundary map for the final
AR-0020 review. They must remain comparable across browser intake, semantic
authoring, runtime hosting, external providers, and presentation mechanisms.

## Non-Goals

- Replacing the canonical WAD-plan implementation with TypeScript before the
  comparison earns that conclusion.
- Moving Node, TypeScript tooling, or a JavaScript engine into `tokimu-core` or
  `tokimu-runtime`.
- Creating a general scripting API, `@tokimu/doom`, or a monolithic TypeScript
  compiler speculatively.
- Treating browser success as lowering, determinism, recovery, or native/WASM
  parity evidence.
- Publishing reviewed Doom or Heretic data through the website merely to make
  the workbench convenient.
- Keeping health, inventory, doors, monsters, map progress, collision truth,
  or replay state in private TypeScript closures.

## Slice 0: Classification And Baseline Contract

### Deliverables

- [x] Create the workbench design document and classify every TypeScript unit
      using AR-0020's inventory schema.
- [x] Record reads, outputs, durable-state owner, semantic authority,
      execution authority, and retained evidence for every experiment.
- [x] Define the common authority-delta artifact and its retained structural
      structural evidence for slices 1 through 9.
- [x] Name the exact WAD-plan observations and requests consumed by the
      TypeScript workbench.
- [x] Retain a Rust/native baseline result from the WAD plan for every behavior
      later compared through TypeScript.
- [ ] Define boundary versioning, structured diagnostics, and unsupported
      behavior without exposing Rust/provider-native objects.

### Acceptance Criteria

- [ ] No TypeScript file is classified only by directory or language.
- [ ] Every comparison has a named baseline and cannot borrow credit from the
      WAD checklist.
- [ ] The workbench can be removed without changing stable engine crates or the
      canonical Doom consumer.
- [ ] Every planned experiment names how its authority delta will be captured
      before implementation begins.

## Slice 1: Browser Asset Intake Into Resource Store

### Deliverables

- [x] Accept one user-selected package/WAD byte source through browser
      TypeScript. The local browser workbench retained the reviewed compact
      package through the Rust/WASM session; drag/drop and fetch remain later
      separate exercises.
      bytes through browser TypeScript.
- [ ] Require an explicit user gesture for local file access.
  - [x] Add the TypeScript button-to-file-input binding. It opens the picker
        only from a click handler, forwards the resulting file to the typed
        Rust/WASM session, and clears the browser input after the request.
- [ ] Submit bytes, source label, media hint, and caller-selected limits through
      a versioned Rust/WASM request.
- [x] Have Resource Store compute and retain identity, content hash, bytes,
      visibility, and diagnostics.
  - [x] Establish the Rust/WASM one-selection session in
        `doom-ts-boundary-workbench-engine`: it bounds a selection to 64 MiB,
        retains bytes in a one-entry Resource Space root, returns schema-v1
        JSON with BLAKE3 fingerprint and retained-byte counts, and atomically
        replaces/disposes the prior selection. ZIP/WAD interpretation remains
        deliberately outside this first request.
- [ ] Return provider-neutral resource observations to TypeScript.
  - [x] Return the Rust-owned canonical-member observation to TypeScript. The
        browser result confirmed `DOOM1.WAD` as an IWAD with 419,602 transient
        derived bytes and 1,264 lumps, while the selected ZIP remained the
        single retained Resource Space resource.
- [ ] Exercise cancellation, repeated selection, duplicate bytes, replacement,
      oversized input, empty input, and interrupted-session behavior.
  - [x] Retain Rust regression coverage for replacement, empty input, empty
        source label, over-limit input, and disposal. The browser has matching
        retained/cancelled/rejected outcomes plus an explicit Clear intake
        action. Browser exercise confirmed repeat selection, cancellation, and
        explicit disposal; interrupted-session evidence remains separate.
- [ ] Record boundary copies, startup bytes, retained bytes, allocation totals,
      and browser memory.

### Acceptance Criteria

- [ ] TypeScript never assigns authoritative Resource Store identity or hashes.
- [ ] TypeScript does not parse paths or filenames to infer WAD/Doom semantics.
- [ ] Bytes remain session-local unless an explicit export/download request is
      made.
- [ ] Malformed or excessive input produces bounded structured diagnostics and
      leaves the prior selected resource usable.

## Slice 2: Archive, WAD, And Map Observation Shell

### Deliverables

- [ ] Let TypeScript request archive inspection, member selection, WAD
      inspection, and map selection through bounded Rust/WASM commands.
- [ ] Present package provenance, archive/member hashes, ordered lumps,
      namespaces, map summaries, and importer diagnostics.
- [ ] Add filters and diagnostic views for markers, duplicates, malformed
      ranges, unsupported lumps, and source indices.
- [ ] Keep TypeScript views derived from immutable observations rather than
      reconstructing archive, WAD, or Resource Space rules.
- [ ] Prove stale observations are rejected after source replacement.

### Acceptance Criteria

- [ ] No TypeScript code parses ZIP or WAD bytes in this canonical observation
      path.
- [ ] No TypeScript object becomes the source of truth for resource, lump, map,
      sector, linedef, sidedef, or thing identity.
- [ ] Browser inspection remains useful when rendering or gameplay is
      unavailable.

## Slice 3: Exact TTSDD Authored-Source Parity

### Deliverables

- [ ] Add Doom-shaped authored examples under the existing frontend examples
      package before proposing a new domain package.
- [ ] Feed the exact checked-in `.ts` files through the frontend rather than
      duplicating their content in Rust string literals.
- [ ] Prove aliased imports, local shadowing, and re-exports through resolved
      Tokimu symbol identity.
- [ ] Compare the lowered semantic plans with equivalent hand-authored Rust
      plans for movement intent and one interaction request.
- [ ] Retain source hashes, semantic-model version, execution mode, reasons,
      diagnostics, and source locations.

### Acceptance Criteria

- [ ] A `lowered` rule either lowers fully or fails with a source-located
      reason; it never becomes runtime implicitly.
- [ ] A coincidental local `rule` function is not recognized as Tokimu
      authoring.
- [ ] Browser bindings or type-check success are not counted as lowering
      parity evidence.

## Slice 4: Lowered Doom Game-Logic Pressure

### Deliverables

- [ ] Author bounded movement-intent, use/activation, pickup, damage, and
      door-state transition cases where the existing semantic model can
      represent them honestly.
- [ ] Keep collision resolution, durable health/inventory, sector movement,
      and world mutation in Tokimu-owned state and systems.
- [ ] Let authored rules query admitted observations and emit commands/signals
      rather than touching engine objects.
- [ ] Compare native and WASM semantic plans and resulting deterministic traces.
- [ ] Record every desired Doom behavior the current rule model cannot express.

### Acceptance Criteria

- [ ] Lowered authored logic produces the same retained plan and trace as its
      Rust baseline within the declared deterministic scope.
- [ ] Unsupported constructs grow an explicit semantic-model finding rather
      than falling back silently.
- [ ] No TypeScript closure or module variable owns durable game state.

## Slice 5: `auto` Execution Manifest

### Deliverables

- [ ] Define a corpus-local execution-manifest schema containing unit identity,
      source hash, semantic-model version, declared mode, resolved mode, and
      reason.
- [ ] Commit the manifest used by the Doom authored examples.
- [ ] Report new lowering opportunities without changing a retained resolution.
- [ ] Reject release-mode source or mode drift until explicitly accepted.
- [ ] Exercise compiler/frontend upgrade, source edit, manifest acceptance, and
      rollback cases.

### Acceptance Criteria

- [ ] The same committed source and manifest resolve identically on native and
      WASM builds.
- [ ] `auto` cannot change execution strategy merely because the frontend gains
      a new lowering capability.
- [ ] Manifest failure is diagnostic evidence, not a runtime fallback.

## Slice 6: Runtime TypeScript Capability Experiment

This slice does not admit runtime TypeScript. It tests the smallest external or
feature-gated provider boundary described by the TTSDD and AR-0020.

### Deliverables

- [ ] Begin runtime units with no authority and inject only named capabilities.
- [ ] Test read-only world queries, bounded command emission, signal emission,
      UI requests, and engine-injected time separately.
- [ ] Keep filesystem, network, DOM, arbitrary WASM exports, raw engine objects,
      ambient timers, and unseeded randomness absent unless the experiment
      grants and records a specific capability.
- [ ] Exercise load, initialize, invoke, suspend, reload, dispose, revocation,
      timeout, panic/exception, and invalid-command behavior.
- [ ] Record every capability read and emitted request as an observation.
- [ ] Retain the runtime host's authority delta across initialization,
      invocation, revocation, reload, exception, and disposal.
- [ ] Compare native and browser-host feasibility without choosing an embedded
      JS engine prematurely.

### Acceptance Criteria

- [ ] Runtime logic cannot mutate durable world state except through validated
      Tokimu commands.
- [ ] Reload loses ephemeral script state while Tokimu-owned durable state
      remains inspectable and unchanged.
- [ ] Capability revocation and runtime failure cannot leave partial mutation
      or falsely report success.
- [ ] The result is still labelled experimental and cannot be enabled by
      default in an engine crate.

## Slice 7: Durable-State And Authority Negative Tests

### Deliverables

- [ ] Attempt to retain health, inventory, keys, door progress, monster state,
      map progression, and replay-relevant values in TypeScript-local state.
- [ ] Demonstrate which lifecycle events lose or fork that state.
- [ ] Attempt direct world mutation, schedule changes, raw resource access,
      oversized commands, stale handles, replay divergence, and unbounded loops.
- [ ] Require explicit rejection or containment for every forbidden path.
- [ ] Reconcile attempted authority with each corresponding slice's retained
      authority delta, including authority surviving disposal.
- [ ] Retain at least one tempting implementation that was rejected because it
      made TypeScript an alternate semantic owner.

### Acceptance Criteria

- [ ] Every durable-state attempt either lowers into Tokimu-owned state or is
      rejected with a specific reason.
- [ ] No forbidden request reaches partial world mutation.
- [ ] Timeouts, exceptions, and reloads have deterministic containment evidence.

## Slice 8: TypeScript WAD Provider Comparison

This optional comparison is an external-provider experiment, not TTSDD
semantic authoring and not the canonical WAD implementation.

### Deliverables

- [ ] Implement only the bounded WAD container observation contract already
      proven by the WAD checklist.
- [ ] Use the same Tokimu-authored synthetic fixtures, malformed fixtures,
      limits, expected observations, and source identities as the Rust provider.
- [ ] Keep parsed TypeScript objects outside Resource Store and trusted-core
      public APIs.
- [ ] Measure copies, allocations/browser memory, parse time, diagnostics,
      bundle size, startup cost, and recovery behavior.
- [ ] Compare implementation complexity, auditability, supply-chain closure,
      native reuse, and WASM portability with the Rust provider.

### Acceptance Criteria

- [ ] The provider matches the canonical observation contract for the admitted
      fixture set or records every divergence.
- [ ] The comparison cannot change Resource Space or WAD semantics to make the
      TypeScript result easier.
- [ ] The report concludes retain, reject, or continue incubation and names the
      ownership and deployment cost.

## Slice 9: Presentation, Input, And Audio Mechanisms

### Deliverables

- [ ] Let TypeScript own DOM/Canvas layout, focus, resize, pointer lock,
      accessibility, file gestures, and presentation controls.
- [ ] Normalize browser keyboard, mouse, and gamepad mechanisms into
      `tokimu-input` requests.
- [ ] Present HUD and diagnostic observations without integrating health,
      movement, timing, or animation locally.
- [ ] Route bounded audio preferences and event acknowledgements without
      deciding Doom game-event meaning or sequencing clocks.
- [ ] Exercise JavaScript-disabled/static fallback for website evidence.

### Acceptance Criteria

- [ ] TypeScript presentation can be replaced without changing world traces.
- [ ] Input normalization produces the same semantic requests as the native
      control for the retained sequence.
- [ ] Presentation frame rate, tab suspension, or DOM lifecycle cannot advance
      simulation truth.

## Slice 10: Native/WASM Boundary And Cost Report

### Deliverables

- [ ] Run equivalent authored-source, asset-intake, observation, command,
      lifecycle, and recovery cases on named native and browser/WASM targets.
- [ ] Record boundary calls, copies, serialized bytes, retained bytes,
      allocations, startup time, frame/update work, binary/bundle size, and
      compile/build time.
- [ ] Separate TypeScript, generated binding, WASM, provider, semantic, and
      renderer costs.
- [ ] Record differences in error timing, numeric behavior, lifecycle, and
      available authority.

### Acceptance Criteria

- [ ] Every performance claim names its workload, target, profile, toolchain,
      host, repetition, and limits.
- [ ] Missing browser, renderer, or runtime-host evidence remains explicit.
- [ ] No candidate wins by omitting work performed by another boundary.

## Slice 11: AR-0020 Findings And ADR Gate

### Deliverables

- [ ] Update AR-0020 with the complete package/unit inventory and observed
      contradictions.
- [ ] Classify each experiment as retain, constrain, reject, promote, or
      continue incubation.
- [ ] Extract only rules that repeatedly changed decisions and can be checked
      mechanically.
- [ ] Draft an ADR if the evidence establishes a stable corpus declaration,
      authoring-package admission rule, execution-manifest release gate,
      runtime-host exception process, or other binding boundary.
- [ ] Update the TTSDD when implementation evidence corrects a draft claim;
      do not rewrite it merely to make a nonconforming experiment look valid.

### Acceptance Criteria

- [ ] Successful browser behavior is not substituted for semantic-authoring
      evidence.
- [ ] The final report identifies what TypeScript can safely own, what it may
      only request, and what Tokimu must own.
- [ ] Any ADR follows retained evidence and names enforcement, migration,
      exception, and reopening behavior.

## Validation

At minimum, each applicable slice should retain:

- strict TypeScript compilation for the frontend workspace and workbench;
- Rust unit/integration tests for semantic requests and observations;
- exact authored-source lowering tests;
- native and `wasm32-unknown-unknown` builds;
- browser automation for asset intake, lifecycle, and presentation mechanisms;
- deterministic native/WASM trace comparison where the behavior claims parity;
- malformed, excessive, stale, interrupted, revoked, and exception paths;
- authority-delta artifacts for slices 1 through 9, backed by retained positive,
  denial, revocation, and disposal evidence where applicable;
- `cargo fmt --all --check` and workspace validation appropriate to touched
  crates.

## Fixture, Security, And Publication Boundary

- Synthetic Tokimu-authored WAD fixtures are the default CI inputs.
- Reviewed Doom/Heretic packages remain governed by the WAD checklist and may
  not enter website deployment merely because this workbench supports browser
  loading.
- User-supplied bytes remain within the browser session unless explicitly
  exported.
- TypeScript and runtime-provider dependencies must be pinned and audited in
  proportion to their execution authority and supply-chain closure.
- Untrusted input and runtime exceptions must cross structured failure
  boundaries with explicit limits and recovery evidence.

## Completion Criteria

This plan is complete when:

- the WAD-plan baseline and TypeScript experiments remain independently
  attributable;
- every TypeScript unit has an AR-0020 classification and authority record;
- every experiment in slices 1 through 9 has a complete authority delta, with
  surviving authority explicitly reconciled or recorded as none with evidence;
- exact authored-source lowering and `auto` manifest behavior have retained
  native/WASM evidence;
- browser asset loading into Resource Store is bounded and recoverable;
- runtime-host and durable-state experiments expose clear accepted and rejected
  authority;
- the optional TypeScript provider comparison has an explicit disposition;
- performance and migration costs are retained without universalizing one
  workload; and
- AR-0020 either yields a binding ADR or records why continued incubation is
  more honest.

## Parking Criteria

Park the experiment when further work requires admitting a runtime host,
creating a stable `@tokimu/*` package, changing Resource Store semantics, or
altering the canonical WAD provider boundary without sufficient AR-0020
evidence. Preserve the smallest reproductions and rejected authority cases so
the eventual ADR is informed by actual pressure rather than recollection.
