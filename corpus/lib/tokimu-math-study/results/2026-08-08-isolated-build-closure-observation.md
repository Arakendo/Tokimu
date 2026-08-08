# Isolated B/C Release Build-Closure Observation

| Field | Value |
| --- | --- |
| Status | One-host fresh-build observation; not a universal compile-time conclusion |
| Date | 2026-08-08 |
| Target | Host default: `x86_64-pc-windows-msvc` |
| Profile | `release` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |
| Host CPU / OS | Not retained: sandbox access to Windows CIM host queries remains denied |
| Repetition | One fresh target directory per isolated candidate |

## Commands

```powershell
Measure-Command { cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-b-provider-backed/Cargo.toml --release --offline --target-dir target/math-study-build-observation-b }
Measure-Command { cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --release --offline --target-dir target/math-study-build-observation-c }
```

## Results

| Candidate | Direct dependency closure | Elapsed time |
| --- | --- | ---: |
| B — provider-backed vocabulary | pinned local `glam` | 3.518 s |
| C — narrow owned implementation | none | 0.322 s |

The B build compiled `glam` and emitted its already-recorded deprecated
`#[must_use]`-attribute warnings. C compiled only the isolated candidate crate.

## Interpretation Limits

The targets are intentionally small and do not model the whole workspace,
incremental behavior, a renderer/application closure, clean dependency caches,
parallel contention, or a developer's normal edit-build loop. Different hosts,
toolchains, target settings, provider features, or candidate growth can reverse
or change the difference. This is retained build-closure evidence, not a
compile-time selection threshold.
