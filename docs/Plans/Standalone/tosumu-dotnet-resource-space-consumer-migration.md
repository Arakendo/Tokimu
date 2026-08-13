# Tosumu .NET Resource Space Consumer Migration

## Status

| Field | Value |
| --- | --- |
| Status | In progress |
| Opened | 2026-08-03 |
| Last updated | 2026-08-04 |
| Owner | Tokimu and Tosumu maintainers |
| Source | `third-party/tosumu/dotnet` |
| Target | `corpus/consumers/dotnet-tosumu-resource-workbench` |
| External dependency policy | Remove all `ClassLibrary` dependencies and assumptions |
| Related plans | [Logical Resource Space](memory-resource-store.md), [Consumer Corpora](consumer-corpora.md), [Support Library Host Adapters](support-library-host-adapters.md) |
| Related reviews | [AR-0009: Resource Store Identity And Kernel Boundary](../../Architectural%20Reviews/AR-0009-resource-store-identity-and-kernel-boundary.md), [AR-0011: Tosumu-Backed Tasset Canonical Asset Output](../../Architectural%20Reviews/AR-0011-tosumu-backed-tasset-canonical-asset-output.md) |
| Related decisions | ADR-0001, ADR-0003, ADR-0005 |

## Purpose

Move the application-shaped .NET inspection workbench out of Tosumu and into
Tokimu's consumer corpus, then make it consume Tokimu's public Resource Space
semantics with Tosumu acting as the persistent storage provider.

This is not a request to move every .NET test out of Tosumu. Tosumu must retain
tests that prove its own CLI packaging, inspection envelopes, errors, and
storage behavior. The migration applies to the WPF application harness and to
tests whose real subject is application composition rather than Tosumu's
standalone product contract.

The governing claim is:

> **Tokimu owns logical resource meaning. Tosumu owns durable storage behavior.
> .NET owns the host application and presentation. The bridge must preserve all
> three boundaries.**

## Primary Composition Claim

The migrated consumer should prove this path:

```text
.NET desktop interaction
        |
        v
Tokimu consumer bridge
        |
        v
Tokimu Resource Space public semantics
        |
        v
consumer-local Tosumu persistent provider
        |
        v
Tosumu public storage and inspection contracts
```

At no point may:

- C# parse Tosumu pages, WAL frames, protectors, or physical records;
- Tosumu define Tokimu store, root, folder, resource, visibility, or address
  meaning;
- Resource Space expose Tosumu keys, SQL tables, page IDs, or CLI DTOs;
- WPF controls or .NET framework objects cross into Tokimu-owned contracts;
- Tokimu core or runtime acquire a .NET, WPF, database, or Tosumu dependency.

## Trigger And Evidence

- `third-party/tosumu/dotnet/Tosumu.WpfHarness` is already an application-shaped
  Windows UI over Tosumu's machine-readable inspection commands.
- `third-party/tosumu/dotnet/Tosumu.Cli.IntegrationTests` correctly proves that
  the packaged Tosumu CLI and its JSON inspection envelopes work from .NET.
- Resource Space is provisionally admitted under ADR-0005 and still needs an
  independent persistent-provider consumer before AR-0009 can select permanent
  extraction, continued incubation, or retirement.
- Tosumu accepted Tokimu's provider-side change request and is available as a
  pinned submodule, making the provider boundary reproducible.
- The existing WPF harness already separates process invocation, workflow,
  presentation state, and WebView visualization well enough to migrate in
  bounded slices rather than through a rewrite.
- The historical WPF design assumes access to the maintainer-local
  `F:\LocalSource\ClassLibrary` repository. That assumption is not portable,
  reproducible, or an accepted dependency of either Tokimu or Tosumu and must
  not survive the migration.
- The support-library plan identifies .NET desktop hosting as a valuable
  consumer boundary, but does not yet justify a reusable framework adapter.

## Current State

### Tosumu-Owned .NET Surface

| Source | Current responsibility | Migration disposition |
| --- | --- | --- |
| `Tosumu.Cli` | Packages `tosumu.exe`; runs commands; deserializes Tosumu inspect envelopes | Keep in Tosumu |
| `Tosumu.Cli.IntegrationTests` | Verifies packaged CLI, round trips, typed inspect DTOs, and CLI failure translation | Keep in Tosumu |
| `Tosumu.WpfHarness` | Active Windows database-inspection application and workflow | Keep in Tosumu; it is not a duplicate Resource Space consumer |
| `WpfHarness.DESIGN.md` | Active Tosumu inspection-companion design, including historical local-library assumptions | Keep in Tosumu; update only through a Tosumu-owned modernization effort |
| PowerShell build/check scripts | Build Tosumu CLI/package/tests/harness together | Split by owner |
| `.artifacts` | Generated package, NuGet cache, and test output | Never migrate or commit |

