import init, { WasmObservationShellSession, } from "./runtime_observation_workbench_engine.js";
import { ObservationShellClient, } from "./src/runtime-observation.js";
import { mapRatatuiKeyboardInput, mapRatatuiWheelDelta, } from "./src/ratatui-input.js";
const output = document.querySelector("#output");
if (!output)
    throw new Error("runtime observation output is missing");
const terminalCanvas = document.querySelector("#ratatui-shell");
const terminalStatus = document.querySelector("#ratatui-status");
if (!terminalCanvas || !terminalStatus) {
    throw new Error("Ratatui shell host is missing");
}
const terminalContext = terminalCanvas.getContext("2d", { alpha: false });
if (!terminalContext)
    throw new Error("Ratatui shell host has no 2D context");
const startupError = document.querySelector("#startup-error");
const startupErrorDetail = document.querySelector("#startup-error-detail");
const runtimeControls = Array.from(document.querySelectorAll("button, input"));
const postRuntimeState = (state, detail) => {
    if (window.parent === window)
        return;
    window.parent.postMessage({
        type: "tokimu-runtime-observation-state",
        state,
        detail,
    }, "*");
};
const postDocumentHeight = () => {
    if (window.parent === window)
        return;
    window.parent.postMessage({
        type: "tokimu-runtime-observation-height",
        height: Math.ceil(document.documentElement.scrollHeight),
    }, "*");
};
const reportStartupFailure = (error) => {
    const detail = error instanceof Error ? error.message : String(error);
    document.body.dataset.runtimeState = "error";
    runtimeControls.forEach((control) => { control.disabled = true; });
    terminalCanvas.tabIndex = -1;
    terminalCanvas.setAttribute("aria-disabled", "true");
    terminalStatus.textContent = `Runtime observation failed to start: ${detail}`;
    output.textContent = JSON.stringify({
        state: "startup_failed",
        error: detail,
    }, null, 2);
    if (startupError && startupErrorDetail) {
        startupErrorDetail.textContent = detail;
        startupError.hidden = false;
    }
    postRuntimeState("error", `Runtime observation failed to start: ${detail}`);
    requestAnimationFrame(postDocumentHeight);
};
postRuntimeState("loading", "Loading the Rust/WASM runtime observation facade...");
const start = async () => {
    await init();
    // One Rust/WASM session powers both the semantic browser controls and the
    // Ratatui terminal. They are two projections of the same runtime scenario.
    const shellWasm = new WasmObservationShellSession();
    const runtime = shellWasm;
    const shell = new ObservationShellClient(shellWasm);
    let commandId = 1;
    let tick = 1;
    const show = (value) => {
        output.textContent = JSON.stringify(JSON.parse(value), null, 2);
        scheduleDocumentHeight();
    };
    const showError = (error) => {
        output.textContent = JSON.stringify({ error: String(error) }, null, 2);
        scheduleDocumentHeight();
    };
    let documentHeightPending = false;
    const scheduleDocumentHeight = () => {
        if (documentHeightPending || window.parent === window)
            return;
        documentHeightPending = true;
        requestAnimationFrame(() => {
            documentHeightPending = false;
            postDocumentHeight();
        });
    };
    const renderRatatuiShell = () => {
        const hostWidth = terminalCanvas.getBoundingClientRect().width;
        const requestedWidth = Math.max(480, Math.floor(hostWidth));
        // The terminal surface is deliberately bounded. Rust selects its cell grid;
        // the browser only supplies the available presentation box.
        const requestedHeight = Math.round(Math.min(576, Math.max(324, requestedWidth * 0.6)));
        const rgba = shellWasm.ratatui_frame_rgba(requestedWidth, requestedHeight);
        const width = shellWasm.ratatui_frame_width();
        const height = shellWasm.ratatui_frame_height();
        if (width === 0 || height === 0)
            throw new Error("Ratatui shell produced an empty frame");
        terminalCanvas.width = width;
        terminalCanvas.height = height;
        terminalContext.putImageData(new ImageData(new Uint8ClampedArray(rgba), width, height), 0, 0);
    };
    const renderRatatuiShellAfter = (action) => {
        try {
            action();
            renderRatatuiShell();
        }
        catch (error) {
            showError(error);
            terminalStatus.textContent = `Ratatui shell error: ${String(error)}`;
        }
    };
    let ratatuiResizePending = false;
    const scheduleRatatuiResize = () => {
        if (ratatuiResizePending)
            return;
        ratatuiResizePending = true;
        requestAnimationFrame(() => {
            ratatuiResizePending = false;
            renderRatatuiShellAfter(() => { });
        });
    };
    const on = (id, action) => document.querySelector(`#${id}`)?.addEventListener("click", () => {
        try {
            show(action());
            renderRatatuiShell();
        }
        catch (error) {
            showError(error);
        }
    });
    on("observe", () => runtime.observation_json(commandId++, 7));
    on("observe-diff", () => runtime.latest_observation_diff_json());
    on("observe-ui", () => runtime.ui_snapshot_json(900, 600, commandId++, 7));
    on("observe-missing", () => runtime.observation_json(commandId++, 99));
    on("move", () => runtime.enqueue_json(JSON.stringify({ id: commandId++, target: 7, authority: "operator", command: { command: "move_by", delta: { x: 0.25, y: 0, z: 0 } } })));
    on("queue-stale", () => runtime.enqueue_json(JSON.stringify({ id: commandId++, target: 7, authority: "operator", expected_revision: 0, command: { command: "set_enabled", enabled: false } })));
    on("apply", () => runtime.apply_json(tick++));
    on("select", () => runtime.select_arm_presentation_json());
    on("play", () => runtime.playback_command_json(JSON.stringify({ command: "play", clip: 0 })));
    on("advance", () => runtime.advance_animation_fixed_step());
    on("shell-execute", () => {
        const input = document.querySelector("#shell-command");
        if (!input)
            throw new Error("shell input is missing");
        return JSON.stringify(shell.execute(input.value));
    });
    on("shell-catalog", () => JSON.stringify(shell.catalog()));
    // Enter is browser interaction only. The command remains raw text until the
    // Rust/WASM shell parses and routes it.
    document.querySelector("#shell-command")?.addEventListener("keydown", (event) => {
        if (event.key !== "Enter")
            return;
        event.preventDefault();
        document.querySelector("#shell-execute")?.click();
    });
    terminalCanvas.addEventListener("pointerdown", () => {
        terminalCanvas.focus({ preventScroll: true });
        terminalStatus.textContent = "Ratatui shell focused. Rust owns the prompt, transcript, history, and command outcome.";
    });
    const dispatchRatatuiInput = (input) => {
        switch (input.kind) {
            case "submit":
                show(shellWasm.ratatui_submit());
                break;
            case "backspace":
                shellWasm.ratatui_backspace();
                break;
            case "clear_prompt":
                shellWasm.ratatui_clear_prompt();
                break;
            case "history_up":
                shellWasm.ratatui_history_up();
                break;
            case "history_down":
                shellWasm.ratatui_history_down();
                break;
            case "scroll":
                shellWasm.ratatui_scroll_by(input.lines);
                break;
            case "append_text":
                shellWasm.ratatui_append_text(input.text);
                break;
        }
    };
    terminalCanvas.addEventListener("keydown", (event) => {
        const input = mapRatatuiKeyboardInput(event);
        if (!input)
            return;
        event.preventDefault();
        renderRatatuiShellAfter(() => dispatchRatatuiInput(input));
    });
    terminalCanvas.addEventListener("wheel", (event) => {
        const input = mapRatatuiWheelDelta(event.deltaY);
        if (!input)
            return;
        event.preventDefault();
        renderRatatuiShellAfter(() => dispatchRatatuiInput(input));
    }, { passive: false });
    // The surrounding documentation shell can resize independently of the window.
    // Observe the actual host box rather than assuming a window resize is the only
    // way the bounded terminal surface changes.
    new ResizeObserver(scheduleRatatuiResize).observe(terminalCanvas);
    new ResizeObserver(scheduleDocumentHeight).observe(document.body);
    window.addEventListener("resize", scheduleRatatuiResize);
    show(runtime.observation_json(0, 7));
    renderRatatuiShell();
    runtimeControls.forEach((control) => { control.disabled = false; });
    terminalCanvas.tabIndex = 0;
    terminalCanvas.setAttribute("aria-disabled", "false");
    document.body.dataset.runtimeState = "ready";
    scheduleDocumentHeight();
    postRuntimeState("ready", "Rust/WASM runtime observation facade ready.");
};
void start().catch(reportStartupFailure);
