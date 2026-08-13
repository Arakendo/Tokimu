# Tokimu Corpus

The `corpus/` directory is Tokimu's architectural corpus.

It is deliberately named `corpus` rather than `examples`: these programs are
executable evidence about whether Tokimu's current architecture can express a
particular behavior, relationship, representation, or application seam
cleanly. Some entries are approachable examples, but tutorial value is
secondary to their architectural claim.

For the underlying philosophy, see
[`docs/example-philosophy.md`](../docs/example-philosophy.md).

## What A Corpus Entry Proves

A good corpus entry answers one primary question:

> Can Tokimu express this behavior naturally through the intended ownership and
> dependency boundaries?

Corpus entries may prove capabilities such as:

- opening a platform window;
- presenting one render pipeline;
- running deterministic simulation rules;
- importing or representing an asset;
- expressing text, layout, input, or interface state;
- reading, writing, or exporting application-owned data;
- composing several already-proven capabilities without bypassing their
  contracts.

The entry should stop growing when its claim is proven. Additional work belongs
in the same entry only when it strengthens that claim or exposes a closely
related failure mode.

## Directory Shape

```text
corpus/
  campaigns/
    */                     sustained, plan-aligned executable evidence
  focused/
    */                     focused proofs grouped by technical domain
  consumers/
    */                     downstream application-shaped composition evidence
  ui/
    hello-ui-*/            focused presentation and interaction examples
  lib/
    */                     incubating shared corpus implementation
  assets/
    */                     mesh, shader, and vector reference assets
  wasm-demo/
    */                     browser presentation fixture
```

### Campaign corpus

`corpus/campaigns/` contains executable evidence belonging to a sustained work
campaign. Its top-level names align with the campaign portfolios under
`docs/Plans/`, so a maintainer can move from intent and checklist to the
relevant fixtures without reconstructing the relationship from filenames.

Campaign folders own navigation, not shared implementation. Reusable but still
incubating code remains under `corpus/lib/`, and downstream application-shaped
compositions remain under `corpus/consumers/`.

See the [campaign corpus index](campaigns/README.md).

### Focused corpus

`corpus/focused/` contains bounded `hello-*` proofs that do not belong to a
sustained campaign. They are grouped by stable technical domains such as
foundations, simulation, observation, data interchange, audio, and networking.
The grouping is navigational; every entry still needs one primary architectural
claim and its own completion record.

See the [focused corpus index](focused/README.md).

### UI corpus

`corpus/ui/` isolates presentation concerns such as text, controls, themes,
layout, interaction, state, scrolling, icons, and font providers. These examples
should consume shared semantics when those semantics have already been proven.
They should not each invent private versions of the same text or control model.

See the [UI corpus index](ui/README.md) for the conceptual grouping.

### Consumer corpus

`corpus/consumers/` contains application-shaped proofs that compose several
Tokimu contracts from a downstream consumer's point of view. These entries are
still repository-owned evidence, not independent production consumers.

See the [consumer corpus index](consumers/README.md) for tier labels and current
entries.

### Corpus libraries

`corpus/lib/` contains implementation shared by multiple corpus entries while
its ownership and API are still being discovered. Code here is incubating
evidence, not automatically a stable Tokimu capability.

Promotion out of `corpus/lib/` requires independent use, an ownership review,
and evidence that the semantic boundary has stabilized. Convenience alone is
not a graduation trigger.

See the [corpus library index](lib/README.md) for current library families and
their ownership rules.

### Corpus assets

`corpus/assets/` contains first-party data used by corpus entries, including
mesh and shader calibration textures and vector references. Its files support
corpus proofs but are not themselves architectural contracts or golden
outputs.

See the [corpus asset index](assets/README.md) for asset organization and
provenance expectations.

## What Does Not Belong Here

Do not use `corpus/` as a home for:

- arbitrary scratch programs;
- unrelated application products;
- benchmarks with no architectural claim;
- generated build output;
- large external reference repositories;
- golden expected results;
- implementation dependencies disguised as sample assets;
- duplicate examples that prove no new seam.

Use these locations instead:

```text
third-party/                 pinned external data corpora
tests/fixtures/golden/       reviewed expected outputs
crates/<crate>/tests/        one crate's public API tests
tests/                       cross-workspace integration tests
target/                      generated and transient corpus output
```

## Admission Checklist

Before adding a corpus entry, record or be able to answer:

- What is its one primary architectural claim?
- Which current boundary does it pressure?
- Why does an existing corpus entry not already prove this?
- What observable result means the proof succeeds?
- What failure would reveal that the architecture is insufficient?
- Which dependencies are necessary for the proof?
- What is explicitly outside the example's scope?

An entry that cannot answer these questions is probably still an experiment.
Experiments are useful, but they should acquire a focused claim before becoming
permanent corpus entries.

## Naming

Use `hello-<capability>` for a focused proof and `hello-ui-<concept>` for a
focused UI proof. Place sustained campaign evidence under the matching
`corpus/campaigns/<campaign>/` portfolio; place other focused proofs under the
closest `corpus/focused/<domain>/` portfolio.

Names should describe the seam under pressure rather than the implementation
library used to satisfy it. Provider-specific examples are appropriate when the
provider boundary itself is the subject of the proof.

Avoid version-number suffixes when a semantic distinction is available. If a
temporary numbered example such as `hello-ui-font2` survives incubation, rename
it once its distinct architectural claim is understood.

## Implementation Rules

- Prefer small, direct, compileable implementations.
- Use public Tokimu APIs when the corpus entry claims to validate a public boundary.
- Do not reach through crate ownership merely to make the screenshot work.
- Keep simulation truth outside rendering and presentation adapters.
- Make backend selection and unsupported behavior diagnostic.
- Control time, randomness, and external inputs when determinism is part of the
  claim.
- Resolve repository assets explicitly; do not depend on an accidental current
  working directory.
- Add local automated tests when important corpus logic can be validated
  without running the full presentation path.

## Corpus Findings And Promotion

Corpus implementation may produce three useful results:

1. the current architecture expresses the claim cleanly;
2. the architecture works but needs refinement;
3. the proposed boundary is wrong or incomplete.

All three are evidence. Do not conceal friction by patching each example in a
different way.

Repeated patterns or boundary failures should be recorded in an Architectural
Review Record under [`docs/Architectural Reviews/`](../docs/Architectural%20Reviews/README.md).
The review determines whether behavior should remain application-owned, continue
incubating, move into a foundational service or capability, or change an
accepted architectural decision.

```text
Focused corpus entry
    ↓
Observed evidence
    ↓
Repeated independent pressure
    ↓
Architectural review
    ↓
Keep / refine / promote / reject / reopen
```

Repetition is evidence for review, not automatic permission to generalize.

## Regression Role

Once accepted into the corpus, an entry also becomes a regression artifact.
Refactors should preserve its primary claim unless an Architectural Review
Record or ADR deliberately changes that contract.

Automated assertions should be added at the narrowest honest boundary. Visual
examples may additionally use reviewed captures or golden fixtures when the
comparison policy is explicit. See
[`docs/testing-strategy.md`](../docs/testing-strategy.md).

## Completion Record

Each substantial corpus entry should have a short `DESIGN.md` or equivalent note that
records:

- purpose and primary proof;
- architectural assertions;
- inputs and observable outputs;
- success criteria;
- non-goals;
- implementation observations and unresolved friction;
- relationships to other corpus entries.

The document should describe what the corpus entry teaches, not narrate every source
file.
