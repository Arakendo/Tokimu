# Tokimu Website Asteroids

This consumer corpus is the compileable template for a playable Asteroids
island on the Tokimu website.

## Build

Requirements:

- Rust target `wasm32-unknown-unknown`
- `wasm-bindgen` CLI
- Node.js and npm

From this directory:

```powershell
npm install
.\build.ps1
```

Serve `dist/` through an HTTP server. Browsers do not load WASM modules
reliably from `file://` URLs.

## Development Boundary

- Modify game truth in `engine/src/lib.rs`.
- Modify browser input, HUD, and pixels in `web/asteroids.ts`.
- Modify presentation styling in `web/styles.css`.
- Do not move collision, score, lives, waves, or game-over behavior into
  TypeScript.

The eventual website integration should register this consumer under the
shared declarative island lifecycle rather than creating a second loader.
