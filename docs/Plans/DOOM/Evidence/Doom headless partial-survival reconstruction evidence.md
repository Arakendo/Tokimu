# Doom Headless Partial-Survival Reconstruction Evidence

## Scope

This record closes Slice 2 of the
[Doom Ordered Source-Occurrence Preparation](../Studies/Doom%20ordered%20source%20occurrence%20preparation.md)
study. It tests one retained partial-survival source SEG without a renderer.
It does not admit a renderer fragment API, Doom screen columns, or a generic
visibility contract.

## Invocation

```powershell
cargo run -p hello-doom-visibility-conformance --bin partial_survival_reconstruction_report
```

Focused validation:

```powershell
cargo test -p hello-doom-visibility-conformance source_occurrence::tests --lib
cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
```

## Result

One source identity, `seg:1/linedef:1/sidedef:2`, was replayed under three
prepared views. The two occurrence identities remained `17` and `18` while
their continuous source-relative intervals changed with the view.

| Pose | Retained source intervals | Excluded source interval | Survivor columns | Forbidden columns |
| --- | --- | --- | ---: | ---: |
| Baseline | `[0, 1/12]`, `[11/12, 1]` | `[1/12, 11/12]` | `15 / 15` | 81 |
| Jitter X +2 | `[0, 5/72]`, `[65/72, 1]` | `[5/72, 65/72]` | `15 / 15` | 81 |
| Nearer Y +16 | `[0, 1/20]`, `[19/20, 1]` | `[1/20, 19/20]` | `9 / 9` | 97 |

For every pose:

- the forbidden middle source interval was omitted;
- every required survivor column was represented;
- 18 reconstructed endpoint observations lay on the original source wall;
- fragment UV width matched source-interval width;
- fragments touching a source endpoint retained the corresponding authored UV
  endpoint.

The exact report fingerprint was:

```text
7c270600723120c12ddbf495fd0524690e060cb45142485cf42c07c2706d11a9
```

## Construction Boundary

The reconstruction path is:

```text
viewer + original source SEG
        ↓
continuous source-ray intersections
        ↓
source-relative survivor intervals
        ↓
validated private occurrences
        ↓
source endpoint and UV interpolation
        ↓
diagnostic-column comparison
```

The diagnostic `320 x 200` column domain is consulted only after continuous
source geometry exists. No screen column is inverse-projected into a world or
source endpoint.

## Conservation And Controls

Seven contribution evaluations balance:

```text
fragmented pose replays       3
thin whole-retain             1
positive whole reject         1
unresolved fail-open          2
                              -
evaluated total               7
```

The three pose evaluations replay one distinct source identity; they are not
three unrelated source contributions.

Negative controls establish:

- near-plane ambiguity fails open and retains the original contribution;
- an unsupported source role fails open and retains the original contribution;
- an empty survivor set rejects only with named positive source authority;
- a thin but non-empty continuous interval remains valid rather than being
  discarded because it falls between diagnostic columns.

## Validation

- 10 focused `source_occurrence` tests passed.
- Strict campaign Clippy passed for all targets.
- The report completed with `status=validated`.
- Windows emitted its existing incremental-cache hard-link fallback warnings;
  they did not change test or Clippy outcomes.

## Finding

The retained partial-survival contradiction can be represented headlessly as
bounded, continuous, source-relative occurrences. The evidence supports
continuing to shared wall/plane boundary preparation in Slice 3. It does not
yet prove ordinary Tokimu presentation lowering, runtime snapshot replacement,
or E1M1 completeness.
