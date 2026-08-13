# Option B Provider-Update Shock And Maintenance Economics

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Update replay | exact local `glam` 0.29.3 `d36e7eef` to 0.33.3 `99287290` |
| Production | retained A at 0.29.3; unchanged |
| A study effort control | approximately 124 active minutes plus separately retained automation/tooling time |

## Camera/Projection API Shock

The frozen Option A update found 86 direct uses of the three deprecated
`Mat4` camera/projection constructors. Its authorized prototype migrated 28
representative sites and deliberately left 58. That is observed A work, not a
hypothetical estimate.

Replaying the same provider pair through the already-present B candidates gives
this accounting:

| Candidate | Public caller edits caused by update | Private B edits | Contract-test edits | Result |
| --- | ---: | ---: | ---: | --- |
| A, direct provider vocabulary | 86 potential strict-warning sites; 28 actually migrated in prototype | N/A | new update regressions | provider organization reaches callers |
| Narrow B, seam already adopted | 0 | three provider constructor adapter bodies plus pin selection | 0 | unchanged public callers/tests pass both pins |
| Full B, vocabulary already adopted | 0 | the same three provider constructor adapter bodies plus pin selection | 0 | unchanged public callers/tests pass both pins |

This proves conditional shock absorption: callers already using either seam do
not change when provider camera organization changes. It does not erase the
one-time adoption cost. Migrating current A to Narrow B still requires changing
the relevant construction sites; the frozen A prototype completed 28/86.
Migrating to Full B additionally changes the five-type public vocabulary and
the representative study already found accessor, setter, crossing, compile,
artifact, and performance costs.

Full B provides no additional insulation over Narrow B for the update that
actually occurred. Both funnel the same three semantic constructors through
the same three private provider adapters.

## Non-Camera Semantic Shock

The 0.29.3-to-0.33.3 audit records a scalar/SIMD consistency change for vector
`min`/`max` with NaN operands. The Full-B observer executes the *same wrapper
source* under both exact providers and obtains different bit results:

```text
0.29.3
left_min_right=[40800000, 3f800000, c0400000]
right_min_left=[40800000, 3f800000, c0400000]

0.33.3
left_min_right=[40800000, 7fc00042, c0400000]
right_min_left=[7fc00042, 3f800000, c0400000]
```

A exposes this provider behavior directly. Narrow B deliberately retains the
same foreign value types, so it does not attempt to contain the change. Full B
owns the type name but its `min`/`max` methods delegate directly, so it also
does not contain the semantic change. Full B could stabilize a chosen policy
only by explicitly admitting, implementing, testing, and maintaining that
policy. Wrapper presence is therefore not evidence of semantic insulation.

Option C owns its mechanics and would receive a local contract/implementation
decision instead of automatic provider drift. That independence is also its
maintenance bill.

## Ordinary New-Operation Simulation

The already-pressured `Vec3::dot` operation provides a bounded replay rather
than an invented API:

| Candidate | Work when caller first requires `dot` |
| --- | --- |
| A | no Tokimu API work when the selected provider already supplies it; caller uses provider method |
| Narrow B | same as A; the semantic-construction seam is intentionally unrelated |
| Full B | add one delegated wrapper method, public documentation/contract judgment, and tests |
| C | add owned arithmetic, documentation/contract judgment, and tests |

Full B and C both pay per-operation public-surface costs. Full B additionally
tracks provider behavior and can still drift unless its contract is stronger
than delegation. Narrow B avoids that shadow-API burden because it owns only
the independently demonstrated camera/projection meaning.

## Work That Does Not Disappear

Under both B candidates, every provider update still requires the Option A
work for:

- immutable source identity, submodule pin, provenance, and source-tree diff;
- complete normal/build/proc-macro closure and selected-feature comparison;
- generated code, unsafe, SIMD/intrinsics, targets, and portability review;
- advisories, licenses, notices, attribution, and redistribution obligations;
- numerical differential, failure, performance, compile, artifact, and target
  gates appropriate to the selected provider behavior;
- rollback/replay, offline local-source enforcement, and maintainer admission;
- actual-browser or unavailable-target evidence where applicable.

B can reduce public source churn. It cannot reduce the foreign implementation
audit merely by moving that implementation behind Tokimu names.

## Recurring Versus One-Time Economics

| Work | A | Narrow B | Full B | C |
| --- | --- | --- | --- | --- |
| adopt candidate from current production | none | one-time constructor-site migration | one-time five-type and crossing migration | one-time implementation/type migration |
| camera provider reorganization after adoption | caller churn or deprecation debt | three private adapters | same three private adapters | none |
| provider source/security audit | recurring | recurring, unchanged | recurring, unchanged | none for `glam`; owned code review remains |
| ordinary provider-supplied operation | immediately available | immediately available | wrapper/contract/test growth | implementation/contract/test growth |
| provider semantic drift | directly exposed | exposed outside narrow seam | exposed unless each wrapper method owns policy | local policy is owned |
| shadow API/documentation burden | provider-owned | three semantic functions | broad and recurring | bounded owned surface |

## Disposition

The update shock strongly supports Narrow B's proportionality: it contains the
observed camera vocabulary change with unchanged callers and only three private
adapter bodies. Full B demonstrates update indirection but no proportional
benefit for this shock, and the NaN control proves that private delegation does
not automatically stabilize broader value semantics.

This does not select Narrow B for production. It establishes the update-
economics evidence needed by the later decision gate while leaving production
A, AR-0029 status, and all ADR-0010 obligations unchanged.