### Tokimu Resource Space Surface

Resource Space currently owns provider-neutral logical semantics for:

- stable store, root, folder, and resource qualification;
- normalized logical addresses and per-store case policy;
- explicit visible and hidden resource state;
- deterministic folder navigation and bounded search;
- immutable source bytes and provider-neutral metadata observations;
- bounded mutation observations and diagnostics;
- deterministic `resource-space-provider-conformance-v1` evidence.

Resource Space does not own durability, transactions, encryption, recovery,
freshness, synchronization, host paths, Tosumu keys, or Tosumu inspection
semantics.

## Ownership Model

| Layer | Owns | Must not own |
| --- | --- | --- |
| Tokimu Resource Space | Logical identities, addresses, hierarchy, visibility, navigation, observations, diagnostics | Database pages, WAL, encryption, host UI, or physical persistence |
| Tosumu provider adapter | Durable mapping, transactions, reopen behavior, provider diagnostics, storage error translation | Resource meaning, WPF presentation, or kernel identity |
| Tosumu | Public storage, integrity, recovery, and inspect contracts | Tokimu resources, application workflows, or .NET UI semantics |
| Tokimu bridge | Bounded command/observation transport between .NET and Rust | Domain meaning not present in public Tokimu contracts |
| .NET workbench | Window lifecycle, interaction, view state, accessibility, and presentation | Tosumu parsing or alternate Resource Space semantics |
| Consumer corpus | Cross-project composition evidence and divergence reports | Stable support library before repeated host pressure exists |

## Migration Rules

### Keep Storage Validation With Tosumu

The following remain in `third-party/tosumu/dotnet`:

- packaging and copying `tosumu.exe` into a .NET package;
- process invocation and cancellation behavior for Tosumu commands;
- JSON inspection envelope compatibility;
- typed Tosumu inspect DTOs and error translation;
- direct Tosumu init/put/get/scan/verify integration tests;
- tests whose failure means Tosumu's .NET-facing contract is broken.

### Move Application Composition To Tokimu

The following move into the Tokimu consumer corpus:

- the WPF window, panes, navigation, command routing, and view state;
- application workflows that open, navigate, and inspect logical resources;
- host-side tests for Tokimu observations and commands;
- Resource Space provider-conformance comparison;
- cross-project diagnostics that distinguish Tokimu, provider, Tosumu, bridge,
  and host failures;
- screenshots and manual Windows evidence for the migrated application.

### Do Not Copy Historical Coupling Forward

The moved workbench must not continue to depend directly on `Tosumu.Cli` as its
primary semantic API. A temporary compatibility pane may expose provider-owned
Tosumu inspection evidence, but ordinary resource operations must pass through
the Tokimu consumer bridge and Resource Space contract.

### Remove `ClassLibrary` Dependencies

The migrated projects must not reference, discover, copy from, or require the
maintainer-local `F:\LocalSource\ClassLibrary` repository. This includes direct
project references, package references produced only by that repository,
absolute paths, source links, build-script discovery, and undocumented runtime
assumptions.

Every historical `ClassLibrary` abstraction must receive one explicit
disposition:

1. **Consumer-local host glue** -- implement the smallest required behavior in
   the consumer when it exists only to host this workbench.
2. **Tokimu-owned semantics** -- migrate the semantic contract into Tokimu only
   when corpus evidence and the accepted Tokimu ownership boundaries justify
   it. Do not move generic WPF or WebView mechanisms into `tokimu-core` or
   `tokimu-runtime`.
3. **Replaceable host/provider mechanism** -- use a maintained upstream package
   or create a separately owned provider/support project after repeated
   consumers prove that boundary. One migrated workbench is not enough evidence
   for a general support library.
4. **Unused or speculative functionality** -- remove or defer it rather than
   porting it preemptively.

Bulk source migration from `ClassLibrary` is prohibited. Any source that is
intentionally adapted must have clear provenance, an ownership reason, and
focused tests; copying a utility because it is convenient is not sufficient.
The default implementation choice is consumer-local code using the .NET base
class libraries and explicit upstream packages.

Known historical candidates begin with this disposition:

