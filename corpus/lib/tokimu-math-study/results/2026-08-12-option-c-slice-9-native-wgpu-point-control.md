# Option C Slice 9: Native WGPU Ordered Bulk Controls

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Complete bounded Slice 9 corpus evidence; actual provider-loss and in-flight cancellation remain deferred |
| Operation | Slice 7 ordered point and AABB/frustum classification controls |
| Provider | WGPU 23, Vulkan, AMD Radeon RX 7900 XTX discrete adapter |
| Command | `cargo run -p tokimu-math-study --release --bin measure_bulk_wgpu_native --offline -- <count>` |

## Scope

The corpus binary owns its WGSL, storage buffers, provider acquisition, command
submission, readback, and disposal. It does not add a Tokimu compute API,
renderer scheduling/buffer vocabulary, authoritative scene state, or Ring 0
provider dependency.

Points and AABBs use the same fixed-seed source generation and inclusive
unit-frustum classification as Slice 8. GPU output remains one ordered flag per
input. The caller compares every readback flag with CPU reference output before
reporting success, so a matching count alone cannot hide an identity/order
mismatch.

The shaders deliberately implement only the corpus unit-cube control. They do
not yet establish arbitrary caller-supplied plane storage, a generic GPU bounds
format, or a Tokimu compute contract.

## Native Observation

| Workload | Count | Candidate count | Adapter request | Device request | Setup | Median warm upload | Median warm dispatch | Median warm readback | Total caller time |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Point | 100K | 12,471 | 149,290,600 ns | 41,854,100 ns | 6,088,800 ns | 384,200 ns | 58,200 ns | 992,700 ns | 413,244,800 ns |
| AABB | 1,861 E1M1 control | 317 | 158,689,900 ns | 43,388,000 ns | 5,975,900 ns | 227,800 ns | 60,700 ns | 584,300 ns | 412,068,400 ns |
| AABB | 100K | 16,030 | 156,903,600 ns | 44,095,500 ns | 6,438,000 ns | 427,800 ns | 58,300 ns | 1,043,400 ns | 415,220,800 ns |
| AABB | 1M | 160,505 | 149,296,900 ns | 42,384,400 ns | 9,230,700 ns | 1,784,100 ns | 67,600 ns | 2,642,600 ns | 429,173,900 ns |

Each successful run agreed exactly with the CPU flags. Warm values are medians
of three same-device, same-buffer lifecycle samples and are retained alongside
the cold adapter/device/setup cost. These are still one-machine observations,
not GPU crossover, browser, or NVIDIA claims. In particular, the E1M1-size
control remains too small to argue that GPU dispatch is useful.

The executable reports `status=completed`, rather than `presented`: this is a
compute/readback control with no presentation surface.

## Failure-Containment Finding

The first run used invalid WGSL field separators. WGPU's unscoped validation
path aborted the executable during shader creation rather than returning the
binary's `Result`. The repaired corpus setup pushes a local `Validation` error
scope before shader/pipeline creation and converts any scoped error to
`provider-validation-rejected: ...`.

This is a corpus-local lifecycle repair. It does not establish a shared Tokimu
diagnostic owner, and it does not replace the broader AR-0024/AR-0027 evidence
about terminal failure delivery. It does show that a provider experiment must
scope creation-time validation if it claims recoverable provider failure.

`measure_bulk_wgpu_native 1000 --invalid-shader` now exits with the explicit
`provider-validation-rejected` result and no process panic. The normal 1K
control still completes and matches the CPU reference after that negative case.

`--cpu-fallback` is a separate, caller-selected bypass control. It computes the
same bounded CPU reference, attempts no WGPU acquisition, and emits exactly one
terminal observation: `status=cpu-fallback`, with the count and candidate count.
It demonstrates that the caller can select a CPU path without a double
CPU/GPU commit; it is not an automatic renderer fallback or a real device-loss
simulation.

## Browser/WASM Execution

`hello-bulk-compute-web` is a separate `wasm32-unknown-unknown` corpus member.
Its DOM host exposes the same fixed 100K ordered-point workload and the same
caller-selected CPU bypass. The fixture built with `--locked --offline`, its
`wasm-bindgen` output was generated, and its local server returned the HTML and
generated module at `http://127.0.0.1:4186`.

An initial DOM-hosted 100K point control established browser execution. The
final revised fixture then reported separate setup/allocation cost and three
same-provider samples:

```text
status=completed; workload=ordered_point; count=100000; candidates=12471;
samples=3; backend=BrowserWebGpu; adapter=; adapter_ms=2.000; device_ms=8.000;
setup_allocation_ms=1.000; warm_upload_ms=1.000; warm_dispatch_ms=0.000;
warm_readback_ms=12.000; total_ms=56.000; build=debug; host=DOM
```

The identical candidate count to the native/Slice 8 reference reflects an
ordered CPU-versus-GPU readback comparison inside the fixture, not a count-only
comparison. The browser/adapter identity was not supplied by the available
observation and is retained as empty rather than inferred. These are debug
browser observations; they establish this host's warm reuse but not
cross-browser/vendor conformance.

The same host also exercised the caller-owned bypass:

```text
status=cpu-fallback; workload=ordered_point; count=100000; candidates=12471;
reason=caller-selected-provider-bypass; observations=1; host=DOM
```

That is evidence that browser caller code can select the bounded CPU outcome
without attempting a second GPU commit. It is not a device-loss simulation or a
shared runtime fallback policy.

The revised browser fixture retains three reused upload/dispatch/readback
samples and a separate `setup_allocation_ms` observation, and exposes a scoped
invalid-WGSL control. Its actual browser failure observation was:

```text
status=provider-validation-rejected; phase=shader-creation;
diagnostic=Error while parsing WGSL: unexpected token; host=DOM
```

The DOM host remained available after this provider rejection. This establishes
bounded shader-creation validation containment, not device-loss or disposal
recovery.

The browser fixture also contains distinct bounded controls for rejected
`count=0` input (before provider acquisition) and explicit destruction of an
idle buffer. Both were manually exercised on the DOM host:

```text
status=input-rejected; input=count=0;
diagnostic=count must be between 1 and 1,000,000; host=DOM

status=disposed; phase=idle-buffer-destroy; observations=1; host=DOM
```

These controls separate local input validation and normal resource release from
the still-open provider-unavailable, device-loss, and in-flight cancellation
cases.

## Slice 9 Boundary And Deferred Pressure

The available WGPU/browser providers did not provide a safe way to manufacture
adapter unavailability, actual device loss, or in-flight cancellation. Those
cases are intentionally deferred rather than simulated and mislabeled. The
known missing NVIDIA coverage also remains explicit. Neither limitation blocks
this bounded candidate experiment: GPU output remains caller-checked corpus
evidence and no Tokimu compute-provider contract is admitted.
