# Ring 0 Dependency Audits

This directory retains the source-review records required by ADR-0010. A
record is evidence for one exact source revision; it is not a permanent
approval for a crate name or a version range.

Use one file per audited upstream repository and revision:

```text
<repository>-<revision>.md
```

When several packages are supplied by one repository, one record may cover the
shared source revision if it names every package and selected feature.

## Record Template

```markdown
# <Repository> Ring 0 Audit: <revision>

## Decision

Retain | Wrap | Replace | Move | Remove | Reject | Provisional

## Identity

- Canonical upstream repository:
- Tokimu mirror or fork, if any:
- Exact commit:
- Source-tree hash or equivalent:
- Packages, versions, and Cargo features:
- Parent submodule path and gitlink:
- Audit date and owner:

## Role And Alternatives

- Native Ring responsibility served:
- Direct callers and public API exposure:
- Alternatives considered: retain, wrap, replace, move, remove:
- Reason this source belongs in Ring 0 rather than an Outer Ring:

## Source And Build Review

- Complete runtime, build-script, and procedural-macro closure:
- Source size and selected build surface:
- Generated code, prebuilt artifacts, or code generation:
- Unsafe code, FFI, inline assembly, and foreign libraries:
- Build scripts, proc macros, environment/filesystem/network behavior:
- Allocation, threading, synchronization, global state, I/O, panic, and error behavior:
- Determinism and native/WASM consequences:

## Legal And Security Review

- License, attribution, notice, patent, and redistribution obligations:
- Advisory sources checked and date:
- Findings and disposition:

## ADR-0008 And ADR-0009 Evidence

- Performance, binary-size, and compile-time evidence:
- Unit, contract, target, malformed-input, and failure evidence:

## Update, Removal, And Reopening

- Owner and update procedure:
- Removal or migration strategy:
- Reopening triggers:
```

The current baseline is intentionally not an admission record. Run
`pwsh -NoProfile -File scripts/audit-ring-zero-dependencies.ps1` to derive and
enforce the configured closure. Until approved source paths are populated, the
command must fail for the known registry sources.