| Historical candidate | Initial disposition |
| --- | --- |
| `MemoryStore` | Replace its proven logical-resource semantics with Tokimu Resource Space. Do not port the historical implementation or preserve its name: in-memory retention is one provider, while Resource Space owns qualified identity, explicit folders, visibility, navigation, and public diagnostics. Graduation remains governed by `AR-0009`. |
| `CompressionTools` | Migrate proven byte-codec and archive behavior through the provider-neutral [Compression And Archive Providers](compression-and-archive-providers.md) plan. Resource Space replaces the historical `MemoryStore` integration; filesystem backup and extraction workflows remain explicit platform or consumer adapters. |
| `WebViewTools` | Replace with the minimum consumer-local WebView2 host behavior; reconsider a support provider only after another consumer repeats it |
| `WpfBlazorTools` | Defer unless the migrated workbench demonstrates a concrete Blazor requirement |
| `HelperClient.Wpf` | Treat as a historical pattern only; implement bounded startup and shutdown behavior locally if required |
| `MonacoTools.WebView` | Reject from the first migration; admit an editor provider only under separate consumer pressure |
| Generic helpers and data structures | Prefer BCL/upstream equivalents or focused consumer-local helpers; do not migrate the library wholesale |

`MemoryStore` is the first explicit semantic migration rather than an example
of source reuse. The replacement is intentionally narrower and safer:

```text
historical MemoryStore behavior
        ↓
Resource Space semantic contract
        ↓
in-memory or Tosumu-backed provider
```

The migration keeps the historical failure evidence--ambiguous roots, hidden
resource discovery, duplicate logical stores, and missing folder objects--but
does not preserve the old implementation's filesystem-shaped assumptions. A
new `ClassLibrary` concept may enter Tokimu only after it receives the same
explicit mapping: Tokimu capability, consumer-local host glue, independent
provider/support project, or rejection/deferment. No dependency is allowed to
remain merely because an older application once used it.

## Proposed Layout

```text
corpus/consumers/dotnet-tosumu-resource-workbench/
  DESIGN.md
  README.md
  src/
    Tokimu.ResourceWorkbench/          # WPF host application
    Tokimu.ResourceWorkbench.Tests/    # host and bridge contract tests
  engine/
    Cargo.toml
    src/                               # Tokimu bridge + consumer-local provider
  fixtures/
    selection-v1.toml
  evidence/
    README.md
  scripts/
    Invoke-Checks.ps1

third-party/tosumu/dotnet/
  Tosumu.Cli/
  Tosumu.Cli.IntegrationTests/
  Invoke-TosumuDotNetChecks.ps1
  Archive/
    WpfHarness.DESIGN.md
```

Exact project names may change during implementation. The ownership split may
not.

## Bridge Boundary

The first bridge should be boring, bounded, and transport-neutral. It may be an
out-of-process JSON-lines executable or another explicit native boundary that
can be tested without launching WPF. It must expose Tokimu-owned commands and
observations rather than Tosumu CLI arguments.

Initial command families should be equivalent to:

```text
session.create_or_open
folder.list
folder.create
resource.put
resource.get
resource.list
resource.set_visibility
resource.move
observation.summary
provider.inspect
```

`provider.inspect` is an explicitly separate provider-diagnostics path. It may
surface bounded Tosumu facts, but those facts cannot alter the meaning of the
Resource Space command results.

Every bridge response must identify:

- schema version;
- command identity;
- success or typed failure;
- Tokimu semantic payload when successful;
- bounded diagnostics with an owning layer;
- provider evidence only when explicitly requested.

## Persistent Mapping Study

The provider implementation is consumer-local until evidence justifies a
reusable adapter. It should document, rather than hide, mappings such as:

| Resource Space concept | Provisional Tosumu representation | Required invariant |
| --- | --- | --- |
| `StoreId` | Durable store metadata key | Survives close/reopen and process restart |
| root | Store-owned canonical record | Exactly one qualified root per store |
| `FolderId` | Durable folder record | Stable across rename and move |
| `ResourceKey` | Durable resource record | Distinct from content identity and address |
| resource bytes | Blob/value record | Exact immutable-byte round trip |
| visibility | Explicit metadata field | Never inferred from host hidden-file state |
| case policy | Immutable store metadata | Conflict on incompatible reopen |
| mutation observation | Tokimu-side bounded observation | Not synthesized from WAL or claimed as sync history |

The mapping must use Tosumu's public surface. If Tosumu lacks an operation, the
consumer records the gap or opens a Tosumu change request; it must not reach
through to physical pages or silently weaken the Tokimu contract.

## Evidence And Divergence Policy

The provider must produce the same public conformance artifact shape as
`resource-space-provider-conformance-v1` wherever semantics overlap.

Every difference must be classified as exactly one of:

1. **Provider-only durable behavior** -- useful Tosumu evidence that does not
   change Resource Space semantics.
2. **Capability-contract refinement** -- a missing provider-neutral semantic
   that requires AR-0009 to reopen before the public contract changes.
3. **Rejected semantics** -- behavior that is Tosumu-specific, host-specific,
   ambiguous, or contrary to the admitted Resource Space boundary.

No divergence may be normalized away solely to make the reports match.

## Implementation Slices

