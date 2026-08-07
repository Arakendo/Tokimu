# ADR-0010: Ring 0 Third-Party Source Admission and Pinning

## Status

Accepted

## Context

ADR-0003 assigns universal engine meaning to Native Tokimu. ADR-0008 then
requires a higher performance and code-quality burden for that Native Ring, and
ADR-0009 requires stronger verification, failure-containment, and recovery
evidence. Those gates are incomplete if code inside the trusted boundary can be
downloaded indirectly, change behind a loose version requirement, or remain too
large or opaque for Tokimu maintainers to inspect.

This ADR uses **Ring 0** as shorthand for ADR-0008's **Native Ring**. The term
describes trusted architectural ownership, not a source directory, crate name,
operating-system privilege level, or native desktop target.

Tokimu currently has third-party code in the Ring 0 build closure:

```text
tokimu-core
├── glam
└── serde
    ├── serde_core
    └── serde_derive
        ├── proc-macro2
        │   └── unicode-ident
        ├── quote
        └── syn
```

`glam` types are also re-exported from `tokimu-core::math`. This means the
question is not hypothetical: some foreign implementation and representation
already participates in trusted code and public API shape.

A `Cargo.lock` file pins resolution for one workspace checkout, but it does not
put the reviewed source under Tokimu's repository-controlled revision graph.
A Git URL with a tag or branch still delegates identity to an external ref. A
Git submodule pins an exact source commit, but pinning alone is insufficient if
Cargo continues compiling registry copies or unresolved transitive crates.

Tokimu therefore needs one policy covering source location, exact identity,
transitive build closure, audit evidence, updates, enforcement, and migration
of the existing dependencies.

## Decision

Ring 0 may use third-party library code only when Tokimu can identify, retrieve,
build, inspect, and audit the exact source that enters the trusted boundary.

> Foreign implementation may support Ring 0. Foreign ownership, opaque code,
> and unreviewed source substitution may not.

Every admitted third-party Ring 0 library must be checked into Tokimu's Git
history as a pinned submodule and consumed from that checked-out source.

### Source and pinning requirements

Each direct third-party Ring 0 library must:

- live under `third-party/ring-0/<library>/` as a Git submodule;
- pin an exact immutable commit through the parent repository's gitlink;
- record the canonical upstream repository and, when applicable, the Tokimu
  fork from which the pinned commit is obtained;
- be consumed by Ring 0 through a Cargo path dependency into that submodule;
- remain visible to normal dirty-submodule checks; Ring 0 entries may not use
  `.gitmodules` settings that hide local source modification;
- disable default features and enable only the reviewed feature set where the
  dependency supports feature selection; and
- remain buildable without Cargo substituting a registry or remote Git copy of
  the same crate anywhere in the Ring 0 closure.

A branch, floating tag, version range, registry checksum, or `Cargo.lock` entry
does not replace the pinned submodule. The lockfile remains useful as a
resolution record, but it is supplemental evidence rather than the source of
truth for Ring 0 code identity.

Submodule directories must remain clean in ordinary Tokimu commits. Required
patches belong in a pinned Tokimu fork or an auditable commit series with clear
upstream provenance; uncommitted edits inside a submodule are not an accepted
dependency state.

### The complete trusted source closure is in scope

The policy applies recursively to third-party source that is:

- linked into a Ring 0 library or executable;
- compiled into Ring 0 through generics, macros, generated modules, or copied
  source;
- executed by a Ring 0 build script; or
- executed as a procedural macro that generates or transforms Ring 0 code.

An admitted direct dependency is not compliant while one of its transitive
runtime, build, or procedural-macro dependencies still resolves from a
registry or remote Git source. Tokimu must do one of the following for every
such dependency:

1. add and audit its exact source as another pinned Ring 0 submodule and use a
   path dependency or workspace patch to select it;
2. disable the feature or build path that requires it;
3. replace the dependency with a smaller auditable implementation; or
4. move the responsibility behind an Outer Ring contract.

Cargo patches must point to pinned local submodule paths. They must not point to
an unpinned sibling checkout, developer-specific path, branch, or network URL.

### Auditability requirements

Source is **auditable** only when maintainers can reasonably inspect the entire
selected closure and explain its behavior at the trust boundary. Source
availability by itself is not enough.

Before admission, retain a Ring 0 dependency audit that records:

- repository URL, exact commit, crate/package versions, and source-tree hash or
  equivalent identity;
