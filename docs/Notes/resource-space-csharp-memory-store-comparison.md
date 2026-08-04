# C# MemoryStore And Resource Space Comparison

## Status

Source-level comparison completed on 2026-08-03. This records what the
existing C# `MemoryStore` actually does and how the incubating Rust Resource
Space intentionally differs. It is not yet a runnable cross-language
compatibility suite.

## Evidence Source

- C# implementation: `F:\LocalSource\ClassLibrary\MemoryStore\InMemoryResourceStore.cs`
- C# test project: `F:\LocalSource\ClassLibrary\MemoryStore.Tests`
- Source test count: 83 `[Fact]` or `[Theory]` cases at review time.
- Rust implementation: `corpus/lib/resource-space`
- Rust consumers: `corpus/hello-resource-space` and the ASP.NET/WASM Asset
  Workbench engine session.

The C# suite was executed on 2026-08-03 with
`dotnet test MemoryStore.Tests.csproj --no-restore`: all 83 tests passed. The
executed public behaviors include case-insensitive URI lookup and overwrite,
XML/JSON/text helpers, copy/move/search, directory import/export, and an
`ImportDirectory_RoundTrip_PreservesStructure` workflow. This is runnable
source-consumer evidence, not a promise that Resource Space must reproduce the
C# API or its case/overwrite policy.

## Shared Workflow

Both systems support the useful document-processing workflow:

```text
select or create a logical root
        |
        v
add named source bytes
        |
        v
resolve related resources by logical address
        |
        v
inspect, decode, or export through an adapter
```

The shared workflow is evidence for a provider-neutral resource capability.
It is not evidence that the two public APIs should be mechanically compatible.

## C# Source Semantics

`InMemoryResourceStore` is a case-insensitive `ConcurrentDictionary<Uri,
byte[]>` behind a `mem:/` base URI. It provides byte, stream, text, JSON, XML,
hashing, copy/move, import/export, wildcard search, and XML resolver
conveniences.

The source establishes these important facts:

- `AddBytes` assigns through the dictionary indexer. The same normalized URI
  silently replaces earlier content.
- `CaseInsensitiveUriComparer` treats casing variants of an absolute URI as
  one key. Its test suite explicitly expects `file.txt` followed by
  `FILE.TXT` to overwrite the first value.
- `NormalizeToUri` accepts either an absolute URI or a relative path resolved
  beneath `BaseUri`.
- `GetDirectories` derives directory names from resource-address prefixes. It
  cannot represent an empty directory because only byte entries are stored.
- `GetFiles` and wildcard search are inferred from the same byte-key set and
  use case-insensitive matching.
- The store exposes a human-readable `BaseUri`, but no stable store identifier
  or create-versus-open registry policy.

These were productive C# conveniences for XML/XSLT and document pipelines.
They also explain the reported failures around ambiguous roots, accidental
same-name stores, hidden entries, and folder navigation.

## Deliberate Rust Replacements

| Concern | C# MemoryStore | Resource Space | Decision |
| --- | --- | --- | --- |
| Store identity | Instance plus display-like `BaseUri`; duplicate logical stores can coexist. | Stable caller-supplied `StoreId` with create/open conflict semantics. | Replace: store identity is explicit, not inferred from a name. |
| Root identity | URI base participates in resolution. | Stable `ResourceRootId`, qualified in every key. | Replace: path text cannot silently select a different root. |
| Resource key | Case-insensitive URI dictionary key. | `ResourceKey` qualified by store, root, and normalized address. | Adapt: URI syntax is provider input, not base identity. |
| Case policy | Always case-insensitive. | Explicit selected case policy per space. | Adapt: portability is visible rather than implicit. |
| Insert collision | Dictionary assignment silently overwrites. | Structured insert/replace/change behavior. | Replace: collision and replacement are separate intent. |
| Folders | Derived from occupied resource prefixes only. | Explicit root and folder nodes, including empty and hidden folders. | Replace: navigation is a first-class contract. |
| Visibility | No portable visibility fact; behavior depends on consumers/import. | Explicit `ResourceVisibility` and visibility-filtered queries. | Replace: hidden entries remain directly addressable. |
| Bytes | Mutable `byte[]` retained by a concurrent dictionary. | Immutable shared bytes with bounded provider retention. | Adapt: callers observe content without sharing mutable provider state. |
| XML/JSON/text | Methods on the store itself. | Replaceable adapters above base resource semantics. | Defer format behavior to format-owned bridges. |
| Import/export | Part of the in-memory store API. | Native and browser adapters outside the base contract. | Preserve logical addresses; do not leak host mechanisms. |

## C# Workflow Chosen For Continued Evidence

The first C# workflow to preserve as executable evidence is the document
bundle pattern:

```text
mem:/project/
    data.xml
    transform.xsl
    common/utilities.xsl
    assets/logo.png
```

The corresponding Resource Space scenario must prove:

1. one explicit store and root contain the four source resources;
2. `common/` and `assets/` exist as navigable folders even before a consumer
   derives paths from entries;
3. direct lookup, folder listing, and same-folder XML/XSLT resolution agree;
4. visible and hidden resources have explicit, consistent behavior;
5. a second store with the same display name does not become identity-equal;
6. an insert collision requires explicit replacement intent.

This is deliberately a semantic test matrix, not an API translation layer.
The Rust XML bridge already covers bounded related-resource lookup.

## Rust Fixture Evidence

`resource-space` now has the first-party
`document_bundle_preserves_explicit_navigation_and_replacement_intent`
regression fixture. It creates the bundle above under one explicit root,
retains `common/`, `assets/`, and an empty `drafts/` folder as navigable state,
checks deterministic folder and resource listings, resolves the related XSLT
resource directly, rejects an accidental `data.xml` insert collision, and then
requires `replace_resource` for the intentional update.

Related base-contract tests cover the remaining matrix dimensions:

- explicit visible/hidden navigation;
- stable store identity despite duplicate display names;
- root qualification and immutable root folders; and
- case-policy and address normalization behavior.

This proves the Rust side of the semantic matrix. It does not turn the C# URI,
stream, async, or XML convenience API into a compatibility requirement.

`hello-resource-space` is the runnable consumer report for that fixture. Its
current observation states that the explicit bundle has two root documents,
five visible folders, `common/utilities.xsl` at a qualified logical address,
and an explicitly retained empty `document-drafts/` folder. Its mutation
report also demonstrates that observations are a bounded diagnostic window
(`16` retained observations, sequences `4..=19`), not an unbounded event log.

## Remaining Evidence

- Capture a purpose-built C# document-bundle report if exact per-step output
  becomes necessary. The existing passing suite is sufficient source-consumer
  behavior evidence, but it does not serialize one shared report format.
- Record intentional differences, especially collision behavior and case
  policy, as expected divergence rather than regressions.
- Do not promise URI, `Stream`, async, filesystem, or XML convenience API
  compatibility from the base Resource Space contract.

## Finding

The C# implementation and the Rust Resource Space converge on a logical,
provider-neutral source-byte boundary. They do not converge on identity,
hierarchy, or replacement semantics, and they should not be forced to do so.
The Rust model preserves the useful workflow while replacing the ambiguity that
generated the original operational failures.
