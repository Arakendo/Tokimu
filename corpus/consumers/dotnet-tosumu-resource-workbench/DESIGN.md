# Tokimu .NET Tosumu Resource Workbench

## Purpose

This consumer corpus validates the composition boundary between a .NET desktop
host, Tokimu Resource Space semantics, and a consumer-local Tosumu persistent
provider.

```text
WPF interaction
    |
    v
versioned Tokimu bridge
    |
    v
Tokimu Resource Space semantics
    |
    v
consumer-local Tosumu provider
```

The WPF host owns window lifecycle and presentation only. It must never parse
Tosumu storage, WAL frames, protectors, or CLI inspection payloads as the
ordinary resource workflow.

## Slice 0 Migration Ledger

| Source item | Disposition | Reason |
| --- | --- | --- |
| `Tosumu.Cli` | Keep in Tosumu | Packages and invokes the standalone Tosumu CLI. |
| `Tosumu.Cli.IntegrationTests` | Keep in Tosumu | Proves CLI packaging and inspect-envelope compatibility. |
| `Tosumu.WpfHarness` application shell | Keep in Tosumu | It is an active Tosumu database-inspection companion, not the source of this Resource Space workbench. |
| Harness coordinators and presenters | Keep in Tosumu | They serve Tosumu inspection workflows; this consumer owns only its Resource Space host composition. |
| WebView2 host mechanism | Deferred | The first Resource Space shell does not need an embedded web surface. |
| `WpfHarness.DESIGN.md` | Keep in Tosumu | It remains the active design record for the Tosumu inspection companion. |
| `dotnet/.artifacts`, `bin`, `obj`, NuGet caches | Reject | Generated output is not source or corpus evidence. |

## Historical Dependency Dispositions

| Historical dependency | Disposition | Current action |
| --- | --- | --- |
| `F:\LocalSource\ClassLibrary` | Reject | No target project, script, or runtime path references it. |
| `WebViewTools` | Consumer-local or upstream mechanism | Deferred until a real web-view requirement returns. |
| `WpfBlazorTools` | Defer | No Blazor requirement in this workbench slice. |
| `HelperClient.Wpf` | Consumer-local host behavior | Do not copy it; add only bounded startup/shutdown behavior if needed. |
| `MonacoTools.WebView` | Reject | An editor is outside this Resource Space proof. |

## Current Host State

The WPF host can explicitly open either an in-memory session or a consumer-local
Tosumu-backed session through the same bounded bridge. Its durable location is
chosen and displayed by the host, but never copied into Resource Space
observations or provider inspection facts. The headless bridge remains
independently verified; WPF must not be replaced with direct Tosumu command
orchestration.

The current host workflow also navigates folders and selects resources using
only `folder.list` and `resource.list` observations. It displays the selected
resource's returned metadata separately from provider inspection; C# does not
derive qualified addresses, visibility behavior, or metadata semantics.

Host file selection is a presentation mechanism only: the host reads the
selected bytes and asks `resource.put` to create the resource in the current
folder. Visibility filtering and selected-resource mutation likewise travel
through the bridge; the host never reimplements resource lookup or visibility
policy.

## Headless Bridge Contract

The first bridge is a versioned JSON-lines executable under `engine/`. Its
commands are Tokimu-owned semantic operations:

```text
session.create_or_open
folder.list / folder.create
resource.put / resource.get / resource.list / resource.move
resource.set_visibility
observation.summary
provider.inspect
```

IDs cross the JSON boundary as decimal strings to avoid cross-language numeric
precision loss. The bridge has no Tosumu CLI, page, WAL, protector, or storage
DTO dependency. `Tokimu.ResourceWorkbench.ContractTests` validates the live
process separately from WPF, including malformed output, stderr, and caller
cancellation.

## Durable Provider Evidence

The bridge has a consumer-local `tosumu` provider mode. It stores a versioned
Resource Space snapshot through Tosumu's public key-value and transaction
surface, then reconstructs Resource Space only through its public APIs on
reopen. The .NET contract runner starts a second bridge process to confirm
that hidden navigation and exact bytes survive the host boundary.

This proves a persistent provider without admitting a reusable adapter or
claiming that the snapshot is a `.tasset` schema. Provider inspection reports
durability without disclosing host paths or physical Tosumu details.

## Next Evidence

The next slice compares the durable provider with the in-memory Resource Space
conformance facts and classifies every durable-only divergence. WPF adopts
ordinary resource workflows only after that evidence exists.
