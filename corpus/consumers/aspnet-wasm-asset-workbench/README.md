# Tokimu ASP.NET WASM Asset Workbench

This Tier 2 consumer corpus entry hosts a TypeScript browser shell in ASP.NET
10 and consumes a corpus-local Tokimu WASM adapter.

Prerequisites:

- .NET 10 SDK;
- Node.js and `npm`;
- Rust's `wasm32-unknown-unknown` target;
- the `wasm-bindgen` CLI.

Build and run:

```powershell
npm install
pwsh -NoProfile -File .\build.ps1
dotnet run
```

Publish a self-contained consumer build into its local `publish/` folder:

```powershell
pwsh -NoProfile -File .\publish.ps1
```

Open the URL printed by ASP.NET and drop an `.svg`, `.cgm`, `.gltf`, `.glb`, or
`.fbx` file into the workbench.

Known-good starting fixtures:

- SVG: `third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/derived/shapes-rect-01-geometry.svg`
- CGM: `third-party/fixtures/webcgm-test-suite/upstream/static10/POLYLN01.cgm`
- GLB: `third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb`
- FBX: `third-party/fixtures/fbx-corpus/upstream/data/maya_cube_7500_binary.fbx`

SVG and CGM currently preview provider-neutral contours. GLB and admitted
static FBX geometry preview through an interactive diagnostic perspective view:
drag to orbit, use the mouse wheel to zoom, and press `R` to reset.

Generated TypeScript and WASM bindings under `wwwroot/app` and
`wwwroot/tokimu` are intentionally ignored.
