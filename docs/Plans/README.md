# Implementation Plans

Plans describe concrete implementation work. They identify scope, ownership,
incremental slices, validation, risks, and completion criteria. Plans are not
architectural authority: evidence that changes an established boundary belongs
in an Architectural Review and an accepted decision belongs in an ADR.

## Campaign Portfolio

Campaign folders collect a controlling plan, supporting studies, and retained
evidence for one sustained body of work. The campaign README is the dashboard;
document location does not imply that work is active.

| Campaign | Status | Controlling document | Current disposition or next action |
| --- | --- | --- | --- |
| [DOOM](DOOM/README.md) | Active | [DOOM WAD Checklist](DOOM/DOOM%20WAD%20Checklist.md) | Continue the current WAD checklist slice and retain source-specific evidence |
| [Native Math](Native-Math/README.md) | Parked | [Foreign-Type Case Study](Native-Math/native-math-vocabulary-foreign-type-case-study.md) | Alternative A retained; B and C remain executable incubation evidence |
| [Coordinate Conformance](Coordinate-Conformance/README.md) | Complete | [Directional Conformance](Coordinate-Conformance/coordinate-frame-directional-conformance.md) | Reopen for a new adapter or contradictory directional evidence |
| [Renderer Reliability](Renderer-Reliability/README.md) | Complete | [Resource Identity And Failure Presentation](Renderer-Reliability/renderer-resource-identity-and-failure-presentation.md) | No shared contract admitted; reopen on independent caller pressure |
| [Textured Presentation](Textured-Presentation/README.md) | Active / incubating | [Alpha-Policy Comparative Corpus](Textured-Presentation/textured-surface-alpha-policy-comparative-corpus.md) | Cutout admitted narrowly; continuous blend remains incubating |
| [Standalone Plans](Standalone/README.md) | Mixed | Individual documents | Promote a plan into a campaign only when sustained related work appears |

## Campaign Layout

Use this shape for new sustained work:

```text
Campaign/
  README.md              campaign dashboard and current state
  controlling-plan.md    primary checklist or implementation plan
  Studies/               bounded alternatives and investigations
  Evidence/              ledgers, results, and dated observations
```

Keep completed and parked material beside its campaign. Do not create a global
archive whose location hides why evidence was collected.

## Status Vocabulary

Use one of these states in plan metadata and campaign dashboards:

- **Proposed** — scoped but not started;
- **Active** — implementation or evidence collection is in progress;
- **Awaiting Review** — evidence is ready for maintainer judgment;
- **Blocked** — progress requires unavailable evidence or authority;
- **Parked** — intentionally dormant with a named reopening trigger;
- **Complete** — acceptance or parking criteria are satisfied; or
- **Superseded** — another named document owns the work.

## External Corpus Plans

External corpus acquisition, coverage, and validation plans live under
[`docs/Libraries`](../Libraries/README.md). Campaign plans should link to those
records instead of duplicating mutable fixture counts.

## Plan Requirements

A useful plan should include:

- the campaign, role, status, parent review, and next action where applicable;
- the problem and evidence motivating the work;
- goals and non-goals;
- current ownership and dependency boundaries;
- small compileable implementation slices;
- tests or corpus evidence for each slice;
- risks, unsupported cases, and explicit diagnostics;
- acceptance, parking, and reopening criteria; and
- links to related ADRs, reviews, notes, examples, and tests.
