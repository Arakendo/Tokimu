# Hello Render Resource Identity Web

Browser/WASM pressure for the corpus-local AR-0024/AR-0027 resource identity
alternatives. It repeats the application-owned, generational, and explicit
lifecycle cases and then uploads two different meshes to one existing WGPU
mesh handle to retain the provider's intentional replacement behavior.

The returned DOM record remains caller-owned after the renderer returns. This
does not admit a shared identity registry, terminal-record owner, diagnostic
store, or renderer fallback policy.

The separate **Run 27 whole-backend replacements** control is the independent
Alternative-A baseline for the scene-resource lifetime study. It retains one
Rust/WASM session, but each replacement drops the previous backend and creates
a fresh device and surface for the same canvas before uploading 64 meshes, 64
sampleable textures, 64 materials, one pipeline, and one camera. Its bounded
record distinguishes logical retirement from physical GPU reclamation, which
remains unobserved. This is deliberately not a reset, arena, generation, or
release contract.

Build and serve the fixture from the repository root:

```powershell
cargo build -p hello-render-resource-identity-web --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/hello-render-resource-identity-web.wasm --target web --out-dir corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/web/pkg
python -m http.server 4177 --directory corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/web
```

Open `http://127.0.0.1:4177`, then select **Run browser identity fixture**.
