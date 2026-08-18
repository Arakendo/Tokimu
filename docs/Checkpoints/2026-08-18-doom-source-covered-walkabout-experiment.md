# 2026-08-18 Doom Source-Covered Walkabout Experiment

## Disposition

E1M1 now has an explicit corpus-private walkabout strategy,
`source-covered-global-shell`, for visually testing whether Doom's ordinary
ordered source-domain participation can suppress the retained far-field
resurrections without inheriting the incomplete-plane failures of the earlier
365-draw candidate.

This is experimental evidence under AR-0030. It is not the default E1M1 path,
a stable Tokimu spatial contract or a renderer feature.

## Preparation Shape

```text
original complete E1M1 prepared shell
        + actual horizontal camera position/heading
        + current Doom runtime map snapshot
                    ↓
ordinary near-first Doom BSP coverage replay
                    ↓
reached source subsector domains
                    ↓
whole original flat draw for reached owner
wall draw if any resolved owner was reached
unresolved ownership retained fail-open
                    ↓
complete ordinary draw set
                    ↓
composition-local prepare-then-replace refresh
                    ↓
unchanged renderer full submission
```

The strategy intentionally does not clip reconstructed floors or ceilings to
SEG endpoint boxes, child boxes or Classic screen spans. A source bound has no
rejection authority over the larger inferred plane geometry. The ordered
replay decides source-domain participation; existing complete meshes realize
participating domains.

## Automated Evidence

The retained six-ray causality report now runs seven candidate controls:

| Control | Expected | Result |
| --- | --- | --- |
| hut-east wall 230 | absent | pass, 0 retained draws |
| wall 247 east | absent | pass, 0 retained draws |
| subsector 104 ceiling reached view | retained | pass, 2 retained draws |
| wall 247 west | absent | pass, 0 retained draws |
| subsector 149 ceiling rejected view | absent | pass, 0 retained draws |
| subsector 104 ceiling rejected view | absent | pass, 0 retained draws |
| nearby wall 135 / SUPPORT2 | retained | pass, 2 retained draws |

Every preparation verifies:

- retained plus rejected draws equals input draws;
- unresolved ownership fails open;
- the renderer receives only existing ordinary draw declarations; and
- no diagnostic or Doom vocabulary enters the renderer boundary.

At the source spawn the two-frame native smoke run reported:

```text
input:     opaque 1823, cutout 26
retained:  opaque 967,  cutout 24
rejected:  flat 446, wall 412
unresolved fail-open: 0
conservation: balanced
```

The first-frame and warm-frame uploads completed successfully. The initial
composition preparation and first runtime refresh differ by one visited
subsector because the runtime refresh uses the realized camera snapshot; the
active frame always receives one complete prepared result.

## Validation

- `cargo fmt --all`
- `cargo check -p hello-doom-e1m1 --bin static_scene`
- `cargo test -p hello-doom-e1m1 --bin static_scene` — 89 passed
- `--ordered-non-presentation-causality-report` — candidate controls 7/7
- native `--render-strategy=source-covered-global-shell --measure-two-frames`
  — completed successfully

## Remaining Visual Questions

The executable evidence cannot yet answer whether the candidate is visually
acceptable under unrestricted movement. Walkabout review must check:

- the spawn room remains complete;
- the known hut and far-field resurrected geometry is absent;
- yaw, translation and pitch do not expose stale or half-prepared sets;
- domain changes do not create objectionable popping at the horizontal view
  boundary; and
- door/platform state remains fresh after source-domain refreshes.

Any failure is a finding about this corpus strategy. It does not justify
changing renderer ownership or promoting Doom BSP concepts into a stable
Tokimu layer.

## Walkabout Result

Maintainer walkabout found the hut area substantially improved, but produced
both reached-domain false retention and source-proxy false omission around the
spawn windows and hut. The strategy is therefore falsified as a sufficient
presentation policy and remains diagnostic-only.

Detailed capture ledger and architectural disposition:
[2026-08-18 Doom Source-Covered Walkabout Falsifiers](2026-08-18-doom-source-covered-walkabout-falsifiers.md).
