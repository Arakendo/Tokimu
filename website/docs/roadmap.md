---
title: Roadmap
description: The current evidence-driven direction for Tokimu.
---

# Roadmap

Tokimu's roadmap is organized around capabilities that can be demonstrated,
measured, and reviewed. It is not a promise that every listed technology has
already reached the same maturity.

## Current direction

<div class="roadmap-list">
  <article>
    <span>Active</span>
    <h2>Presentation geometry</h2>
    <p>Continue hardening shared vector geometry through SVG, icon, UI, and font outline corpus pressure.</p>
  </article>
  <article>
    <span>Active</span>
    <h2>Asset observations</h2>
    <p>Improve provider-neutral inspection and preview boundaries for SVG, CGM, glTF/GLB, and FBX consumers.</p>
  </article>
  <article>
    <span>Active</span>
    <h2>Runtime diagnostics</h2>
    <p>Preserve bounded performance observations without turning the kernel into a general profiler.</p>
  </article>
  <article>
    <span>Incubating</span>
    <h2>WebAssembly consumers</h2>
    <p>Prove ordinary TypeScript applications can consume Tokimu through a bounded WASM API without duplicating engine semantics.</p>
  </article>
  <article>
    <span>Experimental</span>
    <h2>Interactive evidence</h2>
    <p>Harden the first bounded WASM evidence island and admit additional consumers only when each makes a focused, falsifiable claim.</p>
  </article>
</div>

## How items advance

```text
Need appears
    ↓
Focused corpus proof
    ↓
Evidence accumulates
    ↓
Architectural Review
    ↓
Capability admission
    ↓
ADR when the decision becomes binding
```

This sequence lets implementation pressure refine a boundary before the
repository treats it as permanent architecture.

## What this page does not claim

- A listed capability is not automatically production-ready.
- A readable file is not automatically renderable.
- A diagnostic preview is not a complete semantic implementation.
- A published experimental WebAssembly consumer is not a general browser,
  renderer, or SDK guarantee.

See [Capabilities](capabilities/index.md) for the maturity vocabulary used
throughout the site and [Known Limitations](known-limitations.md) for current
constraints.
