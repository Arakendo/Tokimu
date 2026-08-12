# AR-0027: Diagnostic Error Presentation And Standard Error Texture

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-10 |
| Last reviewed | 2026-08-10 |
| Scope | Explicit diagnostic presentation for missing, rejected, or failed visual resources and shader/material paths |
| Trigger | The E1M1 corpus currently omits sky-classified ceiling surfaces honestly, but a black clear/background can resemble missing geometry, failed texture resolution, or shader failure during visual inspection. |
| Related ADRs | ADR-0004, ADR-0007, ADR-0008, ADR-0009, ADR-0012, ADR-0013 |
| Related reviews | AR-0006, AR-0022, AR-0023, AR-0024, AR-0025 |
| Admission exception | None |

## Architectural Question

Should Tokimu admit a standard, provider-neutral diagnostic presentation for
explicit visual failure or omission states—potentially including a conspicuous
error texture/material—and if so, who owns its meaning, construction, fallback
behavior, and presentation limits?

The question is not whether a renderer should silently substitute an asset. It
is whether an application or corpus can deliberately request an unmistakable
diagnostic representation while retaining the original source identity and
failure reason.

## Trigger And Retained Evidence

The static E1M1 corpus has successfully prepared all currently eligible flat
and wall textures. Its exterior black regions instead arise from 74 explicitly
sky-classified ceiling observations that the first static policy intentionally
omits. The visual result is ambiguous to a maintainer: black can also suggest a
missing texture, missing mesh, shader/pipeline failure, or a normal background.

`corpus/assets/PNG/Purple/texture_01.png` is an existing conspicuous corpus PNG
candidate for an opt-in diagnostic stand-in. It is evidence machinery only; its
presence does not make purple checkerboard pixels a Tokimu rendering semantic.

## Ownership Constraints

- The source/application layer owns classification of the condition: e.g.
  source sky omission, missing asset, rejected material declaration, or a
  diagnostic request from a corpus fixture.
- The renderer must not infer failure from absent bytes, alpha, a WAD term, or
  arbitrary shader behavior, then silently replace it with a standard texture.
- Provider adapters may report bounded failures through ADR-0007/AR-0024
  mechanisms, but must not decide source-level recovery policy.
- A diagnostic representation must preserve the original identity and reason
  beside the replacement. Rendering a stand-in does not convert failure into
  successful asset resolution.
- Any native standard asset must meet ADR-0010 provenance requirements and
  remain available without network resolution. A generated pattern may avoid
  asset provenance but creates a different shader/material question.

## Alternatives

### A. Corpus-Local Debug Texture Only

Each corpus consumer may explicitly load an existing checked PNG and bind it
to known diagnostic surfaces. This is the immediate E1M1 experiment. It keeps
the renderer unchanged but may duplicate vocabulary and diagnostics across
consumers.

### B. Tokimu-Owned Diagnostic Presentation Intent With Caller-Supplied Asset

Admit a small semantic request such as `DiagnosticVisual { reason, original }`,
while the application/provider supplies the concrete texture or material. This
separates meaning from mechanism, but needs independent callers before a public
intent is justified.

### C. Tokimu-Owned Standard Error Material/Texture

Ship a specified, provenance-controlled diagnostic texture and ordinary
material profile. This gives consistent visual evidence across native/WASM but
creates asset, color-space, sampling, packaging, and compatibility obligations.

### D. Renderer-Automatic Fallback

Reject initially. A renderer that silently replaces unavailable or invalid
resources can hide lifecycle, authority, source, or shader failures and makes
ordinary rendering appear successful.

### E. Backend-Specific Shader Error Screens

Defer. Backend compiler diagnostics and shader tooling belong to providers and
do not establish a provider-neutral material policy.

## Initial Corpus Plan

1. Add an opt-in E1M1 diagnostic-omission mode using the retained Purple PNG.
   It must be visually unmistakable and must report every replaced source
   identity plus reason.
