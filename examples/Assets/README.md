# Example Assets

This directory contains visual reference assets for the examples workspace.
The PNG files are texture data intended for application to meshes and shader
materials, not UI screenshots or widget artwork.

The visible corpus is especially useful for calibration and diagnostics:

- `texture_01` and `texture_03` provide major grid and center-line guides.
- `texture_04` and `texture_05` exercise diagonal/grid alignment.
- `texture_06` provides repeated crosshair targets.
- `texture_07` and `texture_08` exercise checker and block patterns.
- `texture_09` provides a low-contrast variant of the target pattern.
- `texture_10` through `texture_13` provide labeled or geometric scale
  references such as stairs, doors, windows, and walls.

Color variants are useful for checking material selection, tinting, texture
sampling, filtering, and shader color handling.

## Layout

```text
Assets/
  Vector/              SVG source/reference files
  PNG/
    Dark/              dark texture variant
    Light/             light texture variant
    Red/               red texture variant
    Orange/            orange texture variant
    Green/             green texture variant
    Purple/            purple texture variant
```

The current corpus contains 13 SVG files and 78 PNG files. PNG variants use
the same `texture_01` through `texture_13` naming scheme as the vector files,
where available.

## Usage

Examples should resolve these files from the repository root or an explicit
asset-root configuration. They should not depend on the current working
directory or copy assets into individual example directories.

When an example needs one of these assets, use the PNG variants for mesh or
shader material tests. Use the SVG files when testing vector loading,
triangulation, or vector-to-mesh behavior.

## Reference-corpus status

These files are currently treated as example/reference data, not as a stable
Tokimu asset-provider contract. Before promoting them into a shared fixture or
provider, record their source, license, dimensions, color-space assumptions,
and any required alpha/premultiplication behavior.
