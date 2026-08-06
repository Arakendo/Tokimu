export class ObservationShellClient {
    wasm;
    sequence = 0;
    constructor(wasm) {
        this.wasm = wasm;
    }
    execute(input) {
        return JSON.parse(this.wasm.execute_json(input, this.sequence++));
    }
    catalog() {
        return JSON.parse(this.wasm.command_catalog_json());
    }
}
/** Keeps transport details out of UI components and sequences observations. */
export class RuntimeObservationClient {
    wasm;
    sequence = 0;
    constructor(wasm) {
        this.wasm = wasm;
    }
    observe(selectedEntity) {
        return JSON.parse(this.wasm.observation_json(this.sequence++, selectedEntity));
    }
    latestObservationDiff() {
        return JSON.parse(this.wasm.latest_observation_diff_json());
    }
    observeUi(width, height, selectedEntity) {
        return JSON.parse(this.wasm.ui_snapshot_json(width, height, this.sequence++, selectedEntity));
    }
    enqueue(request) {
        return JSON.parse(this.wasm.enqueue_json(JSON.stringify(request)));
    }
    apply(tick) {
        return JSON.parse(this.wasm.apply_json(tick));
    }
    playback(command) {
        return JSON.parse(this.wasm.playback_command_json(JSON.stringify(command)));
    }
}
