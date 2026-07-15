# Tokimu Future Workspace Layout

| Field      | Value |
| ---------- | ----- |
| Status     | Draft — planning / naming guidance |
| Title      | "Future Workspace Layout" (kept) — a crate-placement and naming guide derived from the kernel/capability/backend model |
| Scope      | Recommended future workspace organization for Rust crates, TypeScript frontends, examples, and backend adapters. This is a layout policy, not a mandate to reorganize immediately. |
| Relates to | `docs/kernel-principles.md`, `docs/semantic-kernel-map.md`, `docs/capability-backends.md`, ADR-0003 |

## 1. Purpose

> The workspace reflects semantic ownership, not implementation convenience.

Tokimu now has enough architectural guidance to say more than "put Rust crates
under `crates/` and examples under `examples/`." The kernel principles,
semantic-kernel map, and capability-backend model together imply a stable
workspace taxonomy:

- a small native kernel;
- first-party foundational services that consume the kernel but do not own world
  truth;
- optional Tokimu-owned capability crates for domain meaning;
- backend adapter crates for concrete libraries;
- separate authoring frontends;
- examples that prove one slice at a time.

This document turns that architectural model into placement and naming rules so
future crates land in the right place for the right reason.

The goal is not an immediate directory reshuffle. The goal is to stop future
workspace growth from becoming accidental.

This document is best read as a consequence of the kernel work, not an
independent filesystem proposal. Once the trusted core, foundational services,
capabilities, and backends are distinguished by ownership, the workspace starts
to organize itself.

## 2. Layout Rules

The workspace should organize crates by ownership role, not by implementation
technology or by whichever feature happened to land first.

## 3. Dependency Direction

Ownership categories are only useful if dependency direction stays aligned with
them. The intended guardrail is:

```text
Kernel
  ↑ forbidden

Foundational services
  ↑

Capability crates
  ↑

Backend adapters
```

Read as dependency permission:

- kernel crates do not depend upward into foundational services, capability
  crates, backend adapters, or frontends;
- foundational service crates may depend on kernel crates, but not on optional
  capability crates or backend adapters unless a narrow, explicit boundary is
  being created and documented;
- capability crates may depend on kernel crates and, where justified, on
  foundational services they consume, but never on backend adapter crates;
- backend adapter crates depend on the capability crates whose contracts they
  implement, and may also depend on kernel/foundational crates as needed;
- authoring frontends depend on Tokimu-owned semantic surfaces, never the other
  way around.

The short rule is: dependency arrows point downward toward more universal
meaning, never upward toward more specialized implementation.

### 2.1 Native kernel crates

These own Tokimu's universal engine meaning and should stay small.

Current / expected members:

- `tokimu-core`
- `tokimu-runtime`
- `tokimu-rule`
- `tokimu` (facade)
- `tokimu-wasm` as an entry surface, not a second kernel

Kernel crates should not depend on heavy specialized libraries, browser-only
packages, OS-windowing stacks, database drivers, or domain-specific capability
implementations.

### 2.2 Foundational service crates

These are first-party, effectively always-present services that consume kernel
meaning but do not own simulation truth.

Current / expected members:

- `tokimu-render`
- `tokimu-platform`
- `tokimu-assets`
- `tokimu-input`
- future small engine-facing support crates if they remain generally required

These are not backend-adapter crates. `tokimu-render` is a foundational service
crate even if it contains a concrete `wgpu` backend today.

### 2.3 Capability crates

These own optional Tokimu-defined domain meaning outside the native kernel.

Expected future examples:

- `tokimu-geometry`
- `tokimu-persistence`
- `tokimu-physics`
- `tokimu-net`
- `tokimu-audio`
- `tokimu-script-host`
- `tokimu-ui` only if an actual engine-facing UI capability is earned

A capability crate exists when Tokimu needs to own a domain's semantic model,
provider contract, diagnostics, and selection rules without forcing one
concrete library into core.

### 2.4 Backend adapter crates

These integrate concrete external libraries behind Tokimu-owned capability
contracts.

Expected naming pattern:

- `tokimu-geometry-occt`
- `tokimu-geometry-truck`
- `tokimu-persistence-sqlite`
- `tokimu-persistence-indexeddb`
- `tokimu-script-host-quickjs`
- `tokimu-physics-rapier`

A backend adapter crate should never be added without a capability crate to own
its meaning first.

### 2.5 Authoring frontends

TypeScript and any future author-facing frontends should stay outside `crates/`
under a separate top-level workspace.

Current / expected shape:

- `frontends/`
- `frontends/packages/tokimu`
- `frontends/packages/rules`
- `frontends/packages/examples`
- future frontend packages only when a Tokimu-owned semantic model already
  exists on the Rust side

The frontend workspace is not an alternate runtime tree. It is an authoring
surface over Tokimu-owned semantics.

### 2.6 Examples and proof apps

Examples remain architecture-driving proofs, not dumping grounds for ad hoc
integrations.

- `examples/hello-window`
- `examples/hello-triangle`
- `examples/hello-asteroids`
- future capability proofs such as `examples/hello-geometry-profile` only after
  the matching capability crate exists

When a capability crate lands, the first backend proof should be paired with one
example or one end-to-end test, not a broad demo suite.

## 4. Recommended Future Tree

Illustrative target shape once the first capability crates and backend adapters
exist:

