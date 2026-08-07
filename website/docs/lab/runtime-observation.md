---
title: Runtime Observation Workbench
description: A shared Rust/WASM runtime-observation session projected through semantic controls and Ratatui.
---

# Runtime Observation Workbench

This lab is a bounded consumer proof for runtime observations, owner-qualified
commands, and embedded terminal presentation. It is not a general browser
terminal and it does not admit Ratatui as a permanent Tokimu provider.

## What This Proves

```text
browser semantic control or terminal input
                    ↓
one Rust/WASM observation-shell session
                    ↓
Tokimu runtime scenario and command routing
                    ↓
semantic control result + Ratatui terminal projection
                    ↓
Tokimu rasterization to RGBA
                    ↓
browser canvas blit
```

The semantic toolbar and the terminal are two projections of the same retained
Rust/WASM `WasmObservationShellSession`. Selecting a presentation target or
advancing playback through the toolbar changes the same runtime state that the
Ratatui terminal renders. Terminal commands use that same session as well.

TypeScript owns browser focus, normalized event forwarding, resize observation,
and whole-frame canvas blitting. It does not parse commands, keep another
runtime, position terminal glyphs, or interpret Ratatui styles.

<section
  class="island-stage runtime-observation-island"
  data-tokimu-island="runtime-observation"
  data-state="idle"
  aria-labelledby="runtime-observation-title"
>
  <div class="island-fallback">
    <p class="eyebrow">Runtime observation / shared-session evidence / on demand</p>
    <h2 id="runtime-observation-title">Inspect one retained runtime session</h2>
    <p>
      Open a bounded runtime scenario. Use the semantic controls, then focus
      the terminal canvas and enter <code>HELP</code>, <code>STATUS</code>, or
      <code>CLEAR</code>. Both surfaces observe the same Rust/WASM session.
    </p>
    <p>
      This is consumer evidence only. It does not claim native/browser session
      handoff, full terminal emulation, or permanent Ratatui admission.
    </p>
    <button class="button button-primary" type="button" data-island-action="activate">
      Open runtime observation workbench
    </button>
    <button class="button button-secondary" type="button" data-island-action="reset" hidden>
      Close runtime observation workbench
    </button>
  </div>
  <div class="island-mount" data-island-mount hidden></div>
  <div class="island-status" role="status" aria-live="polite">
    <span data-island-status-state>Idle</span>
    <span data-island-status-detail>No runtime observation session loaded</span>
  </div>
  <script type="application/json" data-island-config>
    {
      "schema": 1,
      "activation": "explicit"
    }
  </script>
</section>

## Boundary

- Rust/WASM owns the runtime fixture, observations, command catalog, command
  validation, prompt, transcript, history, and Ratatui composition.
- Ratatui owns terminal layout and style composition inside its bounded region.
- Tokimu owns the retained terminal surface and font rasterization.
- The browser owns only the containing island, canvas focus, input forwarding,
  resize notifications, and RGBA frame presentation.

The iframe document loading is not runtime-readiness evidence. The child reports
`loading`, `ready`, or `error` only after Rust/WASM initialization. Startup
failures remain visible inside the consumer and propagate to the containing
island instead of leaving a blank terminal beneath a false ready state.

`AR-0013` remains incubating. The still-open questions are shared live
sessions across standalone/native and embedded hosts, host-level viewport
parity, and permanent provider admission.
