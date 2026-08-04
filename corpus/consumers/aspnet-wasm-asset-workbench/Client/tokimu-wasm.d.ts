export default function init(
  moduleOrPath?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
): Promise<unknown>;
export function engine_status(): string;
export function inspect_asset(fileName: string, bytes: Uint8Array): string;

/** A transient logical resource root for one explicit browser selection. */
export class ResourceSession {
  constructor();
  /** Retains one selected logical resource in the session root. */
  add_resource(name: string, bytes: Uint8Array): void;
  /** Inspects a resource, resolving bounded same-folder dependencies in Rust/WASM. */
  inspect_resource(name: string): string;
  /** Returns selected logical resource bytes for a user-initiated browser download. */
  resource_bytes(name: string): Uint8Array;
  /** Returns bounded resource-root counts for host diagnostics. */
  summary(): string;
}

/** A data-only Tokimu presentation session created from `inspect_asset` output. */
export class PresentationSession {
  constructor(observationJson: string);
  /** Returns Tokimu-discovered provider-neutral target descriptors as JSON. */
  targets(): string;
  /** Applies one JSON presentation command and returns a resolved or rejected JSON result. */
  set_override(requestJson: string): string;
  /** Clears one JSON presentation layer and returns a restored or rejected JSON result. */
  clear_override(requestJson: string): string;
}
