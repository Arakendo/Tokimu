# External MilkDrop Preset Admission

This file is the required record format before a third-party MilkDrop preset
or referenced texture enters the Tokimu corpus. It is deliberately separate
from parser output: parsing a source file does not establish permission to
store, redistribute, or claim compatibility with it.

## Candidate Record

Copy this block once for every proposed file. Leave the candidate unadmitted
until all required fields have an evidence-backed value.

```text
Candidate ID:
Source file name:

Upstream repository or archive:
Upstream revision, release, or immutable URL:
Original author or pack maintainer:
License and redistribution terms:
License evidence location:
SHA-256:

Targeted constructs:
Expected maturity: inspected | partial | compatible | unsupported | invalid
Why this fixture is needed:

Expected supported behavior:
Expected deferred or unsupported behavior:
Expected diagnostic ownership:

Associated textures or external assets:
Asset licenses and hashes:
Reference implementation or compatibility oracle, if any:
Reviewer and admission date:
```

## Admission Rules

- Do not add an external preset, pack, or texture before its redistribution
  terms and source revision are recorded.
- Preserve the original source unchanged under its recorded provenance path.
- Do not label a preset `compatible` solely because it parses or renders.
  Record the exact feature-level evidence and comparison method.
- Treat unsupported sections as expected evidence. Do not delete or silently
  approximate them to make a fixture appear supported.
- Provider behavior, projectM output, and browser rendering are comparison
  evidence. They do not redefine Tokimu visualizer semantics.

## Current Status

No third-party MilkDrop preset or texture is admitted yet. The fixtures under
`assets/` are Tokimu-authored parser and evaluator inputs only.
