# AR-0020: TypeScript Authoring Boundary And Corpus Conformance

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-08 |
| Last reviewed | 2026-08-08 |
| Scope | TypeScript authoring frontend / corpus classification / enforcement |
| Trigger | The corpus contains several kinds of TypeScript, while TTSDD conformance and the distinction between authoring semantics and browser/presentation mechanisms are not mechanically enforced |
| Related design | `docs/Tokimu TypeScript Design Document.md`; SDD sections 5.11 and 5.12 |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005, ADR-0008, ADR-0009, ADR-0011 |
| Related evidence | `frontends/`; `crates/tokimu-ts-frontend`; TypeScript-bearing corpus consumers |
| Related plan | `docs/Plans/DOOM/DOOM TypeScript Boundary Stress Plan.md` |
| Admission exception | None |

## Architectural Question

How should Tokimu classify and enforce TypeScript-bearing corpus work so that
TypeScript can provide browser mechanisms and experimental pressure without
silently becoming a second semantic runtime, bypassing the TTSDD authoring
pipeline, or creating unsupported local authoring vocabularies?

Does repeated evidence justify a binding ADR for TTSDD conformance, and which
rules should remain design guidance until the missing compiler/runtime
mechanisms exist?

## Trigger And Concern

The TTSDD is the source of truth for Tokimu's TypeScript **authoring** surface.
It does not claim ownership of every TypeScript file in the repository. Current
corpus work also uses TypeScript for DOM lifecycle, input forwarding, local
file selection, Canvas presentation, ASP.NET/browser integration, and website
island activation.

Those mechanisms can be architecturally valid. The risk is that the repository
does not yet require each TypeScript-bearing corpus entry to state which role
its TypeScript occupies. A local helper can therefore drift from presentation
mechanism into semantic authoring, durable state, format parsing, scheduling,
or engine policy without crossing an obvious structural gate.

The concern is not "TypeScript exists outside `frontends/`." It is:

> TypeScript role and authority are not mechanically classified, so a corpus
> experiment can accidentally claim TTSDD authoring semantics without passing
> through the TTSDD package, recognition, lowering, manifest, capability, and
> diagnostic boundaries.

## Current Evidence

### Evidence aligned with the TTSDD

- `frontends/` is a separate npm workspace rather than part of the Rust crate
  graph.
- `frontends/packages/tokimu` is the `tokimu` import anchor and re-exports
  `@tokimu/rules`.
- `frontends/packages/rules` contains authoring API shapes and types rather than
  an alternate engine or runtime.
- `frontends/packages/examples` contains lowered and runtime-intent examples
  that depend on the authoring packages, not the reverse.
- `tokimu-ts-frontend` depends on `tokimu-rule`, not renderer, platform, facade,
  Node, or TypeScript tooling.
- `tokimu-core` and `tokimu-runtime` do not depend on the frontend workspace or
  JavaScript tooling.
- Several browser corpus designs explicitly state that TypeScript owns DOM,
  Canvas, file-selection, activation, or presentation mechanisms while Rust
  owns simulation and semantic truth.

### Missing or transitional conformance evidence

- `tokimu-ts-frontend` still recognizes constrained source using a hand-rolled
  text recognizer. It does not implement the TTSDD requirement that recognized
  calls be identified through TypeScript-resolved exported symbol identity.
- The repository has no committed execution manifest implementing stable
  `auto` resolution, source hashes, semantic-model versions, or accepted mode
  migration.
- The authored files in `frontends/packages/examples` type-check, but no
  retained end-to-end test currently feeds those exact files through the Rust
  frontend and proves parity with the engine-owned semantic plan.
- Runtime TypeScript remains a designed but unadmitted execution path. No
  feature-gated host currently proves the TTSDD capability allowlist,
  lifecycle, durable-state boundary, source-map diagnostics, or native/WASM
  posture.
- Corpus-local TypeScript adapters, including presentation-authoring
  precursors, can describe bounded intent but have no common classification or
  conformance declaration.

These are gaps between draft design and retained implementation evidence. They
are not permission to weaken the TTSDD claims or pretend the missing mechanisms
already exist.

## Corpus TypeScript Classification