2. Keep normal E1M1 presentation unchanged; no fallback occurs without the
   explicit diagnostic mode.
3. Exercise at least two distinct conditions: an intentional source omission
   (sky) and a deliberately injected unavailable/rejected visual resource.
4. Repeat on native WGPU and browser/WASM when feasible. Retain source,
   build, backend, and adapter metadata; do not claim pixel identity.
5. Compare a second independent caller before proposing any public intent or
   bundled standard material.

## Acceptance Clamp

No Tokimu-wide admission is justified unless evidence shows all of the
following:

- callers can explicitly distinguish diagnostic presentation from successful
  asset/material resolution;
- original source identity and a bounded reason survive presentation;
- normal renderer behavior does not silently fall back;
- provenance, offline packaging, native/WASM availability, color-space, and
  sampler behavior are established for any bundled asset;
- at least two independent callers need the same semantic capability; and
- ADR-0008/0009 evidence addresses steady-state cost, failure containment, and
  recovery behavior.

## Current Disposition

Begin with Alternative A as a strictly corpus-local E1M1 diagnostic experiment.
Do not admit a renderer fallback, standard texture, public diagnostic-material
API, shader error screen, or source-format-specific visual policy.

## 2026-08-11 Interactive-Door Follow-Up

The Slice 8 dynamic-door experiment supplied an additional, distinct failure
case. Upon ordinary player activation, presentation attempted to re-lower the
affected source wall spans from immutable decoded-map data. The first attempt
failed because `BRNBIGL` was present in the source texture catalog but outside
the static scene's uploaded subset; the provider therefore could not resolve a
source wall extent and returned an explicit diagnostic.

This is not a missing renderer texture: no texture binding was requested and
no shader or WGPU validation failure occurred. It is a source-derived geometry
preparation failure. It nevertheless supports this review's central constraint:
a black gap, a suddenly closed native window, and a diagnostic stand-in would
all make materially different claims.

The corpus repair retains the full source texture-extent catalog as geometry
metadata while continuing to upload only selected eligible rasters. It does
not substitute `BRNBIGL`, bind a fallback texture, or treat the source resource
as successfully prepared. Separately, the native observer retains a bounded
console/stderr diagnostic for recoverable dynamic-door refresh failure rather
than immediately exiting. That containment gap is also direct evidence for
AR-0024.

This adds pressure for Alternative A's explicit corpus-local diagnostic mode,
but does not establish that a standard error texture represents geometry
preparation failure. Future experiments must preserve:

```text
source geometry preparation failed
    != texture bytes unavailable
    != material rejected
    != shader/pipeline failure
    != intentional source omission
```

### Dynamic Resource Identity Follow-Up

The next repair exposed a related but more fundamental failure shape. Newly
materialized `DOORTRAK` spans initially reused mesh-handle values derived from
the mutable opaque draw count. That changed the base of already-uploaded cutout
handles, causing an ordinary door activation to invalidate presentation
resources and close the native observer without an in-window explanation.

The immediate corpus repair reserves disjoint static-opaque, static-cutout,
and dynamic-door mesh-handle ranges. This is deliberately local evidence, not
a claim that numeric ranges are a Tokimu-wide lifetime model. It raises the
following question for future kernel/renderer review:

> Can Tokimu provide a small, explicit resource-identity/allocation discipline
> that makes a live resource's handle stable across ordinary application-side
> dynamic additions, and reports collisions or unresolved handles as bounded
> diagnostics rather than allowing a presentation loop to terminate silently?

Any future admission must preserve application ownership of draw lifetime,
source identity, and recovery policy. A kernel capability may prevent invalid
handle reuse or make it observable; it must not infer that a missing Doom span,
texture, shader, or provider resource should be replaced automatically. The
question also belongs with AR-0024's failure-observation boundary and should
not be answered by adding more corpus-specific exceptions.

Comparative work is tracked by the
[Renderer Resource Identity And Failure Presentation Test Plan](../Plans/Tests/renderer-resource-identity-and-failure-presentation.md).
