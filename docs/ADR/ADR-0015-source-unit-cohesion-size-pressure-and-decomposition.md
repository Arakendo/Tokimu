# ADR-0015: Source Unit Cohesion, Size Pressure, And Decomposition

## Status

Accepted — 2026-08-15, with ADR-0005 pre-admission pilot evidence substitution

## ADR-0005 Evidence Substitution

The accountable maintainer accepted this decision under ADR-0005 before the
Doom composition completed the proposed pre-admission decomposition pilot.
This is permanent admission by evidence substitution, not an assertion that
the then-missing pilot evidence existed at admission time.

### Normal requirement waived

The proposed draft required a successful behavior-preserving decomposition of
the 11,088-line Doom `static_scene.rs` composition before this ADR could be
accepted. The maintainer waived completion before admission; the pilot was
subsequently completed under the accepted conservation boundary.

### Why pre-acceptance completion adds limited decision value

The deferred pilot tested whether the thresholds were calibrated well and whether
the proposed review procedure produces useful private seams. It does not decide
the underlying ownership rule: decomposition must preserve existing ownership,
authority, dependency direction, behavior, and retained failures. Accepting
that conservation rule before the pilot provides the boundary under which the
pilot can proceed and avoids treating an improvised refactor as architectural
evidence.

The pilot could still falsify the threshold calibration, coupling checks, or
review procedure. Such a result would have required revision or supersession of
this ADR; it did not justify performing the pilot without an accepted
conservation boundary.

### Substitute evidence

- `static_scene.rs` already exceeds the exceptional threshold and combines
  independently testable application, Doom preparation, presentation,
  selection, runtime-control, diagnostic, CLI, and regression
  responsibilities.
- AR-0025, AR-0028, AR-0030, Doom sky work, dynamic-sector work, and the Doom
  checklist have independently modified or pressured the same source unit.
- The current Slice 7 checkpoint demonstrates the attribution risk directly:
  structural contribution conservation passes while required geometry remains
  visibly absent.
- ADR-0008 already requires serious decomposition, reuse, code hygiene, and
  proportional review; ADR-0009 already requires retained focused evidence and
  failure identity. This ADR makes their source-unit consequences explicit
  rather than creating a conflicting ownership model.
- Hard line limits, tooling-only reporting, continued local judgment, and
  arbitrary file splitting were considered and rejected in this decision.

### Consequences and accepted risk

- No runtime contract, dependency direction, public API, platform behavior, or
  Native/WASM semantic is admitted by accepting this organizational rule.
- The thresholds could have proved poorly calibrated, and the first pilot could
  have exposed organizational seams that did not reduce coupling. The retained
  completion record did not observe either failure.
- Existing large files are not required to undergo immediate bulk migration.
  Review is triggered proportionally by substantive work and the thresholds in
  this ADR.
- CI remains prohibited from treating line count alone as a failure.
- At admission time, the Doom composition still had to complete the
  checkpointed conservation and post-extraction coupling review described
  below. The retained pilot record now reports that completion.

### Required verification after acceptance

The Doom composition is the first mandatory application of this ADR. Its
decomposition campaign preserved the Slice 7 structural fingerprints and known
missing-geometry falsification while moving implementation into private subject
modules. The post-extraction record below distinguishes completed structural
verification from browser visual evidence that remains owned by the semantic
campaign rather than by this refactor.

Failure of a future mandatory pilot reopens this ADR. The maintainer must revise
the thresholds or procedure, supersede the decision, or record a narrower
retained rule. The accepted status must not be used to declare a failed pilot
successful.

## Context

Tokimu deliberately retains corpus regressions, comparative candidates,
diagnostics, and cross-target evidence. That practice is valuable, but it can
also make a successful corpus composition accumulate unrelated responsibilities
inside one source unit.

The immediate trigger is
`corpus/campaigns/doom/hello-doom-e1m1/src/bin/static_scene.rs`. At the time of
this proposal it is 11,088 lines and contains responsibilities including:

- native application and platform-loop composition;
- command-line option parsing and experimental-mode selection;
- E1M1 scene and runtime-state orchestration;
- global and source-prepared presentation paths;
- candidate-selection experiments;
- camera, input, collision, door, and platform controls;
- interactive console diagnostics and retained reports; and
- a large collection of regressions from several Architectural Reviews and
  Doom campaign slices.

The current Slice 7 prepared-full-submission experiment provides a coherent
checkpoint. Its lowering conservation checks pass, while manual observation
still finds missing geometry. That result must be retained as a
source-preparation falsification. Continuing semantic repair and reorganizing an
11,000-line composition simultaneously would make later failures difficult to
attribute.

