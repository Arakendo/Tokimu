# Tokimu Website

## Status

| Field | Value |
| --- | --- |
| Status | Active |
| Canonical domain | `tokimuengine.org` |
| Supporting domains | `tokimuengine.com`, `tokimuengine.net` |
| Scope | Public project website, documentation publishing, and bounded Tokimu-powered interactive evidence |
| Source discussion | `docs/Conversations/Tokimu Website.md` |
| Related plans | `docs/Plans/Standalone/consumer-corpora.md`, `docs/Plans/Standalone/typescript-shader-material-presentation-control.md` |
| Related decisions | ADR-0001, ADR-0003, ADR-0004, ADR-0005 |

## Purpose

Tokimu needs a durable public website that explains the project, publishes its
documentation, and lets visitors inspect selected engine capabilities through
real browser consumers.

The website should demonstrate Tokimu without making basic access to project
knowledge depend on Tokimu, JavaScript, WebAssembly, WebGPU, or a compatible
browser.

The governing principle is:

> **MkDocs publishes the knowledge; Tokimu turns selected parts of that
> knowledge into executable evidence.**

The site is not merely a brochure and is not an attempt to make Tokimu replace
a conventional web platform. It is a static documentation site containing
bounded Tokimu-powered interactive regions where live execution provides
evidence that static prose cannot.

## Primary Architectural Claim

The website should prove:

> A conventional documentation site can embed Tokimu through public WASM and
> TypeScript boundaries without depending on private engine implementation,
> duplicating engine semantics in browser code, or making documentation
> unavailable when interactive presentation cannot run.

The intended composition is:

```text
Markdown and structured evidence
              ↓
          MkDocs build
              ↓
     durable static HTML/CSS
              ↓
 optional Tokimu WASM islands
              ↓
 executable evidence and inspection
```

## Ownership Boundaries

### MkDocs Owns Publishing

MkDocs and the static-site layer own:

- Markdown rendering;
- document navigation;
- stable URLs;
- page metadata;
- search;
- RSS or journal feeds;
- sitemap and search-engine metadata;
- static fallbacks;
- ordinary HTML accessibility;
- canonical-domain declarations.

### Tokimu Owns Interactive Engine Semantics

Tokimu-powered regions may own:

- provider-neutral asset observations;
- vector and mesh previews;
- presentation overrides;
- runtime and importer diagnostics;
- corpus evidence visualization;
- interactive architecture views backed by real Tokimu data.

Tokimu must not become responsible for:

- rendering ordinary documentation prose;
- global site navigation;
- search indexing;
- basic link behavior;
- essential project information;
- decorative effects that provide no engine evidence.

### TypeScript Owns Browser Interaction

TypeScript may own:

- mounting and unmounting an interactive island;
- file-selection and drag/drop interaction;
- bounded user controls;
- translating browser events into Tokimu requests;
- presenting provider-neutral observations and diagnostics;
- progressive loading and visible fallback state.

TypeScript must not:

- parse SVG, CGM, GLB, glTF, or FBX independently;
- redefine importer or presentation semantics;
- construct renderer-native resources;
- silently substitute Three.js, Babylon.js, browser SVG, or another renderer
  when Tokimu reports unsupported behavior.

### The Browser Owns Mechanisms

The browser may provide:

- DOM and canvas surfaces;
- file selection;
- pointer and keyboard events;
- local byte access after explicit user selection;
- fetch and cache mechanisms;
- accessibility APIs.

Browser mechanisms do not become Tokimu semantic ownership.

## Progressive Enhancement Contract

Every page must remain useful before an interactive island loads.

Each Tokimu-powered region should provide:

1. a static title and explanation;
2. current capability and limitation text;
3. a screenshot, structural artifact, or other static evidence where useful;
4. an explicit control to load or run the interactive experience;
5. visible loading, unsupported, failed, and ready states;
6. an accessible textual result or diagnostic summary.

The website should follow this rule:

> If Tokimu fails to load, the page remains useful. If Tokimu loads, the
> visitor can verify the claim being made.

Interactive modules should load lazily. A visitor reading an ADR or the
homepage should not pay the asset-workbench download and startup cost unless
they choose to run it.

## Domain Policy

`tokimuengine.org` is the canonical public project domain.

Initial domain behavior:

