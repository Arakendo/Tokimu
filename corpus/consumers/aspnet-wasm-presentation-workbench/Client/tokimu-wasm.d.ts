export default function init(
  moduleOrPath?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
): Promise<unknown>;
export function engine_status(): string;
export function presentation_scene(): string;

/** Provider-neutral presentation command boundary owned by Rust/WASM. */
export class PresentationSession {
  constructor(sceneJson: string);
  set_override(requestJson: string): string;
  clear_override(requestJson: string): string;
}
