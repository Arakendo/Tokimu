# Tokimu Semantic Kernel Map

| Field      | Value                                                                                            |
| ---------- | ------------------------------------------------------------------------------------------------ |
| Status     | Draft — exploratory, grow-then-fold                                                               |
| Title      | "Semantic Kernel Map" (kept) — covers primitives, distinctions, clusters, layers, and promotion  |
| Scope      | A discipline for deciding what is a kernel-native concept vs a capability-owned one, and for keeping core vocabulary from drifting. A lens over existing architecture, not a new subsystem. |
| Relates to | ADR-0001 Engine Boundaries, ADR-0003 Capability Ownership Boundary, `docs/kernel-principles.md`   |
| Relates to | `docs/capability-backends.md`, `docs/diagnostics-model.md`, SDD ownership model                   |
| Source     | Adapted from the Tonesu language design method (`docs/Conversations/Tonesu/`)                      |

## 1. Purpose

`kernel-principles.md` argues for a small trusted core. This document supplies
the *filter* that keeps it small: a method for deciding whether a concept is a
kernel primitive, a capability-owned concept, or a backend implementation detail,
plus the anti-drift machinery that keeps `Command`, `Signal`, `Event`,
`Relation`, `Resource`, `Capability`, `Asset`, and `Handle` from slowly melting
into one another like unattended cheese.

The method is borrowed from the Tonesu language project, which faced the same
problem in a different medium: a language needs a small, irreducible set of
semantic roots, explicit boundaries between them, and a strict admission process
so useful-but-derivable ideas do not get promoted to primitives. Tonesu's own
principle applies almost verbatim to Tokimu:

> Stability before extensibility. Fewer, better-defined primitives are always
> preferable to more primitives added prematurely.

The core lesson for Tokimu: *useful is cheap; irreducible is expensive.* Plenty
of things are repeatedly needed — chairs, taxes, and JSON come to mind — without
being ontological atoms. A concept earns kernel status only when repeated
composition attempts fail across multiple unrelated domains.

This is a side document and a lens. It does not add subsystems; it records the
discipline the trusted core should hold to, and can fold into the SDD (or an ADR)
once a real slice exercises it.

## 2. The Primitive Admission Test

Tonesu admits a root only if it is cognitively atomic, combines cleanly, and
carries no hidden metaphor — and only after corpus pressure proves existing
roots cannot express it. The engine translation:

A concept may become a **kernel primitive** only when all of these hold:

1. **Architecturally atomic.** It cannot be defined cleanly using existing
   kernel primitives plus a capability-owned semantic model.
2. **Composes across unrelated domains.** It participates cleanly in many
   constructions spanning domains that have nothing to do with each other.
3. **No hidden metaphor.** Its literal meaning is explicit; it does not quietly
   redefine a neighbor because an API was convenient on Tuesday.
4. **Not merely an implementation convenience.** It expresses engine meaning,
   not a handy struct.
5. **Bounded precisely.** It has strict Includes/Excludes so it cannot absorb its
   neighbors.
6. **Migration cost acknowledged.** Kernel additions are ecosystem-wide changes,
   like a breaking change to a language's root set.

Concrete procedure before adding a native concept:

- attempt at least three decompositions using existing kernel primitives;
- test it in at least three unrelated application domains (e.g. games, CAD,
  technical simulation, databases, authoring tools);
- show that capability-level modeling produces repeated ambiguity or broken
  semantics, not just mild inconvenience;
- write its Includes/Excludes before writing any code.

This mirrors how Tonesu admitted `zi` (mutual transformation): not because
"interaction seems useful," but because static relation, directed causation, and
one-sided change all *failed* across physics, biology, and social domains.
Tokimu should demand equivalent evidence.

## 3. Candidate Kernel Primitive Set

These are the concepts that plausibly pass the test — the vocabulary the trusted
core owns. Treat this as a candidate set under review, not a sacred list. Since
`useful is cheap; irreducible is expensive`, the natural human move is to
immediately produce seventeen candidates; the markers below exist to resist that.

Distinguish three kinds so the list is not read as flat, settled ontology:

- **primitive concept** — irreducible; passes the admission test
- **typed realization** — a domain-typed form of a primitive, not its own root
- **derived compound** — a construction of other primitives; a kernel type, not
  a primitive

