# Doom Private Source-Occurrence Model Evidence

## Scope

This evidence closes Slice 1 of [Doom Ordered Source-Occurrence
Preparation](../Studies/Doom%20ordered%20source%20occurrence%20preparation.md).
It validates a Doom-campaign-private representation only. It does not admit a
Tokimu renderer, platform, core, or runtime contract.

## Retained Manifest

Command:

```powershell
cargo run -p hello-doom-visibility-conformance --bin source_occurrence_model_report
```

Observation:

```text
status=validated; source-contributions=1; partial-occurrences=2; distinct-source-identities=1; whole-retain-generated-geometry=false; unresolved-retains-original=true; shared-boundary-consumers=4; rejected-invalid-controls=7; fingerprint=bd6ae7533d20d59b22bed03d6d21e7e051ff3d0ac3422200dc4ff6cf68b038cf
```

The specimen retains one source contribution as two occurrences over disjoint
continuous source-relative intervals. Source, occurrence, prepared-view,
runtime-snapshot, shared-boundary, and eventual renderer-resource identities
are distinct types. Renderer-resource identity remains absent during source
preparation.

## Representation Findings

- Each partial occurrence owns one non-empty contiguous horizontal interval in
  normalized source space and one finite, non-reversed vertical domain.
- `0..N` is represented by a whole reject for zero survivors or one-or-more
  partial occurrences. An empty partial outcome is invalid, preventing an
  ambiguous empty list from silently deleting work.
- Whole retain preserves the original source contribution and generates no
  replacement geometry.
- Unresolved preparation explicitly retains the original contribution with a
  bounded reason.
- Positive reject authority is distinct from unresolved fail-open behavior.
- Geometry endpoints, normal, wall role, UV domain, material identity, and
  diagnostic label remain attached to source provenance.
- One prepared boundary is referenced by wall, floor, ceiling, and sky
  consumers. Consumers do not reconstruct independent causal values.
- Diagnostic pixel columns do not appear in the representation.

## Invalid Controls

The retained report counts seven rejected controls covering:

1. empty horizontal interval;
2. reversed horizontal interval;
3. non-finite horizontal endpoint;
4. out-of-range horizontal endpoint;
5. overlapping occurrences for one source contribution;
6. source-identity mismatch; and
7. an empty partial outcome.

Focused tests additionally reject empty, reversed, and non-finite vertical
domains.

## Validation

The focused model tests pass:

```text
cargo test -p hello-doom-visibility-conformance source_occurrence::tests --lib
6 passed; 0 failed
```

Strict campaign Clippy passes:

```text
cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
passed
```

`cargo fmt --all` was applied.

The current branch still has an independent pre-existing campaign-test failure:

```text
tests::two_sided_aperture_retains_independent_upper_lower_opening_and_plane_intervals
expected a retained Floor plane key
```

The new occurrence module does not participate in that legacy aperture path.
Slice 1 therefore does not modify unrelated visibility semantics merely to
obtain a green aggregate gate. The failure remains an explicit prerequisite to
a later crate-wide closeout claim.

## Disposition

The private representation satisfies Slice 1. It is sufficient to begin the
headless partial-survival reconstruction in Slice 2. It does not yet prove that
the retained baseline and nearer falsifiers can be reconstructed without
screen-column authority; that is the next experiment.
