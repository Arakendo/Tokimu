# Doom Audio Provider

`doom-audio-provider` is a corpus-private source adapter. It resolves Doom WAD
sound-effect lumps using replacement-friendly last-match lookup, validates the
format-3 header and configured limits, and exposes unsigned eight-bit mono
samples. It can lower those samples to `audio_tools::PcmClip` without choosing
resampling, mixing, playback, or device policy.

The provider also parses bounded MUS scores and lowers their 140 Hz source
timeline into `audio_tools::NoteSequence`. Source channels, event ordering,
logical instrument requirements, controls, pitch bends, offsets, and malformed
input diagnostics remain provider-owned. The parser behavior was cross-checked
against [Chocolate Doom's GPL `src/mus2mid.c` reference](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/mus2mid.c);
this crate contains an independent Rust implementation and does not emit an
intermediate MIDI file.

Gameplay event selection stays in the Doom application. A future playback
provider must consume the provider-neutral request and clip values rather than
learning Doom lump names or formats.
