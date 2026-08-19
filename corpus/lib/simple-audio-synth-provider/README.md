# Simple Audio Synth Provider

This corpus provider converts bounded `audio-tools::NoteSequence` values into
bounded stereo PCM using a deliberately small oscillator implementation. It is
headless evidence for sequencing and PCM handoff, not a General MIDI,
SoundFont, OPL, or production-quality music provider.

Instrument requirements are observed but currently use one common triangle
oscillator. That substitution is reported. Device playback and clocks remain
outside this crate.

For inspection and listening, `encode_pcm16_wave` serializes a resolved clip as
a canonical PCM16 RIFF/WAVE artifact. WAVE remains an optional evidence format,
not the runtime handoff.

The provider supports bounded polyphony, deterministic oldest-voice stealing,
volume, expression, pan, pitch bend, all-notes/all-sounds-off, and controller
reset. Other controls remain visible as ignored-control observations. The PCM
fingerprint is repeatable under the current numeric implementation but is not
yet claimed as a cross-architecture bit-exact contract.
