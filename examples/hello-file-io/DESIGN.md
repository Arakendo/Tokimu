# Hello File IO

`hello-file-io` proves application-owned file reads and writes without adding
filesystem responsibility to `tokimu-core` or `tokimu-runtime`.

The example creates `target/hello-file-io/roundtrip.txt`, writes a UTF-8
payload, reads it back, verifies the bytes, and displays the result.

This is intentionally an example-level proof, not a filesystem service. Future
steps may add missing-file diagnostics, append/overwrite behavior, binary data,
atomic replacement, and a headless persistence contract if independent
consumers justify one.
