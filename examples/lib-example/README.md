# Example Library Playground

This folder holds example-side reusable crates that future Tokimu examples can
consume.

## Folder Map

```text
lib-example/
├── README.md
├── ui-tools/
│   ├── Cargo.toml
│   ├── DESIGN.md
│   └── src/
│       ├── lib.rs
│       ├── controls.rs
│       ├── geometry.rs
│       └── layout.rs
└── ui-framework/
    ├── Cargo.toml
    ├── DESIGN.md
    └── src/
        └── main.rs
```

## Reusable Deliverables

- `ui-tools` owns the reusable geometry and button primitives
- `ui-framework` is the first consumer that proves the primitives feel useful
- future examples should reuse these helpers instead of recreating their own
  button math, hit testing, and layout scaffolding

## Current Scope

- `ui-tools` contains shared UI geometry, button, and layout helpers
- `ui-framework` exercises those helpers in a small framed workspace shell
- the next reusable additions should focus on text, cards, and label-bearing
  controls rather than more one-off panel shapes

## Reusable Element Goals

These are the element families we want to keep cleanly reusable across example
projects:

- text headers and labels
- toolbar buttons and action chips
- status badges and state pills
- content cards and preview tiles
- layout helpers for strips, shells, and framing
- hover and selection state markers

## Design Docs

- [ui-tools design](ui-tools/DESIGN.md)
- [ui-framework design](ui-framework/DESIGN.md)