Line count alone is not an architectural defect. A short file can mix several
owners, while a generated table or cohesive declarative schema can reasonably
be large. The architectural pressure appears when a source unit no longer
communicates one coherent implementation responsibility and consequently makes
ownership, review, testing, navigation, or change isolation materially worse.

Tokimu therefore needs a proportional review rule that detects decomposition
pressure without turning a line quota into architecture, manufacturing public
APIs, or forcing risky reorganizations in the middle of an unresolved
experiment.

> File size triggers review; responsibility boundaries justify decomposition.

> Split by meaning, not by line count.

## Decision

Tokimu hand-maintained source units must represent one coherent implementation
responsibility. Size and responsibility signals trigger an explicit cohesion
review. The review may retain a cohesive unit, require behavior-preserving
decomposition, or record a bounded exception.

This decision applies to Rust, TypeScript, JavaScript, shader source, and other
hand-maintained implementation files. It applies proportionally in every ring;
it does not move Outer Ring or corpus mechanics into the Native Ring.

### Size review triggers

Physical line count is a deliberately inexpensive first signal:

| Hand-maintained source size | Required treatment |
| --- | --- |
| Up to 1,000 lines | Ordinary. No size justification is required. Cohesion may still require review. |
| 1,001-2,000 lines | Inspect cohesion during a substantive modification. |
| 2,001-4,000 lines | Perform and retain an explicit decomposition review. |
| More than 4,000 lines | Presume decomposition debt. Retain the file only with a documented cohesion or sequencing reason. |
| More than 8,000 lines | Exceptional. Active work should normally include a checkpointed decomposition campaign. |

These values are review thresholds, not merge failures or automatic mandates
to create more files. Comments, tests, and documentation embedded in an
implementation source unit count because they affect the same navigation and
review burden.

An automated report may identify units crossing a threshold. CI must not fail
solely because a file has crossed a numeric threshold unless a later decision
admits a specific mechanical gate with demonstrated value.

### Responsibility review triggers

A decomposition review is required regardless of line count when a source unit
shows two or more of these conditions:

- it contains multiple independently testable subsystems;
- it mixes stable composition behavior with experimental candidates;
- it mixes domain semantics with presentation, UI, CLI, or platform concerns;
- it contains several unrelated diagnostic or reporting systems;
- tests require navigating substantial unrelated implementation;
- ordinary changes routinely touch distant, unrelated regions of the file;
- the filename does not communicate a useful owner or responsibility;
- a responsibility can move behind a private module without changing a stable
  contract; or
- multiple campaigns, ADRs, or Architectural Reviews independently modify the
  same unit.

Crossing a size threshold and satisfying one responsibility trigger is also
sufficient to require review.

### Decomposition follows ownership seams

Decomposition must name the responsibility of every extracted module. It must
not produce numbered fragments, arbitrary line buckets, or a directory whose
files only make sense when read as one continuous anonymous source file.

Preferred seams include already-visible responsibilities such as composition,
configuration, source preparation, presentation lowering, diagnostics,
candidate selection, runtime controls, and focused regression families. The
actual code and evidence decide which of those seams exist; this ADR does not
pre-admit that example vocabulary as stable architecture.

The smallest sufficient visibility is preferred:

1. private child module in the same crate or executable;
2. crate-visible module when multiple local units require it;
3. existing public abstraction when the responsibility already belongs there;
4. new public API or crate only after its own callers and admission evidence
   justify that boundary.

Moving code to reduce a file's size does not change who owns its meaning.
Filesystem ownership and architectural ownership are distinct.

### Subject and responsibility identify a source unit

Tokimu adopts a limited organizational lesson from S1000D data modules without
adopting S1000D identifiers, filename codes, XML structures, or publication
processes. S1000D separates the identity of the product subject from the type
of information a standalone module provides. The corresponding source-code
principle is:

> Directory structure communicates subject ownership. Module naming
> communicates implementation responsibility within that subject.

For example:

```text
doom/
    presentation/
        mod.rs
        contribution.rs
        ordered_preparation.rs
        lowering.rs
        diagnostics.rs

    interaction/
        mod.rs
        activation.rs
        collision.rs
        moving_sector.rs
```

`doom/presentation` and `doom/interaction` identify different owned subjects.
The files within each subject identify distinct responsibilities rather than
arbitrary source ranges. The path and filename together should answer both
"what meaning does this belong to?" and "what work does this unit perform?"

Human-readable semantic names are required. Tokimu must not introduce numeric
information codes or opaque filename abbreviations merely to resemble a
technical-publication system.

A small responsibility vocabulary may help reviewers describe demonstrated
seams:

