# Repeated A/B/C Transform Workload Observation

| Field | Value |
| --- | --- |
| Status | Descriptive observation; not a performance conclusion |
| Date | 2026-08-08 |
| Target | `x86_64-pc-windows-msvc` |
| Profile | `release` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |
| LLVM | 22.1.2 |
| Provider | audited local `glam` 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Host CPU / OS | Not retained: sandbox access to Windows CIM host queries remains denied |
| Workload | `baseline_transform_workload`, `provider_backed_transform_workload`, and `owned_transform_workload` |
| Iterations per sample | 100,000 |
| Samples per candidate | 9 |
| Warm-up | One checked execution of each candidate before timing |
| Ordering | Rotated A/B/C order for each sample |

## Command

```powershell
cargo run -p tokimu-math-study --release --locked --offline --bin measure_transform_workload -- 100000 9
```

## Output

```text
iterations=100000
samples=9
baseline_elapsed_ns=min:317300,median:351600,max:571200
provider_backed_elapsed_ns=min:309700,median:357000,max:683600
owned_elapsed_ns=min:286300,median:290200,max:405500
checksum=[30056.43, -9896760.0, -10090.261]
```

## Interpretation Limits

Every timed and warm-up invocation produced the shared checksum. The runner
rotated candidate order, so this is stronger evidence than the prior one-pass
observation that the harness executes equivalent work. The C median is lower
in this process, but the sample ranges overlap and the host is not identified.
This result therefore does **not** establish a provider-wrapper cost, an owned
implementation advantage, or a stable performance disposition.

Before the study may use timing as a decision input, it still needs an
identified host, a defined warm-up/repetition protocol retained across native
and WASM targets, representative caller workloads, binary-size and build-time
observations, and the post-DOOM pressure revisit.
