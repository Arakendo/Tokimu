# AR-0012: Bundled Native Default Font Provider

| Field | Value |
| --- | --- |
| Status | Accepted |
| Opened | 2026-08-04 |
| Last reviewed | 2026-08-04 |
| Scope | Foundational presentation provider |
| Trigger | Native consumers need a complete, deterministic everyday font without expanding the bootstrap bitmap alphabet indefinitely |
| Related ADRs | ADR-0004, ADR-0005 |
| Related evidence | `hello-ui-text`, `hello-ui-font`, `hello-ui-font2`, `hello-ui-text-vectors`, UI tools tests, pinned Departure Mono fixture |
| Admission exception | Provisional admission under ADR-0005 |

## Architectural Question

Should Tokimu provisionally select and bundle Departure Mono as its first-party
native default font provider while keeping text semantics provider-neutral and
font technology outside the trusted kernel?

## Context

ADR-0004 already admits text meaning, measurement, layout, fallback, and
diagnostics as foundational presentation semantics. It deliberately keeps font
files, TTF/OTF parsers, rasterizers, atlases, and renderer resources in
replaceable providers.

Tokimu's hand-built bitmap font has been valuable as a deterministic bootstrap
and emergency diagnostics path. It is not a sustainable everyday alphabet:
adding broad punctuation, symbols, scripts, and typographic refinements by hand
would duplicate mature font-provider work and make a fallback implementation a
hidden owner of product typography.

The current font corpus now contains a pinned, OFL-licensed Departure Mono OTF
fixture alongside Inter, JetBrains Mono, and Noto. The provider-neutral raster,
metrics, layout, and vector-outline paths already consume these fixtures.

## Trigger And Evidence

- Corpus examples: `hello-ui-text`, `hello-ui-font`, `hello-ui-font2`, and
  `hello-ui-text-vectors` exercise native labels, prose, punctuation, metrics,
  rasterization, and vector outlines.
- Automated tests: `ui-tools` tests exercise provider identity, metrics,
  fallback diagnostics, layout, and the canonical Departure fixture choice.
- Audits or diagnostics: repeated native UI work showed that incomplete glyph
  coverage is visible application friction rather than kernel meaning.
- Independent consumers: runtime inspector, CGM inspector, native UI corpus,
  and font corpus all require readable native text, though most still consume
  the incubating `ui-tools` implementation.
- Repeated implementation friction: extending the bitmap alphabet competes
  with provider work and still cannot provide the quality or coverage of a
  real font.
- Missing evidence: final native/WASM packaging cost, broad DPI and
  accessibility review, complex-script fallback, and a second non-corpus
  consumer selecting the default through a stabilized public capability.

## Ownership Analysis

- Text meaning, measurement, layout, fallback policy, and diagnostics remain
  foundational presentation semantics under ADR-0004.
- The Departure Mono asset, OTF parsing, rasterization, and outline extraction
  are provider-owned implementation.
- Selecting a first-party native default is provider resolution policy. It is
  not world truth and is not kernel-native.
- Applications may override the default without changing their semantic text
  requests.
- The provider must not own application text roles, world state, layout truth,
  renderer resources, or ambient operating-system font substitution.

## Dependency Direction

```text
Current and accepted:

application text intent
        -> provider-neutral text semantics and layout
        -> explicit font provider resolution
        -> Departure Mono OTF provider or another selected provider
        -> raster/vector renderer adapter

bootstrap or provider failure
        -> explicit diagnostic
        -> built-in bitmap fallback when configured
```

No font asset or parser dependency enters `tokimu-core` or `tokimu-runtime`.

## Alternatives Considered

### Alternative A: Grow The Built-In Bitmap Font

- Benefits: tiny, deterministic, no external asset.
- Costs: ongoing hand-authored coverage and typography work.
- Failure mode: an emergency fallback silently becomes Tokimu's product font
  and remains incomplete across consumers.

### Alternative B: Use An Ambient System Font

- Benefits: no bundled font payload and familiar platform typography.
- Costs: platform-dependent identity, metrics, availability, and rendering.
- Failure mode: silent substitution changes presentation meaning and defeats
  reproducible corpus evidence.

### Alternative C: Select Inter

- Benefits: strong ordinary UI readability and mature proportional metrics.
- Costs: does not match the compact pixel-oriented native presentation already
  used throughout Tokimu's tools.
- Failure mode: native evidence and the chosen default drift into separate
  visual systems. Inter remains an important comparison and override provider.

### Alternative D: Provide No First-Party Default

- Benefits: maximum neutrality.
- Costs: repeated consumer boilerplate and inconsistent fallback choices.
- Failure mode: applications recreate provider policy ad hoc and may silently
  depend on system fonts.

## Findings

The normal evidence for a permanent default is incomplete. Tokimu has not yet
validated final package size, all target DPIs, accessibility, broad scripts,
or mature native/WASM fallback behavior.

Substitute evidence is sufficient for reversible implementation:

