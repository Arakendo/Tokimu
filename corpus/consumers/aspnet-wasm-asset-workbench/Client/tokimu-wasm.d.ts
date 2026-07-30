export default function init(
  moduleOrPath?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
): Promise<unknown>;
export function engine_status(): string;
export function inspect_asset(fileName: string, bytes: Uint8Array): string;
