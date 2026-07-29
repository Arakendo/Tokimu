# Hello UI Font 2 Comparison Notes

Use this record for a visual review run. Keep it alongside the captured image
or link it from the relevant architectural review record.

## Run

- Date:
- Revision:
- Command:
- Prepared corpus root:
- Operating system/backend:
- Viewport:
- Pixel scale:
- Raster column anchor:
- Vector column anchor:

## Fixtures

| Provider | Format | Resolved path | Bytes |
| --- | --- | --- | --- |
| Inter | TTF |  |  |
| JetBrains Mono | OTF |  |  |
| Noto Sans | TTF |  |  |

## Visual Evidence

- Screenshot or saved image:
- Raster/vector baseline comparison:
- Small text result:
- Large text result:

## Checklist

- [ ] Raster and vector headings share the same baseline.
- [ ] Repeated glyphs preserve advance spacing.
- [ ] Counters remain open.
- [ ] Curve-heavy glyphs have no spikes, gaps, or excessive faceting.
- [ ] Small UI text remains legible beside the raster baseline.
- [ ] Large display text remains smooth without semantic displacement.
- [ ] Missing outlines and unsupported presentations are visible in diagnostics.

## Findings

Record the provider, sample, and exact visible failure for every unchecked item.
Do not convert a visual observation into a semantic guarantee without updating
the plan and the relevant architectural review.
