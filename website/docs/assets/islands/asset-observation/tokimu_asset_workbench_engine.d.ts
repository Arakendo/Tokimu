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

/**
 * A bounded imported resource root owned by the WASM consumer.
 *
 * Browser code supplies explicitly selected byte arrays and logical names.
 * This session owns the provider-neutral hierarchy and dependency lookup; it
 * neither reads browser paths nor asks TypeScript to resolve glTF references.
 */
export class ResourceSession {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Retains one explicitly selected resource beneath the session root.
     *
     * Names must be logical relative addresses. This first browser proof
     * admits same-folder glTF dependencies only, so nested paths are rejected
     * rather than being silently flattened.
     */
    add_resource(name: string, bytes: Uint8Array): void;
    /**
     * Materializes a bounded 7z archive through the same explicit Resource
     * Space path. Archive decoding stays inside the Rust/WASM boundary.
     */
    import_seven_zip(name: string, bytes: Uint8Array): string;
    /**
     * Materializes a bounded TAR archive through the same explicit Resource
     * Space path as the other canonical archive providers. TAR container
     * details stay inside Rust/WASM.
     */
    import_tar(name: string, bytes: Uint8Array): string;
    /**
     * Stages a bounded canonical archive, lowers its safe relative paths into
     * explicit Resource Space folders, and retains each regular entry before
     * selecting a supported document for ordinary inspection.
     *
     * Browser code supplies one archive byte array only. Archive decoding,
     * path validation, extraction, and dependency resolution remain inside
     * the Rust/WASM consumer boundary.
     */
    import_zip(name: string, bytes: Uint8Array): string;
    /**
     * Inspects one selected document, resolving same-folder glTF references
     * through the resource session instead of a frontend-side importer.
     */
    inspect_resource(name: string): string;
    /**
     * Creates an empty, transient imported root for one browser selection.
     */
    constructor();
    /**
     * Returns one selected logical resource for a browser-owned download.
     *
     * The session never opens a browser save dialog or exposes host paths.
     * TypeScript owns the user gesture and turns these bytes into a download.
     */
    resource_bytes(name: string): Uint8Array;
    /**
     * Returns bounded logical-store counts for host diagnostics.
     */
    summary(): string;
}

/**
 * Compresses multiple entries into a 7z archive in WebAssembly environment.
 *
 * This function creates a compressed archive from multiple file entries,
 * designed specifically for WASM targets.
 *
 * # Arguments
 * * `entries` - Vector of JavaScript strings representing file names/paths
 * * `datas` - Vector of Uint8Arrays containing the file data corresponding to entries
 */
export function compress(entries: string[], datas: Uint8Array[]): Uint8Array;

/**
 * Decompresses a 7z archive in WebAssembly environment.
 *
 * This function is specifically designed for WASM targets and uses JavaScript interop
 * to handle the decompression process with a callback function.
 *
 * # Arguments
 * * `src` - Uint8Array containing the compressed archive data
 * * `pwd` - Password string for encrypted archives (use empty string for unencrypted)
 * * `f` - JavaScript callback function to handle extracted entries
 */
export function decompress(src: Uint8Array, pwd: string, f: Function): void;

export function engine_status(): string;

export function inspect_asset(file_name: string, bytes: Uint8Array): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_presentationsession_free: (a: number, b: number) => void;
    readonly __wbg_resourcesession_free: (a: number, b: number) => void;
    readonly engine_status: () => [number, number];
    readonly inspect_asset: (a: number, b: number, c: number, d: number) => [number, number];
    readonly presentationsession_clear_override: (a: number, b: number, c: number) => [number, number, number, number];
    readonly presentationsession_new: (a: number, b: number) => [number, number, number];
    readonly presentationsession_set_override: (a: number, b: number, c: number) => [number, number, number, number];
    readonly presentationsession_targets: (a: number) => [number, number];
    readonly resourcesession_add_resource: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly resourcesession_import_seven_zip: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly resourcesession_import_tar: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly resourcesession_import_zip: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly resourcesession_inspect_resource: (a: number, b: number, c: number) => [number, number, number, number];
    readonly resourcesession_new: () => [number, number, number];
    readonly resourcesession_resource_bytes: (a: number, b: number, c: number) => [number, number, number, number];
    readonly resourcesession_summary: (a: number) => [number, number];
    readonly compress: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly decompress: (a: any, b: number, c: number, d: any) => [number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