| Responsibility | Meaning |
| --- | --- |
| `contract` | Types and behavior intentionally visible across the concept boundary. |
| `state` | Durable or runtime state owned by the subject. |
| `preparation` | Transformation of semantic inputs into an intermediate result. |
| `lowering` | Conversion of owned meaning into another boundary's vocabulary. |
| `adapter` | Integration of a foreign provider, format, or platform mechanism. |
| `diagnostics` | Bounded observations and retained failure evidence. |
| `fixture` | Synthetic or retained evidence input. |
| `tests` | Evidence grouped around a named invariant. |
| `generated` | Machine-produced content explicitly separated from maintained implementation. |

This is descriptive vocabulary, not a required file template. A concept must
not acquire empty or ceremonial `contract.rs`, `state.rs`, or `adapter.rs`
files. Names should become more specific when the actual responsibility is
known; `ordered_preparation.rs` communicates more than `operation.rs`, for
example.

Before accepting an extracted source unit as a coherent seam, the review must
be able to state:

- its subject;
- its implementation responsibility;
- the authority or policy it owns;
- the inputs it consumes;
- the outputs or observations it produces; and
- the responsibilities it explicitly must not own.

A unit that cannot answer those questions is not yet a demonstrated
decomposition seam. It may remain private and local while the boundary is
studied; uncertainty does not justify making it public.

### Successful extraction reduces responsibility coupling

Smaller files are not sufficient evidence of decomposition. Extracted private
modules must not recreate the former monolith through dense conceptual cycles,
broad access to one another's internals, or a shared mutable context that
contains nearly every responsibility in the composition.

After extraction, the review must inspect:

- dependency direction between the new modules;
- which state each module reads and mutates;
- whether a module can be tested through its named responsibility rather than
  through the entire composition;
- whether sibling internals are being routed through the parent merely to
  evade a cycle; and
- whether a broad context parameter is hiding the same ownership ambiguity
  that previously existed in one file.

Some coordination through the composition root is expected. The decomposition
is unsuccessful when most extracted modules still require most of the former
unit's state or policy to perform ordinary local work. In that case the seams
are organizational rather than semantic and must be revised or explicitly
retained as an unsuccessful trial.

> A successful decomposition reduces responsibility coupling, not merely
> physical source size.

### Tests are retained but organized by invariant

Regression growth is evidence that the corpus is working. It is not a reason
to keep every regression in the production source unit that first exposed it.

Tests that need private implementation access may live in private module test
files. Contract and composition tests should use the narrowest honest public or
crate-visible boundary. Test modules and test data should be grouped by the
invariant they retain, such as ordered coverage, plane reconstruction, source
sky behavior, candidate selection, or coordinate embedding.

Splitting tests must not replace focused assertions with broad screenshot-only
coverage or duplicate implementation semantics in test helpers.

### Active experiments use checkpointed decomposition

A size trigger does not authorize an unrelated refactor in the middle of an
unresolved experiment. When an exceptional unit is under active semantic work:

1. reach and document a coherent experimental checkpoint;
2. retain the current passing tests, structural fingerprints, observations,
   and known falsifications;
3. perform behavior-preserving extraction separately from semantic repair;
4. rerun the same evidence gates after each coherent extraction group; and
5. resume the experiment only after the reorganized composition reproduces the
   checkpoint.

If the unit cannot reach a safe checkpoint, the review records why extraction
would create greater attribution risk and identifies the next bounded point at
which decomposition will occur.

### Conservation requirements

A decomposition change must retain:

- architectural ownership and dependency direction;
- public API and visibility unless a separate accepted decision changes them;
- input, output, ordering, lifecycle, and failure behavior;
- structured diagnostic identities and source provenance;
- relevant native and WASM behavior;
- deterministic fingerprints, manifests, or retained observations whose
  contracts have not intentionally changed; and
- the known failure or falsification being investigated.

Mechanical extraction must not quietly fix, suppress, or reinterpret the
active defect. A semantic fix discovered during extraction is recorded and
performed as a separate reviewable change after the conservation baseline is
restored.

### Exceptions

The graduated line thresholds do not apply directly to:

- generated code;
- vendored source;
- machine-produced bindings;
- static lookup tables or data-dominant source units;
- exact corpus artifacts whose value is preserving their original identity;
  or
- a cohesive declarative schema where splitting materially reduces
  comprehension.

An exception must identify the category, distinguish generated/data content
from hand-maintained implementation where practical, and explain why the unit
is more coherent intact. "Splitting is inconvenient" is not sufficient.

### Review record

An explicit decomposition review records:

