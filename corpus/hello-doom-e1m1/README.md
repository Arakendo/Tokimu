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

# ADR-0013 categorical-cutout evidence. This adds the retained masked-middle
# candidates after the unchanged opaque scene using the admitted generic path.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --masked-cutouts

# Fixed source-spawn observer for a normal native screen capture. It maps the
# reviewed player-one THING position and heading into the corpus X/Z world and
# uses the containing sector's vertical midpoint; it is not movement or player
# eye-height policy.
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --spawn-observer
```

If invoked from this `corpus/hello-doom-e1m1` directory instead, replace the
package path with `../assets/DOOM/packages/doom-shareware-corpus-v1.zip`.

The default native overview intentionally does not claim original Doom
plane-span rendering, visibility, gameplay, sky rendering, masked-middle
rendering, or general alpha blending. `--masked-cutouts` is a separate
ADR-0013 path: Doom selects only retained masked-middle candidates, then
declares categorical coverage as discard alpha `<= 0` through Tokimu's generic
cutout capability. It does not change the default opaque scene or admit Blend.

`--spawn-observer` is a fixed evidence camera. For reviewed E1M1 it reports
THINGS record `0`, source position `(1056, -3616)`, angle `90`, sector `38`,
and raw floor/ceiling interval `0..72`; the camera uses the interval midpoint
(`y = 36`) rather than claiming an original-Doom player-height policy.