```
Cluster       Primitive concept          Typed realizations / derived
Identity      Id, Handle, Entity?        EntityId, AssetId, ProviderId,
                                         AssetHandle, MeshHandle, MaterialHandle
State         Resource, Relation
              Component?  — likely derived: typed state + entity association
Process       Rule, Command, Event, Signal, Time
              Schedule?  — likely derived: Rules + Time + ordering policy
Authority     Capability                 Grant?, Scope?, Revocation? — likely
                                         derived: Capability + Handle + scope teardown
Presentation  View                       Camera
Inspection    Diagnostic                 Provenance?, Trace? — likely derived:
                                         Diagnostic + causation metadata / ordering
```

A `?` marks concepts that are candidate primitives *or* derived kernel
constructions; implementation pressure decides. `AssetId`/`ProviderId` are typed
Ids, not separate roots; `MeshHandle`/`MaterialHandle` are typed `Handle`s.
`Entity` may be a primitive (an identity-bearing locus for components and
relations) or reduce to `Id` + world membership + component association;
`Component` and `Schedule` are exactly the sort of cases the admission test
should challenge rather than pre-admit.

Ledger entry format (adapted from Tonesu's Root/Gloss/Includes/Excludes/Notes),
recommended for every drift-prone term, with an explicit status:

```
Primitive:     Capability
Status:        accepted candidate — awaiting implementation proof
Meaning:       scoped authority to invoke a bounded service surface
Includes:      grants, scope, revocation, allowed operations, validation
Excludes:      provider implementation, global service lookup, raw backend handle
Rationale:     required across scripting, persistence, geometry, networking
Typed forms:   CapabilityHandle, CapabilityRequest, CapabilityGrant
```

Ledger status values: `candidate`, `accepted`, `derived`, `rejected`,
`deprecated`. This keeps readers from mistaking the candidate list for settled
ontology.

Illustrative entries under review (the boundaries matter more than the verdicts):

```
Primitive:  Entity
Status:     candidate — test vs "Id + world membership + component association"
Meaning:    identity-bearing world participant
Includes:   component attachment, relation endpoints, lifecycle
Excludes:   components themselves, resources, presentation objects

Primitive:  Component
Status:     candidate/derived — may reduce to typed state + entity association
Meaning:    typed state describing a single entity
Includes:   per-entity data
Excludes:   entity identity, cross-entity links (that is Relation), shared state

Primitive:  Resource
Status:     candidate
Meaning:    world- or runtime-owned state not attached to an entity
Includes:   clocks, configuration, shared simulation state
Excludes:   global singleton, capability grant, backend service, asset identity

Primitive:  View
Status:     candidate
Meaning:    presentation projection of authoritative world state
Includes:   render/text/inspector projections for consumption
Excludes:   ECS query views, camera, UI screen widget, authoritative state
```

`Rule` vs `System` needs implementation proof, not just definitions:

```
Rule (semantic behavior):
  when Ship has Thrusting and Fuel > 0 → emit ApplyAcceleration
System (execution mechanism):
  the Rust executor that evaluates matching entities each fixed tick
```

The terms most worth a ledger entry, because they are most prone to semantic
drift: Entity, Resource, Relation, Command, Signal, Event, Rule, Handle,
Capability, Asset, View, Time, Diagnostic.

## 4. Critical Distinctions

Tonesu's strongest anti-drift tool is its critical-distinctions table (e.g. `ne`
relation vs `pe` part, `go` cause vs `ki` change). Most architectural corruption
begins as vocabulary drift and only later manifests as code that smells like
damp carpet. The engine equivalent:

| Concept A | Concept B        | Required distinction                                                    |
| --------- | ---------------- | ----------------------------------------------------------------------- |
| Command   | Signal           | command requests mutation; signal reports an occurrence                 |
| Event     | Relation         | event is a temporal occurrence; relation is persistent state            |
| Handle    | ID               | handle carries lifecycle validity; ID denotes identity only             |
| Resource  | Capability       | resource is owned state; capability is granted authority                |
| Asset     | Runtime resource | asset is durable/source identity; runtime resource is live realization  |
| Rule      | System           | rule is semantic behavior; system is execution mechanism                |
| View      | World state      | view is presentation/projection; world is authoritative state           |
| Component | Relation         | component describes an entity; relation connects entities               |

The Command / Event / Signal triad is the highest drift risk and deserves a
sharper three-way split, because every queue eventually wants to call its payload
an "event":

