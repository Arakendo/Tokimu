# Tokimu Website Paint

Tier 2 consumer evidence for a bounded Rust/WASM raster-editing application.

The current slice proves both the headless editable document boundary and the
standalone browser adapter. Rust/WASM remains authoritative for decoded pixels,
edit commands, history, preview bytes, and PNG export. TypeScript only lowers
browser interaction and presents explicit byte copies in Canvas.

```powershell
cargo test -p tokimu-website-paint-engine
```

Build the bounded Rust/WASM control boundary with:

```powershell
npm install
.\build.ps1
```

The generated `dist/` contains WASM bindings plus a deliberately small Canvas
workbench. Run it from a local static HTTP server rather than opening
`index.html` directly, because browser WASM module loading requires an HTTP
origin.

For example:

```powershell
python -m http.server 4175 --directory .\dist
```

Then open `http://127.0.0.1:4175/`.

The workbench currently supports New, Open, Pencil, Eraser, exact-match Fill,
Sample, Undo, Redo, Reset, drag/drop, keyboard shortcuts, display-only zoom,
bounded browser observations, and deterministic PNG download. It also releases
its Rust/WASM session when its page is unloaded.

The same reviewed payload is published as the optional
[`Tokimu Paint` website island](../../../website/docs/lab/paint.md). The public
page provides durable static ownership and limitation text before explicitly
loading the workbench.

## Replay Evidence

`PaintReplay` is a serializable, blank-document edit script for corpus work. It
records the document configuration, ordered semantic commands, and terminal
document/history observations when executed. It intentionally excludes Canvas
events, browser timing, renderer state, and imported-image identities; those
remain separate diagnostic boundaries until a second consumer proves a shared
contract.
