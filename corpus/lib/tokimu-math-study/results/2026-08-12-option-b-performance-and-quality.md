# Option B Performance And Code-Quality Gate

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Host | Windows x86-64, release profile |
| Samples | 15 per timed workload |
| Production provider | pinned `glam` 0.29.3 |
| Candidate provider | isolated `glam` 0.33.3 |
| Production migration | none |

All medians below are observations for the named workload and host. They are
not generalized engine budgets or proof of a stable performance contract.

## Narrow-B Semantic-Seam Isolation

The isolated benchmark constructs one valid view, perspective projection, and
orthographic projection per iteration. The direct path calls the selected
provider; the checked path calls Narrow B's bounded contract.

| Pin | Iterations | Direct median | Checked median | Ratio | Absolute checked cost above direct |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0.29.3 | 500,000 | 7.4723 ms | 26.5846 ms | 3.56x | 38.2 ns per three-constructor bundle |
| 0.33.3 | 500,000 | 7.5509 ms | 27.1902 ms | 3.60x | 39.3 ns per three-constructor bundle |

The ratio is material in a constructor-only stress loop; describing Narrow B
as free would be false. The absolute delta is small for a caller that rebuilds
one camera bundle occasionally or once per frame, but the study has not earned
that assumption for mass-camera or bulk-construction workloads. The cost buys
finite-input, degenerate-view, invalid-frustum, and finite-result
classification. No checks were removed, hidden behind debug mode, or replaced
with unsafe shortcuts to improve the result.

## Complete Caller-Shaped Workloads

| Workload | Iterations | A median | Full-B median | C median | Full B vs A |
| --- | ---: | ---: | ---: | ---: | ---: |
| general transform | 1,000,000 | 3.4737 ms | 3.4137 ms | 3.0930 ms | -1.7% |
| stereo camera | 100,000 | 7.2178 ms | 6.8130 ms | 6.8052 ms | -5.6% |
| CAD cursor ray | 100,000 | 3.0015 ms | 2.9389 ms | 12.8411 ms | -2.1% |
| Khronos Box GLB model/floor | 100,000 | 34.6580 ms | 37.3481 ms | 36.0623 ms | +7.8% |
| E1M1 observer camera | 1,000,000 | 44.2988 ms | 44.0324 ms | 50.6488 ms | -0.6% |
| affine inverse isolation | 1,000,000 | 9.0136 ms | 10.6840 ms | 14.5429 ms | +18.5% |

The Full-B GLB and isolated-inverse medians are non-overlapping material
regressions in this run. Their absolute deltas are about 26.9 ns per GLB
iteration and 1.67 ns per inverse. No representation shortcut or inlining
policy was introduced merely to erase them. Under this gate Full B loses the
GLB and inverse workloads pending independent evidence or bounded remediation;
it cannot claim performance equivalence from the other four workloads.

## Full-B Private-Pin Comparison

The same wrapper source was executed for 500,000 iterations under each exact
private pin:

| Workload | 0.29.3 median | 0.33.3 median |
| --- | ---: | ---: |
| transform | 10.6753 ms | 10.7190 ms |
| inverse | 5.4427 ms | 5.2606 ms |
| stereo plus scalar-column handoff | 26.5537 ms | 25.6368 ms |

The private update does not introduce a material regression in these bounded
Full-B controls. This does not supersede the complete-workload result above.

## Allocation And Representation Boundaries

The existing release allocation controls observed zero steady-state
allocations for:

- one million A/B/C transform iterations;
- one million Full-B/C scalar-column upload crossings; and
- 100,000 A/B/C stereo-camera iterations.

All renderer handoffs remain scalar-column copies. No normalization or camera
construction is performed implicitly at that boundary, and no unsafe layout
equivalence was added.

## Compile And Artifact Observations

Fresh target directories were used once per candidate/pin; the immediately
following build is the incremental observation. OS cache state means the cold
numbers are descriptive, not a fair provider-version speed contest.

| Candidate / pin | Cold | Incremental | Candidate artifact |
| --- | ---: | ---: | ---: |
| Narrow B / 0.29.3 | 3.425 s | 73.1 ms | rlib 27,082 bytes |
| Narrow B / 0.33.3 | 1.092 s | 71.0 ms | rlib 27,172 bytes |
| Full B / 0.29.3 | 3.764 s | 106.9 ms | rlib 116,376; DLL 109,056 bytes |
| Full B / 0.33.3 | 1.440 s | 104.3 ms | rlib 115,640; DLL 109,056 bytes |

The 90-byte Narrow-B and 736-byte Full-B rlib differences are compiler
artifacts, not stable size contracts. Full B's larger artifact and incremental
build are expected evidence of its substantially broader wrapper surface.

## Quality Gate

- workspace and both isolated manifests pass formatting checks;
- Narrow B and Full B pass focused all-target strict Clippy against 0.33.3
  without lint suppression;
- the filesystem emitted a non-code hard-link cache warning, which is not a
  candidate lint; and
- strict 0.29.3 output remains blocked by its retained provider-owned
  `#[must_use]` warning flood. `-Awarnings` was used only to make measurement
  output readable, not to classify the old provider as clean.

A focused source review found no hidden state, I/O, process/thread authority,
unsafe block, panic/unwrap path, or shadow numerical source of truth in either
candidate. Narrow B and Full B intentionally repeat some validation/provider
adapter structure because they are competing isolated alternatives. That
duplication must disappear if one candidate is ever admitted; it is not a
proposal to maintain two production semantic constructors.

## Disposition

Narrow B demonstrates a real but bounded checked-construction cost. Full B is
allocation-free and competitive in four complete caller shapes, but fails the
current GLB and inverse performance gates. This is evidence against treating
Full B as a drop-in zero-cost vocabulary, not authorization to weaken its
contract or optimize representation speculatively.
