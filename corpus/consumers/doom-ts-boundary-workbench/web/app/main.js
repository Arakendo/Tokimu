import init, { BrowserIntakeSession } from "../pkg/doom_ts_boundary_workbench_engine.js";
import { bindLocalPackagePicker, disposeIntake } from "./intake.js";
const button = document.querySelector("#select");
const inspect = document.querySelector("#inspect");
const render = document.querySelector("#render");
const renderWorking = document.querySelector("#render-working");
const runRotation = document.querySelector("#run-rotation");
const runRetainedRotation = document.querySelector("#run-retained-rotation");
const mapPrevious = document.querySelector("#map-previous");
const mapNext = document.querySelector("#map-next");
const workingMap = document.querySelector("#working-map");
const renderCutouts = document.querySelector("#render-cutouts");
const renderSelected = document.querySelector("#render-selected");
const renderDiagnosticSky = document.querySelector("#render-diagnostic-sky");
const renderExitsign = document.querySelector("#render-exitsign");
const download = document.querySelector("#download");
const clear = document.querySelector("#clear");
const input = document.querySelector("#package");
const result = document.querySelector("#result");
const canvas = document.querySelector("#scene");
if (button === null || inspect === null || render === null || renderWorking === null || runRotation === null || runRetainedRotation === null || mapPrevious === null || mapNext === null || workingMap === null || renderCutouts === null || renderSelected === null || renderDiagnosticSky === null || renderExitsign === null || download === null || clear === null || input === null || result === null || canvas === null)
    throw new Error("intake DOM is incomplete");