Every TypeScript-bearing corpus entry should be classifiable into exactly one
primary role, with secondary roles named where necessary:

| Class | Allowed ownership | Required boundary | TTSDD relationship |
| --- | --- | --- | --- |
| TTSDD semantic authoring | Author intent expressed through `tokimu` / `@tokimu/*` packages | Recognized symbol identity, validation/lowering or admitted runtime host, explicit execution result | Directly governed by TTSDD |
| Browser/presentation mechanism | DOM, Canvas, user gestures, focus, local file selection, presentation state, transport into Rust/WASM | May forward bounded intent and render observations; must not own simulation truth or recreate Tokimu semantics | Compatible with TTSDD but not itself semantic authoring |
| Corpus-local precursor | Experimental typed adapter used to pressure a proposed authoring domain | Must say which future package/semantic model it pressures, remain local, and avoid claiming stable TTSDD support | Incubating evidence only |
| External consumer/provider integration | Application- or provider-owned TypeScript outside Tokimu authoring packages | Explicit adapter and authority boundary; no reverse dependency into engine semantics | Outside TTSDD authoring surface unless it imports admitted packages |
| Generated binding | Tool-generated JS/TS glue around WASM or another transport | Regenerated from the owning boundary; no hand-authored semantic policy | Mechanism only |

A file's language or directory does not decide its class. Its claimed authority,
imports, outputs, and ownership of durable behavior do.

### Inventory Evidence Schema

The classification inventory must record both the package's declared role and
the semantic effect that survives its boundary. Imports alone are insufficient:
a presentation package can import only browser mechanisms while quietly
producing durable or authoritative application state.

| Field | Question answered |
| --- | --- |
| Package / entry | What independently buildable or reviewable unit is being classified? |
| Primary role | Which class above best describes its intended responsibility? |
| Reads | Which DOM, authored-source, engine-observation, provider, filesystem, network, clock, or other inputs can it observe? |
| Emits | Does it produce bounded requests, presentation state, semantic plans, provider calls, generated bindings, or direct mutations? |
| Durable state | What survives invocation, reload, scene transition, save/replay, or application restart, and who owns it? |
| Semantic authority | Does the TypeScript decide Tokimu meaning, merely express admitted intent, propose experimental meaning, or have no semantic authority? |
| Execution authority | Can it execute only browser mechanisms, request Rust/WASM work, lower into Tokimu semantics, or execute in a hosted runtime? |
| Authority delta | What authority was requested, granted, actually exercised, denied, and retained after disposal? |
| Boundary evidence | Which source, test, diagnostic, manifest, or design statement proves the classification? |

The inventory should make the healthy mechanism path visibly different from
semantic ownership:

```text
TypeScript DOM/input mechanism
    -> bounded request
    -> Rust/Tokimu semantic owner

TypeScript semantic authoring
    -> admitted authoring package
    -> validated/lowered semantic plan
    -> Tokimu-owned execution meaning
```

If the output cannot be described without saying that TypeScript decides what
the world means, the package is not merely a browser/presentation mechanism.

## Invariants Under Review

The following are candidate enforcement rules:

1. TypeScript may claim Tokimu semantic authoring only through an admitted
   `tokimu` / `@tokimu/*` authoring package and an engine-owned semantic target.
2. Browser/presentation TypeScript may observe and submit bounded requests, but
   may not own simulation truth, source-format interpretation, schedule policy,
   deterministic progression, or durable world state.
3. Corpus-local precursors must identify their proposed domain package and
   semantic target, remain explicitly experimental, and define a retirement or
   promotion trigger.
4. A `lowered` unit may never silently become runtime. An `auto` unit may not
   drift execution mode in a release without an accepted manifest change.
5. Runtime-hosted TypeScript is not admitted until a separate provider proves a
   capability-based authority model, lifecycle, recovery, diagnostics, and
   target policy.
6. Generated bindings and browser transports are not evidence that the TTSDD
   authoring pipeline works.
7. New TypeScript semantic domains require concrete corpus pressure and must
   not accumulate in a monolithic `tokimu` compiler or miscellaneous local
   helper package.

## Enforcement Candidates

### A. Documentation-only classification

