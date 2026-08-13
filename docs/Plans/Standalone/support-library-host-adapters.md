# Support Library Host Adapters

## Status

| Field | Value |
| --- | --- |
| Status | Proposed |
| Scope | A top-level `support/` area for reusable host-facing adapters and the corpus evidence required before one is stabilized |
| Related plans | `docs/Plans/Standalone/consumer-corpora.md`, `docs/Plans/Standalone/tokimu-website.md` |
| Related decisions | ADR-0001, ADR-0003, ADR-0005 |
| Initial candidates | Avalonia, Capacitor, Flutter |

## Purpose

Tokimu needs a clear place for optional libraries that help applications consume
public Tokimu contracts from established host ecosystems. Examples include a
.NET desktop host using Avalonia, a TypeScript mobile host using Capacitor, and
a Dart mobile host using Flutter.

These adapters are useful only when they reduce repeated host-specific glue
without moving engine semantics into a host framework. The purpose of this plan
is to establish a disciplined folder, evidence model, and admission rule before
adding a collection of framework wrappers.

The governing rule is:

> **Hosts own application lifecycle and presentation mechanisms. Tokimu owns
> engine semantics. Support libraries adapt the boundary without redefining
> either side.**

## Primary Claim

The support-library area succeeds when a normal application can use a host
framework together with Tokimu through public or explicitly incubating APIs,
without:

- making `tokimu-core` or `tokimu-runtime` depend on .NET, JavaScript, Dart,
  native mobile SDKs, or host build tooling;
- duplicating importer, world, rendering, or asset semantics in the host;
- leaking host framework objects into Tokimu-owned public contracts;
- treating one framework's lifecycle as the universal Tokimu lifecycle.

## Ownership Model

```text
Application domain intent
        |
        v
Public Tokimu semantic contracts
        |
        v
support/<host> adapter
        |
        v
Host framework and platform mechanisms
```

| Layer | Owns | Must not own |
| --- | --- | --- |
| Application | Product state, workflow, UI intent, host composition | Tokimu internal semantics or provider parsing |
| Tokimu public crates | World, runtime, asset, presentation, diagnostics, and capability semantics | Host framework lifecycle or widgets |
| `support/<host>` | Host lifecycle translation, event bridging, package conventions, bounded presentation surfaces | World truth, importer semantics, or a universal UI abstraction |
| Host framework | Windowing, widgets, platform packaging, accessibility mechanisms, native/mobile integration | Tokimu domain meaning |
| `corpus/consumers` | Evidence that the public boundary composes cleanly | Stable reusable host APIs before repeated pressure exists |

This follows ADR-0001 and ADR-0003: support libraries are optional adapters,
not native engine meaning and not dependencies of the core runtime.

## Proposed Layout

```text
support/
  README.md
  avalonia/                 # Added only after Avalonia evidence repeats
  capacitor/                # Added only after Capacitor evidence repeats
  flutter/                  # Added only after Flutter evidence repeats

corpus/
  consumers/
    dotnet-avalonia-*/      # Application-shaped Avalonia evidence
    capacitor-*/            # Application-shaped Capacitor evidence
    flutter-*/              # Application-shaped Flutter evidence

third-party/
  ...                       # Pinned upstream sources only, never support code
```

`support/` owns reusable adapter packages. `corpus/consumers/` owns the
applications that prove whether those packages are justified. A host-specific
sample must not be moved into `support/` merely because it compiles.

## Candidate Profiles

### Avalonia and .NET

Avalonia is the first candidate because it offers a concrete cross-platform
.NET consumer path for desktop tooling and tests. Its initial proof should be
a small consumer corpus, not a general Tokimu UI wrapper.

The first consumer should demonstrate:

- native desktop startup through Avalonia;
- an explicit Tokimu lifecycle boundary;
- normalized host input forwarded through public Tokimu contracts;
- a bounded presentation target or diagnostics view;
- deterministic headless or report-oriented coverage where Avalonia supports
  it;
- no .NET framework object crossing into engine-owned types.

The eventual `support/avalonia/` adapter may own startup convenience and host
event translation. It must not own world state, scene semantics, or a new
widget vocabulary that competes with Tokimu presentation contracts.

### Capacitor

Capacitor is a prospective TypeScript and mobile-WebView host. It should be
considered after an ordinary browser/WASM consumer proves insufficient to
exercise a mobile-specific boundary such as lifecycle suspension, file handoff,
device input, or mobile packaging.

The consumer should use the established Rust/WASM and TypeScript boundary. It
must not introduce a second JavaScript implementation of Tokimu semantics.

### Flutter

Flutter is a prospective Dart and mobile host. It should be considered only
when it supplies independent evidence beyond a Capacitor or browser consumer:
for example, a distinct native-mobile lifecycle, rendering surface, input path,
or application packaging requirement.

The initial goal is not a Flutter renderer or a universal Flutter widget set.
It is a bounded bridge to public Tokimu semantics and diagnostics.

## Common Consumer Contract

Every host consumer must document:

- its consumer tier from `docs/Plans/Standalone/consumer-corpora.md`;
- the public Tokimu crates, WASM API, or explicitly incubating contracts it
  consumes;
- host-owned application state and Tokimu-owned state;
- host lifecycle states mapped to Tokimu lifecycle requests;
- input and presentation boundaries;
- deterministic fixtures and observable reports;
- host-only behavior and known unsupported behavior;
- diagnostics visible to both the host application and a headless/test path
  where meaningful.

The same importer, asset, world, and provider semantics must produce comparable
observations across native Rust, browser/WASM, and host adapters. A host may
present a diagnostic differently; it may not redefine the diagnostic's meaning.

