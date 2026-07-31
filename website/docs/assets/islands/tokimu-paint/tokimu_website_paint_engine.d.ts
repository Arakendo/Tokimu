/* tslint:disable */
/* eslint-disable */

/**
 * Browser-facing adapter for the Paint consumer.
 *
 * Commands and observations are small JSON control records. Preview and
 * export use explicit byte copies so a Canvas, DOM object, or decoder-native
 * object never becomes authoritative application state.
 */
export class WasmPaintSession {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Applies a small semantic command record. Pixel and export buffers are
     * deliberately not transported through this JSON path.
     */
    apply_json(command_json: string): string;
    dispose(): void;
    export_observation_json(): string;
    export_png_bytes(): Uint8Array;
    constructor(width: number, height: number, red: number, green: number, blue: number, alpha: number);
    observation_json(): string;
    /**
     * Opens one admitted encoded source through the Rust raster provider.
     */
    static open(bytes: Uint8Array, format: string): WasmPaintSession;
    preview_bytes(): Uint8Array;
    preview_observation_json(): string;
    redo_json(): string;
    reset_json(): string;
    sample_rgba(x: number, y: number): Uint8Array;
    undo_json(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmpaintsession_free: (a: number, b: number) => void;
    readonly wasmpaintsession_apply_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmpaintsession_dispose: (a: number) => void;
    readonly wasmpaintsession_export_observation_json: (a: number) => [number, number, number, number];
    readonly wasmpaintsession_export_png_bytes: (a: number) => [number, number, number, number];
    readonly wasmpaintsession_new_blank: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly wasmpaintsession_observation_json: (a: number) => [number, number, number, number];
    readonly wasmpaintsession_open: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmpaintsession_preview_bytes: (a: number) => [number, number, number, number];
    readonly wasmpaintsession_preview_observation_json: (a: number) => [number, number, number, number];
    readonly wasmpaintsession_redo_json: (a: number) => [number, number, number, number];
    readonly wasmpaintsession_reset_json: (a: number) => [number, number, number, number];
    readonly wasmpaintsession_sample_rgba: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmpaintsession_undo_json: (a: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
