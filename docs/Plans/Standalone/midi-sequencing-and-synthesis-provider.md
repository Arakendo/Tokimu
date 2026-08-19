# MIDI Sequencing And Synthesis Provider

## Status

In progress. Proposed on 2026-08-02, refined on 2026-08-06 around a first
audible application milestone, and given its first headless sequence,
transport, MUS-import, and synthesis evidence on 2026-08-19. No Standard MIDI
File importer, live MIDI provider, production synthesizer, or audio-output
capability is currently admitted.

The first implementation should incubate in focused corpus libraries and
applications. This plan does not create `tokimu-midi`, `tokimu-audio`, or a
general media framework by itself. Capability admission requires independent
consumer pressure and Architectural Review.

The immediate implementation target is intentionally smaller than the full
plan: authored note events, deterministic sequencing, one tiny software
synthesizer, a WAVE evidence artifact, and one native playback consumer. MIDI
file import, SoundFonts, hardware MIDI, spatial audio, streaming, and browser
playback remain later slices.

## Purpose

Tokimu should support compact, older-style music and sound effects for games,
simulations, tools, and browser experiences without making a particular MIDI
file parser, software synthesizer, instrument bank, or platform audio API part
of the engine's trusted core.

The initial target is a deliberately bounded MIDI pipeline suitable for:

- game music loops and transitions;
- short musical cues and note-driven sound effects;
- deterministic replay and corpus inspection;
- native and future WASM playback;
- provider comparison; and
- optional reuse of synthesized PCM by the existing audio-analysis seam.

MIDI is timed control data, not audio. A MIDI sequence can be inspected and
scheduled without a speaker, while audible output additionally requires a
synthesis provider and an audio-output provider.

## Working Meaning Of "MIDI-Style"

For the first milestone, **MIDI-style audio** means compact, timed note and cue
events driving a small synthesizer. It does not yet mean complete Standard MIDI
File compatibility, General MIDI instruments, a hardware MIDI port, or a
SoundFont player.

The first public vocabulary should therefore describe the semantics Tokimu
actually needs:

```text
NoteSequence
NoteEvent
InstrumentRequirement
Cue
TransportState
PcmBlock
```

It should not acquire `Midi*` names unless the type truly promises MIDI
protocol or file-format behavior. A later MIDI importer may lower `.mid` bytes
into the same note-sequence model, and a later system MIDI provider may consume
compatible events, without either technology defining the application-facing
contract.

This distinction keeps the first implementation useful for basic applications
even if full MIDI compatibility never becomes an admitted Tokimu capability.

## Architectural Thesis

> Applications own musical intent. MIDI semantics own timed musical events.
> Synthesizer providers own sound generation. Audio-output providers own device
> playback.

```text
application cue / score intent
            |
            v
provider-neutral note sequence
notes / instruments / controls / tempo
            |
            v
deterministic sequencer
timed event dispatch / loop policy
            |
            v
synthesis requirement
instrument identity / polyphony / sample rate
            |
            v
synthesizer provider
oscillator / sampler / SoundFont / future hardware
            |
            v
bounded PCM stream
            |
            +--------------------+
            |                    |
            v                    v
audio-output provider       audio analysis
native / browser            optional observer
```

An imported Standard MIDI File is one possible source of a provider-neutral
note sequence:

```text
MIDI file bytes
    -> MIDI importer
    -> validated sequence
    -> sequencer
```

A system MIDI port is a separate optional execution provider:

```text
sequenced MIDI events
    -> system MIDI output adapter
    -> external synthesizer
```

Neither route defines the semantic contract alone.

## Governing Documents

- [`On Audio.md`](../../Conversations/On%20Audio.md) decomposes audio assets,
  playback, spatialization, mixing, effects, and streaming rather than treating
  them as one `play sound` subsystem. This plan intentionally begins with only
  note sequencing, synthesis, and bounded playback.

- [`Tokimu Software Design Document.md`](../../Tokimu%20Software%20Design%20Document.md)
  keeps device and presentation mechanisms outside the trusted simulation
  core.
