export default function init(
  moduleOrPath?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
): Promise<unknown>;
export function engine_status(): string;
export function inspect_asset(fileName: string, bytes: Uint8Array): string;

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