Each slice must compile and validate independently. The source WPF harness
remains runnable until the migrated replacement reaches feature parity for the
slice being removed.

### Slice 0: Freeze Ownership And Baselines

**Objective:** Record what the current .NET projects prove before moving code.

#### Deliverables

- [x] Inventory every project, script, test, fixture, and generated directory
      under `third-party/tosumu/dotnet`.
- [x] Classify every test as Tosumu contract, host application, cross-project
      composition, or generated output.
- [ ] Run and retain the existing Tosumu .NET check summary.
- [x] Record the WPF harness's currently working panes and commands.
- [x] Confirm that `.artifacts`, `bin`, `obj`, package caches, and copied
      executables are ignored rather than migrated.
- [x] Add a source-to-target migration ledger to the consumer `DESIGN.md`.
- [x] Inventory every `ClassLibrary`, `WebViewTools`, `WpfBlazorTools`,
      `HelperClient.Wpf`, and `MonacoTools.WebView` reference or assumption and
      assign one of the four dependency-removal dispositions.

#### Acceptance Criteria

- Every source item has a keep, move, archive, rewrite, or reject disposition.
- Tosumu's retained .NET tests still prove the same Tosumu-owned contracts.
- No generated artifact is mistaken for source.
- The migration can be rolled back to a known commit without reconstructing
  undocumented behavior.
- No retained or target project requires a local `ClassLibrary` checkout to
  restore, build, test, or run.

### Slice 1: Scaffold The Tokimu Consumer

**Objective:** Create a Tokimu-owned consumer shell without changing behavior.

#### Deliverables

- [x] Create `corpus/consumers/dotnet-tosumu-resource-workbench`.
- [x] Add `DESIGN.md`, `README.md`, deterministic fixture selection, and a
      single check script.
- [x] Move or reproduce the minimum WPF shell needed to launch one observation
      pane.
- [x] Implement required host plumbing locally or through explicit upstream
      packages without referencing `ClassLibrary` artifacts.
- [x] Make repository-root and submodule discovery explicit and diagnosable.
- [x] Add the bridge as a bounded Rust corpus member while keeping the WPF host
      outside the Rust workspace.

#### Acceptance Criteria

- The consumer builds from the Tokimu checkout with documented prerequisites.
- The shell launches without directly parsing or mutating Tosumu storage.
- Missing Tosumu, missing bridge, unsupported OS, and missing fixture failures
  are distinct diagnostics.
- Tosumu's original WPF harness remains intact during this slice.
- The new consumer has no project, package, source-link, absolute-path, or
  runtime dependency on `ClassLibrary`.

### 2026-08-04 -- Slice 0 And Slice 1 Scaffold

- **Completed:** Created the Tokimu-owned WPF consumer shell, migration ledger,
  fixture-selection placeholder, evidence boundary, and single host check
  script.
- **Observed source boundary:** The retained `Tosumu.WpfHarness` references
  `Tosumu.Cli` and WebView2 but has no active `ClassLibrary` project/package
  reference. Its historical design contains the local-repository assumptions,
  which are explicitly rejected by the new consumer.
- **Intentional limit:** The shell displays a bridge-pending state. It neither
  invokes Tosumu commands nor claims Resource Space or durable-provider
  behavior before Slice 2 and Slice 3 establish those contracts.

### Slice 2: Establish A Headless Tokimu Bridge

**Objective:** Make .NET consume a Tokimu semantic boundary before migrating
the full UI.

#### Deliverables

- [x] Implement a headless bridge over public Resource Space operations.
- [x] Version the command and observation envelope.
- [x] Add a deterministic in-memory provider mode independent of Tosumu.
- [x] Add .NET contract tests for create/open, folders, resources, visibility,
      navigation, and summaries.
- [x] Ensure process, cancellation, stderr, and malformed-response handling are
      tested outside WPF.

#### Acceptance Criteria

- .NET tests consume Tokimu-owned commands, not Tosumu CLI commands.
- The in-memory mode reproduces the existing Resource Space conformance facts.
- No WPF or Tosumu type appears in the bridge's semantic contract.
- Failures identify bridge, Tokimu command, or host transport ownership.

### 2026-08-04 -- Slice 2 Headless Bridge

- **Completed:** Added `tokimu-resource-workbench-bridge`, a versioned
  JSON-lines process boundary over public `resource-space` semantics. The
  bridge exposes deterministic in-memory session, folder, resource, visibility,
  navigation, summary, and provider-inspection operations.
- **Completed:** Added a dependency-free .NET bridge client and executable
  contract runner. It proves create/open, folder navigation, byte round trip,
  visibility filtering, summaries, and typed unknown-command rejection through
  the live process boundary.
