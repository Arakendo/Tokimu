# Alternative D Initial Findings

| Field | Value |
| --- | --- |
| Status | Interim evidence; expansion intentionally paused |
| Candidate | Bounded upstream-derived scalar `Vec3` |
| Upstream | `glam` 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Derived source size | 125 Rust lines, including tests (2026-08-07) |
| Direct provider dependency | None |
| Provenance artifacts | `README.md` and `UPSTREAM-NOTICE.md` |

## Evidence

- The bounded `Vec3` slice passes the shared initial vector conformance cases
  on native and the study builds for `wasm32-unknown-unknown`.
- Its source carries exact upstream revision, path, dual-license references,
  local modifications, and an explicit no-expansion-without-evidence rule.
- It avoids the retained provider at build and runtime for this slice.

## Cost Visible Before Expansion

- Alternative C's current original scalar/vector/matrix implementation is 557
  Rust lines including tests. Extending D from one derived vector type toward
  the current matrix surface would duplicate both implementation and
  provenance-review work.
- Each D expansion requires a new upstream source-unit selection, attribution,
  license relationship, local-change record, target validation, and update
  policy. That is a real maintenance obligation, not incidental documentation.
- No evidence presently shows that upstream lineage gives D a semantic,
  correctness, performance, or migration advantage over C's original source.

## Interim Disposition

**Do not expand D yet.** Preserve the valid bounded `Vec3` evidence, but defer
additional copied source until a measured deficit in C or a specific upstream
compatibility need makes lineage materially valuable. This prevents the study
from turning “five types” into an unearned partial fork.