const episodeMaps = ["E1M1", "E1M2", "E1M3", "E1M4", "E1M5", "E1M6", "E1M7", "E1M8", "E1M9"];
let workingMapIndex = 0;
let packageRetained = false;
let workingMapRendering = false;
let workingRotationActive = false;
let workingRotationCancellationRequested = false;
let workingWalkaboutActive = false;
let previousWalkStepTime = performance.now();
let nextWorkingPresentationTime = 0;
let mouseDeltaX = 0;
let mouseDeltaY = 0;
const pressedKeys = new Set();
function stopWorkingWalkabout() {
    workingWalkaboutActive = false;
    pressedKeys.clear();
    mouseDeltaX = 0;
    mouseDeltaY = 0;
}
function updateWorkingMapControls() {
    const previous = (workingMapIndex + episodeMaps.length - 1) % episodeMaps.length;
    const next = (workingMapIndex + 1) % episodeMaps.length;
    workingMap.textContent = episodeMaps[workingMapIndex];
    mapPrevious.textContent = `[ ${episodeMaps[previous]}`;
    mapNext.textContent = `${episodeMaps[next]} ]`;
    renderWorking.disabled = !packageRetained;
    runRotation.disabled = !packageRetained;
    runRotation.textContent = "Run 3x map rotation";
    runRetainedRotation.disabled = !packageRetained;
    runRetainedRotation.textContent = "Run 3x retained-session rotation";
    mapPrevious.disabled = !packageRetained;
    mapNext.disabled = !packageRetained;
}
function nextAnimationFrame() {
    return new Promise((resolve) => requestAnimationFrame(() => resolve()));
}
async function runWorkingMapRotation(retainedSession = false) {
    if (workingRotationActive) {
        workingRotationCancellationRequested = true;
        runRotation.disabled = true;
        runRetainedRotation.disabled = true;
        runRotation.textContent = "Stopping after current map...";
        return;
    }
    if (!packageRetained || workingMapRendering)
        return;
    stopWorkingWalkabout();
    workingMapRendering = true;
    workingRotationActive = true;
    workingRotationCancellationRequested = false;
    renderWorking.disabled = true;
    button.disabled = true;
    inspect.disabled = true;
    render.disabled = true;
    renderCutouts.disabled = true;
    renderSelected.disabled = true;
    renderDiagnosticSky.disabled = true;
    renderExitsign.disabled = true;
    download.disabled = true;
    clear.disabled = true;
    mapPrevious.disabled = true;
    mapNext.disabled = true;
    runRotation.textContent = "Stop rotation";
    runRetainedRotation.disabled = true;
    const records = [];
    let diagnostic;
    const started = performance.now();
    outer: for (let round = 1; round <= 3; round += 1) {
        for (let mapIndex = 0; mapIndex < episodeMaps.length; mapIndex += 1) {
            if (workingRotationCancellationRequested)
                break outer;
            workingMapIndex = mapIndex;
            workingMap.textContent = episodeMaps[mapIndex];
            const sequence = records.length + 1;
            result.textContent = JSON.stringify({
                kind: "running-map-rotation",
                round,
                sequence,
                total: episodeMaps.length * 3,
                map: episodeMaps[mapIndex],
                retainedRecords: records.length,
            }, null, 2);
            await nextAnimationFrame();
            const replacementStarted = performance.now();
            try {
                const observation = retainedSession
                    ? await session.render_working_map_retained_session(canvas, episodeMaps[mapIndex])
                    : await session.render_working_map(canvas, episodeMaps[mapIndex]);
                records.push({
                    sequence,
                    round,
                    map: episodeMaps[mapIndex],
                    elapsedMilliseconds: performance.now() - replacementStarted,
                    observation,
                });
            }
            catch (error) {
                diagnostic = String(error);
                break outer;
            }
            // Let the browser present progress and service provider callbacks. This
            // is deterministic replacement pressure, not a claim that one animation
            // frame is enough for physical GPU reclamation.
            await nextAnimationFrame();
        }
    }
    const cancelled = workingRotationCancellationRequested;
    workingRotationActive = false;
    workingRotationCancellationRequested = false;
    workingMapRendering = false;
    workingWalkaboutActive = diagnostic === undefined && records.length > 0;
    previousWalkStepTime = performance.now();
    nextWorkingPresentationTime = 0;
    updateWorkingMapControls();
    button.disabled = false;
    inspect.disabled = !packageRetained;
    render.disabled = !packageRetained;
    renderCutouts.disabled = !packageRetained;
    renderSelected.disabled = !packageRetained;
    renderDiagnosticSky.disabled = !packageRetained;
    renderExitsign.disabled = !packageRetained;
    download.disabled = records.length === 0;
    clear.disabled = !packageRetained;
    result.textContent = JSON.stringify({
        kind: diagnostic === undefined ? (cancelled ? "map-rotation-cancelled" : "map-rotation-complete") : "map-rotation-rejected",
        requestedReplacements: episodeMaps.length * 3,
        completedReplacements: records.length,
        elapsedMilliseconds: performance.now() - started,
        physicalGpuReclamation: "unobserved",
        lifetimeAlternative: retainedSession ? "B-adapter-private-reset" : "A-whole-backend",
        diagnostic,
        records,
    }, null, 2);
}
async function renderCurrentWorkingMap() {
    if (workingMapRendering)
        return;
    stopWorkingWalkabout();
    workingMapRendering = true;
    const mapName = episodeMaps[workingMapIndex];
    renderWorking.disabled = true;
    mapPrevious.disabled = true;
    mapNext.disabled = true;
    result.textContent = `Preparing ${mapName} with grouped sky parity and sector-boundary trim...`;
    try {
        result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_working_map(canvas, mapName), controls: "Click canvas for mouse look; W/A/S/D move; Space/Ctrl vertical; Shift runs; Escape releases mouse." }, null, 2);
        workingWalkaboutActive = true;
        previousWalkStepTime = performance.now();
        nextWorkingPresentationTime = 0;
        download.disabled = false;
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error), map: mapName }, null, 2);
    }
    finally {
        workingMapRendering = false;
        updateWorkingMapControls();
    }
}
await init();
const session = new BrowserIntakeSession();
const unbind = bindLocalPackagePicker(button, input, session, (outcome) => {
    stopWorkingWalkabout();
    result.textContent = JSON.stringify(outcome, null, 2);
    packageRetained = outcome.kind === "retained";
    inspect.disabled = outcome.kind !== "retained";
    render.disabled = outcome.kind !== "retained";
    renderCutouts.disabled = outcome.kind !== "retained";
    renderSelected.disabled = outcome.kind !== "retained";
    renderDiagnosticSky.disabled = outcome.kind !== "retained";
    renderExitsign.disabled = outcome.kind !== "retained";
    download.disabled = true;
    clear.disabled = outcome.kind !== "retained";
    updateWorkingMapControls();
});
clear.addEventListener("click", () => {
    workingRotationCancellationRequested = true;
    stopWorkingWalkabout();
    disposeIntake(session);
    packageRetained = false;
    inspect.disabled = true;
    render.disabled = true;
    renderCutouts.disabled = true;
    renderSelected.disabled = true;
    renderDiagnosticSky.disabled = true;
    renderExitsign.disabled = true;
    download.disabled = true;
    clear.disabled = true;
    updateWorkingMapControls();
    result.textContent = JSON.stringify({ kind: "disposed", retainedResources: 0, retainedBytes: 0 }, null, 2);
});
renderWorking.addEventListener("click", () => void renderCurrentWorkingMap());
runRotation.addEventListener("click", () => void runWorkingMapRotation());
runRetainedRotation.addEventListener("click", () => void runWorkingMapRotation(true));
mapPrevious.addEventListener("click", () => {
    workingMapIndex = (workingMapIndex + episodeMaps.length - 1) % episodeMaps.length;
    void renderCurrentWorkingMap();
});
mapNext.addEventListener("click", () => {
    workingMapIndex = (workingMapIndex + 1) % episodeMaps.length;
    void renderCurrentWorkingMap();
});
document.addEventListener("keydown", (event) => {
    if (!packageRetained || workingMapRendering)
        return;
    if (!event.repeat && (event.key === "[" || event.key === "]")) {
        workingMapIndex = event.key === "["
            ? (workingMapIndex + episodeMaps.length - 1) % episodeMaps.length
            : (workingMapIndex + 1) % episodeMaps.length;
        void renderCurrentWorkingMap();
        return;
    }
    if (workingWalkaboutActive) {
        pressedKeys.add(event.code);
        if (["KeyW", "KeyA", "KeyS", "KeyD", "Space", "ControlLeft", "ControlRight", "ShiftLeft", "ShiftRight"].includes(event.code))
            event.preventDefault();
    }
});
document.addEventListener("keyup", (event) => pressedKeys.delete(event.code));
addEventListener("blur", () => pressedKeys.clear());
canvas.addEventListener("click", () => {
    if (workingWalkaboutActive)
        void canvas.requestPointerLock();
});
document.addEventListener("mousemove", (event) => {
    if (workingWalkaboutActive && document.pointerLockElement === canvas) {
        mouseDeltaX += event.movementX;
        mouseDeltaY += event.movementY;
    }
});
function animateWorkingWalkabout(frameTime) {
    if (workingWalkaboutActive && !workingMapRendering) {
        const forward = Number(pressedKeys.has("KeyW")) - Number(pressedKeys.has("KeyS"));
        const strafe = Number(pressedKeys.has("KeyD")) - Number(pressedKeys.has("KeyA"));
        const vertical = Number(pressedKeys.has("Space")) - Number(pressedKeys.has("ControlLeft") || pressedKeys.has("ControlRight"));
        const hasInput = forward !== 0 || strafe !== 0 || vertical !== 0 || mouseDeltaX !== 0 || mouseDeltaY !== 0;
        if (!hasInput) {
            previousWalkStepTime = frameTime;
        }
        else if (frameTime >= nextWorkingPresentationTime) {
            const deltaSeconds = Math.min(Math.max((frameTime - previousWalkStepTime) / 1000, 0), 0.25);
            previousWalkStepTime = frameTime;
            const yawDelta = -mouseDeltaX * 0.0025;
            const pitchDelta = -mouseDeltaY * 0.0025;
            mouseDeltaX = 0;
            mouseDeltaY = 0;
            const presentationStart = performance.now();
            try {
                session.step_working_model(deltaSeconds, forward, strafe, vertical, yawDelta, pitchDelta, pressedKeys.has("ShiftLeft") || pressedKeys.has("ShiftRight"));
                const presentationMilliseconds = performance.now() - presentationStart;
                // E1M3 and later maps can require several thousand draws for each
                // grouped-parity inspection frame. Coalesce input and leave a bounded
                // recovery interval after each synchronous WASM/WebGPU submission
                // instead of continuously saturating Edge's renderer process.
                nextWorkingPresentationTime = performance.now() + Math.min(Math.max(presentationMilliseconds, 50), 250);
            }
            catch (error) {
                stopWorkingWalkabout();
                result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error), phase: "working-model-walkabout" }, null, 2);
            }
        }
    }
    requestAnimationFrame(animateWorkingWalkabout);
}
requestAnimationFrame(animateWorkingWalkabout);
inspect.addEventListener("click", () => {
    try {
        result.textContent = JSON.stringify(JSON.parse(session.inspect_doom1_wad()), null, 2);
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
});
render.addEventListener("click", async () => {
    stopWorkingWalkabout();
    render.disabled = true;
    result.textContent = "Preparing and presenting the retained E1M1 package...";
    try {
        result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_static_e1m1(canvas) }, null, 2);
        download.disabled = false;
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
    finally {
        render.disabled = false;
    }
});
renderCutouts.addEventListener("click", async () => {
    stopWorkingWalkabout();
    renderCutouts.disabled = true;
    result.textContent = "Preparing and presenting retained E1M1 with corpus-local masked cutouts...";
    try {
        result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_static_e1m1_masked_cutouts(canvas) }, null, 2);
        download.disabled = false;
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
    finally {
        renderCutouts.disabled = false;
    }
});
renderSelected.addEventListener("click", async () => {
    stopWorkingWalkabout();
    renderSelected.disabled = true;
    result.textContent = "Preparing source-spawn E1M1 with corpus-local AABB/frustum selection...";
    try {
        result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_static_e1m1_selected_cutouts(canvas) }, null, 2);
        download.disabled = false;
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
    finally {
        renderSelected.disabled = false;
    }
});
renderDiagnosticSky.addEventListener("click", async () => {
    stopWorkingWalkabout();
    renderDiagnosticSky.disabled = true;
    result.textContent = "Preparing retained E1M1 sky omissions with the opt-in Purple diagnostic stand-in...";
    try {
        result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_e1m1_diagnostic_sky_omissions(canvas) }, null, 2);
        download.disabled = false;
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
    finally {
        renderDiagnosticSky.disabled = false;
    }
});
renderExitsign.addEventListener("click", async () => {
    stopWorkingWalkabout();
    renderExitsign.disabled = true;
    result.textContent = "Preparing the canonical E1M1 EXITSIGN orientation view...";
    try {
        result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_e1m1_exitsign(canvas) }, null, 2);
        download.disabled = false;
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
    finally {
        renderExitsign.disabled = false;
    }
});
download.addEventListener("click", () => {
    canvas.toBlob((blob) => {
        if (blob === null) {
            result.textContent = JSON.stringify({ kind: "rejected", diagnostic: "browser did not encode the canvas as PNG" }, null, 2);
            return;
        }
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = "doom-e1m1-browser-first-frame.png";
        document.body.append(link);
        link.click();
        link.remove();
        setTimeout(() => URL.revokeObjectURL(url), 0);
    }, "image/png");
});
addEventListener("pagehide", () => { unbind(); disposeIntake(session); }, { once: true });
