# CompressionTools Migration Ledger

## Status

Active migration evidence for
[`compression-and-archive-providers.md`](../Plans/Standalone/compression-and-archive-providers.md).
This ledger is descriptive, not an accepted public API or provider choice.

## Pinned Source

| Field | Evidence |
| --- | --- |
| Repository | `https://git.arakendo.com/arakendo/ClassLibrary.git` |
| Revision | `a794476ce48333e13bd54fc7b2c5fdea17b76ad8` |
| Source subtree | `CompressionTools/` |
| Test subtree | `CompressionTools.Tests/` |
| License | MIT, copyright 2025 Arakendo |
| Historical provider | SharpZipLib 1.4.2 |
| Historical resource dependency | `MemoryStore` project reference |

The source inventory contains five hand-authored production files:

- `ArchiveManager.cs`
- `ArchiveNamingScheme.cs`
- `Compressor.cs`
- `FileArchiveManager.cs`
- `FileCompressor.cs`

The test inventory contains 96 xUnit facts:

- `CompressionToolsTests.cs`: 61 facts;
- `FileIOTests.cs`: 35 facts.

Generated `bin/` and `obj/` content is not source evidence.

## Migration Rule

The Rust work preserves useful behavior, not the C# class layout. Static helper
classes, exceptions, callbacks, host paths, and `MemoryStore` objects do not
cross the migration boundary. Every behavior below has an explicit disposition.

## Behavior Disposition

| Historical behavior | Disposition | Tokimu destination or reason |
| --- | --- | --- |
| GZip byte encode/decode | Port | Bounded `CompressionProvider` implementation |
| Brotli byte encode/decode | Port | Bounded `CompressionProvider` implementation |
| Deflate byte encode/decode | Port | Bounded provider with explicit raw/wrapped semantics |
| Compression levels | Adapt | Portable `Fast`, `Balanced`, and `Small` goals; native levels remain provider extensions |
| String encode/decode helpers | Replace | Callers explicitly choose character encoding around the byte contract |
| Compression ratio and size statistics | Adapt | Provider-neutral result observations |
| GZip signature detection | Port | Advisory self-identifying envelope detection |
| Raw Brotli/Deflate detection | Reject | Arbitrary raw payloads are not reliably self-identifying |
| ZIP signature detection | Adapt | Archive-format detection belongs to the archive boundary |
| ZIP create/list/read | Port | Bounded archive-provider contract |
| ZIP add/remove/update | Defer | Requires update, atomicity, and deterministic rewrite evidence |
| ZIP integrity verification | Adapt | Structured CRC/container diagnostics, not a Boolean helper |
| ZIP comparison | Defer | Corpus or diagnostics consumer concern until a second consumer appears |
| TAR create/list/read | Port later | Archive provider after ZIP semantics stabilize |
| TAR.GZ | Port later | Explicit composition of TAR archive and GZip codec providers |
| Entry callbacks | Replace | Bounded manifests, selected-entry reads, or iterators |
| `MemoryStore` compression helpers | Replace | Resource Space transformation bridge using public semantics |
| `MemoryStore` archive helpers | Replace | Explicit Resource Space archive facade and bridge |
| Flattened extraction | Defer | Potentially destructive naming policy requiring consumer evidence |
| Filesystem read/write wrappers | Adapt | Explicit native platform or consumer adapters |
| Directory archive workflows | Adapt | Consumer orchestration over platform, archive, and Resource Space boundaries |
| In-place compression and backup files | Reject from base | Application workflow; never implicit provider behavior |
| Backup directory helpers | Reject from base | Application-owned policy outside codec/archive contracts |
| Batch file compression | Adapt | Consumer orchestration; byte provider remains single-request focused |
| File auto-detection by extension | Reject | Bounded magic inspection only; names are advisory |
| Multi-volume split/assembly | Defer | Slice 9 requires a real consumer and malformed-set corpus |
| Standard ZIP volume naming | Defer | Naming is not archive correctness |
| RAR-style and 7z-style naming | Reject as support claim | Historical naming helpers do not establish RAR or 7z semantics |
| Archive volume discovery | Defer | Platform/resource enumeration policy requires evidence |
| Encryption or password handling | Reject | Not established by source; separate security review if required |

## Ownership Split

```text
byte compression
    codec, bounded transformation, observations

archive container
    entries, manifests, integrity, safe names

Resource Space bridge
    logical identity, folders, explicit mutations

platform adapter
    files, directories, atomic replacement, permissions

application workflow
    backups, recursion, batch policy, user choices
```

No lower layer silently absorbs the decisions of a higher layer.

## Rust Provider Candidates

Provider selection remains open. The following candidates justify focused
compatibility investigation; they do not define Tokimu's portable contract.

