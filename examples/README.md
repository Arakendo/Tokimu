# Tokimu Example Corpus

The `examples/` directory is Tokimu's architectural corpus.

It keeps the conventional Rust repository name `examples` while giving the
directory a stronger role than a collection of tutorials or showcase demos.
Each focused example is executable evidence about whether Tokimu's current
architecture can express a particular behavior, relationship, representation,
or application seam cleanly.

For the underlying philosophy, see
[`docs/example-philosophy.md`](../docs/example-philosophy.md).

## What An Example Proves

A good example answers one primary question:

> Can Tokimu express this behavior naturally through the intended ownership and
> dependency boundaries?

Examples may prove capabilities such as:

- opening a platform window;
- presenting one render pipeline;
- running deterministic simulation rules;
- importing or representing an asset;
- expressing text, layout, input, or interface state;
- reading, writing, or exporting application-owned data;
- composing several already-proven capabilities without bypassing their
  contracts.

The example should stop growing when its claim is proven. Additional work belongs
in the same example only when it strengthens that claim or exposes a closely
related failure mode.

## Directory Shape

```text
examples/
  hello-*/                 focused engine and application corpus examples
  ui/
    hello-ui-*/            focused presentation and interaction examples
  lib-example/
    */                     incubating shared example-side implementation
  Assets/
    */                     mesh, shader, and vector reference assets
```

### Root `hello-*` examples

Root examples pressure general engine seams: platform startup, rendering,
simulation, rule execution, assets, geometry, persistence, and application
composition.

### UI examples

`examples/ui/` isolates presentation concerns such as text, controls, themes,
layout, interaction, state, scrolling, icons, and font providers. These examples
should consume shared semantics when those semantics have already been proven.
They should not each invent private versions of the same text or control model.

### Example libraries

`examples/lib-example/` contains implementation shared by multiple corpus
examples while its ownership and API are still being discovered. Code here is
incubating evidence, not automatically a stable Tokimu capability.

Promotion out of `lib-example` requires independent use, an ownership review,
and evidence that the semantic boundary has stabilized. Convenience alone is
not a graduation trigger.

### Example assets

`examples/Assets/` contains data used by examples, including mesh and shader
calibration textures and vector references. Its files support corpus proofs but
are not themselves architectural contracts or golden outputs.

## What Does Not Belong Here

Do not use `examples/` as a home for:

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

Before adding an example, record or be able to answer:

- What is its one primary architectural claim?
- Which current boundary does it pressure?
- Why does an existing example not already prove this?
- What observable result means the proof succeeds?
- What failure would reveal that the architecture is insufficient?
- Which dependencies are necessary for the proof?
- What is explicitly outside the example's scope?

An example that cannot answer these questions is probably still an experiment.
Experiments are useful, but they should acquire a focused claim before becoming
permanent corpus entries.

## Naming

Use `hello-<capability>` for a focused proof and `hello-ui-<concept>` for a
focused UI proof.

Names should describe the seam under pressure rather than the implementation
library used to satisfy it. Provider-specific examples are appropriate when the
provider boundary itself is the subject of the proof.

Avoid version-number suffixes when a semantic distinction is available. If a
temporary numbered example such as `hello-ui-font2` survives incubation, rename
it once its distinct architectural claim is understood.

## Implementation Rules

- Prefer small, direct, compileable implementations.
- Use public Tokimu APIs when the example claims to validate a public boundary.
- Do not reach through crate ownership merely to make the screenshot work.
- Keep simulation truth outside rendering and presentation adapters.
- Make backend selection and unsupported behavior diagnostic.
- Control time, randomness, and external inputs when determinism is part of the
  claim.
- Resolve repository assets explicitly; do not depend on an accidental current
  working directory.
- Add local automated tests when important example logic can be validated
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
Focused example
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

Once accepted into the corpus, an example also becomes a regression artifact.
Refactors should preserve its primary claim unless an Architectural Review
Record or ADR deliberately changes that contract.

Automated assertions should be added at the narrowest honest boundary. Visual
examples may additionally use reviewed captures or golden fixtures when the
comparison policy is explicit. See
[`docs/testing-strategy.md`](../docs/testing-strategy.md).

## Completion Record

Each substantial example should have a short `DESIGN.md` or equivalent note that
records:

- purpose and primary proof;
- architectural assertions;
- inputs and observable outputs;
- success criteria;
- non-goals;
- implementation observations and unresolved friction;
- relationships to other corpus examples.

The document should describe what the example teaches, not narrate every source
file.

