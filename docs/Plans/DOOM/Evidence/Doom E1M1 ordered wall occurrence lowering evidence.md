# Doom E1M1 Ordered Wall Occurrence Lowering Evidence

## Purpose

This record retains the first real-map correlation between E1M1's continuous
source occurrences and ordinary UV-bearing wall meshes. It is a Slice 6 gate
for the
[Doom Ordered Source Occurrence Preparation](../Studies/Doom%20ordered%20source%20occurrence%20preparation.md)
campaign.

The gate is observation-only. It does not replace the global E1M1 wall
declarations yet. Its purpose is to prove that retained Doom source intervals
have an ordinary mesh destination while preserving the already-demonstrated
opaque versus masked-middle classification.

## Command

```text
cargo build -p hello-doom-e1m1 --bin static_scene
.\target\debug\static_scene.exe corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-strategy=ordered-occurrence-prepared-full --topology-inventory-report
```

Building the executable explicitly matters: `cargo test --bin static_scene`
rebuilds the test harness, not necessarily the directly invoked development
binary.

## Retained Source Observation

```text
source SEG records=732
source SEGs visited=732
whole retained=16
partial retained=16
whole rejected=563
near-plane fail-open=137
retained occurrences=171
occurrence fingerprint=69cbc0a1e53db469
```

## Wall Lowering Observation

```text
occurrences=171
occurrences with wall geometry=135
occurrences without wall geometry=36
unresolved fail-open=0

matched source triangles=303
  opaque=291
  cutout=12

material-resolved source triangles=303
  opaque=291
  cutout=12

clipped source triangles=331
  opaque=319
  cutout=12

lowered wall meshes=321
  opaque=309
  cutout=12

degenerate omissions=10
structural fingerprint=0bcca0e595848b93
occurrence conservation=balanced
category conservation=balanced
material conservation=balanced
```

The 36 occurrences without wall geometry are source traversal occurrences for
which the existing wall provider emits no visible wall tier. They are retained
as an explicit counted outcome rather than treated as an implicit failed mesh.
The 10 degenerate fragments follow the existing narrow degenerate-triangle
omission policy. No unrelated lowering error was skipped.

## Classification Boundary

The cutout classification comes from Doom's authored two-sided middle-texture
observation and exact source identity:

```text
role=Middle
+ linedef identity
+ sidedef identity
+ owning side
+ texture identity
```

It is not inferred from texture alpha bytes. The clipped fragments inherit the
source classification, positions, and UV stream before ordinary Tokimu mesh
lowering. This preserves the AR-0023 ownership result while exercising the new
view-local occurrence path.

The material gate is category-specific. Opaque occurrences resolve their
texture names through the existing opaque wall-material inventory, while
source-authored masked middles resolve through the existing cutout upload
inventory. All 303 matched source triangles resolve through the expected
namespace. Missing material identity remains an explicit fail-open outcome; it
is not repaired by choosing a material from the other category.

## Interpretation

- One retained occurrence may intersect multiple wall tiers and may produce
  more than one clipped triangle.
- Clipping can triangulate a surviving polygon into additional triangles, so
  `331` clipped triangles from `303` matched source triangles is expected and
  is not duplication by itself.
- Ten clipped triangles are geometrically degenerate and therefore do not
  become meshes; all other clipped triangles have an ordinary mesh
  destination.
- Material identity is conserved for all 303 matched source triangles before
  clipping. Replacing a global wall declaration therefore need not invent a
  material or infer its presentation policy.
- The original 1,922 E1M1 contributions remain unchanged. This result is a
  conservation gate, not the Slice 6 prepared-full visual candidate.

## Validation

```text
cargo fmt --all
cargo test -p hello-doom-e1m1 --bin static_scene ordered_occurrence
```

Six focused tests pass. The canonical report exits successfully. The emitted
incremental-cache hard-link warnings are the known Windows filesystem fallback
and do not describe a campaign failure.
