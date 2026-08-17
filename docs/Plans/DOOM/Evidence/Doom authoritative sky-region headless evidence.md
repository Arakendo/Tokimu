# Doom Authoritative Sky-Region Headless Evidence

| Field | Value |
| --- | --- |
| Study | [Doom Authoritative Sky-Coverage Delta Realization](../Studies/Doom%20authoritative%20sky%20coverage%20delta%20realization.md) |
| Slice | 1 — headless authoritative sky-region model |
| Scope | Doom-private synthetic evidence; no stable renderer vocabulary |
| Command | `cargo run -p hello-doom-visibility-conformance --bin authoritative_sky_region_report` |
| Unit gate | `cargo test -p hello-doom-visibility-conformance authoritative_sky --no-fail-fast` |
| Result | 7 passed; 0 failed |

## Retained result

The corrected ordered ledger's terminal-sky positive retained 66 sky-plane
intervals and 2,046 cells. The Doom-private model conserved all of them in two
bounded normalized regions:

```text
input intervals     66
input cells       2,046

modeled regions      2
modeled intervals   66
modeled cells     2,046

omitted intervals    0
removed non-sky      0
fail open        false

structural fingerprint
946bc73732ee4ecbe25d28ddc7237a31e148cd413b3be4ccf875690fb5c01ef7
```

Each region retains the source plane instance, source sector/SEG authority,
source order, prepared-view identity and runtime-snapshot identity. Thirty-three
diagnostic column intervals per region collapse to one normalized horizontal
extent and two knots for each continuous upper/lower boundary. The `320 x 200`
oracle therefore does not become geometry or declaration identity.

## Negative authority controls

| Case | Retained sky intervals | Modeled regions | Relevant result |
| --- | ---: | ---: | --- |
| Paired boundary without retained sky plane | 0 | 0 | 161 paired-boundary columns observed; 0 claimed |
| One-sky negative | 0 | 0 | no paired or plane authority acquired |
| Nearby ordinary aperture | 0 | 0 | no ordinary contribution removed |

This corrects an initially tempting interpretation: a paired-sky boundary is
ordered protocol evidence, but it does not itself authorize a coverage region.
Only a retained `F_SKY1` ceiling interval supplies the tested authority.

## Failure controls

The unit gate constructs three deliberately invalid ledger inputs and proves
that each fails open with one retained outcome:

- missing source authority;
- source SEG absent from admitted order;
- invalid vertical interval.

No case silently hides ordinary geometry. Input interval and cell totals equal
modeled plus omitted totals.

## Claims and limits

Demonstrated:

- a bounded Doom-private semantic region can be derived from a healthy ordered
  ledger without admitting Doom vocabulary to `tokimu-render`;
- the diagnostic raster can remain an oracle while continuous normalized
  boundaries carry the modeled meaning;
- positive, negative and ambiguous authority have distinct outcomes;
- non-sky removal is structurally absent at this seam.

Not demonstrated:

- native or browser presentation of the region;
- a correct depth-bearing realization;
- E1M1 sky-leak repair;
- free-look or jitter behavior;
- any provider-neutral rendering primitive.

Those remain Slice 2 and later work. Candidate 2 remains parked.
