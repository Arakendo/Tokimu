# Tokimu Contribution Admission Guide

| Field      | Value |
| ---------- | ----- |
| Status     | Draft — governance / review guide |
| Title      | "Contribution Admission Guide" (kept) — a checklist for admitting new concepts, crates, capabilities, adapters, and frontends |
| Scope      | A repeatable review process for deciding whether a proposed addition belongs in the kernel, a foundational service, a capability crate, a backend adapter, or an authoring frontend. |
| Relates to | `docs/Tokimu Software Design Document.md`, `docs/kernel-principles.md`, `docs/semantic-kernel-map.md`, `docs/capability-backends.md`, `docs/future-workspace-layout.md`, `docs/Architectural Reviews/README.md`, ADR-0003 |

## 1. Purpose

Tokimu now has enough architectural policy that "should we add this?" should
not be answered ad hoc.

This guide turns the existing architecture documents into a practical admission
checklist. It is not a Rust style guide and not a contributor onboarding guide.
It is a review tool for deciding whether a new concept, crate, API surface,
backend integration, or authoring package belongs in Tokimu at all, and if so,
where.

The goal is to make the next architectural decision more correct, not merely
more consistent.

Durable reviews use an Architectural Review Record under
`docs/Architectural Reviews/`. Review records preserve evidence, findings,
disposition, and reopening triggers. ADRs remain the record of accepted binding
decisions.

## 2. The Review Question

Before asking "where should this go?", ask:

> Who owns this meaning?

That question should determine the rest:

- whether the thing is kernel-native, foundational, capability-owned, adapter-
  owned, or frontend-only;
- which crates may depend on it;
- which existing semantics it may reuse;
- what proof is required before it becomes permanent.

If ownership is unclear, the contribution is not ready.

## 3. Admission Categories

Every proposal should be classified into one of these categories before code is
merged.

### 3.1 Kernel primitive or kernel-native surface

Use this category only for unavoidable engine meaning.

Examples:

- `Command`
- `Signal`
- `Resource`
- time progression
- identity / handle semantics

Kernel additions are expensive because they become part of Tokimu's worldview.

### 3.2 Foundational service

Use this category for first-party services that consume kernel meaning but do
not own simulation truth.

Examples:

- rendering
- platform integration
- normalized input
- asset loading/identity services

### 3.3 Capability crate

Use this category for optional Tokimu-owned domain meaning that should stay
outside the kernel while still having a stable semantic model.

Examples:

- geometry
- persistence
- physics
- scripting host contracts

### 3.4 Backend adapter

Use this category for a concrete external implementation of a capability.

Examples:

- OCCT for geometry
- SQLite for persistence
- QuickJS for runtime-host execution

Backend adapters implement a semantic contract; they do not define it.

### 3.5 Authoring frontend surface

Use this category for TypeScript or other author-facing syntax/tooling layers
that target Tokimu-owned semantics.

Frontend packages do not own runtime truth.

## 4. Mandatory Review Questions

Every architectural contribution should answer these questions explicitly in the
PR description, issue, ADR, or design note.

### 4.1 Ownership

- What meaning is being introduced?
- Who owns that meaning: kernel, foundational service, capability crate,
  backend adapter, or frontend?
- Is this semantic meaning or only an implementation convenience?

### 4.2 Layer

- Which layer does it belong to?
- Why not a lower layer?
- Why not a higher or more optional layer?

### 4.3 Dependency direction

- What new dependencies does it introduce?
- Do those dependencies point downward toward more universal meaning?
- Does any dependency violate the workspace-layout guardrail by depending
  upward into more specialized implementation?

Dependency direction is a semantic statement, not merely a compilation
constraint. Depending on a crate means accepting its worldview.

### 4.4 Existing concept reuse

- Does an existing Tokimu concept already cover this?
- Is this actually a typed realization or derived compound of something that
  already exists?
- Would a rename, narrower contract, or extension of an existing crate remove
  the need for a new one?

### 4.5 Proof requirement

- What example, test, or integration slice proves this addition is needed?
- What concrete ambiguity, failure, or ownership problem does it solve?
- What is the smallest proof that would falsify the proposal if it were wrong?

If the proposal cannot name a proving example or test, it is likely still too
speculative.

## 5. Kernel Primitive Admission

If a proposal adds a new kernel primitive or a new kernel-native semantic term,
it must answer all of the following before implementation:

- What are the term's Includes and Excludes?
- What neighboring concepts must it not blur into?
- Can it be decomposed using existing kernel primitives plus a capability-owned
  model?
- Has that decomposition been attempted in at least three unrelated domains?
- Does repeated use reveal genuine semantic failure rather than mild
  inconvenience?

Required evidence:

1. at least one written ledger-style entry;
2. at least one rejected decomposition attempt;
3. at least one example or test showing why the concept must be kernel-native.

See `docs/semantic-kernel-map.md` for the primitive-admission discipline.

## 6. Foundational Service Admission

If a proposal adds or substantially expands a foundational service crate, ask:

- Is this service broadly required by most Tokimu applications?
- Does it consume kernel meaning without taking ownership of simulation truth?
- Could it be optional domain meaning instead, and therefore belong in a
  capability crate?
