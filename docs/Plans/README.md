# Implementation Plans

Plans describe concrete implementation work. They should identify scope,
ownership, incremental slices, validation, risks, and completion criteria.

Plans are not architectural decisions. If implementation evidence changes an
established boundary, update or create an Architectural Review and ADR rather
than silently treating the plan as authority.

## Current Plans

- [UI Boxes Through Vector Presentation](ui-box-vector-presentation.md)
- [Font Outlines Through Vector Presentation](font-outline-vector-presentation.md)
- [Presentation Geometry Corpus Harness](presentation-geometry-corpus-harness.md)
- [Native Execution and Multithreading](native-execution-and-multithreading.md)
- [XML Tools Incubation Library](xml-tools.md)
- [Performance Diagnostics and Runtime Observation](performance-diagnostics-and-runtime-observation.md)
- [Ring 0 Third-Party Source Audit And Migration](ring-zero-third-party-source-audit-and-migration.md)
- [Native Math Vocabulary And Foreign-Type Case Study](native-math-vocabulary-foreign-type-case-study.md)
- [DOOM TypeScript Boundary Stress Plan](DOOM/DOOM%20TypeScript%20Boundary%20Stress%20Plan.md)
- [Consumer Corpora](consumer-corpora.md)
- [TypeScript Shader, Material, And Presentation Control](typescript-shader-material-presentation-control.md)
- [Particle Simulation And Presentation](particle-simulation-and-presentation.md)
- [Tokimu Website](tokimu-website.md)
- [Tokimu And Tosumu Reciprocal Website Evidence](tokimu-tosumu-reciprocal-website-evidence.md)
- [Tokimu Paint Consumer Corpus](tokimu-paint-consumer-corpus.md)
- [Runtime Observation And Command Corpus](runtime-observation-and-command-corpus.md)
- [Tokimu Observation Shell Consumer Corpus](tokimu-observation-shell-consumer-corpus.md)
- [UI Tools Consumer Safety And Hardening](ui-tools-consumer-safety-and-hardening.md)
- [Streaming RGBA8 Texture Updates](streaming-rgba8-texture-updates.md)
- [Audio-Reactive Visualizers And MilkDrop Compatibility](audio-reactive-visualizers-and-milkdrop-compatibility.md)
- [MIDI Sequencing And Synthesis Provider](midi-sequencing-and-synthesis-provider.md)
- [Tosumu .NET Resource Space Consumer Migration](tosumu-dotnet-resource-space-consumer-migration.md)
- [Compression And Archive Providers](compression-and-archive-providers.md)
- [Tokimu Console Command Window Corpus](tokimu-console-command-window-corpus.md)

## Completed Spikes

- [Networking and Transport Spike](networking-and-transport.md)

## External Corpus Plans

External corpus acquisition, coverage, and validation plans live under
[`docs/Libraries`](../Libraries/README.md). That index records their measured
status and common quality requirements. Implementation plans should link to the
relevant library document instead of duplicating mutable fixture counts.

## Plan Requirements

A useful plan should include:

- the problem and evidence motivating the work;
- goals and non-goals;
- current ownership and dependency boundaries;
- small compileable implementation slices;
- tests or corpus evidence for each slice;
- risks, unsupported cases, and explicit diagnostics;
- acceptance and graduation criteria;
- links to related ADRs, reviews, notes, examples, and tests.
