# Diff Tools Provenance And Behavior Inventory

## Reference Source

The behavioral reference is the local C# project at
`F:\LocalSource\ClassLibrary\DiffTools`, inspected on 2026-08-03.

- License: MIT, copyright Arakendo, 2025.
- Reference revision: local working copy; no source code is copied into this
  Rust crate.
- Scope inspected: `Core/Models`, `UnifiedDiffParser`, `DiffApplicator`, and
  their focused unit tests.

## Behavior Disposition

| Reference behavior | Disposition | Tokimu direction |
|---|---|---|
| Diff documents, files, hunks, lines | Adapt | Immutable, bounded Rust model. |
| Unified-diff parsing | Port later | Explicit admitted dialect and diagnostics. |
| Canonical unified-diff writing | Replace | No writer exists in the source; Tokimu will define one. |
| Exact patch application | Port later | In-memory and atomic by explicit policy. |
| Fuzzy matching | Adapt later | Bounded, scored, and ambiguity-aware. |
| Conflict collection | Adapt later | Structured conflict data, never message-only. |
| Metadata timestamps/author fields | Defer | No stable consumer pressure yet. |
| UI ghost editing and hunk panels | Reject | Presentation consumers own UI. |
| Legacy helper routines | Reject | No stable semantic contract. |
| XML/JSON/binary semantic diff | Defer | Domain adapters remain separate. |

## Supported Unified-Diff Assumptions

No parser is admitted in the first slice. The fixtures reserve the initial
dialect for traditional `---` / `+++` file headers, `@@` hunk ranges, and
context, addition, and removal lines. Git metadata, rename headers, binary
patches, and extension lines remain outside the initial dialect until their
semantics are explicitly selected.

## First-Party Fixtures

The fixture directory contains first-party, MIT-licensed examples created for
this corpus. They are not copied from the C# reference implementation.

- `exact-addition.diff`: a normal single-file addition.
- `fuzzy-offset.diff`: a hunk whose expected range is intentionally stale.
- `malformed-count-mismatch.diff`: a declared count that does not match hunk
  lines.
- `conflict-source.txt`: source text whose changed context is unsuitable for
  exact application.
- `unicode-source.txt`: UTF-8 text with a missing final newline.

## Incubated Generation Evidence

The first generator is `LcsV1`: a deterministic dynamic-programming longest
common-subsequence implementation. It records its algorithm identity in every
generated `DiffDocument`, prefers removal when equal-score edit choices occur,
and rejects inputs whose edit matrix exceeds `DiffLimits::max_edit_matrix_cells`.

This is an intentionally bounded initial algorithm, not a claim that LCS is
the final performance choice for every artifact workload.

## Implementation Dependencies And Replacement Conditions

`diff-tools` currently has no third-party diff, patch, or merge engine. Its
only direct implementation dependencies are infrastructure libraries already
used broadly in Tokimu:

| Dependency | Current purpose | License | Replacement condition |
|---|---|---|---|
| `serde` | Structured contract serialization | MIT OR Apache-2.0 | Replace only if Tokimu selects a different serialization boundary. |
| `serde_json` | JSON artifact comparison input | MIT OR Apache-2.0 | Replace only if JSON stops being an admitted artifact boundary. |
| `thiserror` | Typed error derivation | MIT OR Apache-2.0 | Replace if the workspace error convention changes. |
| `libfuzzer-sys` | Optional parser/applicator fuzz harness | Apache-2.0 | Present only in the standalone `fuzz/` package; replace if the selected fuzzing toolchain changes. |

`LcsV1` is Tokimu-authored deterministic dynamic programming, not an embedded
implementation from a separate diff library. A future algorithm replacement
must retain an explicit algorithm identity in generated documents and compare
its corpus output against the current bounded fixtures before becoming the
default.
