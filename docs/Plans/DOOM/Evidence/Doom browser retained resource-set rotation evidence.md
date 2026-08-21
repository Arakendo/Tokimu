# Doom Browser Retained Resource-Set Rotation Evidence

Date: 2026-08-20

## Claim

The Doom browser working model can replace E1M1 through E1M9 repeatedly through
one ADR-0018 resource-set session while preserving the current map across a
late successor-preparation failure.

This is logical transaction and cross-target presentation evidence. It does
not measure physical GPU overlap or reclamation timing.

## Observed run

External terminal observer run `42344-1787260772989661700` reported:

```text
classification=completed
operation=doom-retained-session-rotation
completedReplacements=27
requestedReplacements=27
elapsedMilliseconds=19217.091
subjectStarted=true
```

The page-side record supplied by the operator independently reports 27/27 in
18,137.4 ms. The difference is expected observer startup/terminal-record
overhead, not a second renderer timing claim.

The page loaded the reviewed local shareware package through an explicit
loopback-only autorun parameter. The bytes entered the same bounded Rust/WASM
intake used by the file picker; TypeScript did not parse package or WAD
semantics.

## Failure containment

After E1M1 presented, the harness fully prepared E1M2 on the CPU and injected
a failure before provider staging:

```text
candidate-map=E1M2
forced-failure=before-provider-staging
preserved-map=E1M1
preserved-resource-set=1
last-known-good-presented=true
backend-creations=1
device-creations=1
surface-creations=1
```

The subsequent real E1M2 candidate then staged and committed normally. The
failure attempt increased replacement attempts but not successful
presentations.

## Final frame and lifetime evidence

The twenty-seventh presentation was E1M9:

```text
strategy=global-full-plus-grouped-sky-parity
opaque=1151
cutouts=20
skywalls=0
sky-planes=255
surface-triangles=2330
edge-conformance-insertions=386
backend-creations=1
device-creations=1
surface-creations=1
replacement-attempts=28
replacements-presented=27
retired-logical-sets=26
retained-provider-session=true
provider resource-set authority=ADR-0018
physical GPU reclamation=unobserved
```

The final E1M8-to-E1M9 commit retired and installed complete mesh, texture,
material, pipeline, camera, and queued-draw inventories. Logical resource
handles were reused only inside the successor set; command submission remained
set-scoped.
The WGPU instance-binding cache reached and retained a bounded high-water mark
of 5,358 bindings during this corpus run; it did not grow with every later map
after reaching the largest submitted scene. This is logical provider evidence,
not a physical allocation or reclamation measurement.

## Terminal-outcome evidence

The observer ran outside the page failure domain and retained a completed
terminal record at:

```text
target/browser-terminal-observer-42344-1787260772989661700.terminal.json
```

The Edge launcher handed off to the page-hosting browser process, so the
observer correctly used page identity, heartbeats, and the explicit terminal
event rather than claiming ownership of the handed-off process. No physical
failure cause is inferred.

## Disposition

- Doom browser map replacement now uses the admitted resource-set transaction.
- Current-map camera updates remain in-set operations and do not reopen raw
  command submission.
- The older whole-backend and adapter-private reset paths are no longer the
  working-model replacement mechanism.
- Physical GPU reclamation timing, peak physical overlap, device-loss recovery,
  and individual handle encoding remain outside this evidence.

## Follow-up visual acceptance

The operator subsequently exercised the browser working-model walkabout and
map switching and accepted the observation on 2026-08-20. One E1M2 join where
a lower ceiling meets the wall below a higher ceiling exposes a bounded
hairline, predominantly white sky crack. It did not widen, flicker, or reveal
recognizable remote geometry during the accepted inspection.

This is retained as a presentation-tolerance limitation, not claimed as Doom
pixel parity. Reopen edge conformance if the crack becomes wider than a
hairline, changes under ordinary movement, or exposes recognizable sky or
unrelated geometry. The operator acceptance is visual evidence; no additional
external terminal record was supplied for this manual session.
