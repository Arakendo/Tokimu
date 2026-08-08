# Foreign Public Type Review Checklist

Use this checklist when a foreign type or trait is proposed for a Native Ring
Tokimu API. It complements ADR-0010's source-admission audit: an auditable
implementation is not automatically acceptable public vocabulary.

Every `N/A` answer needs a local reason. An unresolved item means the public
admission decision remains open; it does not authorize a silent re-export.

## Checklist

### 1. Ownership and contract

- [ ] Name the Tokimu meaning the type represents and why it belongs in Native
      rather than an Outer Ring contract.
- [ ] State what irreducible semantic value Tokimu would lose if this type
      disappeared entirely. Do not infer Native admission merely from current
      availability, convenience, or import count.
- [ ] State whether Tokimu owns only semantics, public vocabulary, and/or the
      executing implementation; do not treat those as the same claim.
- [ ] Search existing Tokimu vocabulary and reject duplicate or competing
      abstractions.
- [ ] Identify real current callers and the exact operations, traits, fields,
      and error/degenerate behavior they require.
- [ ] Evaluate retain, private provider, Tokimu-owned wrapper, narrow original
      implementation, bounded derivation, and outward movement before admitting
      a public foreign type.

### 2. Public-boundary cost

- [ ] Record the foreign names, traits, representation, conversions, and any
      upstream compatibility promise that would cross a public Tokimu boundary.
- [ ] Evaluate serialization, reflection, FFI/layout, GPU upload, WASM,
      authoring-frontend, and downstream-publication consequences.
- [ ] Define migration and rollback paths, including how callers can leave the
      provider vocabulary if a future replacement is required.
- [ ] Keep provider types and conversions out of candidate public signatures;
      any intentional exception is an explicit ADR-0010 admission cost.

### 3. Implementation and supply-chain evidence

- [ ] Complete ADR-0010 source, pin, closure, license, build-script,
      proc-macro, unsafe, update, and security audit requirements.
- [ ] Identify provider seams, allocations, copies, panics, globals,
      synchronization, I/O, and target-specific behavior on relevant paths.
- [ ] If source is copied or derived, preserve exact provenance, modifications,
      upstream fix-detection, license obligations, and an intentionally bounded
      source/API manifest.

### 4. Correctness, performance, and recovery

- [ ] Retain shared baseline/candidate conformance cases that label observed
      provider behavior separately from proposed Tokimu guarantees.
- [ ] Exercise more than API-shaped unit tests: port bounded real callers and
      retain source edits, conversion counts, helpers, provider leaks, and
      rollback steps.
- [ ] Measure only named workloads with target, profile, host/toolchain,
      repetition, and allocation evidence; label missing binary-size,
      compile-time, or WASM-runtime evidence honestly.
- [ ] Cover malformed, degenerate, non-finite, singular, overflow, and failure
      behavior relevant to the type. Apply ADR-0009 containment/recovery
      evidence where a failure could cross a protected boundary.

### 5. Decision and follow-up

- [ ] Record retain, wrap, replace, derive, move outward, or continue-incubation
      disposition with named tradeoffs and reopening triggers.
- [ ] Apply ADR-0008 hygiene/performance and ADR-0011 authority/input/unsafe
      gates to the selected boundary.
- [ ] Do not begin a stable API migration until the applicable ADR, review, and
      validation evidence agree.

## Retrospective Application: `glam` Math (AR-0019)

| Checklist area | Current result | Evidence / remaining gap |
| --- | --- | --- |
| Native meaning and caller inventory | Met for bounded study | ADR-0003 and `operation-inventory.md` identify math meaning and direct callers. Post-DOOM re-scan remains required. |
| Irreducible value if absent | Partially met | `Mat4`/`Vec3` have concrete renderer and corpus pressure. `Vec2`/`Quat` have no direct caller, so their current public availability is not evidence of irreducible Native value. |
| Ownership distinction | Met, unresolved decision | A/B/C/D and AR-0019 separate semantics, vocabulary, implementation, and source ownership. |
| Alternatives | Met for current scope | A/B/C/D are implemented or explicitly paused; outward movement remains a standing boundary question. |
| Public-boundary cost | Partially met | The retained public-boundary scan distinguishes the exposed `Camera` seam from renderer-local scalar-array GPU upload. Real downstream/FFI/serialization/publication/browser-WGPU migration remains absent. |
| ADR-0010 source audit | Met for current `glam` provider | Pinned submodule and dependency audit retained; this does not select public vocabulary. |
| Copy/derivation provenance | Met for D slice | D's notice and manifest are retained; expansion remains paused. |
| Shared correctness | Partially met | A/B/C match bounded transforms, GLB geometry, CAD ray, and animated-node paths. Degenerate/singular behavior is not yet a selected Tokimu contract. |
| Real caller migration | Partially met | Corpus-local bounded ports exist. Full renderer, scene, serialization/FFI, and public API migrations do not. |
| Performance/target evidence | Partially met | Native and WASM builds; zero allocation observations for bounded paths. Repeated target-identified timing, binary size, build time, and WASM execution are missing. |
| Decision | Continue incubation | A remains stable control; C leads the ownership experiment; B remains a transition/comparison seam; D is paused. |

## Reopening Triggers

- A new foreign type or trait is proposed for a Native Ring public API.
- A provider update, advisory, warning escalation, closure change, unsafe
  change, or target regression affects the admitted implementation.
- A real caller requires an unmanifested operation or representation.
- DOOM or another corpus path materially changes object, transform, animation,
  collision, scene, or rendering pressure.
- A stable API/FFI/serialization/publication boundary would expose the type's
  representation or upstream compatibility policy.

## Review-Method Note

This is review guidance derived from AR-0019 evidence, not a binding ADR. It
should be tested against additional foreign-public-type cases before any part
is promoted into admission policy. Its purpose is to prevent a dependency,
implementation choice, or public re-export from silently becoming Tokimu's
permanent ontology without separately proving semantic value, vocabulary cost,
and implementation responsibility.
