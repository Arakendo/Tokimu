# Corpus Campaigns

This directory groups executable corpus evidence by sustained work campaign.
The names intentionally mirror the portfolios under [`docs/Plans/`](../../docs/Plans/README.md).

| Campaign | Plan portfolio | Executable evidence |
| --- | --- | --- |
| [DOOM](doom/README.md) | [DOOM plans](../../docs/Plans/DOOM/README.md) | WAD inspection and E1M1 reconstruction |
| [Native Math](native-math/README.md) | [Native Math plans](../../docs/Plans/Native-Math/README.md) | bulk-compute browser evidence |
| [Coordinate Conformance](coordinate-conformance/README.md) | [Coordinate Conformance plans](../../docs/Plans/Coordinate-Conformance/README.md) | native and browser orientation fixtures |
| [Renderer Reliability](renderer-reliability/README.md) | [Renderer Reliability plans](../../docs/Plans/Renderer-Reliability/README.md) | native and browser resource-identity fixtures |
| [Textured Presentation](textured-presentation/README.md) | [Textured Presentation plans](../../docs/Plans/Textured-Presentation/README.md) | texturing, alpha, streaming, and color-space evidence |

A campaign folder owns navigation and context. Shared implementation remains in
[`corpus/lib/`](../lib/README.md), while application-shaped downstream
compositions remain in [`corpus/consumers/`](../consumers/README.md).