- [`Tokimu TypeScript Design Document.md`](../../Tokimu%20TypeScript%20Design%20Document.md)
  requires TypeScript authoring to lower one-way into Tokimu-owned semantic
  models.
- [`ADR-0001-engine-boundaries.md`](../../ADR/ADR-0001-engine-boundaries.md)
  keeps platform mechanisms out of `tokimu-core`.
- [`ADR-0003-capability-ownership-boundary.md`](../../ADR/ADR-0003-capability-ownership-boundary.md)
  distinguishes Tokimu-owned semantics from replaceable providers.
- [`ADR-0007-kernel-performance-diagnostics.md`](../../ADR/ADR-0007-kernel-performance-diagnostics.md)
  permits bounded performance observations without turning the kernel into an
  audio profiler.
- [`AR-0008-audio-observation-and-visualizer-boundary.md`](../../Architectural%20Reviews/AR-0008-audio-observation-and-visualizer-boundary.md)
  separates PCM acquisition, analysis, visualizer meaning, and renderer
  execution. MIDI output and synthesis are missing evidence, not implied parts
  of the current audio-analysis incubation.
- [`audio-reactive-visualizers-and-milkdrop-compatibility.md`](audio-reactive-visualizers-and-milkdrop-compatibility.md)
  may consume synthesized PCM later, but does not own MIDI sequencing or
  playback.

## Architectural Questions

The corpus must answer these questions before capability admission:

1. Which event, timing, transport, and diagnostic semantics remain stable
   across generated sequences, MIDI files, software synthesis, and platform
   MIDI output?
2. Does a sequencer belong with provider-neutral MIDI meaning, application
   playback policy, or an audio execution layer?
3. Where should tempo conversion stop and runtime scheduling begin?
4. Which instrument identity is portable without making General MIDI or a
   SoundFont bank the Tokimu semantic model?
5. Can native and WASM synthesis consume the same sequence and produce
   equivalent structural observations?
6. Which latency, underrun, voice-limit, and device-loss facts belong to the
   synthesizer versus the audio-output provider?
7. Can synthesized PCM feed the existing analysis seam without coupling MIDI
   semantics to visualizers?

## Ownership Boundaries

### Application owns

- why a cue is played;
- cue selection, transitions, and gameplay meaning;
- pause, resume, stop, loop, and replacement policy;
- whether musical time follows simulation, presentation, or wall time;
- user-facing volume, mute, and accessibility policy; and
- permission to access live MIDI or audio devices.

The application must not parse provider-native device handles or redefine MIDI
file syntax.

### Provider-neutral note-sequence semantics own

- notes, voices/channels, instrument requirements, selected controls, and
  pitch bend;
- explicit tempo and time-signature observations where available;
- bounded tracks and deterministic event ordering;
- sequence identity, duration, and source diagnostics;
- stable instrument requirements rather than provider-native bank objects; and
- explicit unsupported-event reporting.

This layer does not own PCM, audio devices, SoundFont parser objects, platform
ports, or gameplay state.

### MIDI importer owns

- bounded byte parsing;
- variable-length quantities, running status, tracks, and event framing;
- source offsets and import diagnostics;
- conversion from source ticks into the provider-neutral sequence model; and
- explicit rejection of unsupported or malformed input.

The importer is a replaceable provider. MIDI file syntax must not leak into
the sequencer or renderer.

### Sequencer owns

- deterministic mapping from sequence time to due events;
- stable ordering for simultaneous events;
- bounded event dispatch;
- seek, reset, and explicit loop-boundary behavior;
- active-note cleanup on stop, seek, or provider failure; and
- lateness and discontinuity observations.

The sequencer does not own audio-device callbacks, gameplay decisions, or
wall-clock acquisition.

### Synthesizer provider owns

- voice generation and mixing;
- oscillator, envelope, filter, sampler, or bank implementation;
- voice allocation and stealing;
- instrument-bank resolution;
- output sample rate and channel rendering; and
- synthesis-specific diagnostics.

It consumes resolved MIDI events and produces bounded PCM. Its parser objects,
sample-bank handles, and DSP state must not leak into application semantics.

### Audio-output provider owns

