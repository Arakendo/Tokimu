# Doom Audio Provider

`doom-audio-provider` is a corpus-private source adapter. It resolves Doom WAD
sound-effect lumps using replacement-friendly last-match lookup, validates the
format-3 header and configured limits, and exposes unsigned eight-bit mono
samples. It can lower those samples to `audio_tools::PcmClip` without choosing
resampling, mixing, playback, or device policy.

Gameplay event selection stays in the Doom application. A future playback
provider must consume the provider-neutral request and clip values rather than
learning Doom lump names or formats.
