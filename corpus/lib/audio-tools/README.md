# Audio Tools

`audio-tools` is a corpus-incubating library for Tokimu-shaped audio values.
It currently proves two boundaries:

```text
source decoder -> bounded provider-neutral PCM clip
application event -> logical sound request + emission
authored/imported score -> ordered note sequence -> application-clocked transport
```

The library deliberately does not own an audio device, mixer, playback clock,
voice lifecycle, source format, or platform API. It is not an admitted stable
`tokimu-audio` capability. Corpus providers and consumers should use it to
collect evidence until those missing responsibilities have real callers and a
reviewed home.

Current values:

- `PcmClip` retains finite normalized interleaved samples with explicit frame,
  channel, and sample-rate limits.
- `SoundClipKey` identifies application meaning without exposing a source lump
  name or backend handle.
- `SoundEmission` distinguishes listener-relative and world-spatial requests.
- `SoundRequest` combines a logical clip with its emission requirement.
- `NoteSequence` retains a bounded explicit timebase, channel count, duration,
  simultaneous-event order, logical instrument requirements, and note/control
  events without exposing a source parser.
- `SequenceTransport` consumes only caller-supplied time units and makes
  start, pause, resume, finish, and stop/reset transitions explicit.