- **Completed:** The consumer check script now validates forbidden historical
  dependencies, Resource Space plus bridge tests, bridge build, and the .NET
  contract runner. It also verifies malformed JSON, captured stderr, and
  caller cancellation outside WPF.
- **Observed limit:** `provider.inspect` truthfully reports only the in-memory
  provider. No Tosumu CLI, durable store, page, WAL, or provider-native DTO
  crosses the bridge in this slice.

### Slice 3: Add The Consumer-Local Tosumu Provider

**Objective:** Persist the same Tokimu semantics through Tosumu using only
public boundaries.

#### Deliverables

- [x] Implement create/open and stable `StoreId` persistence.
- [x] Persist root, folder, resource, visibility, metadata, and exact bytes.
- [x] Support close/reopen and a fresh-process observation pass.
- [x] Translate Tosumu failures into provider-owned diagnostics without losing
      structured Tosumu evidence.
- [x] Keep provider code consumer-local; do not extract a reusable crate yet.

#### Acceptance Criteria

- Logical IDs survive close/reopen and process restart where the contract
  requires stability.
- Duplicate store IDs and incompatible case-policy reopen attempts fail
  explicitly.
- Move preserves content and metadata at its new qualified address. A stable
  resource identity independent of address remains an AR-0009 open question.
- Hidden resources remain explicit and behave consistently under direct lookup
  and enumeration.
- No physical Tosumu type or key appears in Resource Space observations.

### 2026-08-04 -- Slice 3 Consumer-Local Tosumu Provider

- **Completed:** Added a versioned Resource Space snapshot mapping backed by
  Tosumu's public `KvStore` and transaction surface. The mapping persists the
  store descriptor and immutable case policy, qualified root, explicit folder
  hierarchy, resource metadata and visibility, exact source bytes, and the
  next folder identity.
- **Completed:** Added a fresh Rust bridge-instance test and a fresh .NET
  bridge-process test. Both create a Tosumu session, write hidden content,
  move it to a new qualified address, then reopen the same durable store and
  verify hierarchy, visibility, exact bytes, media metadata, and durable
  provider inspection.
- **Completed:** The Rust provider test also attempts a fresh reopen with an
  incompatible case policy and receives the explicit
  `provider.tosumu.identity_conflict` diagnostic.
- **Completed:** Provider failures are classified under `provider.tosumu.*`.
  Resource Space observations remain provider-neutral; host paths, Tosumu
  keys, physical records, WAL details, and CLI DTOs do not cross the bridge.
- **Observed limit:** This is a consumer-local snapshot mapping, not a general
  Resource Space provider crate or a `.tasset` schema. `ResourceKey` is
  address-qualified today, so move evidence proves retained content and
  metadata at the destination rather than a persistent resource identity. The
  provider has not yet been run against the full provider-conformance artifact
  or Tosumu interruption and corruption cases.

### 2026-08-04 -- Slice 4 Provider Conformance Profile

- **Completed:** The dependency-free .NET runner now emits the
  `resource-space-provider-conformance-v1` `provider-operation-fixture-v1`
  profile. It drives the same public folder, hidden-resource, move,
  visibility, and retrieval workflow through the in-memory provider and a
  Tosumu provider reopened by a fresh bridge process, then requires their
  observations to be structurally equal.
- **Completed:** The profile separates the comparable Resource Space result
  from durable-only evidence. Fresh-process reopen and bounded
  `provider.inspect` facts remain visibly Tosumu-specific and cannot change
  the shared comparison result.
- **Completed:** The profile deliberately avoids the GLTF, XML, JSON, and
  asset-loader observations in the broader `hello-resource-space` artifact.
  Those are adapter-consumer evidence, not requirements that a persistent
  Resource Space provider must reproduce.

### Slice 4: Run Provider Conformance And Classify Divergences

**Objective:** Produce the persistent-provider evidence required by AR-0009.

#### Deliverables

- [x] Add a provider-only conformance profile for the shared folder, hidden
      resource, move, visibility, and retrieval workflow; retain a fresh
      Tosumu bridge reopen as durable-only evidence.
- [x] Run the in-memory and Tosumu-backed providers against equivalent
      deterministic fixtures.
- [x] Emit comparable `resource-space-provider-conformance-v1` artifacts.
- [x] Add durable-only evidence for reopen and provider diagnostics without
      mislabeling it as base-contract behavior.
- [ ] Add durable-only evidence for transactions and integrity without
      mislabeling it as base-contract behavior.
- [x] Classify the observed provider comparison as no divergence: the shared
      semantic observations match structurally. Durable reopen and provider
      inspection are recorded as provider-only behavior.
- [ ] Add focused malformed, corruption, interrupted-write, and resource-limit
      tests where Tosumu can honestly provide evidence.

#### Acceptance Criteria