## Admission Rules

A directory under `support/` is admitted only when all of the following hold:

- [ ] A named consumer corpus demonstrates the host integration using the
      intended public or explicitly incubating Tokimu boundary.
- [ ] The consumer records repeated host-side glue that belongs below the
      application boundary.
- [ ] The proposed adapter API does not expose host objects through
      Tokimu-owned semantic contracts.
- [ ] The adapter can be described without adding a dependency from
      `tokimu-core` or `tokimu-runtime` toward the host ecosystem.
- [ ] At least one deterministic or inspectable acceptance path exists beyond
      a manually observed window.
- [ ] The consumer's friction is reviewed before a generalized wrapper is
      extracted.

One successful demo is evidence for the consumer, not automatic evidence for a
reusable host library.

## Implementation Slices

### Slice 0: Establish The Boundary

- [x] Create `support/README.md` with the folder ownership policy.
- [x] Record the candidate ecosystems and their relationship to consumer
      corpora.
- [ ] Link the plan from the consumer-corpus index if repeated host work begins.

Acceptance criteria:

- `support/` is clearly distinguished from `third-party/` and
  `corpus/consumers/`.
- The documentation explicitly prohibits upward dependencies into
  `tokimu-core` and `tokimu-runtime`.
- No framework dependency is added merely to establish the directory.

### Slice 1: Avalonia Consumer Design

- [ ] Create `corpus/consumers/dotnet-avalonia-workbench/` with a `DESIGN.md`.
- [ ] Select one bounded public Tokimu claim, preferably diagnostics or an
      existing asset/presentation observation.
- [ ] Define native desktop and headless/report-oriented acceptance paths.
- [ ] Record the exact .NET and Avalonia versions and host platforms under
      test.

Acceptance criteria:

- The design identifies application, adapter, host, and Tokimu ownership.
- The selected claim is testable without copying engine semantics into C#.
- Unsupported host paths are visible diagnostics, not silent fallbacks.

### Slice 2: Avalonia Consumer Evidence

- [ ] Implement the smallest Avalonia consumer that exercises the selected
      claim.
- [ ] Capture a deterministic report or screenshot artifact alongside the
      interactive run.
- [ ] Record every application-side bridge and lifecycle translation.
- [ ] Decide whether any repeated bridge belongs in `support/avalonia/`.

Acceptance criteria:

- The consumer calls only declared public or incubating Tokimu surfaces.
- A useful .NET desktop path works without host objects leaking into Tokimu.
- The evidence distinguishes host failure, adapter failure, and Tokimu
  semantic failure.

### Slice 3: Capacitor Decision And Evidence

- [ ] Identify a mobile-WebView requirement not already covered by the browser
      consumers.
- [ ] Create a Capacitor consumer only if that requirement is concrete.
- [ ] Compare its observations with the equivalent browser/WASM consumer.

Acceptance criteria:

- Capacitor adds independent mobile-host evidence rather than a duplicate web
  shell.
- TypeScript remains an adapter and presentation layer, not a semantic engine
  implementation.

### Slice 4: Flutter Decision And Evidence

- [ ] Identify a Flutter-specific requirement that remains unproven by
      Avalonia and Capacitor.
- [ ] Create a bounded Flutter consumer only when that requirement exists.
- [ ] Compare the public Tokimu observations with another host consumer.

Acceptance criteria:

- Flutter supplies independent pressure on the host boundary.
- Dart and Flutter objects remain below the public Tokimu semantic boundary.

### Slice 5: Support-Library Admission Review

- [ ] Review each host consumer's repeated glue, ownership findings, and
      diagnostics.
- [ ] Admit a narrowly scoped `support/<host>` package only where the
      consumer evidence justifies it.
- [ ] Open or update an Architectural Review if a reusable bridge changes a
      capability or public-boundary decision.

Acceptance criteria:

- Each admitted support package has a concrete, documented responsibility.
- No package claims to be a universal UI, mobile, or application framework.
- The outcome is explicit: admit a host package, continue corpus incubation,
  or reject the extraction.

## Non-Goals

- Creating bindings for every UI or mobile framework.
- Making Avalonia, Capacitor, or Flutter a dependency of the engine kernel.
- Replacing native Rust, browser/WASM, or TypeScript consumers.
- Designing a framework-neutral widget abstraction before applications require
  one.
- Treating compilation on one operating system as cross-platform evidence.
- Using a host framework to bypass a missing Tokimu capability.

## Open Questions

- Does Avalonia need a direct native renderer adapter, a WASM/WebView surface,
  or only a diagnostics-oriented first consumer?
- Which lifecycle events must be normalized for desktop and mobile hosts?
- Should host adapters share a small lifecycle helper, or should that remain
  duplicated until at least two hosts demonstrate the same need?
- Are consumer screenshots sufficient evidence for cross-platform behavior, or
  is a platform-matrix report also required?
- When a host only needs asset observations, should it consume the WASM facade
  or a native Rust bridge through a separate ABI?

## Graduation Trigger

The `support/` area remains policy-only until a host consumer repeatedly proves
that an adapter package removes host-specific glue without changing Tokimu
semantics. A specific `support/<host>` implementation graduates when:

- at least one concrete consumer depends on its bounded contract;
- the consumer has recorded repeatable host glue rather than one-off UI code;
- the adapter does not expose foreign host types through Tokimu contracts;
- the adapter has an inspectable acceptance path;
- an Architectural Review finds the extracted boundary stable enough to keep.

Stable boundaries are more valuable than early package names. The folder exists
to make that discipline visible, not to imply that every listed ecosystem is
already supported.