- native device or browser audio mechanisms;
- callback and buffer lifecycle;
- output latency and sample-rate facts;
- underrun, disconnection, permission, and device-loss diagnostics; and
- adaptation from bounded PCM delivery into the platform mechanism.

It must not infer tempo, instrument identity, cue policy, or gameplay meaning.

### Asset capability owns

- identities and bounded byte access for MIDI files and admitted instrument
  banks;
- provenance and license metadata; and
- replacement or reload observations.

It does not interpret MIDI events or synthesize audio.

### Runtime and kernel own

- only existing time, command, and diagnostic semantics needed by callers;
- no MIDI file parsing;
- no synthesizer or audio-device dependency;
- no hidden global transport; and
- no claim that music playback is simulation truth.

If an application makes music state authoritative, it must do so through
explicit world state and commands rather than through provider playback state.

## Initial Semantic Scope

The first sequence model should support only evidence-backed essentials:

- note on and note off;
- channel and velocity;
- program selection through a provider-neutral instrument requirement;
- channel volume and pan;
- sustain pedal;
- pitch bend;
- tempo changes;
- bounded multitrack ordering;
- explicit duration and loop regions; and
- end-of-sequence behavior.

General MIDI may become a compatibility profile. It is not automatically
Tokimu's instrument ontology.

### Phase A application scope

The first audible application proof needs less than the complete semantic
scope above. It should support:

- note on and note off;
- velocity;
- one provider-neutral instrument requirement resolved to a built-in waveform;
- explicit tempo;
- deterministic event ordering;
- bounded polyphony;
- one loop region;
- start, pause, resume, and stop; and
- one music cue plus one independently triggered sound-effect cue.

Program changes, pan, sustain, pitch bend, multitrack import, and richer
controller behavior remain in the broader sequence model but must not block
the first audible proof.

## Non-Goals

The initial work does not attempt to provide:

- a digital audio workstation;
- notation editing or engraving;
- MIDI 2.0 or universal MIDI packet support;
- arbitrary plugin hosting, VST, or Audio Unit compatibility;
- exact emulation of historical synthesizer hardware;
- professional live-performance latency guarantees;
- network MIDI;
- spatial audio;
- compressed media playback;
- a universal audio graph; or
- automatic promotion into `tokimu-core` or `tokimu-runtime`.

## Corpus Strategy

The proof should begin without a device:

```text
Tokimu-authored sequence fixture
    -> deterministic sequencer
    -> ordered event artifact
    -> structural comparison
```

Then add synthesis without live playback:

```text
ordered events
    -> deterministic software synthesizer
    -> bounded PCM artifact
    -> statistics / fingerprint / optional WAVE fixture
```

Only after those seams are stable should a live output provider be added.

Candidate corpus applications:

- `corpus/focused/audio/hello-midi-inspect`: headless sequence and timing evidence;
- `corpus/focused/audio/hello-midi-synth`: deterministic software synthesis and PCM output;
- `corpus/focused/audio/hello-midi`: native audible game-music and cue lifecycle proof; and
- a later website Lab island for browser synthesis and interaction.

The first musical fixture should be intentionally small: a short melody,
bass line, percussion-like cue, tempo change, sustain event, and loop boundary.
It should be generated or authored in-repository so expected semantics are
known exactly.

## External Assets And Licensing

No random MIDI collection or instrument bank should enter the repository.

Before admitting external fixtures:

- record upstream URL and exact revision or release;
- preserve the original license text;
- distinguish fixture redistribution rights from software-library licensing;
- record hashes for selected files;
- keep instrument-bank provenance separate from MIDI sequence provenance; and
- reject any SoundFont or General MIDI bank whose redistribution rights are
  unclear.

A tiny Tokimu-authored oscillator or PSG-style synthesizer is the preferred
first audible provider because it proves sequencing and PCM handoff without
making bank licensing a prerequisite. SoundFont support can follow as an
independent provider.

## Implementation Slices

### Implementation Order

Slice numbers describe architectural concerns, not a requirement to complete
every slice numerically. The first useful path is:

```text
Slice 0: boundary and authored fixture
    -> Slice 1: bounded note events
    -> Slice 2: deterministic transport
    -> Slice 4: tiny software synthesizer
    -> Slice 6: native output adapter
    -> Slice 7: basic application cue consumer
```