- Base semantic observations match or have an explicitly reviewed divergence.
- Durable evidence is visibly separate from provider-neutral conformance.
- No contract is widened merely to accommodate the first persistent provider.
- The report is reproducible without launching WPF.

### Slice 5: Migrate The WPF Workbench Workflows

**Objective:** Make the interactive application present Tokimu and provider
observations rather than directly orchestrating Tosumu commands.

#### Deliverables

- [x] Rewire bounded session selection through the Tokimu bridge for both
      in-memory and Tosumu-backed sessions.
- [x] Add current-folder navigation, resource enumeration, selected-resource
      metadata, and separate provider-inspection panes.
- [x] Add explicit file/open import workflows through the Tokimu bridge.
- [x] Add visibility mutation and filtering controls through the Tokimu bridge.
- [x] Serialize host workflows and JSON-lines bridge requests so a provider
      switch waits for in-flight work; discard refresh results produced by a
      bridge instance that has since been replaced.
- [x] Preserve a separate Tosumu provider-inspection pane where useful.
- [ ] Reuse existing coordinator and presenter patterns only when their
      responsibility remains host-owned.
- [x] Remove direct `Tosumu.Cli` dependencies from ordinary resource workflows.
- [x] Add bounded diagnostics and avoid unbounded UI expansion or log flooding.

#### Acceptance Criteria

- The main workflow visibly exercises Tokimu Resource Space semantics.
- Provider inspection is labeled as Tosumu evidence, not Tokimu resource
  meaning.
- The UI remains responsive under bounded diagnostic and navigation loads.
- Host state, bridge state, Tokimu observations, and provider diagnostics can
  be distinguished during failure.

### 2026-08-04 -- Slice 5 Initial Host Workflow

- **Completed:** Replaced the bridge-pending WPF placeholder with a bounded
  in-memory Resource Space workflow. The host can open a session, create a
  folder, add a visible sample resource, and refresh the resulting summary,
  folder, and resource observations through the JSON-lines bridge.
- **Completed:** Added a separate provider-inspection pane. It renders only
  the bounded `provider.inspect` result and labels it as provider-owned
  evidence rather than Resource Space meaning.
- **Completed:** The WPF project references only the consumer-local bridge
  project. It has no Tosumu CLI, `ClassLibrary`, or local-path dependency.
- **Completed:** The WPF host can now explicitly select an in-memory or
  consumer-local Tosumu-backed session through the same versioned bridge. It
  chooses a durable location under the host's local application-data directory
  and presents that path only as host configuration, never as Resource Space or
  provider inspection meaning.
- **Completed:** Session replacement, mutation, navigation, and refresh now
  share one host workflow gate. The bridge serializes each JSON-lines
  request/response exchange, and refresh verifies that its producing bridge is
  still current before updating WPF state. This prevents a session toggle from
  disposing a bridge underneath an active request or applying old-session
  observations to the new session.
- **Completed:** The host now browses current-folder observations, navigates to
  a selected child folder or the root, and presents selected resource metadata.
  Each view comes directly from bridge commands; the WPF layer does not derive
  hierarchy, visibility, qualified addresses, or metadata behavior.
- **Completed:** Added host-owned file selection that reads a chosen file and
  submits its bytes, filename, and conservative media type through
  `resource.put`. The host never writes a Tosumu record directly.
- **Completed:** Added an explicit resource visibility filter and a
  selected-resource visibility action. Both use `resource.list` and
  `resource.set_visibility`; the UI does not infer Resource Space visibility
  behavior.
- **Completed:** Converted the action row to a wrapping layout and kept the
  body inside a scroll viewer so additional controls and diagnostics do not
  silently make the window unusable.
- **Remaining:** Capture manual Windows interaction evidence under both
  provider modes, including imported-file persistence after a fresh bridge
  reopen and a hidden-resource visibility transition.

### Slice 6: Move Resource Space Tests And Remove Duplicate Claims

**Objective:** Complete Resource Space consumer ownership without weakening
Tosumu's independent database-inspection surface.

#### Deliverables

- [x] Place the Resource Space host application and cross-project contract
      tests in the Tokimu consumer.
- [x] Keep CLI/package/inspect compatibility tests in Tosumu.
- [x] Record that `Tosumu.WpfHarness` is an independent active inspection
      companion, not duplicate source to retire.
- [x] Keep `WpfHarness.DESIGN.md` as Tosumu-owned active design documentation;
      it must not be archived as a migration artifact.
- [x] Split check scripts so each repository validates only the projects it
      owns.
- [x] Remove stale Resource Workbench package references, paths, and
      documentation links from the Tokimu consumer.
- [ ] Address the active Tosumu WPF harness's historical `ClassLibrary`
      strategy in a separate Tosumu-owned modernization plan; it is outside
      this Resource Space consumer migration.

