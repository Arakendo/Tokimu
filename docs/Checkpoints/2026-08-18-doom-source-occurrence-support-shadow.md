# Doom Source-Occurrence Support Shadow Checkpoint

Date: 2026-08-18

## Outcome

The diagnostic source-cell-to-reconstructed-geometry shadow is implemented
and conserved, but presentation installation is paused. Five new walkabout
captures pass their exact-cell expectations and all four plane captures agree
with the finite geometry shadow. A retained historical control exposed an
architectural evidence error: `ceiling-104-reached` is a positive whole-object
occurrence control, not a positive exact-ray presentation control.

No renderer submission or stable contract changed.

## Implementation

The Doom corpus now has `--source-occurrence-support-report`. It:

- freezes the five walkabout rays and complete-shell identities;
- maps each ray into the provider's fixed `320x200` source projection;
- reports final plane-key/cell and wall-tier/cell support;
- compares the source-covered candidate and current ordered preparation;
- clips reconstructed plane triangles by exact plane key, source sector and
  finite source cell in a shadow-only result; and
- verifies conservation, duplicates, degenerates, fragment amplification and
  deterministic fingerprints.

The renderer receives none of this vocabulary.

## Five-Capture Result

| Capture | Exact cell | Geometry shadow | Finding |
| --- | --- | --- | --- |
| ceiling 55 / sector 24 | unsupported | no hit | false retention confirmed |
| ceiling 54 / sector 24 | unsupported | no hit | false retention confirmed |
| ceiling 117 / sector 41 | unsupported | no hit | cascading false retention confirmed |
| wall 241 / middle tier | unsupported | not needed | one matching interval exists, sampled row is not retained; ordered declarations `0` |
| floor 130 / sector 5 | supported | hit at `132.480` | source-covered false omission repaired in shadow |

The capture gate is `5/5`; the plane geometry gate is `4/4`.

For the narrow exact-key-plus-source-sector shadow, representative output was:

```text
ceiling 55 pose  2 draws / 1,233 triangles / 1.271x amplification
ceiling 54 pose  2 draws / 1,290 triangles / 1.330x amplification
ceiling 117 pose 49 draws / 3,966 triangles / 4.089x amplification
floor 130 pose   41 draws / 3,325 triangles / 3.428x amplification
```

All observations had zero duplicate fragments, zero unresolved surfaces and
balanced conservation. Degenerate omissions were bounded at `0..3`.

## Decisive Control Correction

The prior `ceiling-104-reached` ray is:

```text
origin     [1477.330444336, -3594.213134766, 8.994521141]
direction  [-0.792175531, -0.565008104, 0.230702817]
global hit flat:40:CEIL3_5 at 273.102
```

The ordered replay still reaches subsector 104 and creates a partial plane
destination. That remains valid object-occurrence evidence. The exact
presentation evidence is different:

```text
source sample                 column 160, row 62
matching plane-key instances 1
retained intervals at column  none
ordered prepared declarations 2
ordered exact target ray      no hit
new plane-cell geometry ray   no hit
```

The earlier two continuous horizontal occurrence ranges also exclude the
center ray. Therefore the historical label "reached" was correct at object
granularity, while treating its BVH point as a known-good exact presentation
point was not.

The retained exact-cell matrix consequently reports `6/7`, intentionally
failing at this one expectation. It does not weaken the gate to manufacture a
pass.

## Falsified Refinement

Classic plane keys may merge compatible sectors, so a broader shadow using
plane key without source-sector correlation was tested. It did not restore
the ceiling-104 exact ray. It also increased representative output:

```text
ceiling 117 pose 3,966 → 5,497 triangles
floor 130 pose   3,325 → 5,528 triangles
```

The broader association was removed. Correlation alone did not transfer
spatial support authority across sectors.

## Determinism And Validation

- Result fingerprint repeated identically: `026b013b660870ac`.
- `cargo fmt --all` completed.
- `cargo test -p hello-doom-e1m1 --bin static_scene`: `91` passed.
- `cargo test -p doom-geometry-provider`: `39` passed.
- The report deliberately returns a failing hypothesis result:
  `capture-cells=5/5`, `plane-geometry=4/4`,
  `retained-exact-cells=6/7`.
- The existing six-ray ordered occurrence report still passes its original
  object-level semantics and balanced conservation.

## Architectural Finding

The fixed Classic source cells can explain the new false positives, the local
floor false negative, wall 241 and wall 135 without renderer-owned Doom
semantics. They have not yet proved a complete actual-camera realization
policy because the prior positive plane control was asserted at the wrong
granularity.

AR-0030 must decide the next plane acceptance oracle before this shadow can
become presentation-affecting. In particular, arbitrary-pitch realization
cannot infer exact source presence merely from whole-object occurrence.

## Remaining Work

- establish an independently justified exact positive plane control matrix;
- decide the role of fixed Classic rows under arbitrary camera pitch;
- add the remaining scan/return-pose corpus only after that authority is clear;
- measure preparation time after correctness gates are repaired; and
- return to AR-0030 before any live strategy is installed.

One ordinary implementation failure was found during validation: the capture
inventory unit test still expected wall 241 to be observation-only after its
final unsupported disposition became an asserted capture result. The stale
expectation was updated; no production policy changed.
