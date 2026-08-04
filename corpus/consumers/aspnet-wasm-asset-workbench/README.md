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

For an external-buffer `.gltf`, select the document and its same-folder buffer
files together. The browser transfers only explicit file names and bytes;
Tokimu's transient resource session resolves admitted dependencies in Rust/WASM.
Nested paths, host-directory traversal, image decoding, and texture upload
remain outside this first consumer proof.

After inspection, **Download selected source** returns the selected logical
resource bytes through Rust/WASM. The browser owns the user-initiated download
gesture; Resource Space never retains browser file handles or host paths.

The inspector identifies the chosen document and every sidecar before
inspection, then displays the bounded Rust session summary after Tokimu
resolves it. For the external-buffer proof, select both `Box.gltf` and
`Box0.bin` in one chooser or drag/drop operation.

### Resource Space Browser Evidence

Use this bounded manual scenario when validating the generated browser build:

1. Start the workbench with the build instructions above.
2. Select or drop `Box.gltf` and `Box0.bin` together from the known-good
   fixture pair below.
3. Confirm the workbench identifies `Box.gltf` as the document and `Box0.bin`
   as a same-folder sidecar.
4. Confirm the inspection succeeds as `gltf`, reports one resolved external
   buffer, and appends a Rust session summary containing `resources=2`.

This records browser delivery of explicit names and bytes only. The browser
must not parse glTF, resolve `Box0.bin`, or retain provider-native objects.
If the run fails, preserve the visible Resource Session diagnostics rather
than substituting browser-native glTF behavior.

Known-good starting fixtures:

- SVG: `third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/derived/shapes-rect-01-geometry.svg`
- CGM: `third-party/fixtures/webcgm-test-suite/upstream/static10/POLYLN01.cgm`
- GLB: `third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb`
- external-buffer glTF: select both `third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF/Box.gltf` and `third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF/Box0.bin`
- FBX: `third-party/fixtures/fbx-corpus/upstream/data/maya_cube_7500_binary.fbx`

SVG and CGM currently preview provider-neutral contours. GLB and admitted
static FBX geometry preview through an interactive diagnostic perspective view:
drag to orbit, use the mouse wheel to zoom, and press `R` to reset.

Generated TypeScript and WASM bindings under `wwwroot/app` and
`wwwroot/tokimu` are intentionally ignored.
