# TypeScript-Attached Behavior Editor Proposition

Date: 2026-08-10  
Status: deferred product/authoring proposition; not an editor plan, runtime-host
admission, or stable API.

## Proposition

A future Tokimu editor may let a scene element reference a TypeScript-authored
unit, for example a door behavior associated with a Door element. The useful
model is not “a node runs arbitrary JavaScript and owns private state.” It is:

```text
scene element
    -> references an authored TypeScript unit
    -> TTSDD classification and execution decision
    -> Tokimu semantic behavior
    -> Tokimu-owned state and reviewed commands
```

TypeScript would be an author-facing expression of intent. It would not become
an alternate simulation runtime, a hidden world-mutation path, or the durable
owner of game state.

## Editor-facing inspection

If this proposition gains corpus evidence, an editor should make the boundary
visible rather than treating a `.ts` attachment as an opaque executable blob.
Useful inspectable facts include:

- declared and resolved execution mode (for example, lowered versus
  capability-constrained runtime execution);
- requested, granted, exercised, denied, and post-disposal authority;
- explicit capabilities such as observing an activation or emitting a door
  request, alongside denied capabilities such as filesystem, network, or direct
  world mutation;
- durable state owner, including an explicit `none` for script-local durable
  state when appropriate; and
- the engine-owned semantic model/version targeted by the unit.

This turns the AR-0020 authority-delta artifact and the proposed execution
manifest into authoring UX rather than hidden implementation machinery.

## Possible attachment roles

The same language can represent different authority classes. A future editor
must not blur them merely because all happen to be `.ts` files:

| Attachment role | Intended boundary |
| --- | --- |
| Authored rule | Declarative semantic intent lowered into Tokimu-owned behavior. |
| Runtime behavior | Capability-constrained executable unit; admission remains unproven. |
| Editor tool | Editor-only automation, outside application/runtime authority. |
| Presentation script | Presentation mechanism, not simulation truth. |

## Constraints and non-goals

- This does not authorize a JavaScript engine, Node dependency, or TypeScript
  compiler dependency in an engine execution crate.
- It does not authorize per-node private mutable runtime objects.
- It does not claim source/graph round-trip fidelity. An inspector and source
  may be two views of an engine-owned semantic model without being lossless AST
  transforms of one another.
- It does not define serialization, ECS attachment layout, editor UI, or a
  public scripting API.

## Evidence needed before planning

Advance this only after AR-0020 has retained the TypeScript package inventory,
resolved-symbol authoring parity, execution-manifest/`auto` drift evidence, and
runtime-host authority/denial evidence where relevant. Any future corpus should
prove that the referenced unit emits reviewed Tokimu commands while durable
state remains engine-owned.

## Future semantic language-service proposition

An editor-facing TypeScript workflow will likely need language-service support,
but Tokimu should not reimplement TypeScript editor semantics. The proposed
composition is:

```text
TypeScript language service / tsserver
    -> TypeScript syntax, types, imports, symbols, and module resolution
    -> Tokimu semantic frontend
    -> Tokimu-specific diagnostics, lowering result, capability facts,
       semantic-model identity, and source mappings
    -> LSP adapter, CLI, CI, scene inspector, manifest, or build consumer
```

The TypeScript compiler/language service remains the authority for TypeScript
truth. The Tokimu frontend remains the authority for Tokimu semantic truth.
An LSP adapter, if later justified, only presents the combined analysis; it
must not become a second compiler or semantic implementation.

In particular, the LSP, CLI, CI, inspector, manifest generator, and build
must call one shared Tokimu recognizer/lowerer rather than carrying competing
lowering rules. A disagreement in which an editor claims a unit lowers but the
build rejects it is a multiple-source-of-truth defect, not an acceptable
editor-only limitation.

Potential Tokimu-specific editor facts include resolved execution mode, target
availability, lowering completeness, capability/authority delta, semantic-plan
summary, and source-located diagnostics for constructs that cannot lower. Code
lenses or a scene inspector may present the same facts, for example “lowered”,
“Native/WASM available”, or a capability count.

This remains deferred. It requires the same AR-0020 prerequisites as attached
behavior, plus evidence that the Rust frontend's resolved-symbol analysis can
produce stable source mappings and reusable diagnostics for more than one
consumer. A custom LSP protocol surface is not itself evidence that a runtime
TypeScript host or editor scripting API should be admitted.

## Lowering-front-end scope refinement

Tokimu would need compiler/front-end machinery for the **admitted TypeScript
semantic subset**, not a general TypeScript-to-Tokimu compiler and not a
JavaScript runtime in disguise. The intended stages are:

```text
TypeScript source
    -> TypeScript semantic analysis
       (parse, module/type/symbol resolution, source locations, diagnostics)
    -> Tokimu recognition and lowering frontend
       (admitted `@tokimu/*` symbols, supported constructs, capabilities,
        diagnostics, and semantic lowering)
    -> TypeScript-independent Tokimu semantic IR
    -> Tokimu runtime
```

The TypeScript authority should establish that a referenced identifier actually
resolves to an admitted Tokimu export, including aliases and re-exports.
Tokimu then decides the narrower question: whether that valid TypeScript
program can be represented by admitted Tokimu semantics. After successful
lowering, the runtime executes the Tokimu semantic plan, not the source file
and not a JavaScript object graph.

Arbitrary valid TypeScript remains outside that contract. Mutable module-local
state, ambient timers, network access, or unrecognized language/library
constructs must produce source-located Tokimu lowering diagnostics, not silent
fallback or accidental runtime execution.

The TTSDD execution modes remain distinct:

- `lowered` must fully produce Tokimu semantic IR or reject;
- `runtime` would require a separately admitted capability-constrained runtime
  provider; and
- `auto` must resolve through a retained execution manifest so a smarter
  frontend cannot silently change release execution mode.

These modes should eventually be explainable to authors, not only to engine
implementers: “no JS runtime required”, “requires an admitted runtime provider
with these capabilities”, and “a manifest change is required before a newly
available lowering can replace the accepted mode” are useful inspector facts.

An editor graph, forms, generated content, and TypeScript may eventually become
multiple authoring frontends for the same semantic IR. This does not promise
lossless source/graph conversion; it preserves one runtime meaning rather than
four competing definitions of a behavior.

## References

- `docs/Architectural Reviews/AR-0020-typescript-authoring-boundary-and-corpus-conformance.md`
- `docs/Tokimu TypeScript Design Document.md`
- `docs/Plans/DOOM/DOOM TypeScript Boundary Stress Plan.md`