- current line count and responsibility inventory;
- the thresholds and triggers that caused the review;
- the proposed module boundaries, subjects, responsibilities, authorities,
  inputs, outputs, exclusions, and owners;
- the expected dependency direction and post-extraction coupling result;
- whether any public surface or dependency direction would change;
- the checkpoint and conservation evidence;
- the retained disposition: decompose, retain with reason, or defer to a named
  checkpoint; and
- reopening triggers.

A planning document is appropriate for a multi-step extraction campaign. The
plan must distinguish behavior-preserving moves from later semantic repairs.

## Immediate Application To The Doom Composition

`static_scene.rs` exceeds the exceptional threshold and satisfies several
responsibility triggers. It is therefore accepted as active decomposition debt
for the purpose of evaluating this proposal; the debt is not itself evidence
that any contained responsibility belongs in `tokimu-render`, the Doom
geometry provider, or the Native Ring.

The current Slice 7 checkpoint is:

- one coherent prepared-full-submission observation feeds walls, ordinary
  planes, sky intervals, and cutout identities;
- retained-to-lowered contribution accounting has no unexplained loss;
- one zero-area wall fragment remains a named omission; and
- manual fixed-spawn observation still shows missing geometry, so the ordered
  source preparation remains incomplete.

Before further substantial Slice 7 semantic repair, a follow-up decomposition
plan should inventory and test private module seams. Candidate seams to
evaluate—not pre-decided destinations—include:

```text
static_scene/
    mod.rs                         composition root
    cli.rs                         invocation and experiment selection
    runtime.rs                     native loop and mutable E1M1 controls
    console.rs                     corpus-local inspection UI
    presentation/
        global_submission.rs
        ordered_preparation.rs
        lowering.rs
    candidate_selection/
        full.rs
        frustum.rs
    diagnostics/
        reports.rs
        traces.rs
    tests/
        ordered_coverage.rs
        plane_reconstruction.rs
        source_sky.rs
        candidate_selection.rs
        coordinate_embedding.rs
```

The extraction plan must preserve the current missing-geometry falsification.
Making the image look better is not evidence of a successful decomposition.

The completed checkpointed application established private subject folders for:

- observer/view behavior;
- conservative and Doom-owned candidate selection;
- input and inspection controls;
- replayable LOOK/source-ray, candidate, SEG, and campaign diagnostics;
- source presentation models, preparation, lowering, sky spans, and viewport
  conventions;
- mutable application lifecycle, dynamic geometry, and replay reports; and
- composition conservation tests.

Directly owned models, mechanics, formatting, and tests moved with those
subjects. The application object moved as a cohesive private runtime subject
rather than being split into artificial `models`, `interfaces`, or `utils`
files. The executable root remains the composition and experiment-selection
unit. It also retains actively compared legacy SEG/BSP reconstruction mechanics
whose semantic disposition belongs to AR-0025/AR-0030; moving those mechanics
again during this refactor would distribute an unsettled algorithm without
reducing its coupling.

The measured result is:

| Source unit | Lines after formatting | Disposition |
| --- | ---: | --- |
| `static_scene.rs` | 793 | Retain as the thin executable composition root. |
| `runtime/app.rs` | 1,952 | Retain as the cohesive application lifecycle subject. |
| `presentation/legacy_source_protocol/comparison_preparation.rs` | 1,043 | Retain as corpus-private comparison preparation pending AR-0025/AR-0030 disposition. |

The root therefore moved from the exceptional `>8,000` band, through the
presumed-debt and explicit-review bands, below the ordinary `1,000`-line review
signal. The review does not treat 793 as a quality score: it records that the
remaining unit now composes named private subjects. A future semantic decision
that retires or admits the comparative SEG/BSP path should still trigger a
local review because that decision may create a cleaner removal or provider
seam.

The extraction did not change a public API, promote Doom semantics into
`tokimu-render`, or intentionally alter the known Slice 7 missing-geometry
falsification. Focused verification after extraction retained 42 native
composition tests, and strict Clippy passed for the `static_scene` binary.
Incremental-cache hard-link warnings remain an environment/filesystem condition,
not a source warning. Browser visual parity remains evidence for the active Doom
semantic campaign; this structural pilot does not claim a new browser rendering
observation.

The post-extraction coupling review records two deliberate dependencies. The
composition root still coordinates source preparation and comparative SEG/BSP
mechanics across presentation modes, while LOOK diagnostics consume assembled
scene evidence through explicit inputs. Neither child owns or mutates the
application object. Collision and source-special mechanics remain in the corpus
library; the runtime subject owns only their application-level orchestration.
These dependencies are visible and directional rather than hidden behind
pass-through wrappers.