- `tokimuengine.org` serves the website and documentation;
- `tokimuengine.com` permanently redirects to the equivalent `.org` URL;
- `tokimuengine.net` permanently redirects to the equivalent `.org` URL;
- metadata, repository links, feeds, and canonical tags use `.org`;
- alternate domains do not publish independent mirrors.

The `.com` domain may later host or identify a commercial offering only after
such an offering exists and has an explicit relationship to the open-source
project. The `.net` domain may later serve network-oriented infrastructure only
after a concrete service justifies that distinction.

## Initial Information Architecture

The first site map should remain small enough to maintain:

```text
/
├── overview
├── getting-started
├── architecture
│   ├── software-design
│   ├── ADRs
│   └── architectural-reviews
├── capabilities
│   ├── runtime
│   ├── presentation
│   ├── assets
│   └── diagnostics
├── formats
│   ├── SVG
│   ├── CGM
│   ├── glTF-and-GLB
│   └── FBX
├── corpus
│   ├── evidence
│   ├── coverage
│   └── known-limitations
├── lab
│   ├── asset-workbench
│   ├── material-workbench
│   └── diagnostics-explorer
└── journal
```

This is an organizational target, not a requirement to publish empty sections.
Navigation entries should appear only when their first useful page exists.

## Public Capability Language

The website must distinguish maturity precisely.

Suggested public states:

| State | Meaning |
| --- | --- |
| Renderable | Tokimu can lower and present the admitted semantics through the declared path. |
| Previewable | Tokimu can produce a bounded diagnostic preview, but important semantics remain deferred. |
| Inspected | Tokimu can decode and report bounded structure without claiming canonical rendering. |
| Deferred | The capability is intentionally unsupported or awaiting more evidence. |
| Experimental | The capability works through an incubating boundary that is not yet stable. |

The site must not collapse these states into a generic `supported` badge.
Coverage percentages must identify their corpus, admitted profile, and known
exclusions.

## Visual Direction

The website should feel related to Tokimu's existing workbenches:

- technical drafting-grid atmosphere;
- restrained dark surfaces;
- pale cyan or mint evidence geometry;
- warm amber for capability state and attention;
- readable serif display text paired with a practical technical sans or mono
  face;
- explicit labels, measurements, diagnostics, and boundary states;
- motion used to explain state transitions rather than decorate navigation.

The visual system must remain CSS-first for ordinary pages. Tokimu rendering is
reserved for regions that present engine-owned evidence.

The site should avoid visual language that implies a mature general-purpose
product before the engine has earned that claim.

## Website Consumer Corpus

The website should become a repository-owned consumer corpus after its first
Tokimu-powered island exists:

```text
corpus/
  consumers/
    tokimu-website/
```

It is initially a Tier 2 incubating consumer if it depends on `corpus/lib` or
other provisional boundaries.

Its `DESIGN.md` should record:

- the primary composition claim;
- public and incubating dependencies;
- static-site and Tokimu ownership boundaries;
- island lifecycle;
- application-owned state;
- supported browser requirements;
- fallback behavior;
- expected diagnostics;
- accessibility behavior;
- performance budgets;
- security and file-handling policy;
- known friction and deferred capabilities.

The website does not become an independent production consumer merely because
it is publicly deployed. It remains a first-party consumer corpus until an
independently owned application provides external evidence.

## Initial Interactive Proof

The first interactive island should reuse the existing ASP.NET/WASM asset
workbench concepts rather than inventing another engine demonstration.

The bounded first proof should:

- load only after explicit visitor action;
- accept one known fixture and optional local user-selected bytes;
- expose one or two formats whose current maturity can be represented
  honestly;
- show the provider-neutral observation;
- render the admitted preview;
- display structured diagnostics and deferred semantics;
- provide reset and teardown behavior;
- leave the surrounding MkDocs page usable throughout.

The first proof should not attempt every importer, shader editing, runtime
profiling, and architecture visualization simultaneously.

## Security And Privacy

User-selected files should remain local to the browser unless the visitor
explicitly chooses a future upload service.

The first release must:

- avoid uploading selected files;
- state that processing occurs locally;
- enforce bounded file sizes before crossing the WASM boundary;
- bound parser work, diagnostics, geometry, and rendered output;
- handle malformed input without freezing the page;
- avoid evaluating scripts or active content from imported files;
- avoid ambient filesystem, network, DOM, timer, or process access from authored
  shader or asset semantics;
- clear retained file bytes and session state when the island resets or
  unmounts where practical.

Any future server-side upload or artifact-sharing feature requires a separate
security and retention review.

