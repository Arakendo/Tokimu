---
title: Tokimu Paint
description: A bounded Rust/WASM raster-editing consumer corpus.
---

# Tokimu Paint

Tokimu Paint is an experimental consumer corpus for an ordinary raster editing
workflow. It pressures a narrow composition boundary: Tokimu-owned decoded
pixels, edit commands, history, preview bytes, and PNG export meet browser
input, Canvas presentation, and download mechanisms without changing ownership.

## What This Proves

```text
browser file / pointer / keyboard
                ↓
         TypeScript adapter
                ↓
        Rust/WASM PaintSession
                ↓
editable raster document + history
                ↓
      copied preview bytes to Canvas
```

The browser does not decode source images, own the editable buffer, or produce
the exported image. Canvas is a presentation target only.

<section
  class="island-stage paint-island"
  data-tokimu-island="tokimu-paint"
  data-state="idle"
  aria-labelledby="tokimu-paint-title"
>
  <div class="island-fallback">
    <p class="eyebrow">Experimental consumer evidence / on demand</p>
    <h2 id="tokimu-paint-title">Raster workbench</h2>
    <p>
      Activate a bounded Rust/WASM editing session. The current proof supports
      a blank document or admitted PNG, JPEG, and BMP input; pencil, eraser,
      exact-match fill, sampling, undo, redo, and deterministic PNG export.
    </p>
    <p>
      This is consumer evidence, not a general editor or a browser-native image
      fallback. The page remains useful if the interactive payload is unavailable.
    </p>
    <button class="button button-primary" type="button" data-island-action="activate">
      Open raster workbench
    </button>
    <button class="button button-secondary" type="button" data-island-action="reset" hidden>
      Close workbench
    </button>
  </div>
  <div class="island-mount" data-island-mount hidden></div>
  <div class="island-status" role="status" aria-live="polite">
    <span data-island-status-state>Idle</span>
    <span data-island-status-detail>No editable document session loaded</span>
  </div>
  <script type="application/json" data-island-config>
    {
      "schema": 1,
      "activation": "explicit"
    }
  </script>
</section>

## Limits

- The editable document is an application-local, top-down RGBA8 profile.
- PNG export is deterministic; original source encoding is not preserved.
- The current raster providers admit a bounded PNG, baseline JPEG, and BMP
  profile rather than broad image-format compatibility.
- Browser interaction is not a claim that TypeScript or Canvas owns image
  semantics.

See the
[Tokimu Paint consumer design](https://github.com/Arakendo/Tokimu/blob/main/corpus/consumers/tokimu-website-paint/DESIGN.md)
and the
[Raster Image Corpus Testing record](https://github.com/Arakendo/Tokimu/blob/main/docs/Libraries/raster-image-corpus-testing.md)
for the evidence and deferred work.
