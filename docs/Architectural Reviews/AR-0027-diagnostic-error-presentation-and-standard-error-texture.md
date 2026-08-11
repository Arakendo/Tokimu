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

