# Option B Failure, Security, And Provenance Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Candidates | Narrow B and Full B |
| Providers | exact local `glam` 0.29.3 (`d36e7eef`) and 0.33.3 (`99287290`) |
| Native host | Windows x86-64 MSVC |
| WASM host | Node.js through `wasm-bindgen-test-runner` |
| Production migration | none |

## Boundary And Failure Matrix

Narrow B owns three checked semantic constructors. Its public failures retain
only `ConstructionOperation` and `ConstructionFailure`. Full B owns the checked
ordinary-math operations that its pressured corpus callers require, with the
same bounded pair expressed as `MathOperation` and `MathFailure`. Neither error
contains a provider error, provider type, raw input payload, allocation, source
path, or human diagnostic string.

The following cases execute unchanged against both exact provider revisions:

| Pressure | Narrow B | Full B |
| --- | --- | --- |
| malformed typed scalar combinations | invalid frustum | invalid frustum |
| `NaN` / infinity input | non-finite input | non-finite input |
| zero-length input | degenerate view | zero-length normalize / degenerate view |
| collinear basis | degenerate view | degenerate view |
| near-degenerate underflow | degenerate view | zero-length normalize |
| finite-component arithmetic overflow | non-finite result | non-finite result |
| singular matrix | not in Narrow-B surface | singular inverse |
| zero homogeneous projection divisor | not in Narrow-B surface | zero homogeneous W |

The typed candidates do not parse unstructured bytes or dynamic operation
names. An operation outside the closed public surface is therefore absent at
compile time rather than accepted and converted into a runtime provider call.
This is the study's unsupported-operation result; no speculative dispatcher or
`Unsupported` error variant was added merely to make that row executable.

One ordinary defect was found and repaired in the experimental Full-B checked
surface: finite `f32` components could overflow `length_squared`, after which
the provider could return a finite zero from normalization. `try_normalize`
now classifies the overflow as `NonFiniteResult`; an underflowed magnitude
remains a distinct `ZeroLength` rejection. Both pins and both executed targets
agree.

## Containment And Target Results

Native results under each provider:

- Narrow B: 5 contract tests and 4 representative-caller tests pass;
- Full B: 10 internal tests and 6 external contract tests pass; and
- explicit native `catch_unwind` controls confirm the checked rejection matrix
  returns normally without unwinding.

Node/WASM results under each provider:

- Narrow B: all 4 WASM-exported contract tests and 4 caller tests pass; and
- Full B: all 5 WASM-exported external contract tests pass.

The native unwind controls are secondary observations, not a recovery design.
The actual contract is ordinary `Result` return. WASM cannot use unwinding as
the proof, so successful completion of the invalid-input tests is the relevant
no-trap evidence. Actual-browser execution remains unavailable from Slice 6
and is not implied by Node-hosted WASM.

Unchecked compatibility methods retained to measure migration pressure are
not covered by the bounded-failure claim. Full B would need either caller
migration to checked paths or an explicit unchecked policy before stable
admission. The study does not relabel provider panic/non-finite behavior as a
Tokimu guarantee.

## Authority And Lifetime Review

A source scan of the two isolated facades and shared Full-B implementation
found no thread-local or mutable static state, lazy global, filesystem/network
access, thread creation, synchronization owner, heap collection/owner,
callback registration, unsafe block, panic, `unwrap`, or `expect` in candidate
implementation code.

Full B contains four corpus-only `extern "C"` observation probes. They return
plain scalar observations and invoke no host callback or I/O. They are not part
of the proposed stable math contract and do not grant provider authority. The
candidate values own their private provider values directly; no borrowed
provider lifetime or retained provider object crosses the boundary.

## Selected Closure And ADR-0010 Reconciliation

`cargo tree --edges normal,build` reports the same selected runtime closure for
both candidate shapes:

```text
candidate
`-- exact local glam provider
```

The 0.29.3 production submodule is clean at
`d36e7eeff05338c56c4aa8d59fc2615e7963b1b7`. The isolated 0.33.3 source is
clean at `9928729066db87d97fa779e129469721a289beae`. The candidate selects
`default-features = false` and `std` only. `wasm-bindgen-test` and its closure
are dev-only test machinery and do not enter the normal/build runtime closure.

The existing audits remain the source of truth for source identity, legal
obligations, generated source, unsafe/SIMD surface, target behavior, and the
time-bounded advisory checks. B changes none of those obligations:

- foreign `glam` code still compiles and executes in Ring 0;
- the unsafe/SIMD implementation still requires review;
- source provenance, exact pinning, licenses/notices, advisories, target gates,
  rollback, and update diff review remain identical to A;
- private vocabulary does not make foreign implementation Tokimu-authored; and
- every future provider revision remains an ADR-0010 source change.

## Disposition

Slice 8 passes for the bounded checked surfaces and available native/Node-WASM
targets. Wrapper ownership improves diagnostic and semantic ownership but is
not a security or provenance boundary. Narrow B remains the smaller candidate;
Full B has a broader checked-failure and maintenance surface and still exposes
unchecked compatibility paths. Neither candidate weakens ADR-0010, and neither
is admitted by this result.
