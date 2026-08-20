# ADR-0011: Ring-Based Security, Authority, and Trust Boundaries

## Status

Accepted

## Context

Tokimu is designed around a small trusted core, provider-neutral capability
contracts, replaceable mechanisms, world-owned truth, and explicit diagnostic
and lifecycle behavior. Those boundaries reduce accidental coupling, but they
do not by themselves state what a caller, script, provider, importer, browser
client, or remote peer is allowed to observe or mutate.

Security for an engine is not limited to secrets or network authentication. A
malformed asset that requests a 20 GB allocation is an availability failure. A
provider that can mutate unrelated world state is an integrity failure. A
diagnostic that exposes credentials is a confidentiality failure. Code running
in the same process can still exceed its architectural authority even when it
does not cross an operating-system protection boundary.

Tokimu therefore needs a security discipline based on ownership, trust, and
authority rather than directory names or an assumption that in-process code is
implicitly trusted.

This ADR complements, rather than replaces:

- ADR-0003, which decides whether meaning belongs to Native Tokimu;
- ADR-0005, which governs provisional admission and evidence exceptions;
- ADR-0008, which applies proportional performance and code-quality gates;
- ADR-0009, which governs verification, failure containment, and recovery;
- ADR-0017, which makes unexplained host/process disappearance an immediate
  availability and diagnostic conformance failure; and
- ADR-0010, which governs the identity and auditability of third-party source
  admitted to Ring 0.

ADR-0009 asks what happens when an operation fails. This ADR asks what the
operation was allowed to believe, observe, and do in the first place.

## Decision

Tokimu adopts explicit, least-authority security boundaries and proportional
security review gates for the rings defined by ADR-0008.

> Code may observe or mutate only the authority explicitly granted by the
> owning contract. External input is untrusted until validated. Providers do
> not inherit engine authority merely because they run in the same process.

Tokimu security decisions must consider integrity and availability as well as
confidentiality. A boundary is not secure merely because it handles no secret.

### Foundational rules

The following rules apply across all rings:

1. **No ambient authority.** A subsystem receives only the handles,
   capabilities, data, and operations needed for its declared responsibility.
   Global reachability, process membership, object discovery, or possession of
   a provider reference does not grant unrelated authority.
2. **Discovery does not imply authority.** A catalog may report that a
   capability, resource, command, or application exists without granting the
   observer permission to invoke, read, alter, or manage it.
3. **External input is untrusted.** Files, assets, serialized state, browser
   events, IPC, network messages, plugin output, provider responses, and
   foreign-function results must be validated before they influence trusted
   state. Authentication does not make data structurally or semantically
   valid.
4. **In-process is not an authority class.** Rust memory safety and a shared
   address space do not permit a provider, plugin, tool, or presentation layer
   to bypass Tokimu ownership contracts.
5. **Authority is explicit, scoped, attributable, and revocable.** A grant
   identifies who received it, what operation it permits, the objects or
   domain it covers, its lifetime, relevant limits, and the policy decision
   that issued it. Revocation or expiry must produce an explicit denial rather
   than silently retaining access.
6. **Absent authority is denied.** Unknown operations, missing grants, stale
   handles, expanded scopes, and unsupported security states fail closed at
   the protected boundary with safe diagnostics.
7. **Policy and mechanism remain separate.** Tokimu-owned contracts define
   provider-neutral authority and denial meaning. The application or
   composition root decides policy and issues grants. Runtime and platform
   mechanisms enforce those decisions without becoming owners of world truth.

The exact public capability vocabulary is not stabilized by this ADR. Terms
such as `ObserveRuntime`, `MutateRuntime`, `ReadResources`, `WriteResources`,
`ManageApplication`, and `ExecuteExternalProcess` are useful review concepts,
not an admitted enum or role system. New authority concepts still require real
caller evidence and ADR-0003 admission.

### Trust classification

Every protected boundary must identify the source and trust class of its
inputs. At minimum, changes must distinguish as applicable:

- Native Ring state and engine-owned invariants;
- application policy and configuration;
- local user input and local files;
- third-party assets and serialized data;
- plugins, scripts, providers, and foreign libraries;
- browser documents, events, messages, and storage;
- network peers and remote control requests;
- persisted or cached state from an earlier process or version;
- secrets and other sensitive values; and
- FFI, device, subprocess, and operating-system results.

"Trusted" means a source is permitted to make a particular claim; it does not
mean its data is necessarily valid, current, bounded, or authorized for every
operation. Trust is local to a claim and boundary, not inherited transitively.

### Authority ownership and enforcement

