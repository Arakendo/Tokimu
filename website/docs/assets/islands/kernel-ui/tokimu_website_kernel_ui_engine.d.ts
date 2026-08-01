/* tslint:disable */
/* eslint-disable */

export class KernelUiSession {
    free(): void;
    [Symbol.dispose](): void;
    apply(): string;
    cancel_delete(): string;
    confirm_delete(): string;
    dispose(): void;
    constructor();
    observation_json(): string;
    request_delete(): string;
    revert(): string;
    select_resource(resource_id: bigint): string;
    set_filter(value: string): string;
    set_name(value: string): string;
    set_notes(value: string): string;
    toggle_hotspot(): string;
    toggle_visibility(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_kerneluisession_free: (a: number, b: number) => void;
    readonly kerneluisession_apply: (a: number) => [number, number, number, number];
    readonly kerneluisession_cancel_delete: (a: number) => [number, number, number, number];
    readonly kerneluisession_confirm_delete: (a: number) => [number, number, number, number];
    readonly kerneluisession_dispose: (a: number) => void;
    readonly kerneluisession_new: () => number;
    readonly kerneluisession_observation_json: (a: number) => [number, number, number, number];
    readonly kerneluisession_request_delete: (a: number) => [number, number, number, number];
    readonly kerneluisession_revert: (a: number) => [number, number, number, number];
    readonly kerneluisession_select_resource: (a: number, b: bigint) => [number, number, number, number];
    readonly kerneluisession_set_filter: (a: number, b: number, c: number) => [number, number, number, number];
    readonly kerneluisession_set_name: (a: number, b: number, c: number) => [number, number, number, number];
    readonly kerneluisession_set_notes: (a: number, b: number, c: number) => [number, number, number, number];
    readonly kerneluisession_toggle_hotspot: (a: number) => [number, number, number, number];
    readonly kerneluisession_toggle_visibility: (a: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
