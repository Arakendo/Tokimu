# Diff Tools

`diff-tools` incubates provider-neutral text comparison, unified-diff, and
patch-application semantics for Tokimu corpus evidence and consumers.

It deliberately does not own filesystems, repositories, source control,
rendering, editor widgets, or domain-specific document meaning. Callers supply
in-memory documents and present the structured results themselves.

The initial implementation preserves source text facts that a rendered unified
diff alone cannot recover: line-ending policy and whether the source ended in a
newline. Diff generation, parsing, writing, and patch application will build on
that model in later plan slices.

See [`PROVENANCE.md`](PROVENANCE.md) for the C# reference inventory and
[`docs/Plans/Standalone/diff-tools.md`](../../../docs/Plans/Standalone/diff-tools.md) for the
incubation plan.

## Current Contracts

- Exact patch application operates only on caller-owned, in-memory text maps.
- `DiffDocument::reversed` creates a structural inverse and reuses the same
  exact applicator; it does not introduce a second mutation mechanism.
- Fuzzy application is bounded, opt-in, and only applies a uniquely located
  hunk. Ambiguous candidates are preserved as evidence rather than selected.
- Ordered JSON artifact comparison reports the earliest caller-supplied stage
  with structural divergence. It does not infer pipeline ownership from file
  names or implementation details.
- Standard unified-diff final-newline markers are retained as explicit source
  and target format facts. Exact in-memory application rejects a mismatched
  source final-newline state and preserves the declared target state.

## Consumer Recipes

### Compare caller-owned text

```rust
use diff_tools::{diff_text, DiffGenerationConfig, DiffLimits, TextDocument};

let limits = DiffLimits::default();
let before = TextDocument::parse("alpha\n", limits)?;
let after = TextDocument::parse("beta\n", limits)?;
let document = diff_text(
    "note.txt",
    &before,
    "note.txt",
    &after,
    DiffGenerationConfig::default(),
    limits,
)?;
```

### Parse, apply, and report a unified patch

```rust
use std::collections::BTreeMap;
use diff_tools::{apply_exact, parse_unified_diff, DiffLimits, ExactPatchConfig};

let patch = parse_unified_diff(patch_text, DiffLimits::default())?;
let source = BTreeMap::from([("note.txt".to_owned(), "alpha\n".to_owned())]);
let outcome = apply_exact(&patch, &source, ExactPatchConfig::default());

// `outcome.committed` and `outcome.report` are the authoritative result.
```

### Compare structured diagnostic artifacts

```rust
use diff_tools::{compare_json, JsonComparisonConfig};

let comparison = compare_json(&expected, &actual, &JsonComparisonConfig::default())?;
if !comparison.equal {
    // Present `comparison.differences`; do not infer domain meaning here.
}
```

The base library does not decide what a JSON difference means. Runtime,
geometry, and Resource Space adapters retain their typed semantic ownership.

## Workload Evidence

Run the deterministic input-size probes with:

```powershell
cargo run -p diff-tools --example generation_workloads --release
```

They report separate interactive and artifact-sized LCS observations. Timings
are diagnostic evidence for a specific machine and build, not a performance
guarantee or CI threshold.

Image comparison remains with the screenshot and image-evidence layer. Text
comparison cannot honestly establish raster equivalence.

## Adversarial Input Coverage

`tests/adversarial_inputs.rs` retains a deterministic mutation corpus for the
unified parser and exact in-memory applicator. It is intentionally ordinary
`cargo test` coverage rather than a replacement for a dedicated fuzzing
toolchain: malformed inputs must never panic, and bounded limits reject
oversized input before unbounded structural work.

The separate [`fuzz/`](fuzz/) package adds a coverage-guided
`unified-parser-apply` target. It is deliberately not part of the normal Cargo
workspace; see its README for toolchain and execution details.
