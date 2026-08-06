# Tokimu .NET Tosumu Resource Workbench

This Tier 3 consumer corpus entry will test a .NET desktop host consuming
Tokimu Resource Space semantics through a bounded bridge, with Tosumu acting
only as a persistent provider.

The bridge exposes both a deterministic in-memory provider and a
consumer-local Tosumu provider. Neither path parses Tosumu CLI output or
exposes Tosumu pages, WAL records, keys, protectors, or provider-native values
to the .NET host.

## Prerequisites

- Windows;
- .NET 10 SDK with WPF support;
- a Tokimu checkout containing the pinned `third-party/tosumu` submodule.

## Run

```powershell
pwsh -NoProfile -File .\scripts\Invoke-Checks.ps1
dotnet run --project .\src\Tokimu.ResourceWorkbench\Tokimu.ResourceWorkbench.csproj
```

## Current Evidence

- The WPF host is consumer-local and has no `ClassLibrary`, Tosumu CLI, or
  local absolute-path dependency.
- The versioned JSON-lines bridge supports session, folder, resource,
  visibility, navigation, summary, provider inspection, and bounded archive
  inspection/import commands.
- Dependency-free .NET contract checks exercise the live bridge process,
  malformed responses, stderr capture, caller cancellation, and a fresh
  Tosumu-backed bridge process outside WPF.
- The Tosumu provider persists a versioned Resource Space snapshot through
  Tosumu's public key-value and transaction API. It preserves logical IDs,
  hierarchy, exact bytes, metadata, and explicit visibility across reopen.
- The headless runner emits
  `target/resource-space-conformance/dotnet-tosumu-resource-workbench/provider-conformance-v1.json`.
  It compares the same public folder, hidden-resource, move, visibility, and
  retrieval workflow through in-memory and fresh-process Tosumu providers as
  the `resource-space-provider-conformance-v1`
  `provider-operation-fixture-v1` profile. Fresh-process reopen and provider
  inspection remain separately labeled durable-only evidence.
- The bridge also exposes `resource.transform_compression`, which transforms
  one retained resource into an explicitly named destination resource through
  the provider-neutral Resource Space compression facade. The headless runner
  emits
  `target/resource-space-conformance/dotnet-tosumu-resource-workbench/compression-provider-conformance-v1.json`.
  That artifact compares an explicit GZip encode/decode round trip through the
  in-memory session and a fresh-process Tosumu-backed session, including
  public observations, retained source bytes, and restored bytes.
- This compression comparison is durable host-composition evidence only. The
  Tosumu session persists and restores a consumer-local
  `InMemoryResourceSpace` snapshot; it is not an independent persistent
  implementation of the Resource Space contract.
- The bridge also exposes `resource.inspect_archive` and
  `resource.import_archive_subtree`: explicit bounded ZIP, TAR, or 7z
  operations over one retained source resource. The headless runner emits
  `target/resource-space-conformance/dotnet-tosumu-resource-workbench/archive-provider-conformance-v1.json`.
  It retains an opaque ZIP fixture, then compares its source metadata,
  provider-neutral manifest, explicit imported folder tree, retained leaf
  metadata, and exact imported bytes after an in-memory session and a
  fresh-process Tosumu-backed reopen. The .NET host never parses archive bytes
  or receives an archive-library DTO.
- This archive comparison is likewise durable host-composition evidence only;
  it does not establish Tosumu as an independent persistent implementation of
  either Resource Space or the archive bridge contract.
- No source asset, database page, WAL record, protector, or provider-native
  Tosumu value crosses into the host.
- The WPF host now exercises the in-memory bridge workflow directly: it opens
  a session, creates a folder, writes a sample resource, and displays Tokimu
  observations separately from bounded provider inspection. Durable Tosumu
  selection remains the next interactive slice.
- WPF is intentionally Windows-only. The Rust bridge and its focused
  in-memory/Tosumu reopen tests remain headless evidence; the .NET contract
  runner is Windows-host evidence because its transport-failure fixtures use
  Windows batch commands. No Avalonia support layer is admitted from this one
  consumer.

The remaining provider work is transaction, integrity, interruption, and
resource-limit evidence where Tosumu can expose bounded public observations.
The bridge, not this WPF application, owns resource commands and observations.