```
Command = request       — intentional; may be accepted or rejected; targets mutation
Event   = occurrence    — a fact that happened; historical; not inherently addressed
Signal  = communication — a delivered notification of an event, state, or attention
```

Implementation can then test whether `Signal` is merely the delivery form of an
`Event` or deserves separate semantics — rather than blurring them because naming
things is where civilization briefly loses consciousness.

The discipline: do not let these drift into one another. A `Signal` is not a
queued `Command`; a `Resource` is not a global singleton; a `System` is not "any
callback." When two of these blur, fix the vocabulary before the code.

## 5. Cluster Map

Tonesu groups roots into ontological clusters so it can see when a concept
crosses clusters illegitimately. Tokimu's equivalent:

```
Identity      EntityId  AssetId  ProviderId  Handle
State         Component  Resource  Relation
Process       Rule  Schedule  Command  Event  Signal
Authority     Capability  Grant  Scope  Revocation
Time          FixedTime  FrameTime  SimulationTime
Presentation  View  Camera  MeshHandle  MaterialHandle
Inspection    Diagnostic  Provenance  Trace
```

The value is not tidy organization; it is exposing illegitimate crossings. For
example `MeshHandle` lives in Identity/Presentation and must not quietly become
canonical geometry state. A `CollisionEvent` lives in Process and must not become
a permanent Relation unless explicitly materialized as one.

## 6. Cross-Cluster Compounds

Legitimate compounds combine clusters on purpose, and are how domain meaning is
built without new primitives:

```
Capability + Handle    → a revocable authority reference
Command + Provenance   → an attributable mutation request
Asset + Handle         → a live reference to a durable identity
Relation + Time        → a temporal relationship / lease / lifetime binding
View + Query           → an inspected projection of world state
```

If a needed concept is expressible as a clean cross-cluster compound, it is
probably *not* a new kernel primitive — it is a construction. That is the whole
point.

## 7. Model / Reality Layers

Tonesu's sharpest idea is the `su` (structure in the world) vs `to` (conceptual
model of structure) split. Tokimu has an almost identical separation that must
stay distinct — not merely different data formats, but different ontological
layers:

```
source representation     the encoded artifact on disk/wire (e.g. a serialized scene file)
    --parse-->
authoring model           what an author declares (e.g. TypeScript source)
    --lower-->
semantic model            Tokimu-owned meaning the authoring model lowers into
    --evaluate-->
evaluated world state     authoritative runtime state
    --project-->
presentation projection   render/text/inspector projection of world state
```

A file is an encoded representation of meaning, not the meaning itself; that
distinction matters for migrations, partial loading, authoring metadata, and
round-tripping.

The invariants that follow:

- a serialized scene file is not the authoring model, and neither is the world;
- a TypeScript declaration is not the world;
- a renderer mesh is not the CAD model;
- a diagnostic trace is not the command itself.

This reinforces the world-first ownership already in the SDD and the lowering
pipeline in the TTSDD.

## 8. Precision Asymmetry

Tonesu invests precision where its identity depends on it (epistemic state,
agency, causation) and stays deliberately coarse elsewhere. Tokimu should make
the same conscious choice, giving it a recognizable architectural culture:

```
Precise about        Coarser about (until pressure demands more)
- who owns state     - domain-specific geometry
- who may change it  - physics details
- when change occurs - storage engines
- why it occurred    - importer internals
- what is authoritative
- what is derived
```

Maximum precision everywhere is not the goal; a coherent core with clear
strengths and honest limits is. This is Tonesu's Principle 8 applied to an
engine: it is allowed to have weak spots, and a change that blurs a core strength
to slightly improve a weak area should be rejected.

## 9. Tiered Primitive Sets

One caution from the source method: do not transfer the conlang idea too
literally. A language needs a near-closed primitive vocabulary because every
speaker builds meaning from it. A software kernel can allow **capability-local
primitive sets**, so Tokimu does not need one sacred list containing every
semantic atom in the ecosystem.

```
Kernel primitive set        — universal, very hard to change (this document)
Capability primitive sets   — domain-local, Tokimu-owned, versioned independently
                              (e.g. tokimu-geometry: Profile, Solid, Wire, Face)
Backend implementation vocab — private and replaceable (OCCT TopoDS_Shape, …)
```

