# Option C Slice 8: CPU Bulk Reference And Scaling Controls

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | CPU-only corpus evidence; no GPU provider or public capability proposed |
| Workloads | Ordered conservative AABB/frustum classification; ordered point/frustum classification |
| Target/profile | Native release on the recorded development host; `cargo run -p tokimu-math-study --release --bin measure_bulk_references --locked --offline` |
| Semantics | Caller IDs and input order retained; touching a frustum plane remains a candidate; invalid values are explicitly rejected |

## Reference Shape

`bulk_reference.rs` is deliberately a CPU-only semantic control. It accepts
caller-supplied planes and ordered identified bounds/points and returns one
ordered `ClassificationRecord` for every input. It does not sort, batch,
schedule, compact authoritative geometry, mutate a scene, call WGPU, or infer
Doom/AR-0025 topology.

The reference exposes three result observations from the same ordered records:

| Observation | Meaning | Slice 8 status |
| --- | --- | --- |
| Full records | Every input ID plus candidate/rejection reason | Implemented and checksum-controlled |
| Compacted IDs | Caller-derived candidate IDs, in input order | Implemented; no geometry ownership implied |
| Count only | Caller-derived candidate count | Implemented; no information is discarded by the classifier itself |
| GPU-consumed next stage | Provider-side continuation without host readback | Explicitly deferred to Slice 9; CPU evidence must not invent GPU synchronization semantics |

The reusable-output variants are measurement scaffolding only. They distinguish
resident-input/fresh-result allocations from resident-input/reused-result
storage, but do not constitute an admitted public buffer API.

## Validation

The library test suite passed with **60 tests**, including the new controls for:

- conservative plane touching and explicit plane rejection;
- invalid/non-finite point rejection without reordering;
- deterministic input generation;
- reused-output identity/order retention;
- compacted IDs, count-only observations, and deterministic checksums.

The workload generator is fixed-seed and bounded at one million elements. The
largest case deliberately remains host-memory-safe; no unbounded size argument
or GPU allocation is accepted.

## Measured Native Release Controls

Each number is the median of five samples in nanoseconds. Checksums were stable
across one-shot, resident/fresh-result, and resident/reused-result modes for a
given workload and size. Timings are one host observation, not a portability or
GPU-performance claim.

| Workload | Size | One-shot | Resident input, fresh result | Resident input, reused result | Candidates | Checksum |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Ordered AABB | 1K | 13,200 | 10,300 | 10,100 | 165 | `4c5930c834cc46d2` |
| Ordered point | 1K | 21,400 | 7,100 | 8,700 | 130 | `5f3162208857c196` |
| Ordered AABB | E1M1 1,861 | 54,100 | 38,000 | 19,000 | 317 | `8f5c5bfd1d062db1` |
| Ordered point | E1M1 1,861 | 41,000 | 24,600 | 17,500 | 232 | `1a2854c78bb0449e` |
| Ordered AABB | 10K | 221,100 | 157,700 | 102,600 | 1,615 | `fd322abd04404ddc` |
| Ordered point | 10K | 185,300 | 82,400 | 146,000 | 1,200 | `ce4ae9f0aa3c033f` |
| Ordered AABB | 100K | 3,012,300 | 2,345,600 | 1,998,100 | 16,030 | `ebb871e67a72b75c` |
| Ordered point | 100K | 2,045,400 | 1,614,400 | 1,428,200 | 12,471 | `1c78377960104b68` |
| Ordered AABB | 1M | 30,685,700 | 24,354,300 | 20,524,100 | 160,505 | `bc0679af5491e7b3` |
| Ordered point | 1M | 19,947,600 | 16,065,200 | 14,783,500 | 125,019 | `8e92b8ee0c1c8bab` |

The dedicated E1M1-size `1,861` control is a negative/control workload only:
an E1M1 full-submission-sized list does **not** establish a general compute
threshold.

## Interpretation And Limits

- The large 1M controls establish that both semantic references can be
  exercised at a bounded scale without a window or GPU.
- Inputs constructed per observation materially change measured total time;
  later WGPU work must separately account for upload/initialization/readback
  rather than compare a dispatch in isolation with this one-shot path.
- Reusing result storage helps the AABB control at larger sizes, while the 1K
  and 10K point control shows normal measurement noise/branch/cache effects.
  This is not a claim that reuse universally wins.
- Candidate counts are intentionally workload-local and do not identify
  visible Doom geometry or authorize renderer-owned culling.
- Slice 8 validates CPU semantics and scaling controls only. Slice 9 must
  decide whether any WGPU provider experiment earns its own initialization,
  failure, native/browser, synchronization, and fallback evidence.
