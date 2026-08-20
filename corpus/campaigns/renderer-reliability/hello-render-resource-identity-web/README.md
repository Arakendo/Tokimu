# Hello Render Resource Identity Web

Browser/WASM pressure for the AR-0024/AR-0032 resource identity and replacement
alternatives. ADR-0018 now admits the narrow set-level replacement semantics,
while this fixture continues to pressure the provisional implementation. It
also uploads two different meshes to one existing WGPU mesh handle to retain
the provider's intentional in-set replacement behavior.

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

The feature-gated **Run 27 retained-session replacements** path is the private
Alternative-B comparison. It retains one adapter/device/queue/surface while
clearing and rebuilding the complete logical resource set. Two deliberately
adversarial probes follow it:

- **Probe retained-session stale aliasing** submits handles originating in the
  earlier set after the successor reused the same numeric values;
- **Probe retained-session atomicity** forces successor staging to fail after
  reset, then asks whether the preceding scene can still resolve.

These are falsification instruments. A successful reset cycle does not make
the experimental reset seam stable, generational, atomic, or proof of physical
GPU reclamation.

The **Run Alternative C semantic prototype** control is deliberately narrower
than either WGPU pressure path. Pure Rust/WASM commits an E1M1 logical
generation A, injects failure while staging E1M2 generation B, proves A remains
usable, commits a complete B, then proves A's retained handle is stale while
the same local resource key resolves B. It exercises no renderer resources,
provider session, or physical reclamation and admits no engine API.

The feature-gated **Probe Alternative C real-provider staging** control is the
next, separately bounded experiment. It creates one WGPU provider session,
presents resource set A, allocates most of B alongside A, injects a late B
failure, and proves A still presents. Before the second complete B commits, the
fixture retains a set-scoped batch containing A's real draw commands. B reuses
A's local mesh, material, pipeline, and camera keys. After the one-swap commit,
the provider must reject the retained A batch as stale before resolving any of
those handles, while B's scoped batch must still present. The record does not
claim when WGPU physically reclaims A, quantify overlap memory, define public
generation handles, or exercise repeated replacement pressure.

The separate **Run 27 Alternative C staged replacements** control holds that
mechanism fixed and alternates two 64-mesh/texture/material sets on one browser
WGPU session. Every fifth replacement first stages a complete candidate and
then forces a missing-texture material failure; the preceding set must present
all 64 draws before the valid candidate is retried. JavaScript yields across
animation frames between replacements. Each record requires the same logical
inventory after commit, reports the bounded current-plus-candidate source-byte
estimate during overlap, and keeps physical GPU reclamation explicitly
unobserved.

The whole-backend pressure path also correlates each successfully presented
64-resource-family inventory with its predecessor through the same semantic
shadow. The returned record labels this as correlation rather than provider
lifetime evidence: existing WGPU staging remains unchanged and non-atomic.

Build and serve the fixture from the repository root:

```powershell
cargo build -p hello-render-resource-identity-web --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/hello-render-resource-identity-web.wasm --target web --out-dir corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/web/pkg
python -m http.server 4177 --directory corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/web
```

Open `http://127.0.0.1:4177`. Run the whole-backend and retained-session
sequences on fresh pressure objects, then run the stale-aliasing probe before
the destructive atomicity probe. The staged replacement/stale-command probe is
independent and may be run directly on its own fresh pressure object. Its
successful record must include `stale-rejected-before-resource-resolution=true`
and equal nonzero B draw counts before and after scoped submission.