Require TypeScript corpus design documents to state the classification and
ownership boundary, but add no mechanical validation.

This is low cost and immediately useful. It remains vulnerable to drift and
cannot validate symbol identity, execution mode, or semantic parity.

### B. Classification plus retained conformance checks

Add a small manifest or machine-readable declaration for TypeScript-bearing
corpus packages, then validate:

- declared role;
- imports of `tokimu` / `@tokimu/*`;
- whether the package claims semantic authoring;
- forbidden engine-semantic ownership in presentation-only packages where it
  can be detected structurally;
- strict TypeScript compilation;
- exact authored-source lowering parity for admitted TTSDD examples;
- presence and drift of the future execution manifest.

This is the current preferred enforcement direction. It can begin with an
inventory/report mode before becoming a required CI gate.

### C. Immediate binding ADR and CI rejection

Accept all candidate invariants now and reject unclassified TypeScript corpus
packages immediately.

This would create a crisp boundary, but the classification format,
symbol-identity frontend, execution manifest, and runtime host do not yet have
enough implementation evidence. Immediate binding policy risks encoding the
hand-rolled transitional frontend as permanent architecture or forcing browser
mechanisms into an authoring model they do not belong to.

### D. Treat all corpus TypeScript as unconstrained application code

Allow each corpus entry to define its own TypeScript boundary.

Rejected as a direction. Corpus code may be more permissive than Native Ring
code, but it exists to produce architectural evidence. Unclassified semantic
ownership would make that evidence unreliable and allow application-local
vocabulary to become de facto Tokimu authoring architecture.

## Findings

1. Current browser-shell TypeScript is not inherently a TTSDD violation. Most
   reviewed designs explicitly retain simulation and semantic truth in Rust.
2. The TTSDD architecture has a real initial implementation under `frontends/`
   and `tokimu-ts-frontend`; it is not purely aspirational.
3. The strongest current violation risk is implicit role classification, not a
   known reverse dependency into `tokimu-core` or `tokimu-runtime`.
4. Symbol-identity recognition and the execution manifest are the largest gaps
   between TTSDD guarantees and actual enforcement.
5. A browser shell type-checking or successfully calling WASM is not evidence
   that TypeScript semantic authoring lowers correctly.
6. A corpus-local presentation-authoring adapter can be valuable pressure, but
   must not be described as stable `@tokimu/*` authoring until it has an
   admitted semantic target and conformance path.
7. Binding ADR language should follow at least one repository-wide
   classification pass and one end-to-end authored-source parity fixture.
8. The recurring boundary principle is that mechanism does not imply
   ownership. TypeScript use, like foreign implementation use, must be reviewed
   according to the authority and durable outputs it carries rather than its
   language, package name, or directory.
9. A declared capability list is incomplete evidence without its observed
   authority delta. Requested, granted, exercised, denied, and post-disposal
   authority expose over-broad grants, untested claims, failed revocation, and
   hidden durable ownership that a static inventory can miss.

## Disposition

**Under Review.** Adopt classification plus retained conformance checks as the
working direction. Do not declare the TTSDD fully enforced and do not admit a
runtime TypeScript host.

Open a binding ADR when the first inventory and parity fixture show that the
candidate invariants are both necessary and mechanically expressible. An ADR
may then define the minimum corpus declaration, authoring-package admission
rule, release execution-manifest gate, and runtime-host exception process.

## Required Follow-Up

- [ ] Inventory every TypeScript-bearing corpus package and assign one primary
      classification, reads, emitted outputs, durable-state owner, semantic and
      execution authority, and retained boundary evidence.
- [ ] Retain a common authority-delta artifact for each DOOM TypeScript boundary
      experiment, including requested, granted, exercised, denied, and
      post-disposal authority.
- [ ] Identify packages whose implementation contradicts their design claim,
      not merely packages that contain TypeScript.
- [ ] Feed the exact `frontends/packages/examples` authored sources through the
      Rust frontend and retain semantic-plan parity evidence.
- [ ] Replace text-based primitive recognition with TypeScript-resolved symbol
      identity before claiming TTSDD section 3.2 conformance.
- [ ] Design and retain the execution-manifest schema and `auto` drift tests.
- [ ] Add a report-only TTSDD corpus audit before considering a required CI
      gate.