#### Acceptance Criteria

- Tosumu CI validates Tosumu's .NET package and inspection contract without
  requiring the Tokimu checkout.
- Tokimu CI or documented host checks validate the consumer using the pinned
  Tosumu submodule.
- No Resource Space application source is duplicated across repositories.
- Both WPF applications have explicit, non-overlapping ownership and purpose.
- The Tokimu consumer contains no `ClassLibrary` dependency or local-checkout
  discovery path. The active Tosumu harness remains separately tracked until
  its own modernization plan resolves its historical assumptions.

#### 2026-08-04 Scope Correction

The source inventory originally treated `Tosumu.WpfHarness` as a candidate for
migration. Current evidence shows that it is an active Tosumu database
inspection companion with a different semantic subject: Tosumu pages,
verification, keyslots, and storage diagnostics. The Tokimu workbench instead
owns Resource Space composition over a bounded bridge.

These applications may both be WPF hosts, but they are not duplicate source or
replacement candidates. This plan therefore transfers only Resource Space
consumer tests and host composition into Tokimu. Modernizing the Tosumu harness
or removing its historical support-library assumptions requires a separate
Tosumu-owned plan.

### Slice 7: Cross-Platform Host Decision

**Objective:** Decide whether WPF remains sufficient evidence or whether an
Avalonia consumer is justified.

#### Deliverables

- [x] Record Windows-only behavior and host-independent bridge behavior.
- [ ] Run the headless bridge and provider tests on every supported native
      platform available in CI.
- [x] Compare repeated WPF glue with the support-library admission rules.
- [x] Defer an Avalonia migration or companion: one WPF consumer does not yet
      provide independent cross-platform evidence or justify a support library.

#### Acceptance Criteria

- Headless semantic evidence is not blocked on WPF or Windows.
- No `support/dotnet` or `support/avalonia` package is admitted from one app.
- The outcome is explicit: retain WPF, migrate to Avalonia, add a companion,
  or defer cross-platform UI.

#### 2026-08-04 Cross-Platform Boundary

The WPF shell is explicitly Windows-only. The Resource Space contract, the
Tosumu snapshot provider, and their focused reopen test live in the Rust
bridge, which is independently exercised by `cargo test` without launching
WPF. The .NET contract runner currently also contains Windows batch-based
transport-failure fixtures, so it is retained as Windows host evidence rather
than mislabeled as portable.

No repeated host mechanism has justified `support/dotnet`, `support/avalonia`,
or an Avalonia companion. The current decision is to retain WPF as the single
desktop host and defer a second UI framework. CI execution of the headless
bridge on every supported native platform remains open evidence.

### Slice 8: Architectural Review And Graduation Decision

**Objective:** Feed evidence back into Resource Space and Tosumu-backed asset
decisions.

#### Deliverables

- [x] Add the persistent-provider findings to AR-0009.
- [ ] Update AR-0011 with any evidence relevant to Tosumu-backed `.tasset`
      output without treating this workbench as asset-format proof.
- [ ] Select permanent Resource Space extraction, continued incubation, or
      retirement.
- [ ] Decide whether the Tosumu adapter remains consumer-local or has enough
      repeated use for a separate provider crate.
- [ ] Update the Resource Space plan, consumer index, and support-library plan.

#### Acceptance Criteria

- The review records accepted, provider-only, rejected, and deferred findings.
- No database dependency enters `tokimu-core` or `tokimu-runtime`.
- Resource Space graduation is based on independent provider evidence rather
  than the fact that a migration completed.
- Any reusable adapter has at least one additional real consumer or an
  explicitly documented ADR-0005 evidence substitution.

## Validation Matrix

| Boundary | Required validation |
| --- | --- |
| Tosumu .NET package | Existing packaged executable and inspect envelope tests |
| Tokimu bridge | Headless command/observation contract tests |
| In-memory Resource Space | Existing conformance artifact and focused tests |
| Tosumu provider | Equivalent conformance plus durable-only evidence |
| .NET host | Build, launch smoke test, cancellation/error tests, manual UI evidence |
| Cross-process identity | Create, close, restart, reopen, compare observations |
| Failure ownership | Inject host, bridge, Tokimu, provider, and Tosumu failures separately |
| Documentation | Local-link validation and synchronized source/target indexes |
| Dependency independence | Clean restore/build/run with no `ClassLibrary` checkout or locally produced package |

Expected commands will include repository-appropriate variants of:

```powershell
cargo test --workspace
dotnet test third-party/tosumu/dotnet/Tosumu.Cli.IntegrationTests
pwsh corpus/consumers/dotnet-tosumu-resource-workbench/scripts/Invoke-Checks.ps1
```

