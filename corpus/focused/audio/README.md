# Audio Corpus

- `hello-audio-analysis`
- `hello-audio-visualizer`
- `hello-milkdrop`

These entries pressure audio analysis and presentation without making a
particular visualization application engine-owned.

Shared incubation now also includes:

- `corpus/lib/audio-tools` for bounded PCM, logical sound requests, ordered
  note sequences, and caller-clocked transport;
- `corpus/lib/simple-audio-synth-provider` for headless oscillator and WAVE
  artifact evidence;
- `corpus/lib/cpal-audio-output-provider` for corpus-local native device,
  bounded callback queue, mixing, and loss-observation evidence; and
- the Doom campaign's `doom_sound_report`, `doom_music_report`, and
  `doom_audio_playback` consumers for source decoding, headless structure, and
  native lifecycle pressure.
