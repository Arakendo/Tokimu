# Corpus Libraries

This folder holds reusable crates that support multiple Tokimu corpus entries
while their ownership and public contracts are still being discovered.

Code here is incubation infrastructure, not automatically an admitted engine
capability. Promotion requires independent consumers and architectural review.

## Folder Map

```text
corpus/lib/
  compression-provider/            bounded byte-compression provider proofs
  archive-provider/                bounded archive manifest and entry proofs
  archive-file-adapter/            native-only bounded archive file adapter
  resource-space-archive/          explicit archive inspection and entry copies
  resource-space-compression/      explicit logical-resource transformations
  cgm-corpus/                     CGM acquisition and structural evidence
  fbx-corpus/                     FBX acquisition and structural evidence
  gltf-corpus/                    glTF/GLB acquisition and structural evidence
  network-tools/                  replication and transport proofs
  performance-diagnostics-corpus/ runtime observation evidence
  presentation-control/           transient presentation intent and resolution
  presentation-geometry-corpus/   vector and presentation geometry harness
  screenshot/                     deterministic visual evidence helpers
  ui-framework/                   composed UI consumer
  ui-tools/                       incubating UI and vector implementation
  xml-tools/                      XML parsing and query proofs
```

## Library Families

### Data And Interchange

- `compression-provider` incubates provider-neutral GZip, raw Deflate, and raw
  Brotli byte transformations with mandatory decode limits.
- `archive-provider` incubates provider-neutral archive manifests, safe entry
  names, and bounded selected-entry reads. It currently admits a constrained
  ZIP regular-file and directory subset.
- `archive-file-adapter` keeps native file reads and create-new archive
  publication outside archive semantics. It does not add filesystem paths to
  archive providers or claim cross-platform replacement behavior.
- `resource-space-archive` composes bounded archive inspection and selected
  entry reads with explicit Resource Space destinations. It does not mount,
  auto-extract, or reinterpret ordinary retained bytes.
- `resource-space-compression` composes those byte transformations with
  explicit Resource Space lookup, collision, mutation, and provenance
  semantics. Ordinary Resource Space reads remain byte-faithful.
- `cgm-corpus`, `fbx-corpus`, and `gltf-corpus` acquire, classify, and inspect
  external format fixtures without making their format semantics engine-native.
- `xml-tools` provides the incubating XML mechanisms used by format-specific
  corpus work.

### Presentation And Evidence

- `ui-tools` contains shared presentation semantics, geometry, controls, font
  adapters, and vector implementation still under corpus pressure.
- `ui-framework` proves those pieces compose into an application shell.
- `presentation-geometry-corpus` records staged outline, vector, mesh, and
  image evidence.
- `presentation-control` resolves transient target tint, opacity, visibility,
  and emphasis without owning importer truth or renderer resources.
- `screenshot` provides deterministic saved visual evidence.
- `performance-diagnostics-corpus` exercises runtime observation and budget
  reporting.

### Runtime And Movement

- `network-tools` incubates provider-neutral envelopes, codecs, loopback
  transport, and game client/server simulation.

## Ownership Rules

- A corpus library may be reused without being an admitted engine capability.
- Shared implementation remains here while its semantic ownership is uncertain.
- Format providers retain format semantics; corpus harnesses retain evidence
  and reporting concerns.
- Corpus entries should reuse an established helper when doing so tests the
  intended boundary. They should not depend on a helper merely to avoid a few
  local lines.
- Promotion requires independent consumers, stable provider-neutral contracts,
  and architectural review.
- A promoted capability moves to `crates/`; its corpus consumers remain here
  as executable evidence.

## Local Documentation

Each library should keep a `README.md`, `DESIGN.md`, or plan reference that
answers:

- which corpus entries consume it;
- which responsibilities it owns;
- which responsibilities it explicitly rejects;
- what evidence would justify promotion, redesign, or deletion.

Notable design documents:

- [ui-tools design](ui-tools/DESIGN.md)
- [ui-framework design](ui-framework/DESIGN.md)
- [presentation geometry corpus design](presentation-geometry-corpus/DESIGN.md)
