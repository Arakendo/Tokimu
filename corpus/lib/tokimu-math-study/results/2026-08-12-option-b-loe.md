# Option B Level-Of-Effort Ledger

| Field | Value |
| --- | --- |
| Study start | 2026-08-12 |
| Initial observation checkpoint | 2026-08-12T18:55:57-07:00 |
| Actors | maintainer direction; Codex implementation and evidence work |
| Production migration | none |

## Accounting Rules

- **Active effort** is hands-on inspection, implementation, validation, and
  evidence-writing time.
- **Automation wall time** is tool execution that does not require continuous
  active attention.
- **Blocked time** is an environmental or decision gate and is not charged as
  active implementation.
- **Rework** names discarded or repaired work separately.
- **Recurring cost** identifies work that remains necessary for later private
  provider updates even if a B candidate is selected.

## Entries

| Work | Actor | Active effort | Automation wall time | Classification | Recurring? | Result |
| --- | --- | ---: | ---: | --- | --- | --- |
| Freeze A, 0.33.3, Narrow-B, Full-B, and C identities | Codex | 8 min | <1 min | study setup | no | complete |
| Inspect existing Full-B source, surface, crossings, and tests | Codex | 9 min | <1 min | source review | when wrapper changes | complete |
| Compile/list isolated Full-B suite | Codex | 2 min | recorded by command separately | validation | yes | 10 tests; warning flood reproduced |
| Refresh current five-type and constructor pressure | Codex | 13 min | <1 min | caller audit | when caller pressure changes | complete |
| Write control, result, pressure, and effort evidence | Codex | 15 min | N/A | retained evidence | per study/update | complete |
| Define Narrow-B and Full-B provider-neutral contracts | Codex | 22 min | N/A | semantic contract | when owned meaning changes | complete |
| Build and harden dual-provider Narrow-B candidate | Codex | 31 min | 18 sec focused build/test/lint plus provider-warning output | candidate implementation | private adapter changes per update | complete |
| Repair mistaken perspective test oracle | Codex | 4 min | <1 min | test rework | no | separated affine point transform from projective division |
| Refactor Full B for dual exact providers | Codex | 17 min | 5 sec build/test | candidate implementation | private adapter changes per update | one shared wrapper source; three private constructor adapters |
| Add checked failures and independent Full-B suite | Codex | 24 min | 8 sec build/test/lint | contract hardening | when owned meaning changes | 10 unit + 5 external tests pass on both pins |
| Inventory references, ergonomics, and retained evidence | Codex | 12 min | <1 min | source review/evidence | when wrapper changes | 39 namespace refs; 44 delegated inner uses; explicit non-claims |
| Port representative Narrow-B callers and add Full-B chart/storage pressure | Codex | 18 min | 1 min build/test | caller migration | when semantic pressure changes | five Narrow-B scenarios; nine Full-B A/B/C caller modules; no new chart operation |
| Count crossings, mutation friction, and allocation behavior; retain Slice-5 evidence | Codex | 14 min | 8 sec allocation controls | source review/evidence | when callers or renderer boundary change | separate Narrow/Full accounting; all measured crossings allocation-free |
| Add dual-pin Node/WASM harnesses and execute default/`simd128` gates | Codex | 17 min | 58 sec execution plus compile time | target validation | when provider/toolchain changes | A/B/C 14 tests; Narrow B 8 tests; Full B 5 tests pass in every executed combination |
| Compile both candidates for ARM64 and retain representation observations | Codex | 8 min | 29 sec compile/run | target and representation evidence | when provider/toolchain changes | both pins compile with NEON cfg; Full-B native representation equal under both pins |
| Attempt actual-browser attachment and retain Slice-6 evidence | Codex | 7 min | <1 min | browser gate/evidence | when browser is available | no attachable browser; gap retained without substituting Node |
| Build Narrow-B semantic-seam and Full-B dual-pin performance controls | Codex | 18 min | 38 sec compile/run | performance harness | when contract/provider changes | isolated checked construction, transform, inverse, stereo, and handoff costs |
| Execute caller, allocation, compile/artifact, formatting, and strict-lint gates | Codex | 11 min | 74 sec | performance/quality validation | when provider/toolchain changes | mixed Full-B result; GLB/inverse gates lost; zero measured allocations |
| Analyze and retain Slice-7 evidence without speculative remediation | Codex | 9 min | N/A | evidence/decision accounting | when performance evidence changes | Narrow B cost bounded; Full B not declared zero-cost |
| Pressure checked failures, repair overflow classification, and execute both pins on native/WASM | Codex | 18 min | 58 sec build/test/lint | verification and containment | when owned failure semantics or provider changes | bounded failures agree; no native unwind or WASM trap |
| Reconcile candidate authority and exact selected closure with ADR-0010 audits | Codex | 12 min | 3 sec source/closure scans | security/provenance review | every provider update | no new authority; all A provider obligations remain |
| Retain Slice-8 evidence and review disposition | Codex | 9 min | N/A | retained evidence | when failure/security evidence changes | wrapper ownership explicitly not a security claim |
| Replay camera update shock and count A/Narrow/Full caller/adapter effects | Codex | 14 min | 4 sec source/test queries | maintenance economics | every provider API update | Narrow absorbs observed shock; Full adds no proportional benefit |
| Execute non-camera semantic-shock observer and ordinary-operation replay | Codex | 11 min | 4 sec build/run | semantic maintenance | when provider semantics or caller operations change | Full delegation drifts on NaN `min`/`max`; `dot` exposes per-operation cost |
| Retain recurring/one-time Slice-9 economics and disposition | Codex | 10 min | N/A | retained evidence | when update evidence changes | all ADR-0010 implementation-audit work remains |
| Compare real caller ergonomics, trait/field pressure, and ecosystem boundaries | Codex | 13 min | 3 sec source scans | API/ecosystem review | when callers or stable boundaries change | real Full-B `Sum` and mutable-component gaps retained |
| Generate normal/strict Rustdoc and account owned documentation surface | Codex | 7 min | 3 sec documentation builds | documentation evidence | every admitted API change | both experimental surfaces miss docs; Full bill substantially broader |
| Retain Slice-10 evidence without broadening either candidate | Codex | 9 min | N/A | retained evidence | when API/ergonomic evidence changes | Narrow clearer at semantic sites; Full remains bounded and incomplete |
| Cross-review AR-0026, AR-0028, AR-0029, SDD, stereo, and renderer ownership | Codex | 14 min | <1 min source scans | architectural evidence review | when spatial/view evidence changes | semantic roles remain above ordinary math; future portal pressure stays open |
| Execute chart, dual-pin Narrow multi-view, and private WGPU clip controls | Codex | 8 min | 76 sec build/test including known 0.29.3 warning output | focused validation | when provider/camera boundary changes | all focused controls pass; no new operation pressure |
| Retain Slice-11 evidence and update both B/C operation ledgers | Codex | 11 min | N/A | retained evidence | when spatial operation pressure changes | zero operation growth; no stable/public change |
| Build the A/Narrow/Full/C comparative matrix and bounded recommendation | Codex | 18 min | N/A | decision evidence | when candidate evidence changes | continue Narrow incubation; park Full; keep production A |
| Reconcile AR-0029 and explicit admission/resumption gates | Codex | 8 min | <1 min document checks | architectural gate | until maintainer decision | review guidance remains under review; no migration plan authorized |

Active estimate through the Slice 12 recommendation: **421 minutes**. This is an evidence estimate, not a
timesheet-grade measurement. Later entries must retain long-running benchmark,
target-install, browser, and blocked decision time separately.

## Early Maintenance Finding

Full B does not eliminate the recurring ADR-0010 provider-update burden. It may
reduce public caller migration, but the provider still compiles and executes;
its warning, provenance, source-delta, generated-code, unsafe/SIMD, target,
security, and legal work remains. Narrow B is cheaper in wrapper surface but
has the same private-provider maintenance fact.
