# AR-0018: Ring-Based Security, Authority, And Trust Conformance

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-07 |
| Last reviewed | 2026-08-07 |
| Scope | Native Ring / security and trust boundaries / cross-cutting |
| Trigger | ADR-0011 is accepted and needs a durable record of authority, trust, hostile-input, and sensitive-data conformance evidence |
| Related ADRs | ADR-0003, ADR-0005, ADR-0009, ADR-0010, ADR-0011 |
| Related evidence | AR-0015; `docs/Dependency Audits/Ring 0/`; Ring 0 audit script; current Native/WASM and offline validation |
| Admission exception | None |

## Architectural Question

Are Tokimu boundaries enforcing ADR-0011's explicit authority, least-trust,
resource-safety, revocation, and safe-diagnostic requirements at the point a
real capability, input, provider, or external mechanism is introduced?

## Context

ADR-0011 establishes a security discipline without prematurely admitting a
universal role system, authentication service, secrets manager, networking
stack, or sandbox. It distinguishes discovery, parsing, authentication,
authorization, and execution; it treats availability and integrity as security
properties alongside confidentiality; and it requires authority to be explicit,
scoped, attributable, and revocable.

The current Ring 0 repair concerns source provenance, which is one build-time
trust boundary rather than a runtime authorization system. It proves that the
declared Native root can reject registry, remote Git, unapproved local path,
missing, dirty, and ignored submodule sources. It does not prove authority
grants for scripts, providers, browsers, network peers, devices, filesystems,
or subprocesses because no such capability is admitted by this change.

## Trigger And Evidence

- ADR-0010 source checks are now explicit and fail closed: a source must be an
  actual workspace member or a configured, parent-pinned Ring 0 submodule.
- The audit treats a path located under the repository as untrusted unless it
  appears in Cargo's workspace membership or the approved Ring 0 source list.
- The audit rejects dirty submodule worktrees and `.gitmodules` `ignore`
  settings, so discovery of a submodule does not grant it approval.
- `glam` source was mapped to an exact upstream commit, compared with the
  packaged `src/` tree, and retained with its unsafe SIMD surface explicitly
  recorded.
- The offline focused build confirms that a missing registry source is not used
  to satisfy the declared Ring 0 closure.
- Missing evidence includes concrete runtime capability grants, revocation,
  secret handling, path/URI authorization, hostile network or parser inputs,
  browser message validation, and FFI/process isolation tests.

## Ownership Analysis

ADR-0011 owns the accepted cross-ring authority and trust rules. This AR owns
evidence that individual implementation boundaries apply them. ADR-0010 owns
third-party source admission; its audit script is a build-time policy enforcer,
not a Native runtime authority manager.

Applications remain the future owners of policy decisions for users, local
tools, scripts, and remote peers. Providers own mechanisms only through narrow
contracts. Native Tokimu may own provider-neutral capability, scope, denial,
expiry, and revocation meaning when separately admitted, but this review does
not admit any new authority enum or authentication model.

## Dependency Direction

```text
Current build-time boundary:

Cargo metadata + parent Git state
    |
    v
Ring 0 provenance audit
    |
    v
allow only configured local source identity

Future runtime boundary:

application policy and explicit grant
    |
    v
provider-neutral Tokimu contract
    |
    v
scoped provider or tool mechanism

No provider, script, browser client, or remote peer inherits authority from
process membership or discovery alone.
```

## Alternatives Considered

### Alternative A: Treat Source Pinning As Complete Security

- Benefits: a small, measurable goal.
- Costs: leaves runtime authority, hostile input, secrecy, availability, and
  isolation unexamined.
- Failure mode: a pinned dependency is mistaken for a secure application.

### Alternative B: Add A Universal RBAC Or Authentication System Now

- Benefits: an apparently complete vocabulary.
- Costs: invents application identity and policy meaning before real callers.
- Failure mode: Native Tokimu owns arbitrary organization and deployment policy.

### Alternative C: Apply ADR-0011 At Concrete Boundaries

- Benefits: builds security evidence where real source, input, authority, or
  mechanism crossings occur.
- Costs: requires different evidence for build, parser, provider, browser, and
  network work.
- Failure mode: broad principles are recorded but a new boundary skips them.

## Findings

- The provenance audit is a valid build-time least-trust boundary: local path
  location, source discovery, and package version do not substitute for
  configured approval and parent-pinned identity.
- Rejecting registry substitution and dirty/ignored source protects integrity
  and availability of the trusted build closure, but does not establish runtime
  authorization or memory/process isolation.
- `glam`'s unsafe SIMD implementation is recorded and source-pinned; it is not
  thereby proven safe. Its unsafe surface and compiler-warning risk remain
  explicit re-review triggers.
- No present evidence justifies admitting a universal capability grant type,
  secret store, sandbox, network identity layer, or remote-admin contract.
- The next security conformance evidence should follow actual hostile-input,
  provider, path/URI, browser, or network pressure rather than inventing a
  generic security subsystem.

## Disposition

**Continue under review.** ADR-0011 remains binding and unchanged. The Ring 0
source repair is retained as a narrow authority and trust example, while this
record remains open for concrete runtime boundaries that require authorization,
revocation, input admission, sensitive-data handling, or stronger isolation.

## Consequences

- Build-time provenance checks must remain explicit, default-deny, and safe to
  run offline for the declared Ring 0 closure.
- Future capabilities must state input trust, granted operations, scope,
  lifetime, revocation, resource bounds, denial diagnostics, and target
  behavior as applicable.
- No in-process provider or third-party library receives broader Tokimu
  authority merely by being source-pinned or compiled into the same process.

## Required Follow-Up

- [x] Retain the Ring 0 build-provenance trust evidence and source audit.
- [x] Add negative provenance-audit fixtures for unapproved and dirty sources
      without mutating a working Ring 0 submodule.
- [ ] Review the first real path/URI, parser, plugin, browser, network, FFI, or
      subprocess boundary against ADR-0011's full applicable sections.
- [ ] Define a safe, bounded diagnostic artifact for security-relevant audit
      failures when CI retention is introduced.
- [ ] Revisit `glam` before its upstream warning or unsafe surface changes.

## Reopening Triggers

- Any Ring 0 closure accepts an unapproved source or hides local source change.
- A runtime capability introduces ambient authority, an unbounded external
  input, unscoped secret access, or a provider bypass of world ownership.
- A browser, network, path/URI, FFI, device, or subprocess capability is
  introduced without explicit authorization and denial evidence.
- An advisory, source mismatch, unsafe/FFI change, or toolchain failure affects
  the pinned `glam` source.

## Review History

### Cycle 1 -- 2026-08-07

- Status entering review: Proposed.
- New evidence: ADR-0011 accepted; Ring 0 audit enforces explicit source
  approval and local provenance for `tokimu-core`; the retained dependency
  audit names unsafe and update risks.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: build provenance is one concrete ADR-0011 boundary; runtime
  authority, hostile-input, and isolation evidence remains intentionally open.
- Disposition: continue under review.
- Resulting ADR or documentation change: no ADR revision; this record opened.

## References

- `docs/ADR/ADR-0011-ring-based-security-authority-and-trust-boundaries.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/Architectural Reviews/AR-0015-ring-zero-provenance-enforcement-and-audit-closure.md`
- `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
- `scripts/audit-ring-zero-dependencies.ps1`
