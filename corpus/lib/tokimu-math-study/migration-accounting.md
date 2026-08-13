# Representative Migration Accounting

## Scope and Counting Rule

This records only corpus-local study work. A **source edit** means one
corpus-local Rust module that contains an alternative-specific branch or
adapter. It does not count generated targets, tests that only invoke the
fixture, or unchanged stable Tokimu source.

An **explicit conversion** means a candidate/provider matrix handoff at the
retained current-renderer boundary, not an internal B wrapper delegation.

## Source and Boundary Counts

| Item | A control | B: provider-backed vocabulary | C: owned subset | D: bounded derivation |
| --- | --- | --- | --- | --- |
| Stable Tokimu source edits | 0 | 0 | 0 | 0 |
| Candidate-specific corpus modules | 0 | 9 | 9 | 1 (`Vec3` only) |
| Candidate modules counted | N/A | `alternative_b`, `migration_b`, plus 7 shared caller fixtures | `alternative_c`, `migration_c`, plus 7 shared caller fixtures | `alternative_d` only |
| Renderer helper | None | `provider_upload_matrix` unwrap | `provider_upload_matrix` column reconstruction | None: no `Mat4` |
| Observed explicit matrix crossings | 0 | 9 | 9 | 0: blocked |
| Candidate-facing provider signature leak | Existing A public vocabulary | 0 | 0 | N/A |
| Existing renderer provider seam | `tokimu::Camera { view, projection: Mat4 }` | Retained explicit boundary | Retained explicit boundary | Not reached |
| Rollback | No experiment change | Delete corpus-local candidate/fixture modules | Delete corpus-local candidate/fixture modules | Delete isolated derivation |

The seven shared caller fixtures are `hello-3d-mono`, `hello-glb`, `hello-cad`,
`hello-fps`, `hello-hole-punch`, `hello-3d-stereo`, and `hello-asteroids`.
They contain B/C branches but do not alter their original corpus applications.

## Crossing Breakdown

| Retained boundary case | B | C | Evidence |
| --- | ---: | ---: | --- |
| Direct representative upload | 1 unwrap | 1 reconstruction | `migration_b` / `migration_c` upload workload |
| One current `tokimu::Camera` | 2 unwraps | 2 reconstructions | view plus projection handoff test |
| Stereo current cameras | 4 unwraps | 4 reconstructions | left/right view plus projection fixture |
| Orthographic current camera | 2 unwraps | 2 reconstructions | identity plus projection fixture |
| **Total** | **9** | **9** | bounded study set only |

Both B and C have a retained round-trip test at the actual matrix boundary:
candidate matrix → provider matrix → candidate matrix. B uses its crate-private
provider wrapper conversion; C uses the provider column array and its own
`from_cols_array`. The tests are exact for the retained finite camera matrix.

## Interpretation

The count is migration friction, not a claim that conversions allocate or are
universally expensive. Allocation probes retain zero allocations for the
measured paths, while the native and WASM performance records remain separate.
The main provider leak is not inside candidate signatures: it is the present
public renderer `Camera` vocabulary. A selection would need a separate facade
migration decision; this study leaves stable source unchanged.

## 2026-08-12 Option-B Slice-5 Refinement

The earlier table predates the independently measured Narrow-B candidate and
the Doom observer/chart additions. Current detailed accounting is retained in
`results/2026-08-12-option-b-representative-migration.md`.

- Narrow B represents five caller scenarios in one isolated external module,
  changes no value signature, needs no accessor or value conversion, and makes
  eight checked semantic-construction calls through its private adapter.
- Full B now has nine A/B/C-comparable caller modules plus `migration_b`.
  The representative set contains four private wrapper-bearing helper
  signatures, eight scalar accessor substitutions, one column setter, and the
  same nine explicit renderer matrix crossings counted above.
- Doom collision remains source-scalar/integer code and is intentionally not
  forced through either candidate. The Doom observer and FPS movement fixtures
  carry the applicable ordinary vector/matrix pressure.
- Current allocation controls remain zero for A/B/C transforms and stereo and
  for B/C renderer uploads. This does not claim that application-owned output
  collections such as transformed GLB vertices allocate nothing.
