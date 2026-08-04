# AR-0010: Weaver XSLT Resource Resolver Boundary

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | 2026-08-03 |
| Last reviewed | 2026-08-03 |
| Scope | Foundational service / capability / frontend / cross-cutting |
| Trigger | Weaver's documented injected URI resolver may become an independent TypeScript/XML consumer of Tokimu Resource Space |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005 |
| Related evidence | AR-0009; `docs/Plans/memory-resource-store.md`; pinned `third-party/weaver-xslt` source; Weaver `ARCHITECTURE.md`, `SEMANTIC_BOUNDARIES.md`, and `URI_RESOLUTION.md` |
| Admission exception | None |

## Architectural Question

Can a Weaver XSLT consumer obtain selected XML, XSLT, and related resource bytes
through a bounded Tokimu Resource Space adapter without making Tokimu own XSLT
semantics, URI policy, TypeScript execution, or platform I/O?

## Context

Weaver is an external TypeScript-native XSLT project, independently pinned as
the `third-party/weaver-xslt` submodule at
`e7472c6ae2894345f59ed38da38816092af34fea`. Its architecture separates URI
resolution, resource loading, and result publication. Weaver owns base-URI
selection, relative URI resolution, and source identity propagation. Its host
owns allowed schemes, canonicalization for host identity, loading mechanisms,
and write policy.

Tokimu's provisional Resource Space contract separately owns qualified store,
root, folder, and resource identity plus bounded byte lookup. Its current XML
adapter deliberately supports only selected, same-folder resource lookup and
does not interpret XML or SVG semantics.

The overlap is promising but not proof of a shared capability: Weaver's URI
contract is broader than Tokimu's current selected-resource model. This review
keeps those meanings separate while determining whether a small adapter can
make them compose honestly.

## Trigger And Evidence

- Corpus examples: `corpus/hello-resource-space` and the Asset Workbench use
  Resource Space for explicit selected-resource sessions. No Weaver consumer
  has yet used the public contract.
- Automated tests: `resource-space` and `resource-space-xml` test qualified
  lookup and reject local, parent-directory, and unsupported references.
- External design evidence: Weaver documents an injected resolver boundary;
  pure resolution must not load resources, and host policy determines allowed
  schemes and resource access.
- Independent consumers: Weaver would be a third, cross-language consumer only
  after an adapter invokes the public Resource Space API from a real transform.
- Missing evidence: URI mapping, nested-reference policy, cycle behavior,
  structured failure translation, selected-session behavior in a TypeScript
  host, and result-publication handling are all unproven.

## Ownership Analysis

The candidate composition preserves four distinct decisions:

```text
Tokimu Resource Space
    qualified resource identity, folders, visibility, retained bytes

Weaver
    XSLT/XPath semantics, base URI, relative resolution, cache and cycle rules

Weaver-to-Tokimu adapter
    maps an allowed resolved Weaver identity to one selected Tokimu resource
    and translates lookup failure into Weaver's structured resolver result

TypeScript host
    user selection, allowed resolver policy, UI, and any publication target
```

Tokimu must not own XSLT compilation or execution, DOM/XDM representation, URI
syntax, generic URI canonicalization, `xml:base`, network or filesystem access,
or result-document publication. Weaver must not infer Resource Space identity
from display names, host paths, or unqualified strings.

The adapter is a replaceable provider-facing bridge. It may establish a bounded
mapping for a selected session, but it must not turn Resource Space into a
global URI resolver or silently widen access outside selected roots.

## Dependency Direction

```text
Current:

Weaver URI semantics
        |
        v
injected host resolver and loader
        |
        v
host-specific in-memory, filesystem, or browser mechanisms

Candidate:

Weaver URI semantics
        |
        v
Weaver-to-Tokimu resolver adapter
        |
        v
Tokimu Resource Space public contract
        |
        v
selected-session provider bytes
```

Neither `tokimu-core` nor Resource Space may depend on TypeScript, Weaver,
XSLT, DOM/XDM types, or browser/Node resource APIs. Weaver remains the owner of
resolution meaning; Resource Space remains a bounded source-byte mechanism.

## Alternatives Considered

### Alternative A: Weaver Uses Its Existing Host Resolver Only

- Benefits: no cross-project contract or adapter; Weaver remains entirely
  independent.
- Costs: no evidence that Resource Space can serve an XML/XSLT consumer.
- Failure mode: each consumer repeats selected-resource identity and access
  policy differently.

### Alternative B: Direct Host Callbacks Into Resource Space

- Benefits: minimal initial code.
- Costs: URI resolution, authorization, and loading collapse into ad hoc host
  callbacks; diagnostics and identity can drift.
- Failure mode: Weaver's resolver semantics become dependent on incidental
  Tokimu implementation details.

### Alternative C: Narrow Weaver-to-Tokimu Resolver Adapter

- Benefits: exercises two public boundaries while preserving each project's
  ownership; failures can identify resolution, mapping, or byte lookup.
- Costs: requires explicit URI-to-resource mapping and diagnostic translation.
- Failure mode: the adapter quietly becomes a generic URI policy layer.

### Alternative D: Make Resource Space Own Generic URI Resolution

- Benefits: one apparent resolver surface for future consumers.
- Costs: imports XSLT/XML and host-policy assumptions into a provider-neutral
  resource capability.