## Accessibility

Static documentation is the authoritative accessible presentation.

Interactive islands must provide:

- keyboard-accessible activation and controls;
- visible focus;
- textual status and diagnostic output;
- labels for controls and canvas regions;
- reduced-motion behavior;
- sufficient contrast;
- a useful non-canvas summary of the current observation;
- no requirement to drag a file when a file-picker action can express the same
  intent.

Canvas pixels are evidence, not the only source of meaning.

## Performance And Lifecycle

Tokimu must behave as an embedded guest rather than assuming ownership of the
browser tab.

Each island should:

- initialize only after it becomes relevant or the visitor activates it;
- expose loading and startup diagnostics;
- pause or reduce work when hidden;
- avoid a permanent animation loop when the scene is unchanged;
- release event listeners and renderer resources when unmounted;
- keep its canvas and diagnostics within declared layout bounds;
- avoid shifting surrounding document content during startup;
- report meaningful performance-budget warnings through the admitted
  diagnostics model.

Initial budgets should be measured before they become guarantees. The first
evidence record should at least capture:

- compressed JavaScript and WASM payload sizes;
- cold startup duration;
- first useful presentation duration;
- steady-state frame or event-driven render behavior;
- memory growth during repeated load/reset cycles;
- diagnostic volume for malformed inputs.

## Content Sources

The first website should publish curated material rather than exposing the
repository tree directly.

Candidate sources include:

- project overview and getting-started material;
- the SDD;
- accepted ADRs;
- Architectural Review findings;
- roadmap status;
- library corpus summaries;
- selected evidence artifacts;
- public capability and limitation summaries;
- a journal derived from intentionally published engineering notes.

Plans, conversations, working notes, and archives should not automatically
become public navigation. Their publication needs a deliberate audience and
status label.

### Initial Publication Inventory

| Repository material | Publication class | Initial website treatment |
| --- | --- | --- |
| README, SDD, kernel principles, semantic kernel map, capability boundaries | Public reference | Curated into overview and architecture pages; not mirrored blindly |
| TypeScript design and WASM boundary material | Public reference | Curated into the Rust and TypeScript architecture page |
| Accepted ADRs | Public reference | Summarized through the architectural-decision method and bounded claims |
| Architectural Review findings | Public history | Selected findings may support evidence pages; full records remain repository history |
| Roadmap | Public reference | Curated status page using current maturity vocabulary |
| Library corpus records | Public evidence | Published one bounded format page at a time with drift checks |
| Deterministic artifacts and selected screenshots | Public evidence | Linked only when provenance and evidence type are explicit |
| Plans and Conversations | Internal working material | Excluded from website navigation and automatic publication |
| Notes and `.workbench` records | Internal working material | Excluded unless deliberately promoted into a public reference |
| Archive | Deferred history | Repository-accessible but absent from public navigation |
| Engineering journal or feed | Deferred | Requires an intentional editorial and chronology policy |

The website therefore publishes a maintained interpretation of authoritative
repository material. It does not create a second architectural source of truth
and does not treat repository location alone as permission to publish.

## Implementation Slices

### Slice 0: Website Boundary And Content Review

#### Deliverables

- [x] Confirm `tokimuengine.org` as the canonical domain.
- [x] Record redirect intent for `.com` and `.net`.
- [x] Inventory repository documents suitable for public publication.
- [x] Classify documents as public reference, public history, internal working
      material, or deferred.
- [x] Define the initial capability-state vocabulary.
- [x] Select the first interactive proof and its honest support claim.

#### Acceptance Criteria

- [x] The website has one canonical identity.
- [x] Internal conversations and plans are not published accidentally.
- [x] The first release contains no empty navigation categories.
- [x] The first interactive proof is tied to existing code and evidence.
- [x] Public wording distinguishes guarantees from observations.

### Slice 1: Static MkDocs Foundation

#### Deliverables

- [x] Create the website source directory and MkDocs configuration.
- [x] Establish a curated documentation input tree.
- [x] Add overview, getting-started, architecture, roadmap, and known-limitations
      pages.
- [x] Add canonical metadata and sitemap behavior. Feed behavior remains
      deferred until the site publishes chronological content.
- [x] Add a CSS-first Tokimu visual theme.
- [x] Add a static build command suitable for local use and CI.

#### Acceptance Criteria

- [x] The site builds without Rust, WASM, Node, or a GPU after generated
      interactive assets are absent.
