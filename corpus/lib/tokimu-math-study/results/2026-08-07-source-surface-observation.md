# Source-Surface Observation

| Field | Value |
| --- | --- |
| Status | Maintenance/provenance observation; not a quality, binary-size, or performance conclusion |
| Date | 2026-08-07 |
| Scope | Current study source after the CAD and animated-node fixtures |
| Provider | Pinned local `glam` at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |

## Measured Surface

The command below recursively counts `*.rs` files, nonblank PowerShell text
lines, and raw bytes. Candidate counts include their inline unit tests; they
do not include shared study fixtures, docs, dependency source, or generated
build artifacts.

| Surface | Rust files | Lines | Bytes | Interpretation |
| --- | ---: | ---: | ---: | --- |
| A — direct re-export probe | 1 | 68 | 2,325 | Minimal Tokimu source; the provider supplies mechanics. |
| B — provider-backed vocabulary | 1 | 450 | 13,719 | Tokimu owns the wrapper and its private provider seam. |
| C — owned subset | 1 | 494 | 15,807 | Tokimu owns the exercised scalar vector/matrix mechanics. |
| D — bounded derivation | 1 | 139 | 4,064 | Narrow `Vec3` slice only; provenance/update documents are additional required burden. |
| Pinned `glam/src` | 170 | 133,172 | 3,699,334 | Full retained provider source tree, not an estimate of the selected runtime path. |

Alternative B has 41 direct `glam` references in its candidate source. That is
expected delegation evidence, not a leak into its public signatures.

## Command

```powershell
$paths = @(
  'corpus/lib/tokimu-math-study/src/baseline_a.rs',
  'corpus/lib/tokimu-math-study/src/alternative_b.rs',
  'corpus/lib/tokimu-math-study/src/alternative_c.rs',
  'corpus/lib/tokimu-math-study/src/alternative_d.rs',
  'third-party/ring-0/glam/src'
)
foreach ($path in $paths) {
  $files = Get-ChildItem -LiteralPath $path -File -Recurse -Filter '*.rs'
  $lines = 0
  $bytes = 0
  foreach ($file in $files) {
    $lines += (Get-Content -LiteralPath $file.FullName | Measure-Object -Line).Lines
    $bytes += $file.Length
  }
  [PSCustomObject]@{ Path=$path; RustFiles=$files.Count; Lines=$lines; Bytes=$bytes }
}
```

## Limits

- This does not prove an owned implementation is simpler, safer, or cheaper
  over time; numerical maintenance and target-specific optimization cannot be
  reduced to source lines.
- The provider-tree count is not a selected-feature binary-size measurement.
- D's small current count does not make it low-cost: its provenance, upstream
  fix-detection, and compatibility burden grows with every admitted operation.
- Binary size, clean-build time, repeated performance measurements, and WASM
  execution remain separate required evidence.