Native Tokimu may own the provider-neutral meaning of a capability grant,
scope, denial, expiry, and revocation when that meaning has been admitted under
ADR-0003. It must not own application-specific identities, organizations,
roles, subscription plans, or access-control policy.

The application or composition root owns decisions such as which local tool,
script, authenticated peer, or user may receive an admitted authority. A
provider implements only the mechanism requested through its typed contract.
It does not gain the ability to mutate the world, manage applications, read
unrelated resources, write files, open network connections, or start processes
unless that authority is both necessary and explicitly granted.

Authority-bearing handles and requests must be narrow enough to prevent a
caller from obtaining a broader object and reaching protected operations
indirectly. Rendering remains an observer of simulation state. World mutation
must enter through admitted commands, schedules, or other validated mutation
boundaries rather than provider callbacks or presentation objects.

A grant must have a stable identity or equivalent provenance sufficient to
diagnose its issuer, intended subject, scope, and lifecycle without exposing a
secret. Scene unload, provider disposal, application shutdown, disconnection,
or policy change must revoke the related authority where applicable. Cached
authorization decisions may not outlive the facts on which they depend.

### Input admission and resource safety

Untrusted input must be rejected before it can commit invalid trusted state or
cause disproportionate resource consumption. Applicable boundaries must:

- validate framing, structure, type, version, identifiers, and semantic
  invariants;
- validate authority independently from successful parsing or authentication;
- bound sizes, counts, nesting, recursion, decompression ratios, allocations,
  retained data, work per step, and retries before performing the expensive
  operation;
- use checked arithmetic for input-derived offsets, lengths, counts, and
  allocation sizes;
- canonicalize paths and URIs before authorization, while treating resolution,
  authorization, and loading as distinct operations;
- prevent traversal, alternate-encoding, redirect, symlink, alias, or stale
  identity behavior from expanding the authorized scope;
- avoid partially publishing state until validation and authorization succeed;
  and
- prevent rejected or incompletely validated data from poisoning shared
  caches, registries, snapshots, or other sources of truth.

For example, a WAD or image importer may inspect a supplied byte stream, but it
does not thereby receive filesystem write, process execution, network access,
or world-mutation authority. It must validate input-derived allocations and
work before committing an admitted resource.

### Providers, plugins, scripts, and tools

Providers and plugins start with no authority beyond their explicit contract.
They must receive projections or scoped handles rather than broad engine,
world, application, filesystem, or platform objects when narrower inputs are
sufficient. Their output remains untrusted at the receiving boundary.

Scripts and authoring frontends likewise begin with no authority. Tokimu may
inject explicitly granted capabilities, but host filesystem, network, DOM,
process, device, and unrestricted runtime access are not implied by script
execution. A sandbox is one enforcement mechanism; it is not a substitute for
an authority model at the Tokimu boundary.

Developer and observation tools must separate at least observation, runtime
mutation, resource mutation, application management, and external-process
authority when those operations exist. A convenient local operator shell may
receive broad authority only through explicit application policy. That local
choice must not become the default for browser or remote access.

### Network and remote authority

A transport connection establishes neither identity nor authority.
Authentication proves only the identity claim defined by its protocol;
application policy must separately map that identity to scoped Tokimu
authority. Observation and mutation are separate grants.

Network-facing or remote-control boundaries must address, as applicable:

- authenticated identity and authorization at the operation boundary;
- message size, rate, concurrency, retention, and work budgets;
- replay, duplicate, stale, reordered, and partially delivered requests;
- request identity, idempotency, cancellation, timeout, and backpressure;
- connection loss and prompt revocation of connection-scoped authority;
- origin and source validation for browser messaging; and
- safe diagnostic responses that do not reveal secrets, internal paths, raw
  tokens, or unnecessary engine state.

CORS, a private address, a local network, TLS, or a successful login does not
replace operation-level authorization. This ADR does not admit a networking,
identity, authentication, or remote-administration subsystem; those require
separate evidence and architectural decisions.

### Secrets and sensitive data

Secrets must not be stored in world state, ordinary resources, asset payloads,
source control, client bundles, diagnostic context, snapshots, crash artifacts,
or caches unless an explicitly reviewed contract requires that exact storage.
Prefer opaque references and scoped retrieval over copying secret values.

Producers must redact sensitive values before emitting diagnostics or retained
failure evidence. Redaction is not delegated solely to a presentation layer,
because diagnostics can have several consumers. Secret access must be bounded
in lifetime and scope, and rotation or revocation must not require rebuilding
Native Ring semantics.

