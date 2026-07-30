---
title: Architecture
description: Tokimu is organized around world truth, semantic ownership, and replaceable mechanisms.
---

<p class="page-kicker">Architecture / ownership</p>

# Meaning above machinery

Tokimu is a state-processing runtime. It accepts input, rules, assets, and time,
then produces updated world state and observable output.

Its central ownership chain is:

```text
Corpus and applications
          ↓
   World graph / state
          ↓
 Systems, signals, rules
          ↓
      State change
       ↙       ↘
Presentation  Synchronization
       ↓             ↓
Renderer / Platform / WASM / Transport
```

## The trusted center

The world model owns simulation truth. Rendering, platform integration,
authoring tools, persistence, and networking adapt or observe that truth.
Useful capabilities do not automatically belong in the kernel.

## Foundational services

Services such as rendering, input, assets, diagnostics, and presentation expose
Tokimu-owned contracts while concrete providers remain replaceable. Font files,
GPU backends, icon libraries, sockets, and browser APIs should not define the
engine's semantic model.

## Application ecosystems

Tokimu aims to make specialized applications straightforward to build without
absorbing them into the engine. A game, CAD workbench, robotics dashboard, or
technical simulator may own very different domain meaning while sharing the
same runtime services.

<div class="quote-panel">
  <span>Architectural maxim</span>
  <blockquote>Own meaning. Delegate implementation.</blockquote>
</div>

## Source documents

The repository currently treats these documents as architectural truth:

- [Software Design Document](https://github.com/Arakendo/Tokimu/blob/main/docs/Tokimu%20Software%20Design%20Document.md)
- [Kernel Principles](https://github.com/Arakendo/Tokimu/blob/main/docs/kernel-principles.md)
- [Semantic Kernel Map](https://github.com/Arakendo/Tokimu/blob/main/docs/semantic-kernel-map.md)
- [Architectural Maxims](https://github.com/Arakendo/Tokimu/blob/main/docs/architectural-maxims.md)
- [Accepted ADRs](https://github.com/Arakendo/Tokimu/tree/main/docs/ADR)
