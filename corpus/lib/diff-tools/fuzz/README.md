# Diff Tools Fuzzing

This package is intentionally outside the Cargo workspace. It uses
`cargo-fuzz` to exercise the bounded unified-diff parser and exact in-memory
applicator against arbitrary UTF-8 input.

Install the toolchain once:

```powershell
cargo install cargo-fuzz
```

Run the target from this directory:

```powershell
cargo fuzz run unified-parser-apply
```

From the repository root, the repeatable Windows helper discovers the matching
Visual Studio ASan runtime and invokes the same target:

```powershell
.\scripts\run-diff-tools-fuzz.ps1 -Seconds 60
```

The helper keeps its command-line limits aligned with the fuzz target: inputs
are capped at 8 KiB and the default process memory ceiling is 768 MiB. Adjust
the latter only when diagnosing a sanitizer finding or investigating corpus
growth deliberately:

```powershell
.\scripts\run-diff-tools-fuzz.ps1 -Seconds 300 -MaxRssMb 1024
```

## Execution Environment

`cargo-fuzz` requires a nightly Rust toolchain and a sanitizer runtime. On
Windows with Visual Studio Build Tools, make the matching `clang_rt.asan`
directory available on `PATH` before launching the target. For example:

```powershell
$asanDir = 'C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Tools\MSVC\<version>\bin\Hostx64\x64'
$env:PATH = "$asanDir;$env:PATH"
cargo +nightly fuzz run unified-parser-apply -- -max_total_time=60
```

The harness has completed a bounded Windows run with this configuration. A
sanitizer-capable Linux, WSL, or CI environment remains suitable for longer
campaigns. This environment requirement is separate from parser and
applicator behavior: failures to launch the target must not be classified as
diff-tool findings.

The target accepts arbitrary bytes, parses UTF-8 input with small explicit
limits, and, when parsing succeeds, passes the result to exact application
against caller-owned in-memory files. A crash is a contract failure: malformed
input must produce a bounded typed result, never a panic.

`corpus/` contains admitted valid and malformed seeds. Generated crashes and
fuzz artifacts are ignored by Git until a minimized regression case is
deliberately promoted into `../tests/` or `../fixtures/`.

The working coverage corpus also remains ignored. It is a local search cache,
not reviewed evidence; only the four named seeds are versioned. Promote a
minimized input only after confirming that it captures an actionable parser or
applicator regression.
