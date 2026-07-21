# Font Corpus Candidates

These fonts are reference and embedding candidates for Tokimu's UI and glyph
corpus examples. They are not all required in every build. Examples should
select a small, explicit fixture and identify its provider and format.

All three candidates use the SIL Open Font License. Keep the license file with
any copied or embedded font asset.

## Recommended Set

### Inter

- Role: primary UI and proportional Latin font
- Tests: ordinary interface text, mixed-case metrics, buttons, labels, and
  baseline alignment
- Preferred fixture: a static regular TTF for predictable corpus tests
- Source: `third-party/fonts/inter`
- License: `third-party/fonts/inter/LICENSE.txt`

Inter should be the default reference for `hello-ui-font` because it is a
practical UI face with clear glyph metrics and broad everyday coverage.

### JetBrains Mono

- Role: monospace and code-oriented font
- Tests: equal advance widths, punctuation, digits, dense status text, and
  code/editor surfaces
- Preferred fixture: regular TTF
- Source: `third-party/fonts/jetbrains-mono`
- License: `third-party/fonts/jetbrains-mono/OFL.txt`

JetBrains Mono complements Inter by testing whether layout correctly respects
monospace advances rather than assuming proportional typography.

### Noto Sans

- Role: broad-script sans reference
- Tests: fallback coverage, accents, non-Latin samples, and larger
  ascender/descender variation
- Preferred fixture: one focused Noto Sans TTF, not the entire Noto tree
- Source: `third-party/fonts/noto`
- License: `third-party/fonts/noto/LICENSE`

Noto should be used selectively. The repository contains many script-specific
and variable fonts; embedding the entire collection would make the corpus
noisy and unnecessarily large. A practical first fixture is
`NotoSans-VF.ttf`. Noto Serif remains a useful future contrast fixture, but it
should not change the first broad-coverage experiment's variables.

## Suggested Corpus Matrix

| Candidate | Format | Primary evidence |
|---|---|---|
| Inter Regular | TTF | UI baseline and proportional layout |
| JetBrains Mono Regular | TTF | Monospace advances and code text |
| Noto Sans fixture | TTF | Coverage, fallback, and metric variation |

The first two should be stable required fixtures for local corpus examples.
Noto can remain an optional extended-coverage fixture until the font fallback
policy is implemented.

## Identity And Metadata

The reference font is not automatically the product's default font. A corpus
fixture is selected to provide evidence; product typography remains a separate
design decision.

Each embedded or copied fixture should record:

- family and style;
- provider repository;
- exact source revision;
- file format and path;
- license file;
- SHA-256 checksum.

The family name alone is not enough to identify a test input. A changed
upstream variable font can otherwise alter screenshots without changing the
example's source code.

## Initial Test Matrix

```text
Inter
  Hamburgefontsiv
  The quick brown fox jumps over the lazy dog.
  UI labels and buttons

JetBrains Mono
  0O1Il|{}[]()<>
  0123456789
  code and punctuation runs

Noto Sans
  accented Latin
  mixed-script samples
  missing-glyph fallback
  LTR/RTL boundary cases later
```

## Embedding Rules

- Embed only named fixtures, not an entire provider submodule.
- Prefer static TTF fixtures for deterministic baseline tests.
- Use OTF when specifically testing CFF/OTF ingestion.
- Record provider, family, format, and source revision beside the fixture.
- Preserve the corresponding license and attribution.
- Keep runtime font discovery in `ui-tools`; examples should select a logical
  corpus fixture rather than search directories themselves.
