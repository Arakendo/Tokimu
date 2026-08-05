---
title: Ratatui Template Lab
description: Ratatui widgets rendered through a retained Tokimu backend and Tokimu font rasterizer.
---

# Ratatui Template Lab

This laboratory renders a few deterministic terminal-shaped templates from
dummy data. It is not a browser terminal emulator and it does not claim that
Ratatui has been admitted as Tokimu's permanent native presentation provider.

## What This Proves

```text
browser template selector
             ↓
Rust/WASM template selection
             ↓
Ratatui widget composition
        ↓
retained TokimuBackend surface
        ↓
Tokimu font rasterization
        ↓
RGBA frame
        ↓
browser canvas blit
```

Ratatui owns terminal layout and style composition. A corpus-local Rust/WASM
`TokimuBackend` retains Ratatui's changed-cell delivery, and Tokimu's font
provider rasterizes that retained surface. TypeScript owns the browser control
and blits the resulting RGBA frame; it does not position glyphs, interpret
styles, or recreate the template layout.

<section
  class="island-stage ratatui-lab-island"
  data-tokimu-island="ratatui-lab"
  data-state="idle"
  aria-labelledby="ratatui-lab-title"
>
  <div class="island-fallback">
    <p class="eyebrow">Experimental presentation-provider evidence / on demand</p>
    <h2 id="ratatui-lab-title">Ratatui template laboratory</h2>
    <p>
      Compare three dummy-data layouts: a system monitor, an asset inspector,
      and a command transcript. Each layout is composed by Rust/WASM through
      a retained Tokimu backend and Tokimu's Departure Mono font provider before
      the browser sees a completed RGBA frame.
    </p>
    <p>
      This is bounded provider evidence. It contains no host session, live
      telemetry, TQL execution, or browser-owned terminal semantics.
    </p>
    <button class="button button-primary" type="button" data-island-action="activate">
      Open Ratatui template lab
    </button>
    <button class="button button-secondary" type="button" data-island-action="reset" hidden>
      Close template lab
    </button>
  </div>
  <div class="island-mount" data-island-mount hidden></div>
  <div class="island-status" role="status" aria-live="polite">
    <span data-island-status-state>Idle</span>
    <span data-island-status-detail>No Ratatui template projection loaded</span>
  </div>
  <script type="application/json" data-island-config>
    {
      "schema": 1,
      "activation": "explicit"
    }
  </script>
</section>

## Ownership Boundary

- Ratatui is a replaceable layout provider under evaluation.
- Rust/WASM owns template selection, dummy observations, retained cells, and
  Tokimu rasterization.
- TypeScript owns template-control interaction and whole-frame canvas blitting
  only.
- The browser does not interpret Ratatui cells, interpret a shell command,
  derive terminal layout, or replace the provider with browser-native terminal
  behavior.

The native console corpus and the Tosumu inspection island exercise related
evidence with their own consumer-specific state. This page deliberately keeps
its scenes static so provider layout can be examined without importing another
application's semantics.