- [x] Every published page is readable and navigable without JavaScript.
- [x] URLs remain stable across repeated builds.
- [x] The generated site identifies `.org` as canonical.
- [x] Automated accessibility checks find no blocking structural issue in the
      static shell. Manual browser and assistive-technology review remains
      separately tracked in Slice 8.

### Slice 2: Domain And Deployment Baseline

#### Deliverables

- [x] Select GitHub Pages artifact deployment without coupling site content to that
      provider.
- [ ] Configure HTTPS for all three domains.
- [ ] Configure path-preserving permanent redirects from `.com` and `.net`.
- [ ] Add preview deployment for proposed site changes.
- [ ] Add cache rules that distinguish immutable hashed WASM assets from HTML.
- [x] Document rollback and deployment ownership.

#### Acceptance Criteria

- [ ] Each alternate-domain URL resolves to its equivalent `.org` URL.
- [ ] No domain serves a divergent mirror.
- [ ] A failed interactive bundle does not prevent static deployment.
- [x] A previous static deployment can be restored through a reviewed source
      revert and normal Pages redeployment.
- [ ] HTML changes do not remain hidden behind an immutable cache policy.

### Slice 3: Interactive Island Contract

#### Deliverables

- [x] Define the declarative HTML marker or custom-element contract used to
      mount a Tokimu island.
- [x] Define island lifecycle states: idle, loading, ready, unsupported, failed,
      and unmounted.
- [x] Define structured configuration and evidence inputs.
- [x] Add lazy loading and explicit activation.
- [x] Add teardown, listener cleanup, and resource release.
- [x] Add static fallback content inside each mount region.

#### Acceptance Criteria

- [x] Multiple islands can exist without global-state collisions.
- [x] An island can mount and unmount repeatedly without duplicate listeners.
- [x] Failure is visible, bounded, and does not damage page navigation.
- [x] The static fallback remains readable before and after failure.
- [x] The island does not require private Tokimu APIs.

### Slice 4: First Tokimu-Powered Evidence Page

#### Deliverables

- [x] Reuse a bounded asset-workbench flow through public or explicitly
      incubating WASM APIs.
- [x] Include one known fixture with static expected evidence.
- [x] Support optional local file selection under explicit size limits.
- [x] Present observation, preview, diagnostics, and deferred semantics.
- [x] Add reset behavior.
- [x] Add a no-WASM or unsupported-browser state.

#### Acceptance Criteria

- [x] The known fixture produces the expected provider-neutral observation.
- [x] TypeScript does not parse the source format.
- [x] User-selected bytes are not uploaded.
- [x] Malformed or excessive input fails with a bounded diagnostic.
- [x] The page remains useful when WASM cannot initialize.
- [x] Preview dimensions remain bounded as diagnostics change.

### Slice 5: Website Consumer Corpus

#### Deliverables

- [x] Create `corpus/consumers/tokimu-website` after the first island proves the
      category.
- [x] Add a `DESIGN.md` with consumer tier and ownership boundaries.
- [x] Add deterministic semantic checks for the known fixture.
- [x] Add a WASM build and TypeScript typecheck.
- [x] Add a static-page smoke test that runs without the interactive bundle.
- [x] Record direct public and incubating dependencies.

#### Acceptance Criteria

- [x] The website consumes Tokimu as a bounded downstream application.
- [x] Incubating dependencies are labeled and do not appear as stable public
      guarantees.
- [x] Static and interactive validation can fail independently.
- [x] Application meaning remains outside browser rendering and importer
      providers.
- [x] The consumer can identify the first failing composition boundary.

### Slice 6: Evidence And Capability Pages

#### Progress

- The first format record now publishes the bounded W3C SVG geometry profile.
- Website validation compares its official `40 / 525 (7.62%)` coverage claim
  and evidence ledger with `docs/Libraries/w3c-svg-corpus-testing.md`.
- The page labels structural, deterministic CPU, WASM semantic, and browser
  visual evidence separately.
- Repository links expose the selected fixtures, provenance, registered cases,
  and structural golden workflow behind the public summary.
- A second bounded format record now publishes CGM as Previewable, ties its
  15-case selection and stage counts to the authoritative CGM record, and
  keeps unresolved paint semantics explicit.
- Additional format pages remain future corpus growth rather than a requirement
  for this evidence-page proof.

#### Deliverables

