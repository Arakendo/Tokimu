# Resource Space

`resource-space` incubates provider-neutral identity and addressing semantics
derived from the maintainer's C# `MemoryStore` experience.

The name is intentional:

- `MemoryStore` describes one storage provider;
- `VFS` implies filesystem compatibility that this contract does not promise;
- `resource-space` describes a logical hierarchy of roots, folders, and
  addressable resources without selecting a retention or platform mechanism.

This library is provisional under ADR-0005 and AR-0009. Its APIs may move,
change, or be retired as consumer evidence identifies the final ownership
boundary. It is not a stable kernel contract.

The current slice owns:

- explicit store and root identity;
- provider-neutral store provenance for diagnostics without host paths or
  browser/native acquisition details;
- strict logical address normalization;
- qualified resource keys;
- provider-neutral visibility and metadata;
- explicit root and folder navigation.
- immutable in-memory resource bytes through `InMemoryResourceSpace`.
- bounded recursive literal search, kept distinct from direct navigation.
- algorithm-qualified BLAKE3 fingerprints for diagnostics and candidate
  deduplication, plus exact byte comparison separate from resource identity.
- opt-in, bounded, locally ordered mutation observations with structured
  provider-neutral outcomes; observation is disabled by default and is not a
  durable revision or global event stream.

`resource-space-assets` is a separate incubating bridge. It proves that an
immutable `ResourceEntry` can feed `tokimu_assets::AssetLoader` while
`tokimu-assets` continues to own only asset handles and lifecycle. Failed
decodes allocate no asset handle and leave the source resource inspectable.

`resource-space-json` and `resource-space-xml` are likewise separate format
bridges. They prove typed JSON conversion and bounded XML sibling resolution
without making serde, JSON, XML parsing, URI semantics, or source-format
meaning part of the provider-neutral store contract.

It does not emulate a filesystem, parse formats, or create platform paths.

`InMemoryResourceSpace` uses caller-supplied stable store, root, and folder
identifiers. Its `AddressCasePolicy` is enforced at folder mutation boundaries;
callers can obtain normalized names with `resource_name`. Root and folder
removal are empty-only, so no hierarchy operation silently deletes a subtree.
