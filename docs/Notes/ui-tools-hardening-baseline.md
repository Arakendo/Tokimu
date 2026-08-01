# UI Tools Hardening Baseline

## Status

Active Slice 0 evidence for
[`ui-tools-consumer-safety-and-hardening.md`](../Plans/ui-tools-consumer-safety-and-hardening.md).

This is a structural inventory, not a claim that every matching source line is
duplicated or unsafe. It establishes a repeatable starting point for the UI
hardening work after the runtime inspector layout incident.

## Reproduce

Run from the repository root:

```powershell
pwsh -NoProfile -File .\scripts\audit-ui-tools.ps1
```

Use `-AsJson` when a corpus runner or a future report needs machine-readable
output. The script is read-only. It inventories public API declarations, unit
test concentration, and consumer-side marker use; it does not judge code
quality by itself.

## Initial Snapshot

The initial local audit reports:

| Measure | Count | Interpretation |
| --- | ---: | --- |
| `ui-tools` unit tests | 215 | Strong total count, but not a composition guarantee. |
| Root public export statements | 18 | Pre-tier baseline: the root facade remained broad even though it had a small number of export statements. |
| Source public declarations | 641 | A source-level breadth indicator, not a stable API count. |
| SVG tests | 79 | Mature provider/importer evidence. |
| Vector tests | 35 | Mature geometry evidence. |
| Font-outline tests | 23 | Established outline-adapter evidence. |
| Layout tests | 0 | Initial script limitation: cross-cutting layout tests lived in `src/tests/layout.rs`, outside `src/layout.rs`. |
| Controls-local tests | 1 | Interaction/control behavior needs direct evidence. |
| Text-input-local tests | 2 | Too small for a general routing contract. |
| Scroll-local tests | 4 | Too small for nested composition semantics. |
| Corpus `UiRect` references | 388 | A broad marker for local geometry pressure, not a literal count of unsafe layouts. |
| Corpus `UiDrawer` references | 34 | Indicates remaining manual lowering use. |
| Corpus surface-lowering references | 27 | Indicates remaining direct presentation plumbing. |
| Corpus text-lowering references | 25 | Indicates remaining direct presentation plumbing. |
| Corpus renderer-submission references | 119 | A broad marker, including legitimate renderer adapters. |

The source marker counts intentionally include `corpus/lib` and corpus
applications. They are useful for measuring directional change after semantic
composition and owned draw-list paths have independent consumers. They must
not become a goal to blindly minimize: renderer adapters and focused geometry
corpus entries legitimately use lower-level APIs.

## Slice 11 Migration Snapshot

The audit now reports the four named migration consumers separately from the
repository-wide pressure indicators. Relative to the checked-in pre-migration
sources, direct `draw_text` references across those consumers fell from 23 to
3, and `UiDrawer` references fell from 3 to 0. The three remaining direct text
calls belong to CGM diagram labels rendered inside domain geometry, not to its
UI shell. All four consumers report zero direct surface-lowering calls.

`UiRect` references increased from 35 to 40 across the same set. That is not a
regression by itself: the migrated code now names semantic viewport intent,
resolved domain anchors, and explicit viewport-matrix assertions. The audit
therefore treats rectangle counts as pressure evidence while the disappearance
of private drawer and shell-lowering paths is the stronger ownership signal.

Renderer submission references remain intentionally nonzero. Native consumers
still adapt an owned `UiDrawList` to backend commands, while the WASM workbench
serializes structural draw evidence. Neither activity transfers UI meaning to
the renderer or browser.

After Slice 1 introduced explicit API tiers, the audit reports 26 root public
module or export statements and 649 source public declarations. That increase
is expected: the new `consumer`, `provider`, `lowering`, and `diagnostics`
entry points make intended ownership visible without breaking existing corpus
callers. The audit script now also counts the cross-cutting layout test module;
future measurements should be compared with this corrected collection rule.

## Current Quality Gate

As of this baseline, the focused package gate passes:

```powershell
cargo fmt --all -- --check
cargo clippy -p ui-tools --all-targets -- -D warnings
cargo test -p ui-tools
```

The Clippy repair kept SVG diagnostics structured while boxing an optional
nested XML diagnostic. This reduces result-value size without replacing
diagnostic data with a string or changing the importer failure meaning.

## Baseline Performance Evidence

The current runtime evidence remains in
[`ui-presentation-performance-evidence.md`](ui-presentation-performance-evidence.md).
It records the initial `hello-cgm` static-screen symptoms, the confirmed
renderer binding-allocation correction, and outstanding measurement, batching,
and invalidation questions. The runtime inspector is the composition/readability
trigger; `hello-cgm` is the steady-state performance trigger.

## How To Use This Evidence

- Add focused tests where a new composition behavior is introduced.
- Prefer capability categories over aggregate count growth.
- Treat reduced direct geometry/lowering markers as evidence only after a
  shared replacement is proven by multiple consumers.
- Keep native screenshots as manual evidence; use structural layout, routing,
  and draw-list artifacts for authoritative regressions when those contracts
  exist.