```text
tokimu/
├── Cargo.toml
├── README.md
├── docs/
│   ├── Tokimu Software Design Document.md
│   ├── Tokimu TypeScript Design Document.md
│   ├── capability-backends.md
│   ├── kernel-principles.md
│   ├── semantic-kernel-map.md
│   ├── future-workspace-layout.md
│   ├── ADR/
│   └── roadmap.md
│
├── crates/
│   ├── tokimu-core/              # world, identity, relations, resources, time
│   ├── tokimu-runtime/           # app lifecycle, scheduling, plugin orchestration
│   ├── tokimu-rule/              # engine-owned rule semantic model
│   ├── tokimu/                   # facade crate
│   ├── tokimu-wasm/              # wasm entry surface / bindings
│   │
│   ├── tokimu-render/            # foundational presentation service
│   ├── tokimu-platform/          # foundational OS/browser integration service
│   ├── tokimu-assets/            # foundational asset identity/loading service
│   ├── tokimu-input/             # foundational normalized input service
│   │
│   ├── tokimu-ts-frontend/       # TS recognition, validation, lowering host
│   │
│   ├── tokimu-geometry/          # optional Tokimu-owned geometry semantics
│   ├── tokimu-persistence/       # optional Tokimu-owned persistence semantics
│   ├── tokimu-physics/           # optional Tokimu-owned physics semantics
│   ├── tokimu-net/               # optional Tokimu-owned replication/network semantics
│   ├── tokimu-audio/             # optional Tokimu-owned audio semantics
│   └── tokimu-script-host/       # optional Tokimu-owned runtime-host contracts
│
├── adapters/
│   ├── tokimu-geometry-occt/
│   ├── tokimu-geometry-truck/
│   ├── tokimu-persistence-sqlite/
│   ├── tokimu-persistence-indexeddb/
│   ├── tokimu-physics-rapier/
│   └── tokimu-script-host-quickjs/
│
├── frontends/
│   ├── package.json
│   ├── tsconfig.base.json
│   └── packages/
│       ├── tokimu/
│       ├── rules/
│       ├── examples/
│       └── ...
│
├── examples/
│   ├── hello-window/
│   ├── hello-triangle/
│   ├── hello-rule-model/
│   ├── hello-asteroids/
│   ├── hello-geometry-profile/   # only after tokimu-geometry exists
│   └── hello-persistence/        # only after tokimu-persistence exists
│
└── tests/
    ├── integration/
    └── fixtures/
```

## 5. Naming Rules

### 5.1 Core and foundational crates

Use `tokimu-<role>` names for first-party Rust crates that define engine-owned
surfaces.

Examples:

- `tokimu-core`
- `tokimu-runtime`
- `tokimu-render`
- `tokimu-platform`
- `tokimu-assets`

These names should stay plain-English and role-based.

### 5.2 Capability crates

Use `tokimu-<domain>` for a Tokimu-owned optional semantic domain.

Examples:

- `tokimu-geometry`
- `tokimu-persistence`
- `tokimu-physics`

A capability crate name should describe the semantic domain, not the chosen
implementation.

### 5.3 Backend adapter crates

Use `tokimu-<domain>-<backend>` for concrete integrations.

Examples:

- `tokimu-geometry-occt`
- `tokimu-persistence-sqlite`
- `tokimu-script-host-quickjs`

This makes the ownership stack legible in the crate name itself.

### 5.4 Frontend packages

Frontend packages should continue using npm-style names under `@tokimu/*` where
appropriate, but only after a Rust-side semantic target exists.

Examples:

- `@tokimu/rules`
- future `@tokimu/geometry` only after `tokimu-geometry`

## 6. When To Create A New Crate

Before adding a crate, answer the ownership question first.

### Create or extend a kernel crate when:

- the concept is universal engine meaning;
- most Tokimu applications need it;
- it passes the primitive-admission / boundary tests from the side docs;
- it does not require a heavy specialized dependency.

### Create a foundational service crate when:

- the service is first-party and broadly required;
- it adapts platform/presentation/asset/input concerns around the kernel;
- it still should not own world truth.

### Create a capability crate when:

- Tokimu needs to own the semantic model for a domain;
- multiple applications may need the domain, but not all applications;
- competing specialized libraries could implement it;
- the domain should stay outside `tokimu-core` and `tokimu-runtime`.

### Create a backend adapter crate when:

- a capability crate already exists;
- one concrete library has been justified by a real example or integration test;
- the adapter can stay behind Tokimu-owned types and diagnostics.

## 7. Migration Guidance

The current workspace does not need a mass move now.

Recommended sequencing:

1. Keep the current `crates/`, `examples/`, and `frontends/` split.
2. Continue treating `tokimu-render`, `tokimu-platform`, `tokimu-assets`, and
   `tokimu-input` as foundational services rather than misclassifying them as
   backend adapters.
3. Add `adapters/` only when the first real backend adapter crate lands.
4. Add the first capability crate and exactly one backend adapter together with
   one proving example or end-to-end test.
5. Reorganize existing crates only if a real dependency problem or ownership
   confusion appears; do not reshuffle directories merely to satisfy aesthetics.

## 8. Immediate Effect On The Current Workspace

This draft implies only a few near-term rules for today's repository:

- keep current kernel and foundational crates where they are;
- keep TypeScript authoring packages under `frontends/`;
- do not add domain/backend crates directly into `tokimu-core` or
  `tokimu-runtime`;
- when the first optional domain is earned, add both `tokimu-<domain>` and a
  matching place for `tokimu-<domain>-<backend>` rather than folding the work
  into existing crates.

That is enough structure for now. The layout should become more concrete only
when the first capability/backend pair is real rather than hypothetical.
