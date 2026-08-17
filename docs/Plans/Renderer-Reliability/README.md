# Renderer Reliability Campaign

| Field | Value |
| --- | --- |
| Status | Complete |
| Controlling plan | [Renderer Resource Identity And Failure Presentation](renderer-resource-identity-and-failure-presentation.md) |
| Related reviews | AR-0024 and AR-0027 |
| Current disposition | Application-local identity, recovery, and diagnostic presentation retained; no shared contract admitted |
| Next action | Reopen on an independent caller or stronger cross-lifetime requirement |

Alternative designs live in [Studies](Studies/). Native/browser lifecycle,
containment, and diagnostic observations live in [Evidence](Evidence/).

Cross-campaign renderer-boundary evidence also lives here when its immediate
caller is another campaign. The
[AR-0030 transient geometry precedent survey](Evidence/AR-0030%20transient%20geometry%20precedent%20survey.md)
records the lifetime and ownership constraints applied to Doom's private
submission-local geometry experiment.