Slice 3 (MIDI file import) is deliberately deferred until the authored event
and synthesis path is audible and structurally tested. This prevents file
format work from delaying useful application audio or defining the semantic
model accidentally.

The first implementation may incubate in a focused `corpus/lib` library. It
must not create a permanent engine crate merely to make the corpus directory
look tidy.

### Slice 0: Boundary Review And Fixture Definition

Deliverables:

- [x] Open or extend an Architectural Review for MIDI sequencing, synthesis,
      and audio-output ownership.
- [x] Record the relationship to AR-0008 without treating input analysis and
      output playback as one capability.
- [x] Define one Tokimu-authored musical fixture and expected event trace.
- [x] Define explicit event, channel, duration, timebase, and payload bounds.
      Track identity remains a later multitrack refinement.
- [x] Document the first synthesis provider and why it is sufficient evidence.

Acceptance criteria:

- [x] Every implemented type has one named owner.
- [x] No platform audio, MIDI, or synthesis dependency enters `tokimu-core` or
      `tokimu-runtime`.
- [x] The fixture can be inspected without a window, GPU, or audio device.

### Slice 1: Provider-Neutral Note Event Model

Deliverables:

- [ ] Add bounded sequence, track, event, instrument-requirement, and timing
      types in a corpus incubation library.
- [x] Represent simultaneous-event ordering explicitly.
- [x] Add validation for finite values, legal ranges, and sequence bounds.
- [x] Add structured diagnostics for unsupported or invalid events.
- [ ] Serialize a stable structural observation artifact.

Acceptance criteria:

- [ ] Equal fixtures produce byte-equivalent structural observations.
- [ ] Invalid channels, notes, velocities, tempo, and oversized inputs fail
      explicitly.
- [ ] The public corpus contract contains no file-parser or device-native
      objects.
- [ ] The application-facing contract does not claim MIDI file or hardware
      semantics merely because its note model is MIDI-compatible.

### Slice 2: Deterministic Sequencer And Transport

Deliverables:

- [ ] Resolve ticks and tempo changes into deterministic sequence time.
- [ ] Implement start, stop, pause, resume, seek, reset, and bounded advance.
- [ ] Implement explicit loop-region semantics.
- [ ] Flush or restore active notes deterministically across discontinuities.
- [ ] Record due, late, skipped, and rejected event observations.

Current refinement: fixed-rate sequence time, start/stop/pause/resume/reset,
bounded advance, and exact dispatch conservation are implemented. Tempo
changes, seek, looping, active-note restoration, and lateness observations
remain open.

Acceptance criteria:

- [ ] Identical step sequences dispatch identical ordered MIDI events.
- [ ] A tempo change and loop boundary have deterministic expected traces.
- [ ] No wall clock is required by the sequencer.
- [ ] Application-owned time can advance the transport in fixed steps.

### Slice 3: Bounded MIDI File Importer

Deliverables:

- [ ] Parse a deliberately scoped Standard MIDI File subset.
- [ ] Preserve source offsets and track identity in diagnostics.
- [ ] Support bounded tracks, tempo events, running status, and variable-length
      quantities needed by admitted fixtures.
- [ ] Reject malformed lengths, truncated events, oversized tracks, and
      unsupported divisions explicitly.
- [ ] Compare imported and generated sequences through the same observation
      format.

Acceptance criteria:

- [ ] Imported fixtures produce the expected provider-neutral event trace.
- [ ] Malformed fixtures fail without panic or unbounded allocation.
- [ ] File syntax stops at the importer boundary.

### Slice 4: Deterministic Software Synthesizer

Deliverables:

- [x] Implement one small corpus-side oscillator or PSG-style provider.
- [ ] Support bounded polyphony, envelopes, volume, pan, and pitch bend.
- [x] Produce normalized finite PCM in explicit sample-rate/channel blocks.
- [ ] Diagnose voice stealing, missing instruments, and non-finite output.
- [x] Emit bounded PCM statistics and a deterministic artifact fingerprint.
- [x] Write one canonical PCM result as a simple WAVE artifact for listening
      and inspection without making WAVE the runtime audio contract.

