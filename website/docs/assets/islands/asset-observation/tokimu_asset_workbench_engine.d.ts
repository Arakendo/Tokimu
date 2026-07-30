/* tslint:disable */
/* eslint-disable */

/**
 * Stateful, provider-neutral presentation-command boundary for WASM hosts.
 *
 * This owns no importer data or browser rendering state. It only resolves
 * commands against the target descriptors Tokimu emitted in an observation.
 */
export class PresentationSession {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Clears one transient override layer and returns either the restored
     * presentation or a structured diagnostic as JSON.
     */
    clear_override(request_json: string): string;
    /**
     * Creates a command session from the observation JSON returned by
     * `inspect_asset`.
     */
    constructor(observation_json: string);
    /**
     * Applies one bounded transient override and returns either the resolved
     * provider-neutral presentation or a structured diagnostic as JSON.
     */
    set_override(request_json: string): string;
    /**
     * Returns the known targets and source values without exposing provider
     * parser objects or renderer resources.
     */
    targets(): string;
}

export function engine_status(): string;

export function inspect_asset(file_name: string, bytes: Uint8Array): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_presentationsession_free: (a: number, b: number) => void;
    readonly engine_status: () => [number, number];
    readonly inspect_asset: (a: number, b: number, c: number, d: number) => [number, number];
    readonly presentationsession_clear_override: (a: number, b: number, c: number) => [number, number, number, number];
    readonly presentationsession_new: (a: number, b: number) => [number, number, number];
    readonly presentationsession_set_override: (a: number, b: number, c: number) => [number, number, number, number];
    readonly presentationsession_targets: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
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
