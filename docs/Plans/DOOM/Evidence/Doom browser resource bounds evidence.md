# Doom Browser Resource Bounds Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Consumer | `doom-ts-boundary-workbench` |
| Scope | Corpus-private browser startup, decode, and working-model submission budgets |

## Enforced Boundaries

The browser consumer rejects work at explicit boundaries it controls:

- selected package: 64 MiB;
- archive traversal: 2,048 entries, 16 MiB per member, and 64 MiB total;
- WAD: 8,192 lumps, 16 MiB per lump, and 64 MiB total;
- map tables: 100,000 records per principal family, 64 MiB total record bytes,
  and explicit REJECT/BLOCKMAP limits;
- raster globals: 128 MiB total decoded bytes;
- patches/composed textures: 4,096 by 4,096 and 16,777,216 pixels per
  bounded decode/composition;
- browser working-model realization: 20,000 meshes, 2,048 textures, 2,050
  materials, 16 pipelines, one camera, and 100,000 commands;
- estimated working-model payloads: 64 MiB of mesh vertices and 128 MiB of
  source RGBA texture payloads.

The working-model budget is checked after complete CPU preparation but before
the previous retained provider scene is reset or any successor resources are
uploaded. The immutable command list is checked again using its exact count
before the frame is installed. Each rejection names the resource, observed
value, and limit.

`build.ps1` also measures the six uncompressed emitted startup files actually
referenced by the page and rejects a total over 12 MiB. The validated debug
build reported:

```text
Browser startup payload: emitted-bytes=6117127; limit=12582912; files=6; transfer-compression=unmeasured
```

The measured files are the HTML entry, three application JavaScript modules,
the generated WASM JavaScript bridge, and the WASM module. Type declarations
and user-selected WAD bytes are not startup payload.

## Validation

- `cargo test -p doom-ts-boundary-workbench-engine`: 5 passed, including exact
  acceptance at every working-model limit and named rejection above each
  limit;
- `cargo check -p doom-ts-boundary-workbench-engine --target wasm32-unknown-unknown`;
- `pwsh -NoProfile -File corpus/consumers/doom-ts-boundary-workbench/build.ps1`;
- strict TypeScript build as part of `build.ps1`.

These are bounds on Tokimu-controlled inputs, decoded payloads, logical
resources, and provider-submitted source bytes. They are not a promise that
Edge, WGPU, a driver, or a GPU consumes exactly that amount of physical
memory. Physical browser/GPU residency and reclamation remain unavailable and
must not be reported as zero.
