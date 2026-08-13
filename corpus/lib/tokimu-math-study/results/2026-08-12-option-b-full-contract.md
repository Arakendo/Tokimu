# Option B Full Provider-Backed Vocabulary Contract

| Field | Value |
| --- | --- |
| Status | Slice 2 contract; isolated study only |
| Candidate | Full B |
| Foreign surface | none across the candidate boundary |
| Foreign execution | retained private provider; ADR-0010 still applies |

## Boundary Rules

Full B owns value vocabulary and bounded ordinary mechanics. Private provider
values may implement those mechanics, but no provider type, trait, module,
error, layout, or feature name is part of this contract.

All five values are handedness-neutral. Right/left-handed meaning belongs to
named construction or spatial operations, not to `Vec*`, `Quat`, or `Mat4` by
existence. Chart identity, qualified positions, transition orientation,
source-space embeddings, camera lifecycle, and provider clip-depth adaptation
remain above or outside this vocabulary.

## Minimal Surface

| Type | Contract admitted for the study |
| --- | --- |
| `Vec2` | component observation, `ZERO`, construction, array round trip, `Copy`/`Clone`/`Debug`/`PartialEq`; compatibility probe only |
| `Quat` | component observation, `IDENTITY`, XYZW construction, array round trip, `Copy`/`Clone`/`Debug`/`PartialEq`; compatibility probe only |
| `Vec3` | component accessors; zero/one/axis constants; scalar/array construction; ordered arithmetic; component/scalar multiply/divide; length, squared length, distance, dot, cross, finite check, checked normalization, min/max/lerp for finite operands, extend, ordered accumulation |
| `Vec4` | component accessors; explicit-column construction; finite check; truncate; homogeneous use through matrix multiplication and checked projection |
| `Mat4` | identity; column-array construction/round trip; translation, scale, and axis rotation; multiplication; transpose; checked inverse; point/vector transforms; checked projective point transform; final-column read/write; the three Narrow-B checked construction families |

`Vec2` and `Quat` receive no speculative arithmetic. Indexing, hashing,
ordering, serialization, POD/zero-copy, FFI layout, generic numeric traits,
swizzles, Euler policy, interpolation policy, and a general camera namespace
are not admitted.

## Values, Storage, And Operators

- Public components and arrays are semantic scalar values, not promises that
  the private provider has the same memory layout.
- Matrix arrays are column-major logical columns because current renderer and
  corpus crossings require that convention.
- Matrices multiply column vectors on the right. `A * B * p` applies `B` then
  `A` to `p`.
- `transform_point3` uses homogeneous `w = 1`; `transform_vector3` uses
  `w = 0`; projective division is a separate checked operation.
- Ordinary arithmetic follows Rust `f32`/IEEE behavior. Operations whose
  semantic result requires finiteness expose a checked result instead of
  treating NaN or infinity as success.
- No operator may allocate or perform a hidden provider conversion more than
  once per boundary crossing.

## Failure Semantics

The raw value containers may retain any `f32` bit pattern for observation and
transport. Semantic operations classify failures explicitly:

| Operation | Rejection |
| --- | --- |
| normalize | non-finite input or zero length |
| inverse | non-finite input, singular matrix, or non-finite result |
| project point | non-finite input/result or zero/non-finite homogeneous W |
| view/projection constructors | the Narrow-B error contract |

Unchecked compatibility methods, if retained for source-pressure measurement,
must be labeled as such and cannot be the only admitted boundary. Min/max
behavior with NaN operands is not stabilized by this slice; finite operands
are the owned contract.

## Independent Checks And Tolerances

Each operation requires at least one scalar/reference result independent of
the private provider, plus deterministic boundary and fixed-seed metamorphic
cases where meaningful. Common finite comparisons use
`abs_error <= 1e-5 + 1e-5 * abs(expected)`. Required exact checks include array
round trips, identity behavior, categorical rejection, and caller/source
identity outside ordinary math.

Provider differential tests are secondary evidence. Agreement between two
provider revisions does not by itself establish correctness.

## Explicit Non-Claims

Full B does not remove foreign implementation, supply-chain, audit, unsafe,
SIMD, portability, legal, update, or security work. Tokimu-owned names do not
make provider execution Tokimu-owned. The contract is intentionally smaller
than the current `glam` API and may lose operations rather than mirror upstream
breadth.