Current refinement: bounded polyphony, deterministic oldest-voice stealing,
volume, expression, pan, pitch bend, finite-output validation, explicit
instrument substitution, and optional canonical PCM16 WAVE output are
implemented. Envelopes and missing-instrument rejection remain open.

Acceptance criteria:

- [ ] A fixed sequence and configuration produce equivalent PCM artifacts on
      repeated runs within the documented numeric policy.
- [ ] Silence, note lifecycle, sustain, polyphony limit, and stop cleanup are
      covered by tests.
- [ ] Synthesis requires no window, GPU, or live audio device.
- [ ] The WAVE artifact records sample rate, channel count, duration, peak,
      clipping count, and source-fixture identity beside its fingerprint.

### Slice 5: PCM Handoff And Audio Analysis Reuse

Deliverables:

- [ ] Adapt synthesized PCM into the bounded PCM-window seam incubating under
      AR-0008.
- [ ] Preserve sample rate, channels, sequence time, and provenance.
- [ ] Produce one spectrum or band observation from synthesized music.
- [ ] Keep MIDI, synthesis, analysis, and visualizer diagnostics separately
      attributable.

Acceptance criteria:

- [ ] Audio analysis consumes PCM without learning MIDI event semantics.
- [ ] The synthesizer does not depend on a visualizer or renderer.
- [ ] Failures identify the first owning boundary that diverged.

### Slice 6: Native Audio-Output Provider

Deliverables:

- [x] Select a bounded native output mechanism after dependency review.
- [x] Add explicit device-open, sample-rate, latency, underrun, and disconnect
      observations.
- [x] Add a bounded producer/consumer queue with documented overflow policy.
- [x] Keep the device callback free of MIDI parsing and application policy.
- [x] Demonstrate a looping game-style track and interruptible cue.
- [x] Expose a minimal application adapter for play, pause, resume, stop, and
      one-shot cue requests without exposing the device handle.

Dependency decision and evidence (2026-08-19):

- `cpal` 0.18 is admitted only to the corpus-local
  `cpal-audio-output-provider`. It supplies low-level device discovery, stream
  creation, callbacks, and typed samples without choosing a decoder,
  synthesizer, mixer graph, or gameplay policy. No engine crate depends on it.
- The provider uses a fixed-capacity standard-library synchronous command
  channel. Overflow rejects the newest command and increments a bounded
  observation. The callback drains commands without waiting, mixes through a
  preallocated voice table, and uses stack scratch storage rather than a
  per-block heap allocation.
- Source PCM is adapted to the selected sample rate with a deliberately simple
  nearest-frame policy and mapped into the device channel count. This proves
  ownership and lifecycle, not production resampling quality.
- The native Doom consumer opened `Line (2- Yamaha AG03MK2)` through the
  default Windows host at 44,100 Hz, stereo `f32`, with a reported 441-frame
  buffer (nominal 10,000 microseconds). It played a looping five-second
  synthesis of `D_E1M1`, mixed an independent `DSPISTOL` cue, paused, resumed,
  and stopped over 235 callbacks / 104,164 frames with zero content-starvation,
  rejected-command, xrun, device-unavailable, or other device-error reports.
  Manual listening confirmed that the independently queued pistol cue was
  audible over the music path. The retained consumer also prepares and queues
  a separate one-note cue through the same provider-neutral sequence and
  synthesis path; neither cue is parsed in the device callback.
- CPAL reports xruns and invalidated/unavailable devices through the error
  callback. The provider counts those separately and retains the last error;
  device-open/configuration/play/pause failures return typed errors.
- This corpus does not yet claim that `std::sync::mpsc::sync_channel` is a
  production lock-free or deallocation-free real-time queue. Replacement with
  a reviewed real-time handoff remains a promotion concern, not a hidden claim.

Acceptance criteria:

- [x] Live playback uses the same sequencer and synthesizer proven headlessly.
- [x] Device failure degrades into diagnostics rather than a panic or silent
      fallback.
- [x] Warm playback does not allocate or create provider resources per sample
      block without explicit evidence and review.
