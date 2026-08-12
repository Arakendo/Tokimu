# Diagnostic Presentation Comparison Evidence

| Field | Value |
| --- | --- |
| Status | Active — native shared-record comparison and sky-omission fixture retained; browser and second-caller pressure remain open |
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
| Missing/rejected renderer resource | Structured record and explicit rejection | Shared-record fixture only; must not fall back automatically |
| Provider failure | Provider/terminal or host status | Shared-record fixture only; no texture can honestly represent provider state |
| Terminal startup rejection | Invoking caller's terminal result; first failure retained | No: ordinary frame rendering never became valid |

This is intentionally incomplete. The same failure records have not yet been
compared through every presentation surface, no browser observation has been
made, and no second non-Doom caller exists. It therefore supports only
AR-0027 Alternative A experimentation.

The E1M1 `--doom-sky` startup rejection also demonstrates the negative half of
the comparison. A source raster with unresolved coverage cannot truthfully use
the Purple stand-in or proceed to a frame. The native composition ends and the
invoking terminal caller receives the original `SKY1` failure. A secondary
missing-pipeline error is now prevented from replacing it.

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

## Shared-Record Presentation Comparison — 2026-08-11

`hello-render-resource-identity` now presents the same four bounded,
chronologically retained records in two non-visual forms: a structured record
and a console line. The comparison deliberately formats only existing facts:
sequence, phase, operation, category, resource identity when present, caller,
and continuation. It does not create a provider-neutral error text or recovery
contract.

| Meaning | Shared record result | Structured/console presentation | Visual stand-in |
| --- | --- | --- | --- |
| Intentional source omission | `SourcePreparation / IntentionalSourceOmission`, caller `e1m1-sky-omission` | Both retain the original source-side category and continuation | Allowed only as an explicit application choice; the native Purple sky fixture remains the one observed case |
| Source geometry failure | `SourcePreparation / SourceUnavailable`, caller `e1m1-door-refresh` | Both report preparation failure rather than a resolved texture or mesh | Not allowed; a texture would assert geometry that was not prepared |
| Missing renderer resource | `RendererResourceResolution / ResourceUnresolved`, `MeshHandle(44)` | Both retain the unresolved typed resource identity | Not allowed; no resource was resolved |
| Provider failure | `ProviderValidation / ProviderRejected`, caller `hello-shader-backend-diagnostic` | Both retain terminal continuation without inventing a shader screen | Not allowed; provider state cannot honestly be represented as a scene material |

The executable output is retained by the `diagnostic_presentation_keeps_one_record_but_separates_visual_claims` test and the deterministic corpus binary:

```powershell
cargo run -p hello-render-resource-identity --bin hello-render-resource-identity
```

The bounded observation store additionally sorts retained entries by sequence
before presentation. After capacity wraparound, this prevents a console or
structured view from reversing the observed causal order. This is a corpus
fixture property, not a proposal for a global diagnostic store.

The comparison completes the native non-visual alternatives for Slice 5. It
does not claim browser/WASM parity, a shared terminal-record owner, a bundled
Tokimu error material, or a public diagnostic-presentation vocabulary.
