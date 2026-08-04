# Source Provenance

The behavior inventory is derived from the maintainer-owned C# repository at
`ClassLibrary/MemoryStore` and `ClassLibrary/MemoryStore.Tests`.

- Source project: `MemoryStore`
- Source implementation: `InMemoryResourceStore.cs`
- Observed source tests: 83 xUnit facts/theories
- License: MIT, Copyright (c) 2025 Arakendo
- Rust implementation approach: independent implementation from recorded
  semantics; no C# source has been copied into this crate

The source API mixes logical storage, text and format conveniences, hashing,
filesystem import/export, and XML resolution. Those operations are evidence,
not an API template. Format and platform behavior remain outside the base Rust
contract.
