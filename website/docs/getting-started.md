---
title: Getting Started
description: Understand Tokimu's current shape and choose an honest first entry point.
---

<p class="page-kicker">Start / orientation</p>

# Getting started

Tokimu is in early development. The repository is currently most useful to
engine contributors, architecture readers, and people evaluating its corpus
evidence. It is not yet a packaged end-user engine distribution.

## Choose an entry point

<div class="path-grid">
  <article>
    <span>01</span>
    <h3>Understand the model</h3>
    <p>Begin with the architecture overview and learn which layer owns simulation truth.</p>
    <a href="../architecture/">Architecture overview →</a>
  </article>
  <article>
    <span>02</span>
    <h3>Inspect the evidence</h3>
    <p>Review how focused, data, and consumer corpora challenge engine boundaries.</p>
    <a href="../corpus/">Corpus evidence →</a>
  </article>
  <article>
    <span>03</span>
    <h3>Understand authoring</h3>
    <p>See how TypeScript is intended to author high-level content without becoming a second engine.</p>
    <a href="../architecture/rust-and-typescript/">Rust and TypeScript →</a>
  </article>
</div>

For current implementation work, build the Rust workspace and run focused
corpus entries from the
[source repository](https://github.com/Arakendo/Tokimu).

## Current workspace

The public facade and engine services are organized around semantic ownership:

```text
tokimu-core       engine-neutral world concepts
tokimu-runtime    lifecycle, scheduling, and execution
tokimu-render     provider-neutral rendering contracts
tokimu-platform   native and browser mechanisms
tokimu-assets     asset identity and loading
tokimu-input      normalized input state
tokimu-wasm       bounded browser entry surface
tokimu            public facade
```

The `corpus/` tree contains architecture-driving executable evidence, including
focused proofs, external data corpora, shared incubating support, and
application-shaped consumers.

The separate `frontends/` workspace contains the early TypeScript authoring
packages. Those packages describe authored intent and lower toward
language-neutral Rust models; core engine crates do not depend on npm or a
JavaScript runtime.

## Build the workspace

Install stable Rust, clone the repository, then run:

```text
cargo test --workspace
```

Individual corpus entries can be launched by package name. Their README or
`DESIGN.md` files identify the claim being tested, dependencies, expected
diagnostics, and known limitations.

!!! note
    Compilation proves that boundaries still connect. It does not by itself
    prove visual parity, browser lifecycle behavior, or backend correctness.
