# Renderer Resource Identity And Failure Disposition Options

| Field | Value |
| --- | --- |
| Date | 2026-08-13 |
| Plan | [Renderer Resource Identity And Failure Presentation](../renderer-resource-identity-and-failure-presentation.md) |
| Reviews | AR-0024, AR-0027 |
| Status | Accepted 2026-08-13; no contract admitted |

## Evidence Boundary

Native and browser evidence now agrees on the facts relevant to this study:

- application-owned identity can preserve live resources across unrelated
  dynamic additions;
- B, D, and E distinguish the lifecycle failures they claim;
- native and browser retain `ResourceUnresolved` with `MeshHandle(44)`;
- native and browser WGPU providers support deliberate same-handle replacement;
- caller-owned terminal/DOM presentation can retain failure after provider
  return, but page-disposal lifetime is not demonstrated; and
- visual diagnostic substitution is only truthful for the explicitly
  classified source-omission case exercised by Doom.

The evidence does not demonstrate that allocation, terminal-record storage, or
diagnostic presentation must move into the Native Ring or renderer.

## Resource-Identity Dispositions

| Choice | Evidence for | Evidence against / missing | Current reading |
| --- | --- | --- | --- |
| Application/tooling helper | B works on native and WASM; callers retain logical identity and replacement intent | Helpers may duplicate until a second production caller converges | Survives |
| Renderer validation + application allocation | E cleanly distinguishes create/replace/retire and retains provider replacement | No real caller has required a stable public renderer lifecycle API | Incubate |
| Renderer allocation/lifetime owner | Could prevent independent numeric collisions | C was not needed to solve any observed case; transfers ownership and migration cost | Not earned |
| Generational public identity | D best distinguishes stale reuse without larger fixture representation | Requires translation/new renderer identity and no caller yet requires stale-generation semantics | Incubate only |
| Kernel-native identity | Could offer one cross-provider vocabulary | Would make the kernel own presentation-resource lifetime without demonstrated simulation meaning | Reject from current evidence |
| No shared admission | Preserves useful replacement mechanics and lets callers own source/lifecycle policy | Repeated caller duplication could reopen the question | Recommended now |

The recommendation is therefore to retain application-owned allocation and
logical identity, keep deliberate renderer same-handle replacement, and leave
E/D as executable candidates. Do not prohibit replacement as an aliasing fix;
the browser and native evidence prove it is intentional useful behavior.

## Failure-Observation And Terminal Ownership

The shared facts are modest: phase, category, optional resource identity,
caller/correlation, and continuation result can be represented equivalently.
The final owners remain naturally different:

```text
native caller -> terminal / supervisor Result
browser caller -> live DOM host
```

No evidence requires a global mailbox or record surviving page disposal. The
recommended disposition is to retain caller/fixture-owned terminal delivery,
the native first-failure invariant, and corpus-local bounded records. Reopen
only for an independent supervisor/replacement-composition lifetime case.

## Diagnostic Presentation

AR-0027's Alternative A survives. Doom explicitly requested a checked,
offline Purple PNG for 73 retained source-sky omissions on native and browser.
The independent resource-identity fixture proves structured DOM failure
presentation, but correctly does not render an error texture for an unresolved
mesh. It is therefore not a second visual-stand-in caller.

Current recommendation:

- retain corpus/application-supplied diagnostic visuals;
- retain original identity and bounded reason beside any stand-in;
- keep unresolved geometry/resource/provider failures text-only or terminal;
- reject automatic renderer fallback; and
- do not admit a Tokimu-owned standard error texture or public diagnostic
  visual intent until an independent non-Doom visual caller supplies pressure.

## Gate Applicability

No Native Ring or stable public capability is proposed, so ADR-0008 through
ADR-0011 admission evidence is **not applicable**, not silently satisfied. The
corpus candidates remain subject to ordinary tests, linting, bounded storage,
offline provenance, and explicit failure semantics.

## Maintainer Decision

The bounded recommendation is:

> Retain corpus/application-local identity allocation, failure presentation,
> and diagnostic visuals; retain renderer same-handle replacement; incubate
> explicit lifecycle validation and generational identity; admit no shared
> contract now.

The maintainer accepted this disposition on 2026-08-13. The plan closes without
an ADR or SDD change. Future evidence must reopen the appropriate review before
turning any narrower candidate into admission work.
