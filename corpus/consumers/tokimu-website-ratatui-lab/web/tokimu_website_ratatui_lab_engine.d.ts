export default function init(
  input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module,
): Promise<unknown>;

export function template_snapshot(template: string, columns: number, rows: number): string;
export function template_frame_rgba(template: string, columns: number, rows: number): Uint8Array;
export function cell_pixel_width(): number;
export function cell_pixel_height(): number;
export function template_catalog_json(): string;
