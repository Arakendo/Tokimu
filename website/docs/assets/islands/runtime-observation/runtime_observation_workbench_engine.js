/* @ts-self-types="./runtime_observation_workbench_engine.d.ts" */

/**
 * Browser-facing Observation Shell facade.
 *
 * TypeScript submits plain shell text and a transport sequence. Rust owns the
 * command catalog, application argument interpretation, runtime transition,
 * and the resulting bounded projection.
 */
export class WasmObservationShellSession {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmObservationShellSessionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmobservationshellsession_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    advance_animation_fixed_step() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_advance_animation_fixed_step(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    animation_catalog_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_animation_catalog_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} tick
     * @returns {string}
     */
    apply_json(tick) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_apply_json(this.__wbg_ptr, tick);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Exposes discovery as a bounded catalog, not as a browser-owned command
     * grammar or a borrowed shell instance.
     * @returns {string}
     */
    command_catalog_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_command_catalog_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {string} request_json
     * @returns {string}
     */
    enqueue_json(request_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(request_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmobservationshellsession_enqueue_json(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Executes raw owner-qualified shell input at a browser-supplied logical
     * sequence. The returned record is the sole browser-visible command
     * outcome; no runtime request or playback type crosses this boundary.
     * @param {string} input
     * @param {number} sequence
     * @returns {string}
     */
    execute_json(input, sequence) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmobservationshellsession_execute_json(this.__wbg_ptr, ptr0, len0, sequence);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    latest_observation_diff_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_latest_observation_diff_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    constructor() {
        const ret = wasm.wasmobservationshellsession_new();
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmObservationShellSessionFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Returns the same provider-neutral runtime observation used by the
     * graphical controls. This keeps the browser's two views on one Rust
     * session rather than creating parallel scenario state.
     * @param {number} sequence
     * @param {number | null} [selected_entity]
     * @returns {string}
     */
    observation_json(sequence, selected_entity) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_observation_json(this.__wbg_ptr, sequence, isLikeNone(selected_entity) ? Number.MAX_SAFE_INTEGER : (selected_entity) >>> 0);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {string} command_json
     * @returns {string}
     */
    playback_command_json(command_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(command_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmobservationshellsession_playback_command_json(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    playback_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_playback_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    presentation_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_presentation_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Appends raw host text to the Rust-owned prompt. The browser does not
     * interpret commands or construct terminal cells.
     * @param {string} text
     */
    ratatui_append_text(text) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmobservationshellsession_ratatui_append_text(this.__wbg_ptr, ptr0, len0);
    }
    ratatui_backspace() {
        wasm.wasmobservationshellsession_ratatui_backspace(this.__wbg_ptr);
    }
    ratatui_clear_prompt() {
        wasm.wasmobservationshellsession_ratatui_clear_prompt(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    ratatui_frame_height() {
        const ret = wasm.wasmobservationshellsession_ratatui_frame_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Renders the live semantic shell through Ratatui and Tokimu's retained
     * backend. The resulting bytes are an RGBA frame for the browser canvas.
     * @param {number} width
     * @param {number} height
     * @returns {Uint8Array}
     */
    ratatui_frame_rgba(width, height) {
        const ret = wasm.wasmobservationshellsession_ratatui_frame_rgba(this.__wbg_ptr, width, height);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    ratatui_frame_width() {
        const ret = wasm.wasmobservationshellsession_ratatui_frame_width(this.__wbg_ptr);
        return ret >>> 0;
    }
    ratatui_history_down() {
        wasm.wasmobservationshellsession_ratatui_history_down(this.__wbg_ptr);
    }
    ratatui_history_up() {
        wasm.wasmobservationshellsession_ratatui_history_up(this.__wbg_ptr);
    }
    /**
     * @param {number} lines
     */
    ratatui_scroll_by(lines) {
        wasm.wasmobservationshellsession_ratatui_scroll_by(this.__wbg_ptr, lines);
    }
    /**
     * Submits the currently visible prompt through the same semantic shell
     * handler as `execute_json`.
     * @returns {string}
     */
    ratatui_submit() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_ratatui_submit(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    select_arm_presentation_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_select_arm_presentation_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {number} width
     * @param {number} height
     * @param {number} sequence
     * @param {number | null} [selected_entity]
     * @returns {string}
     */
    ui_snapshot_json(width, height, sequence, selected_entity) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmobservationshellsession_ui_snapshot_json(this.__wbg_ptr, width, height, sequence, isLikeNone(selected_entity) ? Number.MAX_SAFE_INTEGER : (selected_entity) >>> 0);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
}
if (Symbol.dispose) WasmObservationShellSession.prototype[Symbol.dispose] = WasmObservationShellSession.prototype.free;

/**
 * Browser-facing observation facade for the runtime corpus.
 *
 * The browser receives owned JSON records and can submit semantic requests.
 * It neither receives a `World` nor parses source GLB data.
 */
export class WasmRuntimeObservationSession {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmRuntimeObservationSessionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmruntimeobservationsession_free(ptr, 0);
    }
    /**
     * Advances only the fixed-step playback policy; it does not mutate the
     * scenario world or create a browser-owned animation model.
     * @returns {string}
     */
    advance_animation_fixed_step() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_advance_animation_fixed_step(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    animation_catalog_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_animation_catalog_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Applies the FIFO command queue at the caller-selected lifecycle tick.
     * @param {number} tick
     * @returns {string}
     */
    apply_json(tick) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_apply_json(this.__wbg_ptr, tick);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Admits one application-owned command into the bounded queue. Command
     * JSON is parsed by Rust and remains only a request until `apply_json`.
     * @param {string} request_json
     * @returns {string}
     */
    enqueue_json(request_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(request_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmruntimeobservationsession_enqueue_json(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Returns the comparison between the two most recent browser-visible
     * observations. The first observation intentionally has no predecessor.
     * @returns {string}
     */
    latest_observation_diff_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_latest_observation_diff_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    constructor() {
        const ret = wasm.wasmruntimeobservationsession_new();
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmRuntimeObservationSessionFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Returns a bounded summary or selected-entity observation.
     * @param {number} sequence
     * @param {number | null} [selected_entity]
     * @returns {string}
     */
    observation_json(sequence, selected_entity) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_observation_json(this.__wbg_ptr, sequence, isLikeNone(selected_entity) ? Number.MAX_SAFE_INTEGER : (selected_entity) >>> 0);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {string} command_json
     * @returns {string}
     */
    playback_command_json(command_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(command_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmruntimeobservationsession_playback_command_json(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    playback_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_playback_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    presentation_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_presentation_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Selects the scenario's explicitly mapped arm target. The target is not
     * guessed from an ECS entity ID by the browser.
     * @returns {string}
     */
    select_arm_presentation_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_select_arm_presentation_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Resolves the current observation into a provider-neutral semantic UI
     * artifact. The browser receives evidence, not renderer resources or a
     * second authoritative layout model.
     * @param {number} width
     * @param {number} height
     * @param {number} sequence
     * @param {number | null} [selected_entity]
     * @returns {string}
     */
    ui_snapshot_json(width, height, sequence, selected_entity) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmruntimeobservationsession_ui_snapshot_json(this.__wbg_ptr, width, height, sequence, isLikeNone(selected_entity) ? Number.MAX_SAFE_INTEGER : (selected_entity) >>> 0);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
}
if (Symbol.dispose) WasmRuntimeObservationSession.prototype[Symbol.dispose] = WasmRuntimeObservationSession.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_344f42d3211c4765: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./runtime_observation_workbench_engine_bg.js": import0,
    };
}

const WasmObservationShellSessionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmobservationshellsession_free(ptr, 1));
const WasmRuntimeObservationSessionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmruntimeobservationsession_free(ptr, 1));

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('runtime_observation_workbench_engine_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