- [x] A basic corpus application can start music and trigger a note-driven
      effect without parsing MIDI or managing PCM buffers itself.

### Slice 7: Game Cue And Transition Consumer

Deliverables:

- [ ] Add an application-owned cue catalog and transition policy in a corpus
      consumer.
- [ ] Exercise music start, loop, replacement, pause, resume, and stop.
- [ ] Exercise one-shot note-driven effects independently of music transport.
- [ ] Record world command, cue decision, sequencer state, and provider state
      as distinct observations.
- [ ] Add deterministic replay of the command sequence without live playback.

Current refinement: the Doom walkabout now provides an opt-in live consumer.
It selects the current map's music, requests decoded pistol and monster-alert
cues from existing gameplay events, and keeps device/cue failure
non-authoritative. The standalone audible proof exercises pause, resume, and
stop. Music replacement across map rotation, application-command replay, and a
fully provider-neutral cue catalog remain open, so this slice is not complete.

Acceptance criteria:

- [ ] Gameplay meaning remains application-owned.
- [ ] Replaying application commands reproduces the structural cue/event trace.
- [ ] Provider playback state is not treated as authoritative world state.

### Slice 8: Browser And WASM Provider

Deliverables:

- [ ] Lower the same sequence and synthesis contract through a browser audio
      mechanism.
- [ ] Require an explicit user gesture where the browser requires one.
- [ ] Report suspended, denied, disconnected, and resumed lifecycle states.
- [ ] Keep TypeScript focused on user interaction and host adaptation.
- [ ] Add a website Lab consumer only after the boundary is stable.

Acceptance criteria:

- [ ] TypeScript does not parse MIDI files or synthesize notes independently.
- [ ] Native and WASM runs expose equivalent structural event observations.
- [ ] Browser lifecycle differences are explicit diagnostics, not hidden
      semantic differences.

### Slice 9: Optional SoundFont And System MIDI Providers

Deliverables:

- [ ] Evaluate a SoundFont provider only after licensed fixture provenance is
      settled.
- [ ] Map provider-neutral instrument requirements into bank/program results.
- [ ] Diagnose missing bank, missing program, unsupported generator, and sample
      limit conditions.
- [ ] Evaluate system MIDI output as a separate optional provider.
- [ ] Preserve the software synthesizer as the deterministic reference path.

Acceptance criteria:

- [ ] Provider replacement does not change sequence semantics.
- [ ] No SoundFont parser, sample-bank, or platform port type leaks upward.
- [ ] External-device output is never required for automated correctness tests.

### Slice 10: Performance And Resource Evidence

Deliverables:

- [ ] Record bounded event-dispatch, synthesis-block, queue, underrun, voice,
      and allocation observations.
- [ ] Separate cold startup and bank load from warm playback.
- [ ] Exercise sustained polyphony and cue-transition stress fixtures.
- [ ] Add application-owned budgets only after representative measurements.
- [ ] Emit kernel-native performance diagnostics only for already admitted
      diagnostic semantics.

Acceptance criteria:

- [ ] Measurements identify their producer and observation scope.
- [ ] Warm-path resource churn is visible and bounded.
- [ ] Performance observations do not become portable guarantees by accident.

### Slice 11: Admission Review

Deliverables:

- [ ] Compare generated sequence, imported file, native synthesis, browser
      synthesis, and optional provider evidence.
- [ ] Record which event, timing, diagnostic, and PCM contracts repeated.
- [ ] Decide whether MIDI semantics, sequencing, synthesis requirements, or
      audio output deserve separate capabilities.
- [ ] Update the SDD and create or revise ADRs for accepted boundaries.
- [ ] Retain rejected and deferred alternatives in the Architectural Review.

Acceptance criteria:

- [ ] Promotion is based on independent consumers rather than one demo.
- [ ] Crate extraction follows stable ownership rather than preceding it.
- [ ] Any admitted API remains provider-neutral and headlessly testable where
      meaningful.

## Diagnostic Vocabulary

The corpus should distinguish at least:

