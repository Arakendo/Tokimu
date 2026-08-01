# Website Kernel UI Consumer

## Purpose

This consumer proves that an ordinary browser application can present and edit
Tokimu-owned resource state through a bounded Rust/WASM observation contract.

```text
browser input
    |
    v
TypeScript presentation adapter
    |
    v
Rust/WASM ResourceWorkbenchModel
    |
    v
provider-neutral observation
```

The browser owns DOM layout, accessibility, and input mechanisms. Rust owns
resource identity, filtering, selection, draft state, command eligibility,
deletion confirmation, and state transitions.

## Corpus Claim

- TypeScript does not reconstruct resource mutation rules.
- DOM elements are presentation targets, not authoritative state.
- Native and browser hosts consume the same application model.
- Compact browser layout does not alter model semantics.
- The page remains useful before the WASM payload is activated.

## Non-Goals

- Admitting UI policy to `tokimu-core`.
- Claiming the DOM and native `ui-tools` renderer are interchangeable.
- General resource persistence, collaboration, or filesystem integration.
- Stabilizing the resource workbench as a public Tokimu API.
