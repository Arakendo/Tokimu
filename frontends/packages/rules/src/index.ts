// @tokimu/rules — the rule authoring surface that maps into the engine-owned
// `tokimu-rule` model. See docs/Tokimu TypeScript Design Document.md.
export type { ExecutionMode } from "./execution.js";
export {
  rule,
  query,
  signal,
  relation,
  command,
  type RuleContext,
  type RuleSpec,
  type RuleDocument,
  type EntityView,
  type EntityRef,
} from "./primitives.js";
export { loweredRule, runtimeAction } from "./runtime.js";
