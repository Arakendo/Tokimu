# MIDI Sequencing And Synthesis Provider

## Status

Proposed on 2026-08-02. No MIDI importer, sequencer, synthesizer, live MIDI
provider, or audio-output capability is currently admitted.

The first implementation should incubate in focused corpus libraries and
applications. This plan does not create `tokimu-midi`, `tokimu-audio`, or a
general media framework by itself. Capability admission requires independent
consumer pressure and Architectural Review.

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

## Architectural Thesis

> Applications own musical intent. MIDI semantics own timed musical events.
> Synthesizer providers own sound generation. Audio-output providers own device
> playback.

```text
application cue / score intent
            |
            v
provider-neutral MIDI sequence
notes / programs / controllers / tempo
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

An imported Standard MIDI File is one source of a provider-neutral sequence:

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

- [`Tokimu Software Design Document.md`](../Tokimu%20Software%20Design%20Document.md)
  keeps device and presentation mechanisms outside the trusted simulation
  core.
- [`Tokimu TypeScript Design Document.md`](../Tokimu%20TypeScript%20Design%20Document.md)
  requires TypeScript authoring to lower one-way into Tokimu-owned semantic
  models.
- [`ADR-0001-engine-boundaries.md`](../ADR/ADR-0001-engine-boundaries.md)
  keeps platform mechanisms out of `tokimu-core`.
- [`ADR-0003-capability-ownership-boundary.md`](../ADR/ADR-0003-capability-ownership-boundary.md)
  distinguishes Tokimu-owned semantics from replaceable providers.
- [`ADR-0007-kernel-performance-diagnostics.md`](../ADR/ADR-0007-kernel-performance-diagnostics.md)
  permits bounded performance observations without turning the kernel into an
  audio profiler.
- [`AR-0008-audio-observation-and-visualizer-boundary.md`](../Architectural%20Reviews/AR-0008-audio-observation-and-visualizer-boundary.md)
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

### Provider-neutral MIDI semantics own

- notes, channels, program changes, controller changes, and pitch bend;
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

General MIDI may be a compatibility profile. It is not automatically Tokimu's
instrument ontology.

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

- `corpus/hello-midi-inspect`: headless sequence and timing evidence;
- `corpus/hello-midi-synth`: deterministic software synthesis and PCM output;
- `corpus/hello-midi`: native audible game-music and cue lifecycle proof; and
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

### Slice 0: Boundary Review And Fixture Definition

Deliverables:

- [ ] Open or extend an Architectural Review for MIDI sequencing, synthesis,
      and audio-output ownership.
- [ ] Record the relationship to AR-0008 without treating input analysis and
      output playback as one capability.
- [ ] Define one Tokimu-authored musical fixture and expected event trace.
- [ ] Define explicit event, track, duration, and payload bounds.
- [ ] Document the first synthesis provider and why it is sufficient evidence.

Acceptance criteria:

- [ ] Every proposed type has one named owner.
- [ ] No platform audio, MIDI, or synthesis dependency enters `tokimu-core` or
      `tokimu-runtime`.
- [ ] The fixture can be inspected without a window, GPU, or audio device.

### Slice 1: Provider-Neutral MIDI Event Model

Deliverables:

- [ ] Add bounded sequence, track, event, instrument-requirement, and timing
      types in a corpus incubation library.
- [ ] Represent simultaneous-event ordering explicitly.
- [ ] Add validation for finite values, legal ranges, and sequence bounds.
- [ ] Add structured diagnostics for unsupported or invalid events.
- [ ] Serialize a stable structural observation artifact.

Acceptance criteria:

- [ ] Equal fixtures produce byte-equivalent structural observations.
- [ ] Invalid channels, notes, velocities, tempo, and oversized inputs fail
      explicitly.
- [ ] The public corpus contract contains no file-parser or device-native
      objects.

### Slice 2: Deterministic Sequencer And Transport

Deliverables:

- [ ] Resolve ticks and tempo changes into deterministic sequence time.
- [ ] Implement start, stop, pause, resume, seek, reset, and bounded advance.
- [ ] Implement explicit loop-region semantics.
- [ ] Flush or restore active notes deterministically across discontinuities.
- [ ] Record due, late, skipped, and rejected event observations.

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

- [ ] Implement one small corpus-side oscillator or PSG-style provider.
- [ ] Support bounded polyphony, envelopes, volume, pan, and pitch bend.
- [ ] Produce normalized finite PCM in explicit sample-rate/channel blocks.
- [ ] Diagnose voice stealing, missing instruments, and non-finite output.
- [ ] Emit bounded PCM statistics and a deterministic artifact fingerprint.

Acceptance criteria:

- [ ] A fixed sequence and configuration produce equivalent PCM artifacts on
      repeated runs within the documented numeric policy.
- [ ] Silence, note lifecycle, sustain, polyphony limit, and stop cleanup are
      covered by tests.
- [ ] Synthesis requires no window, GPU, or live audio device.

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

- [ ] Select a bounded native output mechanism after dependency review.
- [ ] Add explicit device-open, sample-rate, latency, underrun, and disconnect
      observations.
- [ ] Add a bounded producer/consumer queue with documented overflow policy.
- [ ] Keep the device callback free of MIDI parsing and application policy.
- [ ] Demonstrate a looping game-style track and interruptible cue.

Acceptance criteria:

- [ ] Live playback uses the same sequencer and synthesizer proven headlessly.
- [ ] Device failure degrades into diagnostics rather than a panic or silent
      fallback.
- [ ] Warm playback does not allocate or create provider resources per sample
      block without explicit evidence and review.

### Slice 7: Game Cue And Transition Consumer

Deliverables:

- [ ] Add an application-owned cue catalog and transition policy in a corpus
      consumer.
- [ ] Exercise music start, loop, replacement, pause, resume, and stop.
- [ ] Exercise one-shot note-driven effects independently of music transport.
- [ ] Record world command, cue decision, sequencer state, and provider state
      as distinct observations.
- [ ] Add deterministic replay of the command sequence without live playback.

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

Start with Slice 0 through Slice 2 only:

1. author one four-bar corpus fixture in code;
2. represent its notes, program requirement, tempo change, sustain event, and
   loop region;
3. advance it with fixed deterministic steps;
4. emit an ordered JSON event trace and summary; and
5. test stop, seek, loop, and simultaneous-event ordering.

That increment can prove whether the MIDI semantic and timing model is useful
before selecting an audio library, synthesis engine, instrument bank, or device
API.
