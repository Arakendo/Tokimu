# Option B Full Provider-Backed Hardening

| Field | Value |
| --- | --- |
| Status | Slice 4 complete; isolated candidate only |
| Candidate | Full B |
| Providers | exact local `glam` 0.29.3 and isolated 0.33.3 candidate |
| Host / target | Windows x86-64 native, Rust 2024 toolchain in repository evidence |
| Public values | `Vec2`, `Vec3`, `Vec4`, `Quat`, `Mat4` wrappers |
| Production change | none |

## Result

One shared Full-B implementation and one unchanged external contract harness
now pass against either provider pin. The provider selection and the three
camera/projection API adaptations are private to the isolated crate. No
provider type, trait, module, feature, or error appears in the candidate's
public signatures.

This proves private provider switching for the bounded surface. It does not
prove provider replacement is cheap at production scale, remove ADR-0010
audits, or make the executing implementation Tokimu-owned.

## Public Contract And Failure Evidence

Full B retains the five advertised names, but only `Vec3`, `Vec4`, and `Mat4`
have substantial caller-earned mechanics. `Vec2` and `Quat` remain minimal
construction/observation compatibility probes.

Checked semantic operations now retain Tokimu-owned operation and failure
identity:

- normalization rejects non-finite and zero-length input;
- inverse rejects non-finite and singular input and non-finite results;
- projective point transformation rejects non-finite input/result and zero
  homogeneous `w`; and
- right-handed view plus GL-depth perspective/orthographic construction use
  the same bounded validation categories as Narrow B.

Unchecked spellings remain only as explicit compatibility-pressure controls.
They are documented as unchecked and are not the admitted failure contract.
No provider panic or provider error is exposed as Full-B meaning.

## Test Matrix

| Provider | Shared unit tests | External Full-B contract tests | Result |
| --- | ---: | ---: | --- |
| 0.29.3 | 10 | 5 | pass |
| 0.33.3 | 10 | 5 | pass |

The external tests cover exact scalar landmarks, fixed-seed normalization and
inverse properties, independent homogeneous projection arithmetic, view and
orthographic landmarks, degenerate/non-finite/singular rejection, operator
behavior, constants, field accessors, arrays, rotations, scale/translation,
transpose, final-column mutation, and finite checks. The shared parent study
also remains green at 63 unit and six integration tests.

Strict focused Clippy passes without source-warning suppression against
0.33.3. The 0.29.3 run passes with the already-retained generated-source
warning flood suppressed for readable test output; an unsuppressed run still
reproduces that provider-owned warning evidence. Filesystem hard-link cache
warnings are host/tooling observations, not Rust lint failures.

## Provider Reference Inventory

The shared wrapper contains 39 literal provider-namespace references:

| Classification | Count | Purpose |
| --- | ---: | --- |
| private storage and provider construction mechanics | 23 | five private fields, constants, and pressured constructors |
| semantic-constructor adapter calls | 3 | view, perspective, orthographic |
| explicit private conversions | 5 | bounded wrapper/provider crossings |
| provider/layout control tests | 8 | isolation and non-guaranteed representation observations |

There are also 44 `.inner` uses that delegate component access and pressured
ordinary mechanics to the private provider. Validation itself is Tokimu-owned
scalar logic; the only provider numerical predicate used by a checked path is
the private matrix determinant needed to classify singular inversion.

No reference was classified as unnecessary duplication. Some methods mirror
provider spellings because named current callers require them, but broader
swizzles, indexing, serialization, POD/zero-copy, generic numeric traits,
hashing, ordering, and quaternion arithmetic remain deliberately absent.

## Switching Cost Observed

The public wrapper source and external tests are identical for both pins. The
private switching seam consists of:

1. selecting one exact optional provider dependency; and
2. three `cfg`-selected adapter functions for the camera/projection API move.

`cargo tree` shows one selected provider per build. The lockfile records both
because the isolated comparison crate supports both feature selections; a
production selection would still have one exact dependency closure.

## Ergonomics And Claim Limits

- Provider-backed private fields require accessors and explicit final-column
  mutation rather than public field mutation.
- There is no indexing, generic numeric interoperability, serialization,
  reflection, POD/FFI, or guaranteed layout contract.
- The candidate currently observes provider-equal size/alignment only as a
  non-binding measurement. No unsafe layout conversion relies on it.
- Provider conversions are crate-private. Renderer/resource crossings still
  need measurement in Slice 5.
- Full B owns names and bounded failure semantics, not implementation,
  provenance, SIMD, security, legal, or supply-chain independence.

The key unresolved comparison is therefore economic rather than feasibility:
does this larger wrapper buy enough caller insulation beyond Narrow B to pay
for accessor friction, private crossings, and an indefinitely maintained
compatibility surface?
