# AR-0033 Slice 1 Semantic-Shadow Evidence

Date: 2026-08-20

## Scope

This slice compares corpus-private semantic shadows for:

- Alternative B: scoped replacement behind an existing identity;
- Alternative C: an explicitly dynamic resource class; and
- Alternative D: submission-local transient presentation data.

The types live only in the Doom browser workbench. They do not implement a
provider operation, change `tokimu-render`, select a public handle shape, or
admit an update contract.

## Shared Checks

All three shadows retain session and resource-set scope. The comparison proves:

- foreign-session and retired-set targets reject before local resource lookup;
- failure after preparation preserves the prior visible realization;
- simulated failure after partial provider allocation preserves the prior
  visible realization or transient frame;
- a successful operation becomes visible only at an explicit commit or scoped
  submission boundary;
- a whole-set commit wins over an older in-set candidate or transient
  submission, which then rejects as stale before reused-key lookup;
- no shadow exposes raw provider/backend submission.

## Alternative Results

### B: existing-identity replacement

Alternative B survives the semantic checks. An existing scoped command remains
valid and observes revision `N+1` only after commit. Failed candidates leave
revision `N` authoritative. A whole-set commit invalidates an older update
candidate even when the successor reuses its local key.

This is the smallest surviving persistent-resource semantic for the concrete
console texture.

### C: explicit dynamic class

Alternative C also survives. It rejects an update to a resource not declared
dynamic and commits an update to a declared-dynamic resource using the same
failure and ordering machinery as B.

The shadow exposes its extra cost clearly: the dynamic declaration adds an
eligibility gate, but it does not eliminate B's scoped transaction, failure,
or ordering requirements. The console alone has not earned that additional
resource classification.

### D: transient submission

Alternative D survives at the abstract submission level without persistent
resource identity. Failed preparation/allocation retains the last submitted
frame, and a submission scoped to a retired set rejects.

This does not prove that Tokimu can sample a transient console raster through
the current renderer. No transient texture payload or bounded provider upload
lifetime exists yet. D therefore remains semantically viable but mechanically
unproven for this caller.

## Validation

- `cargo test -p doom-ts-boundary-workbench-engine`: 11 passed, including
  three focused semantic-shadow tests.
- `pwsh -NoProfile -File .\build.ps1` regenerated the browser/WASM bindings,
  passed TypeScript compilation, and retained 6,304,997 emitted startup bytes
  under the 12,582,912-byte limit.
- Browser provider execution belongs to Slice 2 and is not claimed here.

## Architectural Finding

Alternative B is the smallest surviving provider-test candidate. Advancing it
to Slice 2 cannot be accomplished through the current ADR-0018 session: that
session deliberately exposes scoped submission and camera upload but no
same-set texture replacement. A real native/browser experiment therefore needs
one deliberately selected experimental authority shape. Exposing the backend
is disallowed; adding a stable session method would prematurely decide the
contract under review.

The next decision is whether AR-0033 authorizes a feature-gated,
explicitly-experimental set-scoped texture-update transaction in
`tokimu-render` solely for real-provider evidence. That decision should be made
before implementation rather than hidden in the Doom workbench.
