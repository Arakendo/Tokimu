# Candidate-Isolated Release Link Output Observation

| Field | Value |
| --- | --- |
| Status | Descriptive link-output observation; not an application binary-size conclusion |
| Date | 2026-08-08 |
| Target | `x86_64-pc-windows-msvc` |
| Profile | `release` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |
| LLVM | 22.1.2 |
| Provider | audited local `glam` 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Workload | One candidate's shared transform workload, held through `std::hint::black_box` |
| Link targets | `measure_transform_binary_a`, `_b`, and `_c` |

## Commands

```powershell
cargo build -p tokimu-math-study --release --locked --offline --bin measure_transform_binary_a
cargo build -p tokimu-math-study --release --locked --offline --bin measure_transform_binary_b
cargo build -p tokimu-math-study --release --locked --offline --bin measure_transform_binary_c
Get-Item target/release/measure_transform_binary_{a,b,c}.exe
```

## Output Sizes

| Candidate | Executable bytes |
| --- | ---: |
| A — direct `glam` control | 123,904 |
| B — provider-backed vocabulary | 123,904 |
| C — narrow owned implementation | 124,416 |

The C output is 512 bytes larger than A/B for this target and workload.

## Interpretation Limits

Each executable calls only its own workload, preventing all A/B/C mechanics
from being linked merely because one comparison binary imports them together.
However, Rust code generation, linking, standard-library startup, the host,
and this artificially small workload dominate these outputs. The observation
does not measure a renderer, scene, application dependency closure, asset
pipeline, debug information, compression, LTO policy, WASM output, or final
distribution format. It must not be used to select a stable math vocabulary.
