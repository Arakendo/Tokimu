# AR-0007: Semantic UI Composition Boundary

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-01 |
| Last reviewed | 2026-08-01 |
| Scope | Foundational presentation and consumer-safe UI composition boundary |
| Trigger | Native and WASM consumers now share one headless semantic tree, layout, interaction, and draw-list path |
| Related ADRs | ADR-0001, ADR-0003, ADR-0004, ADR-0005 |
| Related evidence | UI hardening plan, runtime inspector, CGM viewer, UI layout corpus, runtime-observation WASM workbench, `ui-tools` tests |
| Admission exception | None |

## Architectural Question

Has provider-neutral semantic UI composition earned a first-party Tokimu
capability boundary, and which responsibilities must be separated from the
current corpus-side `ui-tools` package before extraction?

This review distinguishes admission of an architectural responsibility from
promotion of the package that currently incubates it. A large or widely used
crate is not automatically a valid engine capability.

## Context

The UI corpus began with independent examples that constructed rectangles,
text, interaction regions, and renderer submissions directly. Runtime
inspector pressure exposed that this approach let ordinary consumers create
overlapping text, inconsistent hit regions, duplicated layout math, and
excessive draw submissions without violating any API.

The hardening work established a shared path:

```text
application intent
        ↓
semantic UiTree
        ↓
headless resolution
        ├── resolved bounds and clips
        ├── interaction and focus
        ├── fit and diagnostics
        └── deterministic UiDrawList
                ↓
        renderer adapter
```

The path is now consumed by independent native applications and by a WASM
consumer. It remains usable without a window, GPU, or live renderer. That is
strong evidence for a semantic presentation boundary.

The current `ui-tools` package is not that boundary. It also directly contains
or depends on TTF/OTF parsing, XML/SVG import, Lucide and font provider support,
vector lowering, tessellation, and corpus-specific compatibility APIs. Those
technologies are useful incubation neighbors but do not share one owner.

## Evidence

- `hello-runtime-inspector`, `hello-cgm`, and `hello-ui-layout` consume the
  shared semantic composition path under different application pressure.
- `runtime-observation-workbench-engine` consumes the same contracts through a
  WASM boundary rather than reproducing layout or interaction semantics in
  TypeScript.
- Headless tests cover deterministic tree resolution, layout constraints,
  clipping, stacking, focus, events, scroll behavior, text fit, draw-list
  ordering, diagnostics, and invalidation.
- Public consumer integration tests exercise the preferred
  `ui_tools::consumer` surface rather than internal modules.
- The semantic path has no renderer, platform, window, or GPU dependency.
- The package dependency graph still includes provider and geometry
  technologies. This is evidence against promoting `ui-tools` wholesale.

## Accepted Findings

The following responsibilities form a coherent foundational presentation
capability candidate:

- semantic node identity, order, parentage, roles, and content intent;
- provider-neutral constraints, layout, visibility, clipping, stacking, and
  scroll resolution;
- interaction regions, deterministic hit testing, focus traversal, and
  semantic input events;
- theme-role resolution without application-owned visual constants;
- deterministic renderer-neutral draw-list lowering;
- bounded diagnostics and invalidation evidence tied to stable node identity.

This capability observes application or world state. It does not own
simulation truth or mutate the world implicitly.

## Deferred Findings

- Extracting a first-party `tokimu-ui` package is deferred until the semantic
  implementation can be separated from provider-backed modules without
  weakening current consumer tests.
- Stable text and icon package extraction remains governed by ADR-0004.
- Vector capability promotion remains governed by AR-0001 and its independent
  consumer criteria.
- Preferred and maximum sizing, complete text overflow policy, richer control
  vocabulary, full event routing, and renderer batching policy remain corpus
  work rather than implied stable API.

## Rejected Findings

- Promote the entire `ui-tools` package because it is large or broadly used.
- Move TTF/OTF parsers, XML/SVG importers, Lucide assets, tessellators, GPU
  resources, window mechanisms, or renderer caches into a semantic UI
  capability.
- Let UI composition become an alternate owner of application or simulation
  state.
- Treat module names or compatibility re-exports as settled engine package
  boundaries.

## Dependency Direction

The intended direction is:

```text
application and world observations
        ↓
semantic UI composition
        ├── provider-neutral text and icon intent
        └── renderer-neutral presentation requests
                ↓
provider and renderer adapters
```

No semantic UI package may depend upward on a renderer, platform, window,
browser, GPU backend, or application-specific shell. Provider implementations
may consume semantic contracts; semantic contracts must not consume provider
implementation identities.

## Disposition

**Continue incubation.** The semantic UI composition responsibility is
accepted as a coherent foundational presentation capability candidate. Package
extraction is deferred because the current `ui-tools` crate still co-locates
several replaceable provider and geometry technologies.

No ADR changes are required by this review cycle. ADR-0004 already establishes
the applicable provider-neutral presentation ownership rule. A future package
admission requires a separate accepted disposition and ADR update.

## Graduation Triggers

- Internally separate semantic composition from font, SVG/XML, icon, vector,
  and tessellation implementations.
- Preserve the native and WASM public-consumer test suite across that split.
- Keep structural tree, layout, interaction, draw-list, and diagnostic
  artifacts stable, or version intentional schema changes.
- Close or explicitly defer the remaining semantic contract gaps named in the
  UI hardening plan.
- Demonstrate a dependency graph with no provider or backend implementation
  dependency in the proposed first-party semantic package.

## Reopening Triggers

Reopen this review when the internal semantic/provider split is complete, a
new independent consumer exposes a missing ownership boundary, or provider
substitution changes semantic layout or interaction behavior.

## Review History

### Cycle 1 — 2026-08-01

Native and WASM consumers established an independent semantic composition
boundary. The review accepted the ownership finding and deferred package
promotion until provider technologies can be separated honestly.