- [ ] Review corpus-local authoring precursors and record promotion, continued
      incubation, or retirement criteria.
- [ ] Create an ADR only after the inventory and conformance fixture establish
      a stable enforceable rule.

## Reopening And Escalation Triggers

- TypeScript begins to parse a source format or recreate Tokimu semantic
  decisions to complete a corpus consumer.
- A browser/presentation package starts owning durable simulation state,
  deterministic progression, or schedule policy.
- A local adapter is published or described as a stable Tokimu authoring API.
- An `auto` unit changes resolved execution mode without an accepted manifest
  update.
- A runtime host, JS engine, Node dependency, or TypeScript compiler dependency
  is proposed inside an engine execution crate.
- A new `@tokimu/*` domain package is proposed without a concrete corpus and an
  engine-owned semantic target.
- Repeated audit findings show that documentation-only classification does not
  prevent drift.

## Review History

### Cycle 1 -- 2026-08-08

- Status entering review: Proposed.
- Evidence reviewed: TTSDD v0.2.0; the `frontends/` workspace; the current
  hand-rolled Rust frontend; TypeScript-bearing browser, website, ASP.NET,
  presentation-authoring, runtime-observation, asset, and external-consumer
  corpus designs.
- Findings: the principal enforcement gap is implicit role classification.
  Current authoring packages preserve the intended dependency direction, while
  symbol-identity recognition, execution manifests, exact authored-source
  parity, and runtime-host enforcement remain incomplete.
- Disposition: move to Under Review and begin a repository-wide classification
  inventory. Defer a binding ADR until evidence produces a stable mechanical
  rule.
- Resulting ADR or documentation change: none; AR-0020 records the question and
  enforcement study.

### Cycle 2 -- 2026-08-08

- Status entering review: Under Review.
- New evidence: external review agreed that language and directory are
  insufficient classifications and identified semantic outputs as a missing
  inventory dimension. Imports can look harmless while durable or
  authoritative state crosses the boundary.
- Findings: the inventory must record reads, emitted outputs, durable-state
  ownership, semantic authority, execution authority, and the evidence for
  each classification. This makes browser mechanism, admitted semantic
  authoring, and experimental precursors distinguishable by observable
  boundary effects.
- Disposition: retain Under Review. Proceed with the enriched inventory before
  drafting an ADR; keep `auto` manifest stability and runtime-host capability
  authority as likely future binding rules requiring implementation evidence.
- Resulting ADR or documentation change: expanded the classification inventory
  schema; no ADR created.

### Cycle 3 -- 2026-08-08

- Status entering review: Under Review.
- New evidence: external review of the separate DOOM TypeScript Boundary Stress
  Plan supported its aggressive placement experiments, its separation from the
  canonical WAD plan, and its use of exact authored source, execution manifests,
  runtime-host denial cases, and a TypeScript provider comparison.
- Findings: every experiment needs a comparable authority delta: requested,
  granted, actually exercised, denied, and surviving-after-disposal authority.
  This distinguishes a narrow successful use from an over-broad grant and makes
  revocation, lifecycle containment, and hidden durable state reviewable across
  otherwise different TypeScript roles.
- Disposition: retain Under Review. Require the authority-delta artifact across
  slices 1 through 9 and use the collected results in the final ADR gate.
- Resulting ADR or documentation change: added the shared authority-delta
  evidence contract to the plan and AR inventory schema; no ADR created.

## References

- `docs/Tokimu TypeScript Design Document.md`
- `docs/Tokimu Software Design Document.md`
- `frontends/README.md`
- `frontends/packages/tokimu/`
- `frontends/packages/rules/`
- `frontends/packages/examples/`
- `crates/tokimu-ts-frontend/`
- `crates/tokimu-rule/`
- `corpus/focused/simulation/hello-fps-web/`
- `corpus/consumers/aspnet-wasm-presentation-workbench/`
- `corpus/consumers/runtime-observation-workbench/`
- `corpus/consumers/tokimu-website-asteroids/`
- `corpus/consumers/tokimu-website-paint/`
- `corpus/consumers/weaver-xslt-resource-space/`
