import { rule, type RuleContext, type RuleDocument, type RuleSpec } from "./primitives.js";

// Intent-first convenience wrappers over `rule()`.
//
// These exist so an author can make execution intent obvious at the call site
// instead of threading an `execution` field through every spec. They are thin on
// purpose: the engine-owned meaning still lives in the rule document, not here.

/** Declare a rule that must lower into Tokimu-owned semantics. */
export function loweredRule(name: string, spec: Omit<RuleSpec, "execution">): RuleDocument {
  return rule(name, { ...spec, execution: "lowered" });
}

/** Declare a rule that stays in the TypeScript runtime host. */
export function runtimeAction(name: string, run: (ctx: RuleContext) => void): RuleDocument {
  return rule(name, { execution: "runtime", run });
}
