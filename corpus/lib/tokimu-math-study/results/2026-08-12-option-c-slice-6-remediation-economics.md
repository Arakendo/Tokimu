# Option C Slice 6: Ring 0 Update And Remediation Economics

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Bounded maintenance model; not a dependency-update authorization |
| Scope | Alternative A direct `glam` provider versus C0/C1 owned scalar candidate |
| Governing policy | ADR-0005, ADR-0008, ADR-0009, ADR-0010, ADR-0011 |

## Purpose

This is a responsibility model, not a line-count contest. It asks what a
maintainer must actually do when an important math issue occurs. C ownership
does **not** make an implementation correct or secure; it moves diagnosis,
proof, optimization, portability, and rollback responsibility to Tokimu.

The known A baseline is pinned `glam` revision
`d36e7eeff05338c56c4aa8d59fc2615e7963b1b7`. Its current warning cleanup route
to the first released upstream repair changes about 170 files, 29,090 insertions
and 10,156 deletions, including generated and SIMD source. That is concrete A
review pressure, not a claim that an update is unsafe or that C should win.

## Four Update Classes

| Class | A: pinned foreign provider | C: owned candidate | Proportional response |
| --- | --- | --- | --- |
| Compiler-warning/toolchain cleanup | Identify upstream fix, choose exact revision, review source/diff/closure/features/generated+unsafe surface, rerun Ring 0 gates, retain old/new audit and rollback gitlink | Identify whether warning is in Tokimu code; make the smallest source change, rerun scalar/native/WASM/conformance/performance controls, retain regression and rollback commit | Current `glam` warning is this class; its released update is substantive rather than a routine bump |
| Bounded correctness fix | Prove exact provider behavior and caller impact; review upstream patch plus all selected changed closure; differential/target regression then switch submodule only after ADR-0010 review | Reproduce with independent oracle and contract test; repair only selected operation; prove reference/C1 behavior, native/WASM, malformed input, and affected callers | Neither source ownership nor upstream reputation replaces the reproduced contract test |
| Target regression | Identify target/backend/feature trigger and whether regression is provider, toolchain, or adaptation; compare prior/new pins and selected target code; preserve fallback or retain old pin | Minimize target-specific branch; prove scalar reference equivalence, unavailable-target behavior, native/WASM divergence cause, and maintenance owner | A target optimization must not be silently accepted; C target code cannot bypass its scalar control |
| Critical security fix | Identify exposure in the compiled Ring 0 closure; isolate or move outward if possible; use ADR-0005 provisional path only when full audit cannot precede mitigation; bound revision, exposure, missing audit, date/exit condition | Identify whether the defect is in owned logic, build pipeline, or release process; contain exposure, patch with adversarial regression, review authority/inputs, and release rollback/forward plan | Urgency reduces elapsed time, not audit responsibility; self-authorship is not security evidence |

## Required Responsibilities By Alternative

| Responsibility | A direct provider | C owned candidate |
| --- | --- | --- |
| Discovery | Upstream advisories/releases, toolchain and local corpus failures, closure scan | Local tests/corpus/oracle discrepancies, toolchain and target failures |
| Exact review unit | Parent gitlink, upstream diff, source tree, feature and Cargo closure | Tokimu commit/range, affected operations and numerical contract |
| Provenance | Submodule commit, upstream URL, source-tree identity, closure provenance | Tokimu source history and review; any oracle remains outside executed Ring 0 code |
| Correctness proof | Provider behavior plus Tokimu contract/caller regression | Independent scalar/reference behavior plus Tokimu contract/caller regression |
| Unsafe/SIMD/build review | Required for every selected upstream delta and transitive build/proc-macro change | Required only if C actually adds it; C1 currently adds neither |
| Target validation | Re-run native/WASM/affected target matrix for the new provider | Re-run native/WASM/affected target matrix for the affected owned operation |
| Rollback | Revert audited gitlink/lock/patch atomically to known revision | Revert owned commit while retaining failing regression and issue record |

## Stagnation Trigger

If important fixes repeatedly cannot be reviewed, pinned, or safely applied
under the current A closure, maintainers must reopen either ADR-0010
proportionality or the volume of foreign Ring 0 execution. They must **not**
silently weaken audit requirements or suppress the evidence. Conversely, if C
cannot produce correct, portable, performant repairs within its small declared
surface, the study must retain that as evidence that A may be lower risk.

## Present Disposition

- For A, ADR-0010 treats every provider revision as a Ring 0 source change;
  warning-only, correctness, and target fixes do not receive a lighter update
  gate. A time-sensitive security mitigation may use ADR-0005 provisionally,
  but must retain the missing audit and exit condition. A permanent lighter
  provider-update class would require an ADR-0010 revision, not local practice.
- For C, the ordinary owned source review is governed by the applicable
  ADR-0008/0009/0011 evidence. It cannot be called an ADR-0010 exception
  because it contains no foreign provider revision.
- A remains an auditable but costly foreign-provider path under ADR-0010.
- C0/C1 removes that provider execution only if a future migration earns it; it
  replaces broad provider review with direct Tokimu numerical maintenance.
- The C1 affine optimization is a reminder that owned code creates active
  performance debt rather than eliminating it.
- No ADR-0010 adjustment, dependency update, or Alternative C selection is
  recommended by this model.