- [x] Add format pages backed by corpus reports rather than hand-maintained
      support claims.
- [x] Publish admitted corpus scope, coverage, and known exclusions.
- [x] Link static screenshots and structural artifacts where appropriate.
- [x] Add a clear generated-at date and source revision to evidence.
- [x] Distinguish native, deterministic CPU, WASM semantic, and browser visual
      evidence.

#### Acceptance Criteria

- [x] Coverage claims identify their denominator and admitted profile.
- [x] A stale or missing report is visible rather than silently replaced by an
      old claim.
- [x] Structural artifacts remain authoritative for structural claims.
- [x] Browser screenshots are labeled as visual evidence rather than semantic
      truth.
- [x] Capability labels match the state vocabulary in this plan.

### Slice 7: Accessibility, Performance, And Security Hardening

#### Progress

- The first island now exposes a uniquely associated textual report and a
  concise polite live announcement for each observation.
- Canvas remains labeled evidence and explicitly points assistive technology
  to the authoritative textual summary and report.
- Local input size is checked before bytes are read or sent to WASM;
  diagnostics remain browser-bounded.
- Presentation remains event-driven and suppresses resize redraws while the
  document is hidden or the island is offscreen.
- Reduced-motion and forced-color behavior are covered by executable website
  contract tests.
- WASM startup, inspection, first useful evidence, and Canvas presentation are
  reported as separate observations rather than one ambiguous startup number.
- Published island assets have executable per-file and aggregate size budgets.
  The 2026-07-30 source-tree measurement is 887,389 bytes against the 1 MiB
  aggregate ceiling.
- Empty, malformed, binary-corrupted, and entity-bearing SVG inputs exercise
  bounded diagnostic behavior.
- Thirty-two activation/reset cycles require exactly one release per mount and
  retain only the controller's delegated click handler.

#### Deliverables

- [x] Add keyboard and screen-reader contract checks for island controls and
      evidence output.
- [x] Add reduced-motion and forced-color stylesheet contracts.
- [x] Measure payload, startup, first-useful-presentation, and reset behavior.
- [x] Add hidden-page and offscreen presentation behavior.
- [x] Add malformed, oversized, and adversarial local-file fixtures.
- [x] Add a visible local-processing and privacy statement.
- [ ] Verify teardown does not retain unbounded memory or event handlers.

#### Acceptance Criteria

- [x] The first interactive proof is operable without drag input.
- [x] Essential results are available as text.
- [x] Repeated activation and reset remain within documented handler and
      release-count bounds.
- [x] Oversized input is rejected before expensive parsing.
- [x] Hidden or inactive islands do not consume continuous presentation work.
- [x] No selected file leaves the browser in the initial implementation.

### Slice 8: Public Launch Review

#### Progress

- Public wording was reviewed against the SDD's Rust/WASM and TypeScript
  ownership boundaries, accepted presentation ADRs, relevant Architectural
  Reviews, and current website consumer evidence.
- Stale claims describing the deployed site, WASM island, and interactive
  evidence as pending were corrected without promoting experimental browser
  execution into a general engine guarantee.
- A post-build crawler now checks every generated page for canonical `.org`
  metadata, descriptions, resolvable internal links and assets, `CNAME`, and
  `.nojekyll`.
- The same generated-site check proves that useful homepage knowledge and
  static evidence context survive without JavaScript.
- Known launch limitations and the evidence-first maintenance process are
  published on the limitations page and recorded in the website consumer
  design.
- Live HTTP review on 2026-07-30 confirmed `.org` serves GitHub Pages with
  `200 OK`. DNS forwarding for `.com` and `.net` is now configured, but the
  observed public edge still showed the prior Squarespace behavior: `.com`
  redirected to non-path-preserving `http://tokimuengine.org`, while `.net`
  served a Squarespace root page and lost paths when redirecting. Propagation
  and path-preserving HTTPS behavior remain external verification work.
- Supported-browser interaction and heap-level retention remain manual
  evidence gaps.

#### Deliverables

- [x] Review public claims against the SDD, ADRs, Architectural Reviews, and
      current corpus evidence.
- [x] Review domain redirects, canonical metadata, and broken links.
- [x] Review static behavior with JavaScript disabled.
- [ ] Review interactive behavior on supported browser targets.
- [x] Record known launch limitations.
- [x] Define the post-launch evidence and content maintenance process.

#### Acceptance Criteria