This ADR does not establish a secrets manager, cryptographic suite, or key
storage format. Applications and replaceable providers own concrete secret and
credential mechanisms unless a later decision admits narrower native meaning.

### Unsafe Rust, FFI, devices, and subprocesses

`unsafe` Rust and FFI do not receive an exemption from ring boundaries. Their
call sites must be narrow and documented with the invariants required for
memory, lifetime, thread, ownership, and unwinding safety. Input-derived
pointers, lengths, handles, callbacks, and return values must be validated at
the safe boundary.

Foreign code that can corrupt memory, abort the process, escape the intended
authority surface, or retain authority after revocation cannot support a claim
of in-process containment. If continued operation after such behavior is a
requirement, the risky mechanism must be isolated behind an appropriate
process, operating-system, browser, or equivalent boundary and exercised by
ADR-0009 recovery evidence.

Subprocess and device access must use explicit executable or device identity,
arguments, environment, working scope, inherited handles, permissions,
lifecycle, output bounds, and termination policy. Untrusted input must not be
concatenated into a shell command or used to select broader authority.

### Native/WASM security parity

The same provider-neutral authority and denial meaning must apply on native
and WASM targets when the capability exists on both. Different mechanisms are
expected, but target-specific convenience must not silently broaden authority.

The browser sandbox, same-origin policy, and user permission prompts are
additional boundaries, not Tokimu authorization. Browser events, URL data,
storage, `postMessage` traffic, imported modules, and server responses remain
untrusted inputs. Authority held by a server or native host must never be
embedded into downloadable client code merely to make a browser consumer work.

### Full Native Ring security gate

Every non-mechanical Native Ring implementation, dependency, behavior, or API
change must answer the applicable checklist before merge. A maintainer may mark
an item not applicable only with a local reason.

The checkboxes are a reusable review template. Answers and evidence belong in
the change, pull request, Architectural Review Record, or another retained
artifact; unchecked boxes do not indicate incomplete work in this ADR.

#### Threat, trust, and authority

- [ ] The change identifies the assets or invariants being protected and the
      relevant integrity, availability, and confidentiality risks.
- [ ] Every external or cross-ring input has an explicit source and trust
      classification.
- [ ] The subjects, permitted operations, scopes, policy owner, enforcement
      point, lifetime, and revocation conditions are stated.
- [ ] Discovery, authentication, parsing, authorization, and execution are not
      treated as interchangeable approval steps.
- [ ] Missing, stale, revoked, unknown, and expanded authority is denied at the
      protected boundary.
- [ ] The change introduces no ambient or transitive authority through a
      global, broad object, callback, provider handle, or leaked native object.

#### Input and resource protection

- [ ] Input-derived work, allocation, retention, recursion, decompression,
      retry, and concurrency are bounded before expensive or irreversible work.
- [ ] Validation precedes publication into world state, registries, caches,
      persistent data, and other shared sources of truth.
- [ ] Path, URI, resource, redirect, and alias handling cannot expand the
      authorized scope after the policy decision.
- [ ] Duplicate, stale, replayed, reordered, and partially completed requests
      have defined behavior where they can occur.
- [ ] Denial and malformed input preserve unrelated trusted state and cannot be
      converted into silent fallback or unbounded retry.

#### Sensitive data and mechanisms

- [ ] Diagnostics, metrics, logs, snapshots, crash artifacts, browser output,
      and retained test fixtures do not expose secrets or unnecessary sensitive
      data.
- [ ] Unsafe, FFI, subprocess, device, filesystem, network, and persistent
      storage authority is absent or is explicitly scoped and reviewed.
- [ ] Third-party code in the Ring 0 closure satisfies ADR-0010 and its enabled
      features do not introduce unreviewed authority or mechanisms.
- [ ] Native and WASM paths preserve the same security meaning, or the target
      difference is explicit and tested.
- [ ] Revocation, shutdown, unload, disconnection, and replacement remove
      authority and sensitive retained state as promised.

#### Security evidence

- [ ] Tests cover successful use, permission denial, scope escape attempts,
      stale or revoked authority, and malformed hostile input as applicable.
- [ ] Resource-exhaustion and availability abuse cases are tested at practical
      bounds rather than requiring unsafe machine-wide exhaustion.
- [ ] A provider, browser, worker, renderer, device, or process disappearance
      cannot be mistaken for a denied request, successful containment, or an
      unavailable measurement; terminal outcome closure follows ADR-0017.
- [ ] Network or asynchronous boundaries test replay, duplicate, rate, timeout,
      cancellation, and disconnect behavior as applicable.
