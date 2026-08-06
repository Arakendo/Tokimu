/* tslint:disable */
/* eslint-disable */

/**
 * Browser-facing Observation Shell facade.
 *
 * TypeScript submits plain shell text and a transport sequence. Rust owns the
 * command catalog, application argument interpretation, runtime transition,
 * and the resulting bounded projection.
 */
export class WasmObservationShellSession {
    free(): void;
    [Symbol.dispose](): void;
    advance_animation_fixed_step(): string;
    animation_catalog_json(): string;
    apply_json(tick: number): string;
    /**
     * Exposes discovery as a bounded catalog, not as a browser-owned command
     * grammar or a borrowed shell instance.
     */
    command_catalog_json(): string;
    enqueue_json(request_json: string): string;
    /**
     * Executes raw owner-qualified shell input at a browser-supplied logical
     * sequence. The returned record is the sole browser-visible command
     * outcome; no runtime request or playback type crosses this boundary.
     */
    execute_json(input: string, sequence: number): string;
    latest_observation_diff_json(): string;
    constructor();
    /**
     * Returns the same provider-neutral runtime observation used by the
     * graphical controls. This keeps the browser's two views on one Rust
     * session rather than creating parallel scenario state.
     */
    observation_json(sequence: number, selected_entity?: number | null): string;
    playback_command_json(command_json: string): string;
    playback_json(): string;
    presentation_json(): string;
    /**
     * Appends raw host text to the Rust-owned prompt. The browser does not
     * interpret commands or construct terminal cells.
     */
    ratatui_append_text(text: string): void;
    ratatui_backspace(): void;
    ratatui_clear_prompt(): void;
    ratatui_frame_height(): number;
    /**
     * Renders the live semantic shell through Ratatui and Tokimu's retained
     * backend. The resulting bytes are an RGBA frame for the browser canvas.
     */
    ratatui_frame_rgba(width: number, height: number): Uint8Array;
    ratatui_frame_width(): number;
    ratatui_history_down(): void;
    ratatui_history_up(): void;
    ratatui_scroll_by(lines: number): void;
    /**
     * Submits the currently visible prompt through the same semantic shell
     * handler as `execute_json`.
     */
    ratatui_submit(): string;
    select_arm_presentation_json(): string;
    ui_snapshot_json(width: number, height: number, sequence: number, selected_entity?: number | null): string;
}

/**
 * Browser-facing observation facade for the runtime corpus.
 *
 * The browser receives owned JSON records and can submit semantic requests.
 * It neither receives a `World` nor parses source GLB data.
 */
export class WasmRuntimeObservationSession {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advances only the fixed-step playback policy; it does not mutate the
     * scenario world or create a browser-owned animation model.
     */
    advance_animation_fixed_step(): string;
    animation_catalog_json(): string;
    /**
     * Applies the FIFO command queue at the caller-selected lifecycle tick.
     */
    apply_json(tick: number): string;
    /**
     * Admits one application-owned command into the bounded queue. Command
     * JSON is parsed by Rust and remains only a request until `apply_json`.
     */
    enqueue_json(request_json: string): string;
    /**
     * Returns the comparison between the two most recent browser-visible
     * observations. The first observation intentionally has no predecessor.
     */
    latest_observation_diff_json(): string;
    constructor();
    /**
     * Returns a bounded summary or selected-entity observation.
     */
    observation_json(sequence: number, selected_entity?: number | null): string;
    playback_command_json(command_json: string): string;
    playback_json(): string;
    presentation_json(): string;
    /**
     * Selects the scenario's explicitly mapped arm target. The target is not
     * guessed from an ECS entity ID by the browser.
     */
    select_arm_presentation_json(): string;
    /**
     * Resolves the current observation into a provider-neutral semantic UI
     * artifact. The browser receives evidence, not renderer resources or a
     * second authoritative layout model.
     */
    ui_snapshot_json(width: number, height: number, sequence: number, selected_entity?: number | null): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmobservationshellsession_free: (a: number, b: number) => void;
    readonly __wbg_wasmruntimeobservationsession_free: (a: number, b: number) => void;
    readonly wasmobservationshellsession_advance_animation_fixed_step: (a: number) => [number, number, number, number];
    readonly wasmobservationshellsession_animation_catalog_json: (a: number) => [number, number, number, number];
    readonly wasmobservationshellsession_apply_json: (a: number, b: number) => [number, number, number, number];
    readonly wasmobservationshellsession_command_catalog_json: (a: number) => [number, number, number, number];
    readonly wasmobservationshellsession_enqueue_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmobservationshellsession_execute_json: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly wasmobservationshellsession_latest_observation_diff_json: (a: number) => [number, number, number, number];
    readonly wasmobservationshellsession_new: () => [number, number, number];
    readonly wasmobservationshellsession_observation_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmobservationshellsession_playback_command_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmobservationshellsession_playback_json: (a: number) => [number, number, number, number];
    readonly wasmobservationshellsession_presentation_json: (a: number) => [number, number, number, number];
    readonly wasmobservationshellsession_ratatui_append_text: (a: number, b: number, c: number) => void;
    readonly wasmobservationshellsession_ratatui_backspace: (a: number) => void;
    readonly wasmobservationshellsession_ratatui_clear_prompt: (a: number) => void;
    readonly wasmobservationshellsession_ratatui_frame_height: (a: number) => number;
    readonly wasmobservationshellsession_ratatui_frame_rgba: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmobservationshellsession_ratatui_frame_width: (a: number) => number;
    readonly wasmobservationshellsession_ratatui_history_down: (a: number) => void;
    readonly wasmobservationshellsession_ratatui_history_up: (a: number) => void;
    readonly wasmobservationshellsession_ratatui_scroll_by: (a: number, b: number) => void;
    readonly wasmobservationshellsession_ratatui_submit: (a: number) => [number, number, number, number];
    readonly wasmobservationshellsession_select_arm_presentation_json: (a: number) => [number, number, number, number];
    readonly wasmobservationshellsession_ui_snapshot_json: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_advance_animation_fixed_step: (a: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_animation_catalog_json: (a: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_apply_json: (a: number, b: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_enqueue_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_latest_observation_diff_json: (a: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_new: () => [number, number, number];
    readonly wasmruntimeobservationsession_observation_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_playback_command_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_playback_json: (a: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_presentation_json: (a: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_select_arm_presentation_json: (a: number) => [number, number, number, number];
    readonly wasmruntimeobservationsession_ui_snapshot_json: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
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
