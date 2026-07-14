# Tokimu Frontends

This workspace holds authoring frontends separate from the Rust engine crates.
The design is specified in
[docs/Tokimu TypeScript Design Document.md](../docs/Tokimu%20TypeScript%20Design%20Document.md).

Packages:

- `tokimu` — the authoring anchor. Authors `import { rule, query } from "tokimu"`;
  it re-exports the stable surface from the domain packages.
- `@tokimu/rules` — the rule authoring API and types, mapping into the
  engine-owned `tokimu-rule` model. Supports both `lowered` and `runtime`
  execution intent.
- `@tokimu/examples` — authored content that consumes the authoring packages.
  It is not an authoring API; it depends on the packages above, never the reverse.

Boundaries:

- Authoring packages contain API surface and types only — no engine, no runtime.
- Authored content lowers into Tokimu-owned semantics via the Rust
  `tokimu-ts-frontend` host; the engine never imports these packages.

Development:

```sh
npm install
npm run typecheck
```
