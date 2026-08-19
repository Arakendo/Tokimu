# CPAL Audio Output Provider

This corpus-incubating provider adapts bounded `audio-tools::PcmClip` values to
a native output device through CPAL. It owns device selection, stream callbacks,
sample-rate/channel adaptation, a bounded command queue, and playback-loss
observations. It does not parse music, synthesize notes, choose gameplay cues,
or expose CPAL device and stream handles to applications.

The audio callback drains a fixed-capacity command queue and mixes from a
preallocated voice table. Queue overflow rejects the newest command explicitly.
Warm callbacks neither allocate nor create provider resources per sample
block. The current standard-library bounded channel is nonblocking at its API
boundary, but this corpus does not yet claim a lock-free or deallocation-free
real-time callback. Silence while a loop is expected is recorded as a
content-starvation callback; CPAL xrun and device-unavailable errors are
recorded separately.

This is native corpus evidence, not an admitted `tokimu-audio` capability.