## Alternatives Considered

### No shared rule

Continue relying on local judgment. This avoids process but leaves recurring
corpus success able to concentrate implementation, tests, and experiments in
one contested unit without a required review point.

Rejected because the Doom composition shows that the pressure is already
material and attribution risk is affecting active architecture work.

### Hard maximum line count

Fail formatting or CI when a source file exceeds a fixed size.

Rejected because line count is not responsibility, creates arbitrary splits,
and mishandles generated, data-heavy, and declarative units.

### Tooling report without an architectural rule

Publish source-size statistics and leave their meaning unspecified.

Rejected as insufficient by itself. A report can locate pressure but cannot
decide whether to retain, decompose, or preserve ownership during extraction.

### Cohesion review with graduated triggers

Use size as an inexpensive signal, require responsibility analysis, and
conserve behavior and ownership through checkpointed decomposition.

Proposed because it addresses the demonstrated failure mode without making
filesystem layout a new architectural authority.

## Consequences

### Positive

- Large or contested units receive review before navigation and attribution
  costs grow without bound.
- Corpus regressions can continue accumulating without forcing all evidence
  into one production file.
- Private modules can improve isolation without manufacturing public APIs or
  changing ring ownership.
- Active experiments retain a clear before/after checkpoint.
- Exceptions remain possible for genuinely cohesive generated, data, or
  declarative units.

### Negative

- Decomposition reviews and conservation reruns add near-term work.
- Private module boundaries may expose hidden coupling that requires careful
  sequencing.
- A large cohesive change can temporarily cross a threshold and require a
  written disposition even when eventual extraction is straightforward.
- Numeric triggers may be mistaken for quality scores unless reviews continue
  to emphasize responsibility.

## Acceptance Record

The maintainer accepted that:

- the graduated thresholds are useful review signals rather than source caps;
- the responsibility triggers are concrete enough for repeatable review;
- private extraction does not imply public or Native ownership;
- checkpoint and conservation requirements protect active corpus evidence;
- exemptions cover legitimate large units without becoming a general escape;
  and
- the Doom composition is the appropriate first mandatory verification
  campaign.

The Doom pilot's pre-acceptance timing requirement was the evidence substituted
under ADR-0005. The later checkpointed pilot completed successfully: the root
left the exceptional and presumed-debt bands, private subjects retained their
ownership, focused tests and strict Clippy passed, and the known semantic
falsification was not recast as a refactor success. This completion validates
the procedure for this Rust corpus unit without converting the graduated
thresholds into hard limits.

The accepted pilot checkpoint is commit `140e5b5`. Continued decomposition
after that checkpoint reduced the executable root from 3,939 to 793 lines.
Startup and command-line composition moved into a private `startup` subject;
retained source-protocol mechanics moved into focused comparison-preparation,
classic-BSP, and screen-projection subjects. Slice 7's canonical comparison
alternatives are now separate private strategy files rather than interleaved
conditionals:

- A — `global_full_submission.rs`;
- B — `prepared_full_submission.rs`; and
- C — `prepared_frustum_filtered.rs`.

The selection seam is corpus-local and deliberately small. C invokes B's
ordered Doom preparation before selecting the generic frustum/AABB post-filter;
it is not another preparation implementation. This layout makes strategy
switching explicit without admitting strategy vocabulary into Tokimu's public
renderer API. The known prepared-full missing-edge falsification remains open
and was not changed by this structural work.

## Reopening Triggers

Revisit this decision if:

- reviews repeatedly produce ritual justification without changing outcomes;
- the thresholds create arbitrary fragmentation or substantial churn;
- generated/data-heavy files are still misclassified;
- private extraction repeatedly requires accidental public APIs;
- decomposition changes cannot reliably conserve native/WASM evidence; or
- another language or artifact class demonstrates materially different
  maintenance pressure that requires its own policy.

## Related Decisions And Reviews

- ADR-0001: Engine Boundaries
- ADR-0003: Capability Ownership Boundary
- ADR-0005: Admission Evidence And Maintainer Exceptions
- ADR-0008: Native Kernel Ring Performance And Code Quality
- ADR-0009: Ring-Based Verification, Failure Containment, And Recovery
- AR-0025: Camera Candidate Selection And Visibility Culling
- AR-0030: Tokimu Render Preparation And Submission Framework

## External Reference

- [S1000D principal concepts](https://s1000d.org/?page_id=101) — reference
  model for standalone information units whose identifiers distinguish subject
  identity from information purpose. Tokimu borrows only that organizational
  distinction; this ADR does not claim or require S1000D conformance.
