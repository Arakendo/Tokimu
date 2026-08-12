# Diagnostic Presentation Comparison Evidence

| Field | Value |
| --- | --- |
| Status | Active — Alternative A native sky-omission fixture implemented and visually observed |
| Related review | AR-0027 |
| Related plan | [Renderer Resource Identity And Failure Presentation Test Plan](renderer-resource-identity-and-failure-presentation.md) |
| Scope | Corpus-local comparison only; no renderer fallback or public diagnostic material contract |

## Explicit Sky-Omission Stand-In

`hello-doom-e1m1` now accepts `--diagnostic-sky-omissions`. It is deliberately
separate from the normal scene path:

```text
normal E1M1
    retained sky source observations
    -> no submitted sky draw

diagnostic E1M1 (explicit flag)
    retained sky source observations
    -> re-lower only those surfaces
    -> Purple/texture_01.png stand-in
    -> original subsector / sector / plane plus reason retained beside draw
```

The Purple PNG is read with the corpus raster provider, explicitly prepared as
`ColorSrgb`, and uploaded with its own high, corpus-local texture/material and
mesh-handle range. Its retained SHA-256 is
`0e0c2dfff301d16919ec1c5f977dbae4743f0c840aa11f1d255c2fec3291159e`.
It is not looked up through the WAD, does not change a normal source texture
result, and is never selected automatically.

The fixture prints bounded records of the form:

```text
reason=intentional-source-sky-omission;
original=flat subsector=<n> sector=<n> plane=Ceiling ...;
stand-in=Purple/texture_01.png
```

Run the native evidence path from the workspace root:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --diagnostic-sky-omissions --spawn-observer --spawn-yaw-plus-90
```

## Current Classification

| Meaning | Appropriate presentation now | Stand-in allowed? |
| --- | --- | --- |
| Intentional source sky omission | Explicit corpus Purple surface plus retained record | Yes, only under the flag |
| Source geometry preparation failure | Console/terminal diagnostic | No: a texture would claim geometry exists |
| Missing/rejected renderer resource | Structured record and explicit rejection | Not yet tested; must not fall back automatically |
| Provider failure | Provider/terminal or host status | No: a texture cannot honestly represent provider state |

This is intentionally incomplete. The same failure records have not yet been
compared through every presentation surface, no browser observation has been
made, and no second non-Doom caller exists. It therefore supports only
AR-0027 Alternative A experimentation.

## Native Visual Observation — 2026-08-11

The native E1M1 observer showed the Purple stand-in on retained sky ceilings.
At adjacent black regions, the local `LOOK` command reported:

```text
look: no prepared triangle intersects the center ray
```

This distinguishes the two observations in the same scene:

```text
purple surface
    = explicitly requested diagnostic representation of a retained sky omission

black region + no prepared triangle
    = presentation/geometry coverage is absent at this corpus stage
      (not asserted to be a failed texture binding)
```

The result supports the review's no-automatic-fallback constraint. A global
error texture would have hidden that second, materially different finding.
