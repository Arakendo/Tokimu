# Doom Source-Topology Synthetic Admission Evidence

Date: 2026-08-16

This evidence supports Slice 2 of [Doom Source-Topology Admission Over Complete
Geometry](../Studies/Doom%20source-topology%20admission%20over%20complete%20geometry.md).
It records a Doom-study-local classifier over immutable source-labelled
contributions. It does not admit a Tokimu renderer, platform, or kernel API.

## Command

```powershell
cargo run -p hello-doom-visibility-conformance --bin topology_admission_report --quiet
```

Focused validation:

```powershell
cargo test -p hello-doom-visibility-conformance --lib topology_admission --quiet
cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
```

The focused suite passed 12/12 tests and strict Clippy passed. Inventory
observation is also checked not to mutate the fixture's structural manifest.

## Results

| Fixture | Admitted | Rejected | Unresolved fail-open | Fingerprint |
| --- | ---: | ---: | ---: | --- |
| open aperture | 6 | 0 | 0 | `97fbae9a74c7ab2c0a7bb55fa1b66e82576d03a617a0dea767c49c7761992747` |
| terminal solid | 3 | 3 | 0 | `5157305853296eba056e5a2461b67fe32c5bc057f98264c87c1d44de1196097f` |
| paired sky | 6 | 0 | 0 | `2f915787530a71d0f8e82229fd6c9f5321aee250d1c24339fd92704abaa3e994` |
| one-sky identity | 6 | 0 | 0 | `345a7f5d15153036f7b04121ff3832f64c3daa3f97e5ef9283dfe03227ad9254` |
| vertical aperture | 6 | 0 | 1 | `8baaa834846a66d3a7fda9438b1c49cd004b1f8e8dafa34f499d218651dc8182` |
| masked middle | 3 | 0 | 0 | `89831de07d41ccfc39116139c51f7a6fc82d6148526fc782e0505d32f7eb016f` |
| ambiguous near plane | 2 | 0 | 1 | `88c0af8d7713ef047c718c662383476b5c41eecbf24bf77b1fb0012aeaf1a86a` |
| door closed | 1 | 0 | 0 | `32129e8beb6c9ef047e976d70b10da9ed54f20cd3e934cd9cc6790535e21e9d5` |
| door open | 0 | 1 | 0 | `3657e988c31e908ddd3c605dfea0ebc0d5435defe9c632bb5e5f13580506d723` |
| platform low | 1 | 0 | 0 | `bdae32ab67df8da700ed9ca43f04e43345f142f99645ea8e3620d6db4a166d9a` |
| platform raised | 1 | 0 | 0 | `31642cc4def4dcc488e6b19fafa828b3eefd3186aecb611207ac8d97aeb2a895` |

## Findings

- A source-terminal solid range supplies positive rejection authority. The
  otherwise-identical open aperture retains the far source occurrence.
- Paired-sky and one-sky identity alter plane meaning without independently
  granting or removing source reachability.
- Upper, lower, middle/cutout, floor, ceiling, and sky-plane families retain
  separate contribution identities.
- A masked middle remains admitted but never becomes terminal merely because
  its quad spans an opening.
- Door and platform evidence consumes explicit current-height snapshots only.
  The fixture owns no activation, ticking, waiting, or reversal policy.
- Near-plane and unsupported projection ambiguity is explicitly fail-open.
  Absence from a trace is not rejection authority.
- The classifier preserves immutable source identities and fixture structural
  manifests; it neither reconstructs nor mutates geometry.

## Baseline limitation retained separately

The full crate test run currently has one inherited failure in
`two_sided_aperture_retains_independent_upper_lower_opening_and_plane_intervals`.
The provider retains the source floor mark but no projected floor span survives
at that pose, while the older assertion requires one. The admission matrix does
not use that missing projected span as rejection authority, and the focused
suite plus strict Clippy remain green. This limitation must be resolved or
reclassified separately rather than being charged as a Slice 2 admission
failure.

## Slice 3 whole-contribution falsifier

Command:

```powershell
cargo run -p hello-doom-visibility-conformance --bin whole_contribution_falsifier_report --quiet
```

The same immutable partial-paired-sky source geometry was observed at baseline,
bounded horizontal jitter, and a nearer viewer pose. Alternative A is the
explicit whole-contribution control. Alternative B applies the study-local
topology classifier and is forbidden from constructing fragments.

| Pose | B far result | Rejected unrelated contributions | Invalid overlap columns if retained whole | Required survivor columns | Ordinary depth authority in overlap |
| --- | --- | --- | ---: | ---: | --- |
| baseline `[0,-96]` | admitted: ordered source SEG | none | 81 | 15 | no |
| jitter `[2,-96]` | admitted: ordered source SEG | none | 81 | 15 | no |
| near `[0,-80]` | admitted: ordered source SEG | none | 97 | 9 | no |

The empty unrelated-rejection set is deliberate: this fixture contains no
positive terminal-solid rejection provenance. Slice 2's terminal-solid fixture
separately proves that the classifier can reject when that authority exists.
Absence or paired-sky identity is not promoted into rejection merely to improve
the result.

The far contribution must survive in two side intervals but must not appear in
the central overlap. Retaining it whole exposes source-invalid coverage that
ordinary depth cannot remove; rejecting it whole loses required coverage. The
result persists across both movement controls while the source geometry remains
unchanged.

Result: **falsified**. Whole-contribution Boolean admission is insufficient and
Alternatives B/C must not advance to E1M1 under this plan. View-local partial
presentation has earned investigation in AR-0030.

The result is stronger than a failed optimization filter. It establishes that
the required Doom-owned preparation may map one source contribution to zero,
one, or multiple view-local occurrences. Calling that operation a prefilter
would conceal the demonstrated fragmentation/synthesis responsibility.

The measured columns are a falsification instrument rather than a proposed
contract. Follow-up work must not canonize the `320 x 200` diagnostic grid as
Tokimu or Doom semantic vocabulary. The next representation study should keep
source-relative correspondence continuous where possible and compare a
Doom-local view-occurrence representation realized first as ordinary
view-local triangles. A bounded screen-local primitive is reserved for a later
test only if ordinary triangles cannot preserve the required coverage.

Falsifier fingerprint:
`e79bb365ef3c1d8bb77dcce721cef1d5a08c1394a1370ffe4a6d35aef8ba94db`.