- malformed or unsupported MIDI source;
- event or track limit exceeded;
- invalid tempo or timing division;
- unknown instrument requirement;
- instrument bank or program unavailable;
- polyphony limit and voice stealing;
- late, skipped, duplicated, or reordered event;
- active-note cleanup after seek or stop;
- PCM queue overflow or underrun;
- audio device unavailable, suspended, or disconnected;
- browser permission or gesture requirement; and
- provider replacement or reload.

Diagnostics must identify the owning boundary and remain bounded. They should
not dump unbounded event streams or sample data.

## Validation Matrix

| Evidence | Headless | Native | WASM | Authoritative claim |
| --- | --- | --- | --- | --- |
| Generated sequence trace | yes | yes | yes | MIDI semantics and ordering |
| Imported MIDI trace | yes | yes | yes | importer compatibility |
| Deterministic PCM artifact | yes | yes | planned | synthesis correctness |
| Live speaker playback | no | yes | planned | provider lifecycle only |
| System MIDI output | no | optional | optional | adapter compatibility only |
| Audio-analysis observation | yes | yes | planned | PCM seam reuse |

Audible quality is useful review evidence but is not a substitute for structural
event and PCM assertions.

## Risks

### MIDI becomes synonymous with one file format

Mitigation: keep generated event sequences and file import as separate sources
of the same semantic model.

### General MIDI becomes Tokimu's permanent instrument ontology

Mitigation: treat bank/program mapping as a compatibility profile resolved by
providers.

### Device timing leaks into deterministic sequencing

Mitigation: advance the sequencer with explicit caller-owned time and adapt its
output to device buffers afterward.

### SoundFont licensing blocks reproducible fixtures

Mitigation: begin with a Tokimu-authored oscillator provider and admit banks
only with recorded redistribution rights and hashes.

### Live playback hides structural failures

Mitigation: make headless event and PCM artifacts authoritative for automated
validation.

### Audio scope expands into a DAW or universal graph

Mitigation: keep the first proof to game cues, compact music, bounded synthesis,
and explicit provider seams.

## Graduation Criteria

The work is ready for admission review when:

- generated and imported sequences share one deterministic semantic model;
- at least two independent consumers use the sequence/sequencer contracts;
- one headless synthesis provider and one live output provider consume the same
  event stream;
- native and WASM structural observations agree where both exist;
- provider failures and resource bounds are explicit;
- no device, parser, SoundFont, or browser objects leak into semantic APIs;
- the audio-analysis seam can consume synthesized PCM without MIDI coupling;
  and
- external fixture and instrument-bank provenance is documented.

## Suggested First Increment

Build the shortest path to an audible, inspectable application proof:

1. author one four-bar corpus fixture in code;
2. author one short sound-effect cue using the same note-event contract;
3. represent their notes, instrument requirements, tempo, and loop region;
4. advance them with fixed deterministic steps;
5. synthesize them through a tiny Tokimu-authored oscillator provider;
6. emit ordered event JSON plus a deterministic PCM fingerprint and WAVE
   artifact; and
7. play the looping music and interruptible cue through one native output
   adapter controlled by a basic corpus application.

MIDI file import is not part of this first increment. It begins only after the
authored-note path proves the semantic model, scheduling, synthesis, artifact,
and application-control boundaries.

## Phase A Definition Of Done

Phase A is complete when:

- [ ] Headless tests prove deterministic event ordering, looping, transport,
      note cleanup, and PCM fingerprints.
- [ ] The generated WAVE artifact is non-silent, finite, bounded, and reports
      clipping explicitly.
- [ ] A basic native corpus application can start, pause, resume, and stop one
      music cue and trigger one overlapping sound-effect cue.
- [ ] The application supplies musical intent without parsing MIDI, choosing
      an audio device, or managing PCM buffers.
- [ ] The synthesizer and output adapter can be tested independently through
      the shared note-event and PCM handoffs.
- [ ] Missing output devices, unsupported instrument requirements, queue
      pressure, and invalid events produce bounded diagnostics rather than
      silent fallback.
- [ ] No claim is made yet for MIDI files, General MIDI, hardware MIDI,
      SoundFonts, spatial audio, streaming, browser playback, or production
      mixing.
