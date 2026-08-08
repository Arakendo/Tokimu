# Initial Alternative A/B Transform Workload Run

| Field | Value |
| --- | --- |
| Status | Informational only; not a performance conclusion |
| Date | 2026-08-07 |
| Target | `x86_64-pc-windows-msvc` |
| Profile | `release` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |
| LLVM | 22.1.2 |
| Provider | audited local `glam` 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Selected provider features | `default-features = false`, `std` |
| Host CPU / OS | Not retained: sandbox access to Windows CIM host queries was denied |
| Workload | `baseline_transform_workload` and `provider_backed_transform_workload` |
| Iterations | 100,000 |

## Command

```powershell
cargo run -p tokimu-math-study --release --locked --offline --bin measure_transform_workload -- 100000
```

## Output

```text
iterations=100000
baseline_elapsed_ns=402900
provider_backed_elapsed_ns=402800
checksum=[30056.43, -9896760.0, -10090.261]
```

## Interpretation Limits

Both alternatives produced the same checksum in this invocation. The elapsed
times are a single, short measurement and are not evidence of equivalence,
regression, or zero-cost abstraction. Repeat measurements on an identified host
with warmup, repetition statistics, allocation evidence, binary-size evidence,
and the WASM measurement path are required before an Alternative B performance
disposition.

## Allocation Observation

```powershell
cargo run -p tokimu-math-study --release --locked --offline --bin measure_transform_allocations -- 100000
```

```text
iterations=100000
baseline_allocations=0
provider_backed_allocations=0
checksum=[30056.43, -9896760.0, -10090.261]
```

The counter is reset immediately before each workload and the executable fails
on a nonzero count. This supports only the ordinary value operations exercised
by this workload; it is not evidence that every candidate API, conversion
boundary, caller migration, or future DOOM-driven workload is allocation-free.

## Alternative C Extension

After C gained its original `Mat4` implementation, the identical workload was
run with the same target, profile, toolchain, and iteration count:

```text
iterations=100000
baseline_elapsed_ns=322400
provider_backed_elapsed_ns=327400
owned_elapsed_ns=302200
checksum=[30056.43, -9896760.0, -10090.261]
```

```text
iterations=100000
baseline_allocations=0
provider_backed_allocations=0
owned_allocations=0
checksum=[30056.43, -9896760.0, -10090.261]
```

The owned timing is not a performance result: it is one short invocation on an
unidentified host. Its retained value is that all three candidates now execute
the same checked workload, while C's allocation observation is also zero for
that bounded path.

## Adapter-Boundary Measurement Correction

The first B/C upload-boundary harness used a constant checksum and the compiler
eliminated the provider-backed loop. That result was discarded. The corrected
workload passes each matrix and observed upload component through optimization
barriers:

```text
iterations=100000
provider_backed_upload_elapsed_ns=112800
owned_upload_elapsed_ns=101500
checksum=0
```

This single corrected run is still not comparative performance evidence. It
only establishes that the harness now executes both representative conversion
paths rather than optimizing one away.

The corresponding counted-allocation run reports:

```text
iterations=100000
provider_backed_upload_allocations=0
owned_upload_allocations=0
checksum=0
```

This confirms neither representative upload conversion allocates in this
process after the counter reset; it does not imply that a complete renderer
migration is allocation-free.
