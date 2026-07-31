export default function init(moduleOrPath?: RequestInfo | URL): Promise<unknown>;

export class WasmPaintSession {
  constructor(width: number, height: number, red: number, green: number, blue: number, alpha: number);
  static open(bytes: Uint8Array, format: string): WasmPaintSession;
  free(): void;
  dispose(): void;
  apply_json(commandJson: string): string;
  observation_json(): string;
  undo_json(): string;
  redo_json(): string;
  reset_json(): string;
  sample_rgba(x: number, y: number): Uint8Array;
  preview_bytes(): Uint8Array;
  preview_observation_json(): string;
  export_png_bytes(): Uint8Array;
  export_observation_json(): string;
}
