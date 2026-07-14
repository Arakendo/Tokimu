import type { ExecutionMode } from "./execution.js";

// The recognized Tokimu authoring primitives.
//
// IMPORTANT: these functions are not meant to run in a bare Node/TS context.
//
// - In `lowered` mode they are recognized by the Tokimu lowering pass at build
//   time and translated into engine-owned semantics. The bodies below are never
//   executed for lowered rules.
// - In `runtime` mode they are bound to a Tokimu runtime host that supplies real
//   implementations behind a narrow, versioned API.
//
// Calling them directly (outside a host, outside lowering) is a programming
// error, so they fail loudly rather than pretending to work.
const RECOGNIZED_ELSEWHERE =
  "This Tokimu authoring primitive is recognized by the lowering pass (lowered mode) " +
  "or bound by the runtime host (runtime mode). It cannot run in a bare TypeScript context.";

/** The fixed-step context handed to a rule body. Only engine-owned time is exposed. */
export interface RuleContext {
  /** Fixed simulation step in seconds. There is deliberately no wall-clock time. */
  readonly fixedDelta: number;
  /** Emit a declared signal so tools can inspect a rule's effects. */
  emit(signal: string): void;
}

/** A minimal typed view of an entity inside a `query()` iteration. */
export interface EntityView {
  readonly id: number;
  get<T>(component: string): T;
  set<T>(component: string, value: T): void;
}

/** A reference to another entity, used by `relation()`. */
export interface EntityRef {
  readonly id: number;
}

/** The declarative spec for a rule. Matches the object form the Rust host parses. */
export interface RuleSpec {
  execution?: ExecutionMode;
  inputs?: readonly string[];
  outputs?: readonly string[];
  signals?: readonly string[];
  run?: (ctx: RuleContext) => void;
}

/** The lowered/normalized rule document produced by `rule()`. */
export interface RuleDocument {
  name: string;
  execution: ExecutionMode;
  inputs: string[];
  outputs: string[];
  signals: string[];
}

/**
 * Declare a Tokimu rule. Returns a normalized document; the `run` body is
 * either lowered at build time or executed by the runtime host, depending on
 * the declared execution mode.
 */
export function rule(name: string, spec: RuleSpec = {}): RuleDocument {
  return {
    name,
    execution: spec.execution ?? "auto",
    inputs: [...(spec.inputs ?? [])],
    outputs: [...(spec.outputs ?? [])],
    signals: [...(spec.signals ?? [])],
  };
}

/** Iterate entities matching the named components. Recognized at lowering time. */
export function query(...components: string[]): Iterable<EntityView> {
  void components;
  throw new Error(RECOGNIZED_ELSEWHERE);
}

/** Emit a named signal. Recognized at lowering time. */
export function signal(name: string, payload?: unknown): void {
  void name;
  void payload;
  throw new Error(RECOGNIZED_ELSEWHERE);
}

/** Assert a directional relationship toward a target entity. Recognized at lowering time. */
export function relation(kind: string, target: EntityRef): void {
  void kind;
  void target;
  throw new Error(RECOGNIZED_ELSEWHERE);
}

/** Request a named command the runtime applies after the step. Recognized at lowering time. */
export function command(name: string, payload?: unknown): void {
  void name;
  void payload;
  throw new Error(RECOGNIZED_ELSEWHERE);
}
