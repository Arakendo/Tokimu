# FBX Corpus Fixtures

This directory contains a deliberately selected subset of the public `ufbx`
repository test data. It is evidence for FBX importer development, not an
official Autodesk conformance suite and not Tokimu's canonical model format.

## Pinned Source

- Repository: `https://github.com/ufbx/ufbx.git`
- Revision: `fcc5d6ba444cfd3eb80677dba5e37e493941abe5`
- Tag at selection: `v0.23.0`
- License used for redistribution: MIT alternative in upstream `LICENSE`

The upstream repository is dual-licensed under MIT or the Unlicense. Tokimu
retains the complete upstream notice and selects the MIT alternative for these
fixtures.

## Layout

```text
upstream/
    LICENSE
    data/
        selected FBX inputs
        selected OBJ reference outputs
selected/
    selection-v1.toml
    feature-matrix.md
```

The files under `upstream/` are byte-identical copies from the pinned revision.
The selection manifest records every FBX source and dependency checksum.

## Scope

Selection v1 contains:

- 23 FBX cases;
- 14 logical scenes;
- 10 ASCII and 13 binary encodings;
- Blender, Maya, 3ds Max, and synthetic failure cases;
- FBX versions 5800, 6100, 7400, 7500, and 7700;
- 13 OBJ reference artifacts used only as optional structural evidence.

The selection covers minimal cubes, version and encoding pairs, big-endian
binary data, UV sets, instancing, axis variants, materials, Unicode names,
animation, skinning, blend shapes, and expected-invalid input.

The complete upstream repository contains substantially more exporter and fuzz
coverage. It is intentionally not vendored. Run the preparation script when a
fresh byte-identical copy of the selected files is needed:

```powershell
pwsh -NoProfile -File .\scripts\prepare-ufbx-fbx-corpus.ps1
```

Ordinary tests and verification never access the network:

```powershell
pwsh -NoProfile -File .\scripts\verify-ufbx-fbx-corpus.ps1
```

## Boundary

These fixtures may shape an FBX importer and provider-neutral imported-model
evidence. They do not admit `ufbx` types, FBX object graphs, transform quirks,
or material classes into Tokimu engine contracts.