- ADR-0004 already fixes the semantic/provider ownership boundary;
- several unrelated presentation consumers require complete native text;
- four font providers exercise the same provider-neutral contracts;
- Departure Mono is pinned, checksum-recorded, and OFL licensed;
- the selection can be changed without changing application semantics; and
- the bitmap fallback preserves a no-external-asset diagnostic path.

Mechanically waiting for more examples would add little ownership evidence.
The unresolved questions concern provider quality and packaging, not whether
font assets belong in the trusted kernel.

The accountable maintainer is the Tokimu project maintainer, Arakendo. The
accepted risks are a bundled asset payload, provisional visual preference,
limited script coverage, and future migration to a stronger default provider.

## Disposition

Accepted. ADR-0004 is revised to provisionally admit Departure Mono as Tokimu's
first-party native default font provider under ADR-0005. This admits provider
selection and bundling only. It does not admit font technology or provider
identity to the trusted kernel, stabilize a final crate layout, or remove the
built-in bitmap bootstrap fallback.

## Consequences

Native consumers gain one deterministic, complete everyday font selection and
no longer need to grow the bitmap alphabet for ordinary presentation. Corpus
and application code can still select Inter, JetBrains Mono, Noto, project
fonts, or future providers explicitly.

Tokimu must distribute the matching OFL license and retain fixture provenance.
Provider resolution remains fallible and diagnostic. A missing bundled asset
must not silently become an ambient system font.

## Required Follow-Up

- [x] Revise ADR-0004 with the provisional provider decision.
- [x] Pin and checksum the Departure Mono fixture and preserve its license.
- [x] Add provider-side native-default resolution without changing semantic
      text requests.
- [x] Add Departure Mono to the font comparison corpus.
- [ ] Capture native visual evidence at ordinary and integer-multiple sizes.
- [ ] Measure native and WASM distribution impact before permanent admission.
- [ ] Exercise explicit missing-provider fallback and diagnostics in a consumer.

## Reopening Triggers

- native or WASM distribution cost is unacceptable;
- Departure Mono fails readability, accessibility, DPI, or target-platform
  review;
- broad-script or shaping requirements expose a default/fallback contract flaw;
- provider identity leaks into application semantic text APIs;
- licensing or provenance can no longer be reproduced;
- another bundled provider offers materially stronger evidence; or
- the bitmap fallback becomes unavailable when provider loading fails.

## Review History

### Cycle 1 -- 2026-08-04

- Status entering review: Proposed
- New evidence: pinned Departure Mono provider; four-provider font comparison;
  established bitmap, raster, and vector-outline paths
- Participants or reviewers: Arakendo and implementation review assistance
- Findings: provider selection is reversible and cross-cutting; parser and
  asset ownership remain outside the trusted kernel
- Disposition: Accepted with provisional admission under ADR-0005
- Resulting ADR or documentation change: revision of ADR-0004

### Cycle 2 -- 2026-08-04

- Status entering review: provisional native-default provider admitted
- New evidence: `tokimu-console-command-window` exercises Departure Mono in a
  focused native command-window corpus with lowercase input, punctuation,
  scrolling transcript output, editable prompt focus, and provider-boundary
  diagnostics. The corpus keeps Tosumu TQL and Ratatui terminal-cell behavior
  outside engine crates.
- Boundary check: `cargo tree -p tokimu-core` and
  `cargo tree -p tokimu-runtime` contain no Tosumu, Ratatui, or Departure Mono
  dependency.
- Findings: Departure Mono is readable for the reviewed native console
  presentation and supports a terminal-like everyday consumer without growing
  the bootstrap bitmap alphabet. The result remains manual native evidence,
  not a claim of complete DPI, accessibility, broad-script, distribution-size,
  or WASM validation.
- Disposition: retain provisional admission; no crate-boundary change
- Resulting ADR or documentation change: none

### Cycle 3 -- 2026-08-05

- Status entering review: provisional native-default provider admitted
- New evidence: the console corpus now retains deterministic cell-layout and
  CPU-raster artifacts, uses provider metrics for measured transcript wrapping,
  and presents a sustained native command viewport with editable input,
  history, scrolling, clipping, and resize behavior.
- Findings: Departure Mono remains readable at the reviewed native console
  dimensions, and its metrics are sufficient for deterministic wrapping and
  bounded raster evidence. The provider remains replaceable through ADR-0004;
  no font parser, asset identity, or provider-native type entered shell,
  Tosumu, core, or runtime semantics.
- Limits: this evidence does not complete accessibility, broad-script,
  cross-DPI, integer-multiple visual review, missing-provider consumer
  fallback, WASM packaging, or distribution-size measurement.
- Disposition: retain provisional admission; no permanent-default or
  crate-boundary change
- Resulting ADR or documentation change: console readiness findings now feed
  the Tosumu inspection-island prerequisites and AR-0013

## References

- `docs/ADR/ADR-0004-foundational-presentation-text-and-icons.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `third-party/fonts/README.md`
- `corpus/lib/ui-tools/DESIGN.md`
- `corpus/ui/hello-ui-font2/DESIGN.md`
- `corpus/consumers/tokimu-console-command-window/DESIGN.md`