- license, attribution, notice, patent, and redistribution obligations;
- selected features and why each is required;
- the complete runtime, build, and procedural-macro dependency closure;
- source size and the portions actually selected by the build;
- unsafe code, FFI, inline assembly, generated source, and prebuilt artifacts;
- build scripts, procedural macros, environment reads, filesystem/network
  access, and other build-time behavior;
- allocation, threading, synchronization, global state, I/O, panic, and error
  behavior relevant to Ring 0;
- determinism and native/WASM portability consequences;
- public foreign types or traits exposed through Tokimu APIs;
- known security advisories and the method/date used to inspect them;
- performance, binary-size, and compile-time evidence required by ADR-0008;
- unit, contract, target, malformed-input, and failure evidence required by
  ADR-0009;
- serious alternatives, including implementing the narrow requirement in
  Tokimu or moving it outward; and
- an owner, update procedure, removal/migration strategy, and reopening
  triggers.

Audit records belong under:

```text
docs/Dependency Audits/Ring 0/<library>-<revision>.md
```

An audit may conclude that the source is too large, generated, macro-heavy,
unsafe, entangled, or poorly specified for Tokimu to review confidently. That
is a rejection result, not missing paperwork to waive casually.

Automated vulnerability, license, dependency, and unsafe-code scanners may
support the audit. They do not replace source review or architectural judgment.

### Public API exposure is a separate admission cost

Using a foreign implementation privately does not automatically admit its types
or traits into Tokimu's public vocabulary.

A third-party type exposed from a stable Ring 0 API requires explicit review of:

- the semantic promise Tokimu is making through that type;
- upstream semantic-versioning and representation risk;
- serialization, reflection, FFI, WASM, and authoring-frontend consequences;
- the cost of wrapping or migrating callers later; and
- whether Tokimu or an Outer Ring should own the public representation instead.

Public re-export is an architectural commitment and must be recorded in the
dependency audit or a dedicated ADR. Convenience is not sufficient evidence.

### Update policy

Every dependency update is a source change to Ring 0, even when the upstream
version describes itself as a patch release.

An update must:

- move the submodule gitlink to one reviewed commit;
- retain the old and new revisions and an upstream diff reference;
- review added, removed, generated, unsafe, build-script, and proc-macro code;
- compare the complete Cargo dependency and feature closure;
- update licenses, advisories, and audit findings;
- rerun the applicable ADR-0008 and ADR-0009 gates; and
- update path patches, lockfiles, CI evidence, and attribution together.

Automated tooling may propose an update and produce diffs. It may not merge a
Ring 0 dependency update solely because compilation and tests pass.

A time-sensitive security update may use ADR-0005's provisional path when a
full audit cannot finish before mitigation. The exception must bound the
revision and exposure, record missing audit work, and name the evidence or date
that will complete, replace, isolate, or remove it.

### Build and CI enforcement

Tokimu must add a machine-checkable Ring 0 dependency audit that:

- derives the relevant dependency closure from Cargo metadata rather than a
  hand-maintained crate list;
- permits Tokimu workspace packages and approved local Ring 0 submodule paths;
- rejects registry, remote Git, developer-local, or unapproved path sources in
  the runtime/build/proc-macro closure;
- verifies required submodules are initialized at the parent-pinned commits;
- rejects dirty Ring 0 submodules in release and CI validation;
- records selected features and reports closure changes; and
- fails explicitly when source is missing instead of downloading an unnoticed
  substitute.

The audit should remain reproducible after the required submodules and Rust
toolchain are present. Ring 0 validation must not require Cargo to fetch library
source from the network.

### Publication and release artifacts

An official Tokimu binary, library artifact, or release validation must compile
Ring 0 from the pinned local source closure. Producing the same crate version
from a registry is not evidence that it contains the reviewed commit.

Cargo path overrides do not propagate automatically to downstream consumers.
Ring 0 crates must therefore remain unpublished until a reviewed publication
strategy proves how consumers receive or reproduce the audited source closure.
That strategy may package reviewed source, publish Tokimu-controlled audited
revisions with immutable identity, or select another mechanism that provides
equivalent source evidence. It may not silently fall back to ordinary registry
resolution while claiming compliance with this ADR.

Release archives must either include the required Ring 0 source or identify the
exact submodule initialization procedure and fail clearly when that source is
absent.

### Scope exclusions and separate trust roots

This ADR does not require the following to become Tokimu submodules:

- the Rust compiler, standard library, Cargo, linker, or target toolchain;
- operating-system, browser, GPU-driver, or hardware implementation code;
- source assets and data corpora that are not compiled or executed as Ring 0
  code; or