- [x] The site describes current Tokimu rather than only its ambition.
- [x] Every interactive claim has static context and an explicit failure state.
- [x] The launch does not imply that inspected or previewable formats are fully
      supported.
- [x] Website deployment does not change engine ownership boundaries.
- [x] The website consumer corpus records any public API friction found during
      launch.

## First Release Definition

The first public release is complete when:

- `tokimuengine.org` serves a readable static MkDocs site;
- `.com` and `.net` redirect to `.org`;
- the overview, architecture, roadmap, and known-limitations pages are useful;
- one Tokimu WASM island loads only after explicit activation;
- the island exercises one bounded, honest engine capability;
- the same page remains useful without JavaScript or WASM;
- local file handling is bounded and does not upload data;
- static, Rust/WASM, TypeScript, and browser validation responsibilities are
  documented;
- the website is recorded as a first-party consumer corpus.

The first release does not require every format page, a shader playground,
interactive ADRs, a diagnostics explorer, or a custom documentation renderer.

## Validation Matrix

| Boundary | Required evidence |
| --- | --- |
| Static documentation | MkDocs build, link check, no-JavaScript review |
| Canonical identity | `.org` metadata and path-preserving redirect checks |
| Island lifecycle | mount, ready, failure, reset, and unmount tests |
| WASM contract | Rust tests and `wasm32-unknown-unknown` build |
| TypeScript adapter | strict typecheck and bounded request/response tests |
| Asset observation | deterministic known-fixture result |
| Visual presentation | manual browser evidence plus bounded layout checks |
| Accessibility | keyboard, focus, text alternative, contrast, reduced motion |
| Security | size limits, malformed inputs, local-only file policy |
| Performance | payload, startup, first useful result, idle work, reset memory |

## Non-Goals

This plan does not initially provide:

- a Tokimu-rendered Markdown engine;
- Tokimu-owned global site navigation;
- a browser IDE;
- unrestricted public file hosting;
- server-side asset conversion;
- user accounts;
- a package registry;
- a benchmark leaderboard;
- a universal asset viewer;
- automatic publication of every repository document;
- silent renderer or importer substitution;
- proof that a first-party public website is an independent production
  consumer.

## Risks

### The demonstration overwhelms the documentation

Mitigation: static content remains authoritative, and interactive modules load
only after explicit activation.

### The website overclaims format support

Mitigation: use precise maturity states and generate claims from reviewed
corpus evidence.

### The site becomes a second application framework

Mitigation: keep ordinary web behavior in MkDocs, HTML, and CSS. Share only
Tokimu semantic consumers whose reuse is independently demonstrated.

### WASM startup harms the first visit

Mitigation: isolate and lazy-load hashed bundles, provide static evidence, and
measure first-useful-presentation time.

### Imported files expose visitors or infrastructure to risk

Mitigation: process bounded bytes locally, reject excessive inputs early, and
require a separate review before any upload path exists.

### Public deployment is mistaken for external adoption

Mitigation: label the website as a first-party consumer corpus and preserve the
distinction between public deployment and independent ownership.

### Documentation drifts from evidence

Mitigation: give evidence reports source revisions and dates, and fail visibly
when generated evidence is stale or unavailable.

## Graduation And Reopening Triggers

The website island boundary may justify reusable first-party support when:

- at least two different site experiences need the same mount and lifecycle
  semantics;
- another independent browser consumer needs the same integration contract;
- repeated glue belongs clearly to Tokimu rather than to the website;
- extraction preserves static-site independence and provider-neutral meaning.

Reopen the plan or an owning Architectural Review if:

- the site requires private engine APIs;
- TypeScript begins duplicating importer or presentation semantics;
- WASM and native consumers require different application meaning;
- interactive regions cannot remain bounded and independently disposable;
- public evidence repeatedly cannot be generated from existing diagnostics or
  corpus artifacts;
- accessibility requires semantic information unavailable above the renderer.

## Completion Criteria

This effort is mature enough to leave active planning when:

- the static site is durable, accessible, canonical, and independently useful;
- Tokimu-powered regions behave as bounded progressive enhancements;
- at least one public interactive page consumes Tokimu through the declared
  WASM and TypeScript boundaries;
- capability claims are traceable to evidence;
- domain, privacy, security, performance, and failure policies are documented
  and exercised;
- the website consumer records architectural friction without disguising it as
  local glue;
- future interactive experiences can be added without changing who owns
  documents, application meaning, engine semantics, or pixels.