| Need | Candidate evidence | Work still required |
| --- | --- | --- |
| GZip and Deflate | `flate2` 1.1.9, MIT OR Apache-2.0, default pure-Rust backend | Selected for incubation; native tests and WASM compilation pass |
| Brotli | `brotli` 8.0.4, BSD-3-Clause AND MIT, safe-Rust streaming API | Selected for incubation; native tests and WASM compilation pass; decoder does not distinguish truncation from malformed data |
| ZIP | `zip` 8.6.0, MIT, defaults disabled with `deflate` only | Selected for bounded read/write incubation; deterministic writing, native tests, and WASM compilation pass; browser runtime evidence remains open |
| TAR | Rust `tar` ecosystem | Verify regular-file subset, extended-header policy, license, native, and WASM behavior |

No concrete provider should be admitted until its exact revision/version,
license, enabled features, and target compatibility are recorded.

## Test Migration

The 96 historical facts are evidence categories rather than a required
one-to-one Rust test count. Migration proceeds by boundary:

- codec round trips become provider conformance cases;
- malformed/truncated/high-expansion inputs become bounded decoder fixtures;
- archive entry operations become manifest and selected-entry tests;
- unsafe names and duplicate normalized names become adversarial fixtures;
- Resource Space cases use the public bridge against in-memory and durable
  providers;
- filesystem and backup tests remain consumer/platform evidence;
- multi-volume tests remain deferred and visibly inventoried.

First-party fixture bytes must be generated or checked in with provenance.
No historical binary output is copied implicitly from `bin/` or `obj/`.

## Current Implementation Evidence

`corpus/lib/compression-provider` now establishes an incubating contract for:

- `Gzip`, `Brotli`, and `Deflate` codec identity;
- separate encode and decode requests;
- semantic encode goals;
- input, output, and expansion limits;
- structured observations and failure categories;
- advisory GZip envelope detection with raw Brotli/Deflate reported as unknown;
- provider-independent contract tests with no filesystem or Resource Space
  dependency;
- a `flate2` 1.1.9 provider for GZip and raw Deflate;
- a `brotli` 8.0.4 provider for raw Brotli with semantic quality mapping;
- streaming output and expansion-limit enforcement before output is appended;
- round trips across empty, small, UTF-8, binary, pseudo-random, and repetitive
  payloads under all three semantic compression goals;
- malformed-header and truncated-stream rejection, with the Brotli provider's
  coarser malformed classification recorded explicitly;
- native tests and `wasm32-unknown-unknown` compilation.

`corpus/focused/data-interchange/hello-compression` independently consumes the public contract across
all three codecs and semantic goals, rejects a bounded high-expansion decode,
and writes `target/hello-compression/report.json` as structural evidence.

`corpus/lib/resource-space-compression` now establishes the first explicit
Resource Space composition boundary:

- source bytes are resolved through public logical lookup;
- encode or decode, codec, destination, metadata, and collision policy are
  caller-selected;
- transformation succeeds before any destination mutation is attempted;
- reject and replace behavior are distinct and observed;
- source and result keys plus content fingerprints remain in bounded evidence;
- ordinary reads continue to return the exact retained bytes;
- no `MemoryStore`, filesystem, persistence-provider, or archive type crosses
  the bridge.

The headless corpus retains source, encoded, and decoded resources together and
proves source preservation plus byte-identical decoded content. Cross-provider
evidence through a Tosumu-backed host remains open.

`corpus/lib/archive-provider` now establishes the first archive boundary:

- ZIP format, manifest, ordered entry observations, normalized names, selected
  reads, and independent archive/count/entry/total-output/path limits;
- stored and Deflate compression observations without provider-native ZIP
  objects or numeric compression methods escaping the contract;
- rejection of traversal, absolute and drive-prefixed names, normalized
  duplicates, symlinks, encrypted entries, malformed/truncated central
  directories, CRC corruption, and exceeded limits;
- pinned `zip` 8.6.0 with default features disabled and only `deflate` enabled;
- deterministic ordered writing with fixed timestamps and permissions, safe
  normalized names, duplicate rejection, Stored/Deflate policy, and bounded
  input/output;
- native conformance tests and `wasm32-unknown-unknown` compilation.

`corpus/focused/data-interchange/hello-archive` consumes one first-party immutable ZIP fixture through
the public contract, reads one selected entry byte-for-byte, rejects an input
budget violation, writes the same ordered entries twice with byte-identical
results, reads that generated archive back through the public contract, and
retains `target/hello-archive/report.json`.

`corpus/lib/resource-space-archive` composes bounded inspection and one
selected-entry copy with public Resource Space semantics. It preserves source
identity and bytes, requires explicit destinations and collision policy, and
performs no mounts or automatic extraction. `hello-archive` now supplies the
first application-shaped evidence for that bridge.

The work does not yet contain execution cancellation, whole-tree Resource Space
archive import/export, archive-backed folder views, TAR semantics, or browser
runtime parity.