- dev-only test and analysis tools that cannot affect produced Ring 0
  artifacts.

Those remain trust inputs and need their own versioning, CI, platform, fixture,
or development-tool policies. A tool stops being excluded when it generates,
rewrites, or links code shipped as Ring 0; build scripts and procedural macros
are therefore explicitly in scope.

### Existing dependency migration

This decision does not silently grandfather the current `glam`/`serde` closure.
On acceptance, the following become a finite migration set:

```text
glam
serde
serde_core
serde_derive
proc-macro2
quote
syn
unicode-ident
```

Migration must determine, rather than assume:

- whether `glam` remains the correct Ring 0 math representation and whether its
  public re-export remains acceptable;
- whether serialization derives belong in Ring 0 or can move behind a narrower
  engine-owned or Outer Ring boundary;
- which upstream repositories and exact commits supply the complete closure;
- whether the procedural-macro closure is small and stable enough to audit; and
- whether path patches preserve correct native/WASM builds and published-crate
  behavior.

Until migration completes:

- no new third-party Ring 0 dependency or feature may be added;
- changes touching the migration set must record the known nonconformance and
  may not expand the closure without an ADR-0005 exception; and
- Tokimu must not describe the Ring 0 build as fully source-audited under this
  ADR.

The migration should proceed as a focused plan and compileable increments. The
ADR does not predetermine whether the final result retains, wraps, replaces, or
moves each current dependency.

## Alternatives Considered

### Depend on crates.io and rely on `Cargo.lock`

Rejected for Ring 0. The lockfile pins package resolution and checksums, but it
does not make the reviewed upstream source part of Tokimu's pinned repository
graph or guarantee maintainers audited the code that entered the trust
boundary.

### Use remote Git dependencies pinned to commits

Rejected for Ring 0. An exact Git revision is stronger than a version range but
still leaves source retrieval outside the initialized Tokimu tree and makes
offline audit and source-path enforcement less direct.

### Use `cargo vendor` snapshots

Rejected as the default Ring 0 source-of-truth mechanism. Vendoring can support
offline builds, but a copied snapshot loses the explicit upstream commit
relationship and review workflow that the project requires here. A generated
vendor directory may be a derived build artifact, not the authoritative source.

### Ban all third-party Ring 0 code

Rejected as a universal rule. Reimplementing foundational math,
serialization-support, or similarly mature code can increase defect and
maintenance risk. Strict admission, source pinning, bounded audits, and a bias
toward small dependencies provide a more useful boundary.

### Admit the library only in an Outer Ring

Preferred whenever Tokimu can preserve universal semantics through an
engine-owned contract while moving specialized implementation outward. This is
not always possible for foundational representation or compile-time support,
but it must be considered before Ring 0 admission.

## Non-Decisions

This ADR does not:

- admit any specific current or future dependency into Ring 0;
- declare the existing `glam` re-export permanently acceptable;
- require third-party source to be forked when an unchanged upstream commit is
  sufficient;
- permit opaque binaries because a wrapper crate is auditable;
- require Outer Ring providers, corpora, fonts, tools, and website dependencies
  to use the Ring 0 source policy;
- define the Rust toolchain pinning or compiler-trust policy;
- replace license counsel, security response, ADR-0008, or ADR-0009; or
- claim that auditability guarantees correctness or absence of malicious code.

## Consequences

Ring 0 builds gain an inspectable source chain rooted in the Tokimu commit. A
reviewer can initialize pinned submodules, derive the complete trusted closure,
inspect the exact code, and verify that Cargo did not substitute an unaudited
copy.

New Ring 0 dependencies become deliberately expensive. Large dependency trees,
procedural macros, build scripts, unsafe code, generated sources, and public
foreign types carry visible admission and update costs. This pressure should
naturally keep specialized libraries in Outer Rings and favor small,
engine-owned contracts.

The immediate cost is real migration work. The current `tokimu-core` dependency
closure is not compliant until its source is pinned as submodules, redirected
through local paths, audited recursively, or reduced by moving/replacing code.
CI and clean-checkout workflows must initialize the approved Ring 0 submodules.
Ring 0 crate publication remains blocked until the publication requirements
above have a reviewed implementation.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/contribution-admission-guide.md`
- `docs/future-workspace-layout.md`
- `docs/kernel-principles.md`
- `docs/Tokimu Software Design Document.md`
- `Cargo.toml`
- `Cargo.lock`
