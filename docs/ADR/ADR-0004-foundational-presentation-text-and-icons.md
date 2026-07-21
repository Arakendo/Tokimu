# ADR-0004: Foundational Presentation Text and Icons

## Status

Accepted

## Context

Every Tokimu application eventually needs to communicate information to a
person, a tool, or another presentation target. Native UI, debug overlays,
diagnostics, launchers, game menus, terminal adapters, headless reports, and
future web frontends all need text. Many also need iconography.

Recent UI, font, SVG, and Lucide corpus examples showed that application code
should not need to understand font formats, glyph rasterization, SVG parsing,
or renderer buffers merely to request a title, warning, body label, or save
icon. Those concerns otherwise fragment application semantics and cause each
frontend to rebuild measurement, baseline layout, fallback behavior, and
diagnostics differently.

At the same time, font parsers, operating-system font discovery, icon
libraries, shaping engines, GPU atlases, and renderer backends are specialized
and replaceable. Admitting them to `tokimu-core` would violate the small trusted
core and capability ownership discipline established by ADR-0001 and ADR-0003.

## Decision

Text and icon semantics are part of Tokimu's **foundational presentation
capability**. The trusted core remains small.

> Applications communicate intent. Providers communicate implementation.

The ownership model is:

```text
Trusted core
    world identity and state, time, commands, handles, diagnostics, versions

Foundational presentation capability
    text and icon semantics, measurement, layout, handles, fallback policy,
    diagnostics, renderer-neutral draw requests

Replaceable providers and renderer backends
    TTF/OTF/system-font parsing, icon libraries such as Lucide, shaping,
    rasterization, atlas packing, GPU upload, and draw submission
```

The foundational text contract owns provider-neutral concepts including:

- text specifications and semantic roles;
- opaque font handles and provider-qualified font identity;
- text measurement, advances, baseline, ascent, descent, line gap, and visible
  bounds;
- line layout, wrapping, clipping, alignment, and spacing policy;
- glyph placement and renderer-neutral draw requests;
- explicit fallback policy and structured missing-font or missing-glyph
  diagnostics.

The foundational icon contract owns:

- semantic icon identity, including a provider-qualified escape hatch for
  project-local icons;
- icon measurement, sizing, layout, color inheritance, and diagnostics;
- renderer-neutral icon draw requests.

TTF, OTF, system-font, WOFF2, Lucide, project-local icon, rasterizer, shaping,
atlas, and renderer implementations are providers or backends. They resolve
external technology into Tokimu-owned contracts and may be substituted without
changing application meaning.

Public text and icon contracts expose opaque Tokimu handles and identities. No
parser-native font object, SVG document, icon-library object, atlas allocation,
or renderer-native object may leak through the author-facing API.

Measurement and layout must be usable without a window, GPU, or live renderer
when supplied with an appropriate metrics-capable provider. Rendering remains
optional execution of the resulting draw requests.

Provider resolution and failure are explicit and diagnostic:

```text
request -> provider resolution -> success | configured fallback | diagnostic
```

There is no silent ambient system-font substitution or silent provider switch
that changes a document's presentation meaning without inspection evidence.

The intended dependency direction is:

```text
tokimu-core
    depends on no presentation/provider/renderer implementation

tokimu-text and tokimu-icon
    depend on tokimu-core

tokimu-font-* and tokimu-icon-*
    depend on the corresponding semantic capability

text/icon renderer adapters
    depend on semantic capabilities and rendering contracts
```

The names above describe ownership direction, not an immediate crate split.
`examples/lib-example/ui-tools` remains the evidence layer while the APIs are
proven by independent corpus examples.

## Current Implementation Status

The evidence-layer implementation now contains prototype contracts for:

- provider-qualified font identity and generation-checked font handles;
- provider-neutral text measurements, line baselines, alignment, overflow,
  and metrics-only headless providers;
- explicit missing-glyph policy and text diagnostics;
- provider-qualified icon identity and generation-checked icon handles;
- explicit missing-icon and provider-unavailable diagnostics;
- backend-neutral vector icon assets and vector-provider resolution;
- renderer-neutral text draw requests.

These contracts are intentionally still example-side. The current decision is
to continue incubation in `ui-tools`; extraction into `tokimu-text` and
`tokimu-icon` waits for an independent consumer and a more complete provider
and shaping story. This status does not weaken the architectural admission of
presentation semantics; it records that the implementation boundary has not
yet earned a first-party crate split.

## Graduation Trigger

`tokimu-text` and `tokimu-icon` graduate from `ui-tools` when:

- at least one non-example consumer depends on the semantic contracts;
- no provider implementation details leak through the public API;
- the corpus remains stable across provider implementations;
- the new consumer exposes no architectural boundary requiring redesign.

Crate extraction is intentionally deferred because moving code is inexpensive
compared with stabilizing semantics. Breaking changes to semantic contracts
require architectural review; provider implementation changes do not, provided
they preserve the semantic contracts and declared compatibility expectations.

## Consequences

Applications gain a stable way to request and lay out text and icons without
coupling to a particular font family, font file format, icon library, parser,
or graphics backend. This supports native, headless, terminal, and future web
presentation paths with one semantic vocabulary.

Tokimu must now preserve provider-neutral handle, measurement, layout,
diagnostic, and fallback contracts as they mature. Provider implementations may
evolve independently only when they preserve those contracts and their declared
compatibility.

This decision adds a deliberate boundary between semantic layout and execution.
That produces more explicit provider resolution and failure handling, but avoids
putting specialized dependencies into the trusted core.

The following remain deferred until corpus evidence and real consumers require
them:

- final crate graph and package names;
- a specific TTF/OTF parser, shaping engine, or rasterizer;
- glyph atlas strategy and GPU upload policy;
- bundled default font or icon assets;
- kerning, complex shaping, bidi/RTL, rich text, and text editing;
- WOFF2 decoding and system-font discovery policy.

## References

- ADR-0001 Engine Boundaries — `docs/ADR/ADR-0001-engine-boundaries.md`
- ADR-0003 Capability Ownership Boundary —
  `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- Kernel Principles — `docs/kernel-principles.md`
- Foundational Presentation Text and Icon TODO —
  `.workbench/Todos/foundational-presentation-text-icon.md`
- Text Presentation Corpus v1 — `.workbench/Todos/text-presentation-corpus-v1.md`