- Is it being introduced because the engine needs the service, or because one
  external library is convenient?

Foundational services are first-party and broadly required, but they still do
not define world truth.

## 7. Capability Crate Admission

If a proposal adds a capability crate, it must justify why the domain should be
Tokimu-owned but not kernel-native.

Required questions:

- What semantic model does Tokimu need to own for this domain?
- Why should that model live outside `tokimu-core` and `tokimu-runtime`?
- What competing implementations or platform constraints make a backend seam
  desirable?
- What application slice earns the capability now?

Required evidence:

1. one concrete use case or example;
2. one draft semantic model or provider contract;
3. one statement of what remains backend-specific and must stay out of the
   capability crate.

See `docs/capability-backends.md` and ADR-0003 for the ownership boundary.

## 8. Backend Adapter Admission

If a proposal adds a backend adapter crate, it must prove that the semantic
contract already exists and that one concrete backend is now justified.

Required questions:

- Which capability crate owns the semantics?
- Which provider contract is being implemented?
- What Tokimu-owned types cross the boundary?
- What foreign-library types are explicitly forbidden from crossing it?
- What target/platform constraints affect provider selection?

Required evidence:

1. one capability crate already exists or lands in the same slice;
2. one end-to-end example or test proves the backend choice;
3. diagnostics show why provider selection succeeded or failed;
4. no raw foreign objects leak into engine-owned or author-facing APIs.

## 9. Frontend Surface Admission

If a proposal adds a TypeScript package or expands the authoring surface, ask:

- What existing Tokimu-owned semantic model does it target?
- Is it defining authoring syntax, or is it trying to invent runtime meaning on
  the frontend side?
- Could the same need be satisfied by improving lowering, diagnostics, or
  reflection on the Rust side instead?
- Does it preserve the rule that the frontend is not an alternate runtime tree?

Frontend packages should only appear after the Rust-side semantic target exists.

## 10. Crate Creation Checklist

Before creating any new crate, answer yes to all applicable questions:

- Is the ownership category explicit?
- Is the dependency direction acceptable?
- Is an existing crate insufficient for a principled reason, not an aesthetic
  one?
- Is the name based on semantic role or domain rather than implementation
  accident?
- Is there at least one example or test that earns the crate?
- If this is optional, is it correctly modeled as a capability or adapter
  rather than being pushed into the kernel?

If several answers are still vague, do not create the crate yet.

## 11. Typical Rejection Reasons

The most common reasons to reject or defer a proposal should be explicit.

Reject or defer when:

- the contribution exists only for convenience of one implementation;
- the ownership layer is unclear;
- the dependency direction points upward;
- an existing Tokimu concept already covers the meaning;
- the proposal introduces a backend before the capability semantics exist;
- the proposal promotes a useful concept to the kernel without admission
  evidence;
- there is no proving example, test, or integration slice.

Useful is cheap. Tokimu should admit unavoidable meaning, not every convenient
abstraction.

## 12. Concept And Crate Retirement

Admission is only half of governance. Tokimu also needs a principled way to
retire concepts, crates, capabilities, adapters, or API surfaces when the
architecture becomes simpler without them.

A thing may be retired when one or more of these becomes true:

- ownership moved elsewhere more cleanly;
- decomposition became possible using better-understood existing concepts;
- two concepts turned out to be one concept with better boundaries;
- the implementation pressure that justified the addition disappeared;
- a capability or adapter was superseded by a clearer Tokimu-owned contract;
- an authoring surface duplicated meaning that now belongs elsewhere.

Retirement should follow the same evidence-based discipline as admission.
Questions to answer:

- What meaning is being removed, merged, or relocated?
- What now owns that meaning instead?
- What evidence shows the old boundary is no longer the best one?
- What examples, tests, or callers prove the replacement is sufficient?
- What migration, compatibility, or deprecation path is required?

The goal is not churn. The goal is to let Tokimu shed concepts that were once
useful but are no longer architecturally necessary.

## 13. Architectural Review Records

Open an Architectural Review Record when admission evidence needs durable
analysis before a binding decision is justified. This includes proposed kernel
or foundational admission, unclear ownership, new capability boundaries,
repeated corpus friction, and reconsideration of an accepted ADR.

The review record answers:

- what question was reviewed;
- what evidence triggered it;
- what ownership and dependency findings resulted;
- whether the proposal is accepted, incubating, deferred, rejected, or requires
  no architectural change;
- what corpus pressure would justify reopening it.

An accepted review that changes architecture must produce or revise an ADR. A
deferred or rejected review remains valuable without becoming policy. See
`docs/Architectural Reviews/README.md` and its `TEMPLATE.md`.

## 14. Minimal PR Template

For changes with architectural weight, the author should be able to answer this
briefly:

```text
Ownership:
Layer:
Evidence:
Proof:
Dependency direction:
Existing concept reused:
Why not lower/higher layer:
Retirement/deprecation impact:
Backend-specific details kept out:
```

If that template cannot be filled in clearly, the change likely needs another
design pass before merge.
