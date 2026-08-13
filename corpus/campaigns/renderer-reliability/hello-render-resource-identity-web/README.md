# Hello Render Resource Identity Web

Browser/WASM pressure for the corpus-local AR-0024/AR-0027 resource identity
alternatives. It repeats the application-owned, generational, and explicit
lifecycle cases and then uploads two different meshes to one existing WGPU
mesh handle to retain the provider's intentional replacement behavior.

The returned DOM record remains caller-owned after the renderer returns. This
does not admit a shared identity registry, terminal-record owner, diagnostic
store, or renderer fallback policy.

Build and serve the fixture from the repository root:

```powershell
cargo build -p hello-render-resource-identity-web --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/hello-render-resource-identity-web.wasm --target web --out-dir corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/web/pkg
python -m http.server 4177 --directory corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/web
```

Open `http://127.0.0.1:4177`, then select **Run browser identity fixture**.
