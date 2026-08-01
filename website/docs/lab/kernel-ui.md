---
title: Kernel UI Workbench
description: A bounded Rust/WASM resource-editing consumer for Tokimu state and UI semantics.
---

# Kernel UI Workbench

The Kernel UI Workbench is a consumer corpus for editing application-owned
resource state through a bounded browser adapter. It is called a kernel UI
example because it demonstrates UI around stable Tokimu meaning, not because
DOM controls or application-specific resource fields belong in `tokimu-core`.

## What This Proves

```text
browser pointer / keyboard
              ↓
       TypeScript adapter
              ↓
 Rust/WASM ResourceWorkbenchModel
              ↓
 provider-neutral observation
              ↓
       accessible DOM view
```

Rust owns resource identity, filtering, selection, draft state, command
eligibility, and delete confirmation. TypeScript owns browser input and DOM
presentation. The two hosts, native `ui-tools` and browser DOM, consume the same
application model without pretending their presentation mechanisms are equal.

<section
  class="island-stage kernel-ui-island"
  data-tokimu-island="kernel-ui"
  data-state="idle"
  aria-labelledby="kernel-ui-title"
>
  <div class="island-fallback">
    <p class="eyebrow">Experimental UI consumer evidence / on demand</p>
    <h2 id="kernel-ui-title">Resource control room</h2>
    <p>
      Activate a Rust/WASM resource editing session. Filter and select stable
      resource identities, edit draft fields, toggle presentation flags, apply
      or revert changes, and exercise a model-owned delete confirmation flow.
    </p>
    <p>
      This is evidence for UI composition and state ownership, not admission of
      resource-editor policy into the trusted kernel.
    </p>
    <button class="button button-primary" type="button" data-island-action="activate">
      Open kernel UI workbench
    </button>
    <button class="button button-secondary" type="button" data-island-action="reset" hidden>
      Close workbench
    </button>
  </div>
  <div class="island-mount" data-island-mount hidden></div>
  <div class="island-status" role="status" aria-live="polite">
    <span data-island-status-state>Idle</span>
    <span data-island-status-detail>No resource editing session loaded</span>
  </div>
  <script type="application/json" data-island-config>
    {
      "schema": 1,
      "activation": "explicit"
    }
  </script>
</section>

## Ownership Boundaries

- DOM nodes are presentation targets, not authoritative resource state.
- TypeScript does not decide whether Apply, Revert, or Delete is legal.
- The resource model remains application-owned corpus evidence.
- `ui-tools` provides the native presentation path; the browser provides a
  separate consumer path over the same state transitions.
- Closing the island releases its WASM session.

See the
[website consumer design](https://github.com/Arakendo/Tokimu/blob/main/corpus/consumers/tokimu-website-kernel-ui/DESIGN.md)
and the
[native resource workbench design](https://github.com/Arakendo/Tokimu/blob/main/corpus/consumers/ui-resource-workbench/DESIGN.md)
for the complete evidence claim.
