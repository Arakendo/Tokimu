# Option B Narrow-Seam Hardening Evidence

| Field | Value |
| --- | --- |
| Status | Slice 3 complete; isolated candidate only |
| Host | Windows x86-64 MSVC |
| Production provider | `glam` 0.29.3 / `d36e7ee` |
| Reviewed provider | `glam` 0.33.3 / `9928729` |
| Public caller source | identical integration test under both features |

## Candidate Shape

The independent `alternative-b-narrow-seam` crate exposes exactly three
checked construction functions plus bounded operation/failure categories. It
retains the current foreign `Vec3`/`Mat4` value vocabulary by design, but no
public function or error names `glam::camera`, a provider module, or a provider
error.

| Measure | Result |
| --- | ---: |
| candidate source | 190 lines / 167 nonblank |
| contract test source | 177 lines / 164 nonblank |
| provider construction references | 6 total: 3 for each exact revision |
| provider-specific public error references | 0 |
| contract tests | 4 |

Each public operation validates once, invokes exactly one private provider
construction function, and scans the resulting 16 scalar values once. The
private revision switch changes only those three construction calls:

```text
0.29.3: Mat4 associated constructors
0.33.3: private glam::camera functions
```

No public caller source changes between revisions.

## Contract Evidence

The same downstream integration tests pass against both exact provider trees.
They independently check:

- eye/target view-space mapping;
- orthonormality and positive basis determinant;
- perspective near/far mapping to GL NDC `[-1, 1]`;
- orthographic X/Y/depth extent mapping;
- non-finite input classification;
- zero and collinear view rejection;
- invalid-frustum classification;
- a finite near-collinear view; and
- a finite-input/non-finite-result control.

The first perspective test incorrectly used `transform_point3` as though it
performed projective division. It failed against 0.29.3 (`-0.1` rather than
NDC `-1`) and was repaired to compute clip Z/W independently from the matrix
columns. This was a test-harness semantic mistake, not a provider difference,
and is retained because Full B must keep affine point transformation separate
from checked projective transformation.

## Validation

```text
cargo test --manifest-path .../alternative-b-narrow-seam/Cargo.toml
    0.29.3: 4/4 passed

cargo test --manifest-path .../alternative-b-narrow-seam/Cargo.toml \
  --no-default-features --features provider-033
    0.33.3: 4/4 passed

cargo clippy ... --no-default-features --features provider-033 \
  --no-deps --all-targets -- -D warnings
    passed

cargo test -p tokimu-math-study --locked --offline
    focused shared study: 63 unit tests and 6 integration tests passed

cargo fmt --all -- --check
cargo fmt --manifest-path .../alternative-b-narrow-seam/Cargo.toml -- --check
    passed
```

Unsuppressed 0.29.3 Clippy remains blocked by the already retained provider
generated-source warning flood. With that known provider warning class
suppressed for the focused run, the Narrow-B code passes strict Clippy. This is
not reported as a clean provider gate.

## Placement Result

If AR-0029 later admits this seam, the smallest existing placement is
`tokimu-core::math`:

- it already owns the five public ordinary-math names;
- construction is pure, engine-neutral, and provider-independent in meaning;
- `tokimu-render` remains the owner of `Camera` lifecycle and defaults; and
- WGPU clip-depth conversion remains private renderer/provider adaptation.

No new crate, camera service, renderer state, or lifecycle abstraction is
earned. This placement result is experimental guidance, not authorization to
modify the stable module.

## Claim Limit

This proves provider-update shock absorption for the three construction
families on one native host. It does not prove actual browser parity,
representative runtime migration, whole-workspace admission, performance, or
that Full B is warranted.
