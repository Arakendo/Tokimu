# Tokimu Consumer Corpus

Consumer corpus entries are application-shaped proofs that compose several
Tokimu contracts from the perspective of a downstream application.

They differ from focused `hello-*` entries:

- focused entries prove one architectural seam;
- consumer entries pressure composition, lifecycle, diagnostics, and target
  integration across several already demonstrated seams.

Every entry must identify its consumer tier:

- **Tier 1** uses only published-intent Tokimu APIs;
- **Tier 2** also uses explicitly named incubating libraries from `corpus/lib`;
- **Tier 3** intentionally validates a concrete provider or backend.

Repository-owned consumer entries remain corpus evidence. They are not
independent production consumers.

## Entries

- [`aspnet-wasm-asset-workbench`](aspnet-wasm-asset-workbench/DESIGN.md) is a
  Tier 2 ASP.NET 10 and TypeScript host that transfers dropped asset bytes into
  a Rust/WASM inspection adapter.
- [`tokimu-website-paint`](tokimu-website-paint/DESIGN.md) is a Tier 2
  Rust/WASM raster-editing consumer that keeps editable document truth below
  browser and Canvas mechanisms.
