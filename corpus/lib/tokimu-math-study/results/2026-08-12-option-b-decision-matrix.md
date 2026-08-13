# Option B Slice 12: Comparative Decision Matrix

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Recommendation prepared; explicit maintainer acceptance required |
| Production state | Alternative A on audited `glam` 0.29.3 remains unchanged |
| Recommendation | Continue Narrow B incubation; park Full B; retain A in production and C0/C1 as executable evidence |
| Non-claim | No stable API, provider update, migration, ADR, or production placement is authorized |

## Candidate Matrix

| Dimension | A: provider types + direct constructors | Narrow B: provider types + Tokimu constructors | Full B: five Tokimu wrappers | C0/C1: owned subset |
| --- | --- | --- | --- | --- |
| semantic ownership | foreign constructor vocabulary despite Tokimu camera meaning | Tokimu owns exactly three checked camera/projection families | Tokimu names broad ordinary math, but delegated methods can still inherit provider semantics | Tokimu owns admitted numerical behavior |
| public coupling | five provider types and constructor organization are public | five provider value types remain public; camera constructor organization is insulated | provider types private, but wrapper surface becomes a large public commitment | no foreign public or executing math closure |
| implementation trust | audited foreign Ring 0 implementation | same audited foreign Ring 0 implementation | same audited foreign Ring 0 implementation | Tokimu implementation and maintenance responsibility |
| observed 0.29.3 -> 0.33.3 shock | 86 dated caller sites; 28 migrated in prototype | after adoption: zero caller changes, three private adapter changes | same zero/three result; no extra benefit for this shock | no provider update, but local compiler/numerical work remains |
| adoption migration | none while A retained; update prototype already measured | one-time migration of direct constructors; value callers unchanged | broad type, field, trait, storage, and renderer crossing migration | broad type/mechanics migration |
| conversions and crossings | none | zero value conversions | nine explicit renderer crossings in bounded migration; scalar crossings allocation-free | Tokimu values directly, subject to full migration |
| performance | mature production control | checked bundle adds about 38-39 ns in stress control; real frame impact unproven | mixed: four caller controls competitive; GLB +7.8%, inverse +18.5% | C0 general inverse materially slower; C1 repairs demonstrated affine GLB pressure; non-affine work open |
| targets | production native/WASM history plus update candidate gates | native both pins; Node-WASM default/SIMD both pins; ARM64 compile; actual browser unavailable | same bounded Node-WASM/ARM compile; actual browser unavailable | native and DOM/WASM chart evidence; broader production migration absent |
| checked failure behavior | direct provider behavior | bounded operation/category errors; invalid native/WASM calls contained | bounded checked methods; unchecked compatibility paths remain outside claim | explicit owned policy and test burden |
| API/documentation size | provider API is already broad and externally documented | 16 experimental missing-doc items concentrated in three functions/errors | eight top-level declarations, 60 inherent members, 10 trait impls, and substantially larger documentation bill | bounded owned manifest, but every admitted operation needs policy/docs/tests |
| caller ergonomics | familiar provider API; upstream vocabulary leaks | clear RH/GL intent at pressured sites; otherwise familiar values | familiar finite math, but real callers still need missing `Sum`, field/column mutation, and other provider ergonomics | coherent owned vocabulary with compatibility/migration cost |
| semantic drift | provider behavior visible | provider ordinary-value behavior visible; constructor contract checked | wrapper `min/max` NaN result changed across pins despite unchanged wrapper source | Tokimu controls drift but owns every edge case |
| recurring ADR-0010 burden | full provider update audit | identical full provider update audit | identical full provider update audit | no foreign math provider audit; local implementation review remains |
| spatial/chart pressure | chart works; raw types do not own meaning | chart remains above math; no seam growth | same `2520c9de` trace without absorbing chart meaning | same trace; zero operation growth |
| current readiness | retained production choice | credible bounded candidate with missing actual-browser/runtime and stable-admission gates | bounded experiment, but benefits do not justify measured breadth/cost | credible future replacement candidate, not production-ready |

## Evidence-Bearing Recommendation

### Continue Narrow B incubation

Narrow B is the only B alternative whose benefit is proportional to the
observed problem. It absorbs the actual `glam::camera` update shock, makes
Tokimu's already-owned right-handed/GL-depth intent explicit, preserves existing
value ergonomics, and does not move renderer, chart, source, or input ownership
into math.

It is not recommended for admission yet because:

1. the study obtained Node-WASM and compile evidence but no actual-browser B
   execution;
2. AR-0029 still lacks the retained actual GLB runtime/browser observations it
   names as a gate;
3. its approximately 38-39 ns checked-constructor stress delta is bounded but
   has not been judged against a real mass-camera/frame budget;
4. stable documentation, final placement, semver, rollout, and rollback have
   intentionally not been performed before maintainer selection; and
5. production A is healthy enough that no emergency migration pressure exists.

### Park Full B

Full B proves that provider-backed Tokimu names are feasible, but it does not
stabilize delegated semantics automatically, does not absorb the observed
camera shock better than Narrow B, loses two performance gates, expands public
documentation and migration cost substantially, and still lacks real caller
ergonomics. Reopen it only if a non-camera provider API shock or multiple
independent public foreign-type boundaries demonstrate value Narrow B cannot
provide.

### Retain A and C evidence

A remains the single production vocabulary while the recommendation is under
review. The update study showed its maintenance path was Routine rather than
pathological, so retaining it is safe and evidence-based. C0/C1 remain the
executable ownership alternative and should receive future CAD/AR-0026 pressure;
this B study did not invalidate the earlier decision to avoid migration.

## AR-0029 Disposition Recommendation

Keep AR-0029 **Under Review as review guidance**, not an ADR yet. The narrow
semantic proposition survived this study, but the actual-browser/runtime and
explicit maintainer gates remain open. If maintainers later select Narrow B
after those gates, write or update the binding ADR before production migration.
If maintainers reject Narrow B, close AR-0029 with no stable change and retain
this corpus as update-shock evidence.

## Reopening / Resumption Pressure

Resume Narrow B admission work only when an actual browser is available and a
maintainer wants to evaluate the remaining GLB/browser, real workload budget,
documentation, and placement gates. Reopen Full B only for an independently
demonstrated non-camera public-vocabulary failure. Reopen C migration when
additional substantial callers keep its operation surface bounded while its
performance specializations remain few and reviewable.

## Maintainer Gate

The recommendation is deliberately non-binding. Production remains A and the
study must not create a migration plan or stable API until maintainers choose
one of the plan's bounded dispositions explicitly.