The final script must not depend on a developer's global package cache, local
Tosumu checkout, or unrecorded environment variable.

The consumer check script must also reject active project or script references
to `F:\LocalSource\ClassLibrary` and the retired local-only helper packages.

## Risks And Mitigations

### Risk: Moving Tosumu's Own Tests By Accident

Mitigation: classify every test before moving it. If a failure means Tosumu's
package or inspection contract broke, the test stays with Tosumu.

### Risk: A C# Resource Space Reimplementation

Mitigation: C# consumes versioned Tokimu observations and commands. It owns no
alternate address normalization, visibility, folder, or identity model.

### Risk: Tosumu Becomes The Resource Space Contract

Mitigation: require the in-memory bridge mode and compare provider-neutral
artifacts before Tosumu is introduced.

### Risk: Process Bridge Becomes A Permanent Accidental Protocol

Mitigation: version and bound it, but describe it as a consumer bridge until a
second .NET or native host demonstrates reusable demand.

### Risk: WPF Blocks Cross-Platform Evidence

Mitigation: keep conformance, provider, and bridge tests headless. Treat WPF as
one host presentation mechanism, not the semantic test runner.

### Risk: Secret Or Unlock Material Leaks Through Diagnostics

Mitigation: retain Tosumu's explicit secret-input paths, redact bridge logs,
never place passphrases or recovery material in command arguments or artifacts,
and test diagnostic redaction.

### Risk: Two Repositories Become Impossible To Validate Independently

Mitigation: Tosumu tests remain self-contained. Tokimu pins Tosumu as a
submodule and owns the cross-project checks. Neither repository relies on an
uncommitted sibling checkout.

### Risk: `ClassLibrary` Is Recreated Inside Tokimu

Mitigation: migrate only abstractions justified by an accepted Tokimu semantic
boundary. Keep one-off WPF/WebView glue consumer-local, prefer focused upstream
packages for mechanisms, and require repeated consumer evidence before creating
a reusable support/provider project.

## Non-Goals

- Moving Tosumu's CLI package or inspect DTO compatibility tests into Tokimu.
- Making Tokimu the owner of Tosumu's storage, encryption, WAL, recovery, or
  inspection semantics.
- Promoting Resource Space directly into `tokimu-core`.
- Stabilizing a general .NET, WPF, Avalonia, FFI, or JSON-RPC SDK.
- Retaining a dependency on `F:\LocalSource\ClassLibrary` or its locally built
  packages.
- Porting `ClassLibrary` wholesale into Tokimu, Tosumu, or the consumer corpus.
- Replacing Tosumu's TUI or command-line product roadmap.
- Making WPF cross-platform.
- Defining `.tasset` schema or publishing semantics in this consumer.
- Inferring synchronization events from Tosumu WAL records.
- Claiming Tosumu durability, security, or freshness beyond Tosumu's own
  documented and tested guarantees.

## Open Questions

- Should the first bridge be an out-of-process executable, a C ABI, or a
  WebView/WASM boundary? The out-of-process path is preferred until evidence
  shows it prevents a required capability.
- Does Tosumu currently expose the atomic operations needed to persist a full
  Resource Space mutation without partial logical state?
- Which Tosumu public schema surface should store the provider mapping while
  remaining independent from future `.tasset` schemas?
- Should provider inspection remain in the workbench after ordinary Resource
  Space workflows are complete, or become a separate diagnostic tool?
- Is WPF still useful after semantic migration, or does Avalonia add enough
  independent platform pressure to justify replacing it?
- Does a second consumer need the Tosumu provider before it can graduate from
  consumer-local code?
- After `MemoryStore` has completed its Resource Space migration, does any
  remaining `ClassLibrary` concept have two independent consumers and a
  provider-neutral semantic boundary, or should it remain consumer-local or
  be rejected?

## Completion Criteria

This migration is complete when:

- Tosumu retains all tests needed to validate its standalone .NET-facing
  contracts;
- the application-shaped workbench lives under Tokimu's consumer corpus;
- ordinary workbench operations consume Tokimu Resource Space semantics;
- Tosumu supplies persistent behavior only through a bounded provider adapter;
- equivalent in-memory and Tosumu-backed conformance evidence is retained;
- every divergence has an explicit classification;
- AR-0009 records the independent persistent-provider result;
- duplicated WPF source and stale cross-repository paths are removed;
- no active project, script, or runtime path depends on `ClassLibrary` or an
  artifact available only from that repository;
- both repositories build and test independently from clean checkouts.

## Pause Point

Slices 0 through 4 form the first useful stopping point. At that point the
headless .NET consumer and Tosumu provider should have produced the evidence
needed by AR-0009 even if the full WPF workflow has not yet moved. UI migration
must not block the architectural result.
