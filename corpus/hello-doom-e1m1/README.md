# E1M1 Static Presentation Corpus

This corpus consumer prepares a bounded, static E1M1 scene from the reviewed
compact Doom package. It keeps WAD interpretation and source identities at the
corpus edge, then submits only ordinary textured meshes and materials to
Tokimu's renderer.

Run these commands from the repository root:

```powershell
# Headless preparation report: source omissions, texture/material inventory,
# and renderer-neutral draw count.
cargo run -p hello-doom-e1m1 --bin hello-doom-e1m1 -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD

# Native first-frame overview of the prepared opaque static scene.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD

# Corpus-only AR-0023 cutout comparison. This adds the retained masked-middle
# candidates after the unchanged opaque scene using a local binary-alpha shader.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --masked-cutouts
```

If invoked from this `corpus/hello-doom-e1m1` directory instead, replace the
package path with `../assets/DOOM/packages/doom-shareware-corpus-v1.zip`.

The default native overview intentionally does not claim original Doom
plane-span rendering, visibility, gameplay, sky rendering, masked-middle
rendering, or general alpha blending. `--masked-cutouts` is a separate
corpus-only AR-0023 path: it declares binary coverage as discard alpha `<= 0`
with depth writes and does not change the default opaque scene or admit a
renderer alpha API.
