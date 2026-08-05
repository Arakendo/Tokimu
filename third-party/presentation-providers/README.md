# Presentation Provider Source Corpora

This directory contains pinned upstream source used to inspect and validate
replaceable presentation-provider boundaries. A source checkout is evidence,
not automatic admission into Tokimu engine crates.

## Ratatui

- Upstream: <https://github.com/ratatui/ratatui>
- Pin: `v0.29.0` (`28732176e1adb4cddba45c2d6b2b27abf7a46f79`)
- Purpose: inspect the embedded backend, buffer, diff, cursor, resize, and test
  contracts used by AR-0013 and the console/Ratatui consumer corpora.

The source review identified two useful boundaries:

```text
Runtime delivery:
Terminal::flush
    -> Buffer::diff
    -> Backend::draw(changed cells)
    -> retained Tokimu cell surface

Diagnostic evidence:
Terminal::draw
    -> CompletedFrame.buffer
    -> complete frame snapshot
```

`TestBackend` remains useful for deterministic tests. The target embedded
provider is a custom backend that consumes Ratatui's changed-cell iterator and
passes a bounded retained surface into Tokimu presentation without making
TypeScript or a browser canvas the owner of cell semantics.
