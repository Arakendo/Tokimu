// Execution intent for a Tokimu rule.
//
// This mirrors `ExecutionMode` in the engine-owned `tokimu-rule` crate so the
// authoring surface and the semantic model agree on the same three words.
//
// - "lowered": compile into Tokimu-owned semantics; deterministic by construction.
// - "runtime": run in the TypeScript runtime host; flexible, not determinism-guaranteed.
// - "auto":    lower if possible, otherwise fall back to runtime and report the outcome.
export type ExecutionMode = "auto" | "lowered" | "runtime";
