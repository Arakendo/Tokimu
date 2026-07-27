# Khronos glTF Sample Assets Fixtures

This directory preserves a pinned, deliberately small selection from the
official Khronos glTF Sample Assets repository for Tokimu importer corpus
work.

Tokimu does not claim complete glTF conformance by carrying these files. The
initial selection validates only source acquisition, glTF JSON structure,
external buffer resolution, and GLB container framing.

## Layout

```text
upstream/Models/       Verbatim selected Khronos model subtrees
upstream/LICENSES/     License texts required by the selected models
selected/              Tokimu selection and feature records
provenance.json        Source repository and pinned revision
```

The complete upstream repository is intentionally not vendored. Showcase
models and textures are much larger than the first importer proof requires.
`scripts/prepare-khronos-gltf-corpus.ps1` acquires the exact selected files
from the pinned revision.

## First Selection

- `Triangle/glTF/Triangle.gltf` plus `Triangle.bin`
- `Box/glTF-Binary/Box.glb`

The complete `Triangle` and `Box` model directories are retained locally so
later source-encoding comparisons can use the same logical models without
changing the v1 coverage numerator.

Model-specific provenance and license declarations remain in each model
directory. Full CC0 and CC-BY-4.0 license texts are under
`upstream/LICENSES/`.

## Validation

```powershell
pwsh -NoProfile -File .\scripts\verify-khronos-gltf-corpus.ps1
cargo test -p gltf-corpus
```

The tests stop at structural source inspection. They do not yet prove glTF
accessor decoding, Tokimu mesh lowering, or rendering.
