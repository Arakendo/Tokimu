# Textured Box Native Manual Observation

| Field | Value |
| --- | --- |
| Date | 2026-08-09 |
| Target | Native WGPU window |
| Observer | Project maintainer |
| Consumer | `hello-textured-box` |
| Result | Passed manual composition observation |

## Observed

The pinned Khronos Box rendered with the independent first-party PNG texture
visibly applied through the supplied-UV `Textured3d` path.

After the fixture UV scale was deliberately changed to `3.25`, the sampler
comparison became visibly meaningful:

- clamp holds edge texels for out-of-range coordinates; and
- repeat tiles the grid texture across the Box faces.

This is native visual evidence of the supplied UV, normalized RGBA8 upload,
material texture binding, sampler declaration, back-face-culling, and WGPU
presentation composition. It is not browser evidence, pixel equivalence, a
PNG decoder conformance result, a glTF material-import result, or an
alpha/cutout result.

## Reproduction Controls

- `M`: grid, dark-door, green-door.
- `R`: point/clamp, point/repeat, linear/clamp, linear/repeat.
- `X`: UV identity, U-flip, U/V swap.

The window title reports the selected texture, sampler mode, and UV mode.

## Alpha Boundary

The selected PNG fixtures contain no transparency. `Textured3d` uses the
existing explicit pipeline blend policy; the present native observation uses
the opaque profile. Cutout/alpha-test behavior is intentionally unsupported
until a separate first-party alpha source and a reviewed policy are admitted.