- Failure mode: Resource Space becomes a hidden filesystem/network policy
  engine.

### Alternative E: Promote Weaver Or XSLT Support Into Tokimu

- Benefits: direct integration.
- Costs: violates Tokimu's Rust-engine boundary and prematurely admits a
  language-specific execution system.
- Failure mode: Tokimu owns foreign execution semantics without independent
  engine-wide pressure.

## Findings

Current documentation supports the following preliminary findings:

- Weaver and Resource Space agree that identity, resolution, authorization,
  loading, and publication are separate concerns.
- A selected Resource Space session could provide an explicit in-memory source
  to Weaver without granting ambient filesystem or network access.
- Relative URI resolution and URI canonicalization remain Weaver and host
  semantics; Resource Space is not yet a valid general URI authority.
- The same qualified resource may need a Weaver-visible URI chosen by the
  adapter. That mapping is not a proof that URI identity equals Resource Space
  identity.
- A successful adapter would be independent consumer evidence for AR-0009,
  not evidence that XSLT belongs in Tokimu.

Uncertainties remain around nested and cyclic references, `xml:base`, text and
binary resource kinds, output publication, native/WASM parity, and whether the
adapter can stay smaller than Weaver's own host resolver contract.

## Disposition

**Proposed for focused evidence collection.** No integration, package
extraction, or admission follows from the external design review. The next
useful evidence is one bounded transform that resolves preselected related
resources only through the public Resource Space API and reports denied or
missing resources structurally.

## Consequences

- AR-0009 can name Weaver as a reviewed prospective consumer, but it does not
  count Weaver toward admission evidence until a real consumer exists.
- Resource Space must retain qualified byte lookup without learning URI or
  XSLT semantics.
- Any future bridge should live outside `tokimu-core`, `tokimu-runtime`, and
  the Resource Space semantic contract.
- Diagnostics must preserve the requesting Weaver operation and source context
  where Weaver supplies them; Tokimu lookup diagnostics must not be flattened
  into ambiguous strings.

## Required Follow-Up

- [ ] Design a selected-session URI mapping that cannot escape admitted roots.
- [ ] Create a minimal Weaver fixture with a stylesheet, source XML, and one
      related selected resource.
- [ ] Implement an adapter using only the public Resource Space API.
- [ ] Test successful sibling lookup plus missing, parent-directory, unknown
      scheme, and denied-resource failures.
- [ ] Verify Weaver's interpreter and generated execution paths use the same
      resolver behavior where both are available.
- [ ] Record whether the adapter adds evidence to AR-0009 without requiring a
      Resource Space API redesign.

## Reopening Triggers

- a real Weaver adapter requires Resource Space to parse, canonicalize, or
  authorize arbitrary URIs;
- selected-session lookup cannot preserve Weaver's required source identity or
  diagnostic context;
- multiple XML/XSLT consumers require the same adapter semantics;
- a browser or native host cannot preserve the bounded mapping;
- result-document publication pressures Resource Space into owning writes;
- implementation shows a smaller decomposition than a resolver adapter.

## Review History

### Cycle 1 -- 2026-08-03

- Status entering review: Proposed.
- New evidence: Weaver architecture, semantic-boundary, and URI-resolution
  documents were reviewed from `F:\LocalSource\TS XSLT\docs`.
- Participants or reviewers: project maintainer and Codex.
- Findings: both projects make host access explicit, but they assign different
  and compatible meanings to URI resolution and resource byte retention.
- Disposition: Proposed for focused evidence collection; no implementation or
  admission decision.
- Resulting ADR or documentation change: AR-0009 now records Weaver as a
  reviewed prospective consumer; no ADR change.

### Cycle 2 -- 2026-08-03

- Status entering review: Proposed.
- New evidence: Weaver is now pinned as `third-party/weaver-xslt` at
  `e7472c6ae2894345f59ed38da38816092af34fea`. The
  `corpus/consumers/weaver-xslt-resource-space` runner passed a controlled
  XML/XSLT source-buffer baseline through interpreter and auto/native paths
  without coupling TypeScript tooling into any Tokimu crate. The paths matched
  semantically; a leading literal stylesheet newline remains recorded as
  execution evidence rather than normalized away globally.
- Findings: Weaver's documented `ResourceResolver` contract is a precise fit
  for the candidate bridge, but the currently exposed `XsltProcessor` surface
  does not yet accept that resolver. A real selected-session adapter would be
  dishonest until Weaver exposes the contract or an equivalent public seam.
- Disposition: retain Proposed. Exercise source-buffer transforms now; defer
  the Resource Space adapter, resolver failures, and admission evidence until
  the public resolver surface exists.
- Resulting ADR or documentation change: added
  `docs/Plans/weaver-xslt-resource-space-consumer-corpus.md`; no ADR change.

## References

- `docs/Architectural Reviews/AR-0009-resource-store-identity-and-kernel-boundary.md`
- `docs/Plans/memory-resource-store.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `third-party/weaver-xslt/docs/ARCHITECTURE.md`
- `third-party/weaver-xslt/docs/SEMANTIC_BOUNDARIES.md`
- `third-party/weaver-xslt/docs/URI_RESOLUTION.md`
