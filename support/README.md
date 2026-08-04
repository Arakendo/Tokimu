# Tokimu Support Libraries

`support/` is the home for reusable host-facing integration libraries that
help applications consume Tokimu through a concrete ecosystem.

Examples may eventually include:

- `avalonia/` for .NET desktop hosts;
- `capacitor/` for TypeScript and mobile WebView hosts;
- `flutter/` for Dart and mobile hosts.

This directory is not an engine dependency bucket. `tokimu-core` and
`tokimu-runtime` must never depend on a support library, its host framework,
or its build tooling. A support library adapts an already-public Tokimu
contract to a host lifecycle, input surface, package system, or presentation
mechanism.

Consumer applications remain under `corpus/consumers/`. They provide the
evidence that a host bridge is useful and expose friction before a reusable
support library is stabilized. External upstream source remains under
`third-party/`, not here.

No host implementation is admitted to this directory until a concrete consumer
corpus demonstrates a repeatable host-side need.

See `docs/Plans/support-library-host-adapters.md` for the admission plan.