- [ ] Unsafe or foreign boundaries retain focused tests for invalid lengths,
      lifetimes, handles, callbacks, error returns, and panic or unwind behavior
      as applicable.
- [ ] Security-relevant diagnostics have stable identity and useful provenance
      without disclosing the rejected secret or payload.
- [ ] A corrected security defect retains the smallest safe regression fixture
      and reopens affected assumptions, audit records, or ADRs when warranted.

### Minimum Outer Ring security gate

Outer Ring code may use local policy and helpers without designing a universal
authorization framework. Every non-mechanical change must still answer the
applicable minimum gate:

- [ ] External inputs and their trust level are identified.
- [ ] The code requests only the authority needed for its declared job and does
      not redefine or bypass Native Ring security meaning.
- [ ] Input is validated and resource-bounded before it reaches trusted state
      or an irreversible external operation.
- [ ] Paths, URIs, commands, diagnostics, and retained artifacts do not leak or
      expand authority through unchecked input.
- [ ] Denial, revocation, lifecycle cleanup, and partial failure are explicit.
- [ ] At least one focused test covers the most credible denial, hostile-input,
      or scope-boundary failure introduced by the change.
- [ ] Documentation and diagnostics make no security or isolation claim beyond
      the enforcement boundary actually provided.

An Outer Ring change must apply the applicable full-gate sections when it:

- accepts network traffic or remote mutation requests;
- parses untrusted binary data, archives, compressed data, or recursive input;
- resolves paths or URIs, writes files, or modifies persistent/shared state;
- hosts plugins, scripts, downloaded code, or user-authored execution;
- handles credentials, tokens, private data, or other secrets;
- uses unsafe Rust, FFI, native libraries, subprocesses, devices, or privileged
  operating-system APIs;
- creates a shared cache, registry, or process-wide authorization decision; or
- crosses into or changes a Native Ring authority contract.

When a change crosses rings, the stricter gate applies at the crossing. When
classification is uncertain, treat the boundary as Native until ownership is
resolved. ADR-0005 governs any provisional evidence exception; convenience or
schedule pressure does not silently weaken a security boundary.

### Security findings and updates

A credible security finding must be contained first within existing authority:
disable or revoke the affected capability or provider, reject the vulnerable
input, or remove the exposed path without inventing a silent fallback. Recovery
and incident evidence follow ADR-0009. Changes to admitted third-party Ring 0
source follow ADR-0010 even when the update is urgent; ADR-0005 may document a
time-bounded provisional exception when its requirements are met.

Security reports and retained evidence should disclose enough for maintainers
to reproduce and verify the correction without unnecessarily publishing
secrets, personal data, live credentials, or a dangerous payload. A material
finding must reopen the affected trust assumption, authority contract, test
strategy, and review record rather than being treated only as a local patch.

## Consequences

- Native Ring changes now carry explicit proof of least authority, hostile
  input handling, resource safety, revocation, and security regression evidence.
- Outer Ring experiments remain inexpensive unless their actual authority or
  exposure creates a higher-risk boundary.
- Providers, tools, scripts, and clients cannot infer authority from discovery,
  process membership, authentication, or access to a broad engine object.
- Security review remains coupled to architectural ownership rather than a
  directory, crate, deployment target, or generic compliance checklist.
- Some existing broad handles, operator modes, parsers, provider callbacks,
  diagnostics, and client/server seams may require decomposition or narrower
  grants as audits apply this decision.
- Explicit denial and bounded hostile-input behavior add design and test work,
  but reduce the chance that integrity or availability failures become part of
  Tokimu's permanent contract.

## Non-Decisions

This ADR does not:

- establish a universal user, role, group, ACL, RBAC, or policy language;
- admit networking, remote administration, plugin hosting, scripting, a
  secrets manager, or a process supervisor into Native Tokimu;
- choose authentication protocols, cryptographic algorithms, certificate
  policy, credential storage, or platform sandbox technology;
- require every provider to run out of process or ban all unsafe Rust;
- claim that auditability under ADR-0010 proves third-party code is secure;
- promise multi-tenant isolation inside one Tokimu process; or
- guarantee security merely because a checklist was completed.

Those decisions require concrete threats, consumers, and evidence. This ADR
defines the authority and trust discipline they must satisfy.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0017-observable-terminal-failure-and-host-crash-conformance.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/kernel-principles.md`
- `docs/semantic-kernel-map.md`
- `docs/capability-backends.md`
- `docs/diagnostics-model.md`
- `docs/Tokimu Software Design Document.md`
- `docs/Tokimu TypeScript Design Document.md`
