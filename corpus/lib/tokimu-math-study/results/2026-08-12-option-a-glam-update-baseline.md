# Option A `glam` Update Baseline

| Field | Observation |
| --- | --- |
| Captured | 2026-08-12 |
| Update candidate | `glam` 0.33.3; not yet pinned or admitted |
| Current gitlink | `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` (`glam` 0.29.3) |
| Current selection | local path, `default-features = false`, `features = ["std"]` |
| Toolchain | `rustc 1.95.0`; `cargo 1.95.0`; `x86_64-pc-windows-msvc`; LLVM 22.1.2 |
| Workspace state | pre-existing Option C and unrelated AR changes are dirty; candidate pin movement is intentionally deferred |

## Isolation finding

The root worktree already contains uncommitted Option C study changes in both
`Cargo.toml` and `Cargo.lock`, as well as related corpus and review evidence.
The `glam` submodule itself remains clean and at the recorded 0.29.3 gitlink.
Fetching the 0.33.3 tag added only an object to the submodule repository; it did
not move its checkout or alter the parent gitlink.

This baseline therefore supports source review and current-pin observations,
but it is not a clean upgrade branch. Slice 3 must not move the pin until the
pre-existing work is committed or an isolated worktree/branch is authorized.

## Current-pin validation

Commands ran from the workspace root on the toolchain above.

| Command | Result | Warm wall time | Retained observation |
| --- | --- | ---: | --- |
| `cargo clippy -p tokimu-core --all-targets -- -D warnings` | exit 0 | 582 ms | 4,896 generated-swizzle `unused_attributes` diagnostics from `glam`; dependency warnings are not promoted to errors by this invocation |
| `cargo test -p tokimu-core --locked --offline` | pass | 713 ms | 29 unit tests passed; no failures |
| `cargo build -p tokimu-core --target wasm32-unknown-unknown --locked --offline` | pass | 541 ms | `glam` reported 4,900 warnings, including the 4,896 generated-swizzle diagnostics; `tokimu-core` also reported the host filesystem hard-link warning |
| `scripts/audit-ring-zero-dependencies.ps1` first attempt | blocked | 15,242 ms | Cargo metadata attempted to download uncached optional package `minicov 0.3.8`; sandbox network was unavailable |
| `scripts/audit-ring-zero-dependencies.ps1` approved retry | pass | 2,032 ms | closure contains local `glam` 0.29.3 with only `std`; `minicov` was downloaded for metadata resolution but is not in the admitted Ring 0 execution closure |

The warning baseline is intentionally recorded even though core Clippy exits
successfully. A passing command does not mean the selected foreign source is
warning-clean.

## Baseline limitations

- These are warm, narrow core observations, not clean-build compile-time or
  binary-size measurements.
- Actual browser/WASM execution and caller-shaped performance controls remain
  to be captured after isolation and before candidate admission.
- The audit's metadata download is an audit-tool reproducibility finding, not
  evidence that `minicov` executes in Tokimu Ring 0.
