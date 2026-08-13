# Option C0 Slice 3 Boundary Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Slice 3 complete as bounded corpus evidence; no production admission |
| Candidate | `src/alternative_c.rs`, reused by the dependency-free isolated crate |
| Toolchain | Rust/Cargo 1.95.0, `x86_64-pc-windows-msvc` |
| Plan | `docs/Plans/Native-Math/Studies/ar-0019-option-c-owned-math-and-bulk-compute.md` |

## Added Evidence

- `bounded_numerical_probe` is a dependency-free executable with an optional
  deterministic seed and a deliberately capped `4,096` cases. Its retained
  256-case invocation used seed `3235823838` and produced checksum
  `-34.195366`.
- A 128-case fixed-seed scalar column-array projection reference independently
  checks C0's checked perspective-divide path. It does not call C0's
  homogeneous-vector helper, so it can detect a mistake shared by provider and
  candidate differential comparisons.
- The isolated host integration test surrounds 1,536 repeated ordinary value
  operations with a test-only `GlobalAlloc` counter and observed zero allocation
  calls. The allocator forwarding contains `unsafe` solely in the test harness;
  the candidate source remains safe and contains no allocation API.
- The retained translated/rotated conditioning fixture accepts smallest scale
  `1e-2` and rejects `1e-3` through `1e-10` under the selected two-sided
  `1e-3` identity residual. This is deliberately a bounded C0 rejection
  observation, not a general condition-number or production threshold.

## Surface Accounting

| Observation | First hardening | Boundary evidence | Change |
| --- | ---: | ---: | ---: |
| Shared source lines | 979 | 1,008 | +29 |
| Shared nonblank lines | 860 | 887 | +27 |
| Shared source bytes | 30,196 | 31,415 | +1,219 |
| Shared local tests | 9 | 10 | +1 |
| Candidate `unsafe` occurrences | 0 | 0 | 0 |
| Isolated manifest dependencies | 0 | 0 | 0 |

Current C0 source SHA-256:

```text
c3e6aed82050aa85fe187da762669a0f673f8d5319b13948c4c0de34275c3b9c
```

The `unsafe`-free candidate result is scoped to
`src/alternative_c.rs`. It must not be misreported as a claim that every
host-only test instrument is safe Rust.

## Validation

Passed offline:

```text
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --locked --offline
  11 tests passed

cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --test ordinary_value_allocations --locked --offline
  1 test passed; measured allocation count = 0

cargo run --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --bin bounded_numerical_probe --locked --offline -- 3235823838 256
  completed; checksum = -34.195366

cargo clippy --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --all-targets --locked --offline -- -D warnings
  passed

cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --target wasm32-unknown-unknown --locked --offline
  passed (compile-only)
```

Windows may report Cargo incremental-cache hard-link fallback warnings. They
are host filesystem behavior, not candidate source diagnostics. The broader
study also retains the independent, known pinned-`glam` warning flood.

## Disposition

Slice 3 demonstrates a narrow, safe scalar candidate with bounded generated,
independent-reference, degenerate, conditioning, and host-allocation evidence.
It does **not** decide Option C, prove target-wide allocation behavior, add an
ABI/FFI contract, admit a public Tokimu math vocabulary, or authorize replacing
production `glam`.

Slice 4 must next obtain actual WASM execution/parity and representation
observations before comparing build/binary/performance costs in Slice 5.