This maps exactly onto ADR-0003's three tiers. `Profile` and `Solid` compose
well *inside* geometry but not across Tokimu generally, so they are
`tokimu-geometry` primitives, not kernel primitives. The admission test is run
per tier: a capability crate may admit its own primitives under the same
discipline, without expanding the kernel.

Provisional escalation rule: each capability owns its own primitive ledger and
admission test, but kernel promotion requires evidence that the concept cannot
remain capability-local across at least three unrelated capabilities. That gives
a clean escalation path with a deliberately hostile review board:

```
backend detail
  → capability-local primitive
  → cross-capability construction
  → kernel primitive (only under repeated failure to stay lower)
```

Keep this evidence-oriented, not bureaucratic. Some genuinely kernel-level
concepts surface from one domain first yet clearly affect everything; the rule
must not tell a fundamental concept "ontology denied, please invent two more
subsystems." A future `AuthorityScope`, for instance, might prove foundational
before three capability crates exist. The bar is convincing cross-cutting
evidence, not a mandatory headcount of capabilities.

## 10. Worked Verdicts

Applying the test, as the source conversation did:

- **`Handle` → kernel.** Cannot be cleanly derived; stable identity, revocation,
  lifecycle validation, and stale-reference protection are cross-cutting concerns
  under persistent pressure from assets, geometry, scripting, databases, physics,
  rendering, and sessions.
- **`Capability` → kernel.** Not merely a relation or handle; it encodes granted
  authority, scope, revocation, and allowed operations, under pressure from
  script hosts, geometry, persistence, networking, editor tooling, and platform.
- **`PhysicsBody` → capability (`tokimu-physics`).** Decomposes cleanly into
  entity + transform + velocity + shape data + mass/inertia + physics capability.
  Needed in games and simulations, rarely in CAD, never in databases. Not
  universal.
- **`Profile` → capability (`tokimu-geometry`).** Decomposes into ordered
  geometric elements + plane + closure constraint + geometry semantics. Not
  broadly useful outside geometry.

The recurring shape: "important" repeatedly tries to bully its way into
"primitive." The test is what stops it.

## 11. Relationship to Existing Docs

- **ADR-0003** fixes the native/capability/backend ownership tiers; this document
  supplies the admission test that decides which tier a concept lands in.
- **`kernel-principles.md`** names the trusted-core discipline; this document is
  the vocabulary filter that keeps that core small, and shares its "handles, not
  raw ownership" and "revocable scoped capabilities" invariants.
- **`capability-backends.md`** defines the provider/registry mechanism; the
  tiered primitive sets here explain why `Profile`/`Solid` live in a capability
  crate rather than core.
- The SDD remains the authority. Candidates here graduate into it (or an ADR)
  once a concrete slice exercises them.

## 12. Open Questions

- Whether Tokimu should maintain a living primitive ledger file (Section 3
  format) for the drift-prone terms, and where it lives relative to the SDD.
- Whether the critical-distinctions table (Section 4) should be promoted into the
  SDD as normative once the M2/M8/M9 vocabulary stabilizes.
- Whether `Signal` is merely the delivery form of an `Event` (Section 4) or
  deserves separate semantics — the highest live drift risk.
- Which Authority/Inspection candidates (`Grant`, `Scope`, `Revocation`,
  `Provenance`, `Trace`) settle as primitives vs derived kernel constructions
  once revocation and provenance have real implementations.
- Whether a future coupled-operation concept (constraint solve, collision
  resolution, transaction commit, bidirectional binding, synchronization
  barrier) ever earns a kernel primitive — the Tonesu `zi` situation — or stays a
  capability-level construction.
- Confirming the Section 9 escalation rule in practice: does a real
  cross-capability concept survive to kernel promotion, or stay capability-local?

> Note: the source Tonesu docs under `docs/Conversations/Tonesu/` are an archived
> reference for method, not Tokimu-normative. They contain their own internal
> open questions (e.g. `ki` scope, `to` subdivision); those belong to that
> project and are intentionally not reconciled here.

## References

- ADR-0001 Engine Boundaries — `docs/ADR/ADR-0001-engine-boundaries.md`
- ADR-0003 Capability Ownership Boundary — `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- Kernel Principles — `docs/kernel-principles.md`
- Capability Backends — `docs/capability-backends.md`
- Diagnostics Model — `docs/diagnostics-model.md`
- Tokimu Software Design Document — `docs/Tokimu Software Design Document.md`
- Source method — `docs/Conversations/Tonesu/` (primitives, semantic-map, principles)
