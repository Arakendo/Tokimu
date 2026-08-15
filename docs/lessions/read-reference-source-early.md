# Read Available Reference Source Earlier

When Tokimu is reconstructing an existing format or behavior and inspectable
reference source is available, consult that source early enough to eliminate
incorrect models before they accumulate implementation and corpus machinery.

Reference source is evidence, not architectural authority. It can explain what
the original system did, which input facts mattered, and which operations were
ordered or stateful. It does not decide what Tokimu should own, which boundary
should expose the behavior, or whether Tokimu should reproduce the historical
implementation.

## When to Read It

Source inspection is especially valuable when:

- several plausible models explain the same initial observation;
- two bounded repairs fail in opposite ways;
- correctness depends on traversal order, accumulated state, clipping, or
  source-format conventions that are difficult to infer from a final image;
- a compatibility implementation exists but its durable invariant is unclear;
- a synthetic fixture cannot yet distinguish mechanism from semantics.

Do not begin with broad source archaeology for an ordinary local defect. Start
with a bounded question, inspect the narrow subsystem that can answer it, and
stop when the result can be expressed as a falsifiable invariant.

## Recommended Workflow

1. Retain the corpus observation and name the competing hypotheses.
2. Ask a narrow source question, such as which record owns a decision or what
   state is updated before a later contribution is admitted.
3. Inspect the original implementation and, when practical, a faithful modern
   continuation. Agreement helps distinguish durable behavior from one
   implementation's accident; disagreement identifies a compatibility choice.
4. Record the behavior separately from the historical mechanism.
5. Build or refine a small synthetic fixture that can falsify the extracted
   invariant.
6. Use corpus evidence and Tokimu's accepted boundaries to decide ownership and
   implementation. Do not copy the reference architecture by default.

The intended relationship is:

```text
corpus observation
    -> bounded hypotheses
    -> reference-source inspection
    -> extracted invariant
    -> synthetic falsification
    -> Tokimu ownership and implementation decision
```

## Doom Sky Lesson

The Doom sky investigation first tested two reasonable world-space models: a
paired-sky depth boundary and exact sky-plane geometry. One model incorrectly
clipped valid hut geometry; the other still allowed unrelated sector geometry
to survive.

Reading the classic Doom wall and plane rendering paths clarified the missing
causal relationship: sky painting consumes viewer-relative coverage produced
by ordered wall and plane processing. Sky geometry is not itself the authority
that determines all visibility.

That source result did not mean Tokimu should copy Doom's renderer. It supplied
a better invariant for the synthetic campaign: source-authorized presentation
can require partial, viewer-relative coverage, so a whole-SEG Boolean candidate
decision may be too coarse. A partial-coverage fixture could then test that
claim directly.

Earlier bounded source inspection would have retired the world-space depth-wall
branch sooner. The corpus was still necessary to determine that Doom owns the
source protocol while the generic Tokimu renderer should remain unaware of
visplanes and Doom-specific clipping.

## Guardrails

- Preserve source locations and the precise question they answered in retained
  evidence; avoid unsupported appeals to how the original engine worked.
- Separate required behavior, compatibility quirks, and historical machinery.
- Prefer independent tests over translating source code line for line.
- Recheck licenses and provenance before copying any implementation. Reading an
  implementation as an oracle does not authorize importing it.
- If the extracted rule changes a stable Tokimu boundary, use an ADR or
  Architectural Review rather than promoting this lesson into policy.
