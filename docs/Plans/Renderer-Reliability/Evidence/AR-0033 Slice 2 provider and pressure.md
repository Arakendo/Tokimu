# AR-0033 Slice 2 Provider And Pressure Evidence

Date: 2026-08-20

## Scope

This slice tests a feature-gated, texture-only implementation of Alternative B
against native WGPU and browser WebGPU. The operation targets one existing
source texture in the current ADR-0018 resource set. Width, height, color-space
interpretation, material dependency, mesh, pipeline, camera, and command
topology remain fixed.

At the time of this evidence run the implementation was experimental. AR-0033
subsequently accepted the narrow invariant as ADR-0019. The evidence does not
admit general resource mutation, descriptor changes, mesh updates, or a
dynamic-resource class.

## Provider Shape

`WgpuResourceSetSession` exposes two feature-gated experimental operations:

```text
begin fixed-descriptor texture update
    -> allocate texture and dependent provider bindings separately
    -> current set remains untouched

commit candidate
    -> verify provider-session authority
    -> verify current resource-set identity
    -> verify target and fixed descriptor
    -> swap texture realization and dependent provider bindings
```

The session still does not implement `Renderer` or expose `WgpuBackend`.
Provider-internal accessors are visible only inside the WGPU module. Stale set
and wrong provider-session checks precede target texture lookup.

## Native WGPU

The native ADR-0018 fixture passed the complete sequence on Vulkan / AMD Radeon
RX 7900 XTX:

- a prepared update was dropped and the prior one-draw command batch still
  presented;
- a second update committed to resource set 3 and rebound its one dependent
  material;
- the same scoped command batch remained valid and presented one draw;
- 27 further updates committed in the same set, with five prepared candidates
  deliberately dropped first;
- the logical inventory remained one texture, material, mesh, pipeline, camera,
  and two commands;
- a later whole-set commit selected set 4 and the older set-3 update candidate
  rejected as stale before lookup;
- provider diagnostics remained zero.

One retained run measured 242 microseconds for the single update and 1,937
microseconds for the one-resource Alternative-A whole-set transaction. An
earlier run measured 201 and 24,126 microseconds respectively. These are local
CPU interval observations, not performance guarantees; their variation is
retained rather than averaged into a contract.

## Browser WebGPU

The 16x16 procedural-texture probe completed with:

```text
set-A=1
set-after-update=1
set-B=2
failed-update-preserved=true
existing-command-remained-valid=true
initial/failed/update/whole-set draws=8
stale-rejected-before-resource-lookup=true
provider-diagnostics=0
```

The browser timer observed approximately 1 ms for the single update and 2 ms
for an eight-mesh/texture/material Alternative-A transaction. Its millisecond
resolution and browser scheduling make it comparative evidence only.

The separate pressure run used the console's exact 960x264 RGBA8 payload size:

```text
updates=27
preparedDrops=5
resourceSets=1
logicalInventoryStayedFixed=true
sourceBytesPerUpdate=1,013,760
providerDiagnostics=0
elapsedMilliseconds=546.9
physicalGpuReclamation=unobserved
```

Tokimu's external browser terminal observer classified the run `completed` in
1,480 ms and retained the bounded result at:

`target/browser-terminal-observer-44128-1787286933930860900.terminal.json`

The earlier detailed update/order probe was also externally classified
`completed`; its retained result is:

`target/browser-terminal-observer-37952-1787286667874885800.terminal.json`

These records establish page-level terminal closure under ADR-0017. They do
not establish physical GPU reclamation timing or a physical cause beyond what
the provider explicitly reported.

## Independent Caller

The renderer-resource-identity fixture is a non-console procedural-texture
caller. It generates changing RGBA8 content and retains the same texture,
material dependency, set, and commands. It converges with the Doom-console
pressure on fixed-descriptor texture-content replacement.

No independent mesh-content or topology caller was exercised. The evidence
therefore supports keeping mesh updates separate and unadmitted rather than
generalizing a texture transaction into `update_resource`.

## Ordinary Findings Resolved

- The first implementation could not access the session backend across sibling
  Rust modules. Provider-internal accessors fixed module privacy without adding
  a public backend escape.
- The external observer originally retained only completion, not the bounded
  probe result. Completion now accepts optional detail, and the rerun retained
  the complete experiment record.
- The F: volume reached zero free bytes because Rust incremental caches had
  grown beyond 60 GB. Only regenerable repository `target` incremental caches
  were removed; source and evidence were untouched.

## Non-Claims And Gate

- physical WGPU allocation reuse/reclamation is unobserved;
- descriptor changes are unauthorized;
- mesh, material-semantic, pipeline, and camera updates are unauthorized;
- individual resource-handle encoding remains undecided;
- broader resource mutation remains unadmitted.

The implementation evidence reached the admission gate. The maintainer
subsequently selected the narrow fixed-descriptor texture-content transaction;
ADR-0019 records the binding decision and keeps every broader mutation outside
the contract.

## Post-Admission Conformance

After ADR-0019 acceptance, the experimental feature flag was removed and the
same corpora were rebuilt against the ordinary provider-neutral
`RenderTextureContentUpdateLifecycle` surface. The stable preparation method
accepts only the current texture handle and RGBA8 bytes; callers cannot supply
a replacement descriptor.

Native Vulkan/WGPU repeated the full failure, commit, 27-update, whole-set
ordering, and stale-candidate sequence with zero provider diagnostics. The
browser WebGPU pressure run also completed 27 updates with five prepared drops,
one resource set, fixed logical inventory, and zero provider diagnostics. The
external observer retained `contract=ADR-0019` and classified the run
`completed`:

`target/browser-terminal-observer-36132-1787287740036110700.terminal.json`

The final full-workspace test run stopped at
`presentation-geometry-corpus`, without identifying an ADR-0019 failure. An
immediate isolated rerun passed all 26 library tests. This is retained as a
single non-reproduced validation observation rather than reported as a clean
workspace-wide pass or assigned to the texture transaction without evidence.

## References

- `docs/Architectural Reviews/AR-0033-scoped-in-set-presentation-resource-updates.md`
- `docs/ADR/ADR-0018-atomic-staged-render-resource-set-replacement.md`
- `corpus/campaigns/renderer-reliability/hello-render-resource-identity/`
- `corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/`
- `corpus/campaigns/renderer-reliability/hello-browser-terminal-observer/`
