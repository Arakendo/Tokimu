import init, { BrowserIntakeSession } from "../pkg/doom_ts_boundary_workbench_engine.js";
import { bindLocalPackageDrop, bindLocalPackagePicker, disposeIntake, submitSelectedPackage, type IntakeResult } from "./intake.js";
import {
  beginObservedOperation,
  completeObservedOperation,
  operatorCompleted,
  rejectObservedOperation,
  terminalObservationEnabled,
} from "./terminal-observer.js";

const button = document.querySelector<HTMLButtonElement>("#select");
const inspect = document.querySelector<HTMLButtonElement>("#inspect");
const render = document.querySelector<HTMLButtonElement>("#render");
const renderWorking = document.querySelector<HTMLButtonElement>("#render-working");
const observeConsoleControl = document.querySelector<HTMLButtonElement>("#observe-console-control");
const runRotation = document.querySelector<HTMLButtonElement>("#run-rotation");
const completeObservedWalkabout = document.querySelector<HTMLButtonElement>("#complete-observed-walkabout");
const mapPrevious = document.querySelector<HTMLButtonElement>("#map-previous");
const mapNext = document.querySelector<HTMLButtonElement>("#map-next");
const workingMap = document.querySelector<HTMLElement>("#working-map");
const renderCutouts = document.querySelector<HTMLButtonElement>("#render-cutouts");
const renderSelected = document.querySelector<HTMLButtonElement>("#render-selected");
const renderDiagnosticSky = document.querySelector<HTMLButtonElement>("#render-diagnostic-sky");
const renderExitsign = document.querySelector<HTMLButtonElement>("#render-exitsign");
const download = document.querySelector<HTMLButtonElement>("#download");
const clear = document.querySelector<HTMLButtonElement>("#clear");
const input = document.querySelector<HTMLInputElement>("#package");
const dropTarget = document.querySelector<HTMLElement>("#drop-package");
const result = document.querySelector<HTMLElement>("#result");
const canvas = document.querySelector<HTMLCanvasElement>("#scene");
if (button === null || inspect === null || render === null || renderWorking === null || observeConsoleControl === null || runRotation === null || completeObservedWalkabout === null || mapPrevious === null || mapNext === null || workingMap === null || renderCutouts === null || renderSelected === null || renderDiagnosticSky === null || renderExitsign === null || download === null || clear === null || input === null || dropTarget === null || result === null || canvas === null) throw new Error("intake DOM is incomplete");

const episodeMaps = ["E1M1", "E1M2", "E1M3", "E1M4", "E1M5", "E1M6", "E1M7", "E1M8", "E1M9"] as const;
let workingMapIndex = 0;
let packageRetained = false;
let workingMapRendering = false;
let workingRotationActive = false;
let workingRotationCancellationRequested = false;
let workingWalkaboutActive = false;
let workingConsoleOpen = false;
let previousWalkStepTime = performance.now();
let nextWorkingPresentationTime = 0;
let mouseDeltaX = 0;
let mouseDeltaY = 0;
const pressedKeys = new Set<string>();

function stopWorkingWalkabout(): void {
  workingWalkaboutActive = false;
  pressedKeys.clear();
  mouseDeltaX = 0;
  mouseDeltaY = 0;
}

function applyWorkingConsole(action: () => string): void {
  try {
    const observation = action();
    result!.textContent = JSON.stringify({
      kind: "browser-doom-console-updated",
      observation,
      controls: "Backquote toggles; Enter submits; Backspace edits; Escape closes.",
    }, null, 2);
  } catch (error) {
    result!.textContent = JSON.stringify({
      kind: "rejected",
      diagnostic: String(error),
      phase: "browser-doom-console-update",
    }, null, 2);
  }
}

function setWorkingConsoleOpen(open: boolean): void {
  workingConsoleOpen = open;
  pressedKeys.clear();
  if (open && document.pointerLockElement !== null) document.exitPointerLock();
  applyWorkingConsole(() => session.set_working_console_open(open));
}

function updateWorkingMapControls(): void {
  const previous = (workingMapIndex + episodeMaps.length - 1) % episodeMaps.length;
  const next = (workingMapIndex + 1) % episodeMaps.length;
  workingMap!.textContent = episodeMaps[workingMapIndex];
  mapPrevious!.textContent = `[ ${episodeMaps[previous]}`;
  mapNext!.textContent = `${episodeMaps[next]} ]`;
  renderWorking!.disabled = !packageRetained;
  observeConsoleControl!.disabled = !workingWalkaboutActive;
  runRotation!.disabled = !packageRetained;
  runRotation!.textContent = "Run 3x ADR-0018 rotation";
  completeObservedWalkabout!.disabled = !terminalObservationEnabled || !workingWalkaboutActive;
  mapPrevious!.disabled = !packageRetained;
  mapNext!.disabled = !packageRetained;
}

interface RotationRecord {
  sequence: number;
  round: number;
  map: string;
  elapsedMilliseconds: number;
  observation: string;
}

function nextAnimationFrame(): Promise<void> {
  return new Promise((resolve) => {
    let complete = false;
    const finish = (): void => {
      if (complete) return;
      complete = true;
      resolve();
    };
    requestAnimationFrame(finish);
    // Automated evidence may run in a background Edge window where rAF is
    // aggressively throttled. The timer only yields browser work; it does not
    // replace or certify presentation.
    setTimeout(finish, 250);
  });
}

async function runWorkingMapRotation(): Promise<void> {
  if (workingRotationActive) {
    workingRotationCancellationRequested = true;
    runRotation!.disabled = true;
    runRotation!.textContent = "Stopping after current map...";
    return;
  }
  if (!packageRetained || workingMapRendering) return;

  const observedOperation = "doom-retained-session-rotation";
  beginObservedOperation(observedOperation);

  stopWorkingWalkabout();
  workingMapRendering = true;
  workingRotationActive = true;
  workingRotationCancellationRequested = false;
  renderWorking!.disabled = true;
  button!.disabled = true;
  inspect!.disabled = true;
  render!.disabled = true;
  renderCutouts!.disabled = true;
  renderSelected!.disabled = true;
  renderDiagnosticSky!.disabled = true;
  renderExitsign!.disabled = true;
  download!.disabled = true;
  clear!.disabled = true;
  mapPrevious!.disabled = true;
  mapNext!.disabled = true;
  runRotation!.textContent = "Stop rotation";
  const records: RotationRecord[] = [];
  let failedPreparationProbe: string | undefined;
  let diagnostic: string | undefined;
  const started = performance.now();

  outer: for (let round = 1; round <= 3; round += 1) {
    for (let mapIndex = 0; mapIndex < episodeMaps.length; mapIndex += 1) {
      if (workingRotationCancellationRequested) break outer;
      workingMapIndex = mapIndex;
      workingMap!.textContent = episodeMaps[mapIndex];
      const sequence = records.length + 1;
      result!.textContent = JSON.stringify({
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
        if (sequence === 2) {
          failedPreparationProbe = await session.verify_failed_working_map_preserves_current(canvas!, episodeMaps[mapIndex]);
        }
        const observation = await session.render_working_map_retained_session(canvas!, episodeMaps[mapIndex]);
        records.push({
          sequence,
          round,
          map: episodeMaps[mapIndex],
          elapsedMilliseconds: performance.now() - replacementStarted,
          observation,
        });
      } catch (error) {
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
  button!.disabled = false;
  inspect!.disabled = !packageRetained;
  render!.disabled = !packageRetained;
  renderCutouts!.disabled = !packageRetained;
  renderSelected!.disabled = !packageRetained;
  renderDiagnosticSky!.disabled = !packageRetained;
  renderExitsign!.disabled = !packageRetained;
  download!.disabled = records.length === 0;
  clear!.disabled = !packageRetained;
  result!.textContent = JSON.stringify({
    kind: diagnostic === undefined ? (cancelled ? "map-rotation-cancelled" : "map-rotation-complete") : "map-rotation-rejected",
    requestedReplacements: episodeMaps.length * 3,
    completedReplacements: records.length,
    elapsedMilliseconds: performance.now() - started,
    physicalGpuReclamation: "unobserved",
    lifetimeAlternative: "ADR-0018-retained-resource-set-session",
    failedPreparationProbe,
    diagnostic,
    records,
  }, null, 2);
  if (diagnostic === undefined) {
    completeObservedOperation(observedOperation, JSON.stringify({
      completedReplacements: records.length,
      requestedReplacements: episodeMaps.length * 3,
      failedPreparationProbe,
      finalObservation: records.at(-1)?.observation,
    }));
  } else {
    rejectObservedOperation(observedOperation, diagnostic);
  }
}

async function renderCurrentWorkingMap(): Promise<void> {
  if (workingMapRendering) return;
  stopWorkingWalkabout();
  workingMapRendering = true;
  const mapName = episodeMaps[workingMapIndex];
  const observedOperation = "doom-manual-walkabout";
  beginObservedOperation(observedOperation);
  renderWorking!.disabled = true;
  mapPrevious!.disabled = true;
  mapNext!.disabled = true;
  result!.textContent = `Preparing ${mapName} with grouped sky parity and sector-boundary trim...`;
  try {
    result!.textContent = JSON.stringify({ kind: "presented", observation: await session.render_working_map(canvas!, mapName), controls: "Click canvas for mouse look; W/A/S/D move; Space/C vertical; Shift runs; Backquote opens the embedded console; Escape releases mouse." }, null, 2);
    workingWalkaboutActive = true;
    previousWalkStepTime = performance.now();
    nextWorkingPresentationTime = 0;
    download!.disabled = false;
  } catch (error) {
    result!.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error), map: mapName }, null, 2);
    rejectObservedOperation(observedOperation, error);
  } finally {
    workingMapRendering = false;
    updateWorkingMapControls();
  }
}

await init();
const session = new BrowserIntakeSession();
const receiveIntakeOutcome = (outcome: IntakeResult) => {
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
};
const unbindPicker = bindLocalPackagePicker(button, input, session, receiveIntakeOutcome);
const unbindDrop = bindLocalPackageDrop(dropTarget, session, receiveIntakeOutcome);
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
function runWorkingConsoleProof(): void {
  const observedOperation = "doom-browser-console-adr0019";
  beginObservedOperation(observedOperation);
  try {
    workingConsoleOpen = true;
    const opened = session.set_working_console_open(true);
    const typed = session.insert_working_console_text("CAMERA");
    const submitted = session.submit_working_console();
    const closed = session.set_working_console_open(false);
    const reopened = session.set_working_console_open(true);
    const detail = {
      contract: "ADR-0019",
      sequence: "open>type-CAMERA>submit>close>reopen>present",
      opened,
      typed,
      submitted,
      closed,
      reopened,
      wholeSetControl: JSON.parse(session.observe_ar0033_console_whole_set_control()),
      semanticShadows: JSON.parse(session.observe_ar0033_console_semantic_shadows()),
    };
    result!.textContent = JSON.stringify({ kind: "browser-doom-console-complete", ...detail }, null, 2);
    completeObservedOperation(observedOperation, JSON.stringify(detail));
  } catch (error) {
    result!.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error), phase: "browser-doom-console-adr0019" }, null, 2);
    rejectObservedOperation(observedOperation, error);
  }
}
observeConsoleControl.addEventListener("click", runWorkingConsoleProof);
runRotation.addEventListener("click", () => void runWorkingMapRotation());
completeObservedWalkabout.addEventListener("click", () => {
  operatorCompleted("doom-manual-walkabout");
  completeObservedWalkabout.disabled = true;
  result.textContent = JSON.stringify({
    kind: "operator-completed",
    observation: "manual Doom walkabout completed under the external terminal observer",
  }, null, 2);
});
mapPrevious.addEventListener("click", () => {
  workingMapIndex = (workingMapIndex + episodeMaps.length - 1) % episodeMaps.length;
  void renderCurrentWorkingMap();
});
mapNext.addEventListener("click", () => {
  workingMapIndex = (workingMapIndex + 1) % episodeMaps.length;
  void renderCurrentWorkingMap();
});
document.addEventListener("keydown", (event) => {
  if (!packageRetained || workingMapRendering) return;
  if (!event.repeat && event.code === "Backquote" && workingWalkaboutActive) {
    event.preventDefault();
    setWorkingConsoleOpen(!workingConsoleOpen);
    return;
  }
  if (workingConsoleOpen) {
    event.preventDefault();
    if (event.code === "Escape") {
      setWorkingConsoleOpen(false);
    } else if (event.code === "Enter") {
      applyWorkingConsole(() => session.submit_working_console());
    } else if (event.code === "Backspace") {
      applyWorkingConsole(() => session.backspace_working_console());
    } else if (!event.ctrlKey && !event.metaKey && !event.altKey && event.key.length === 1) {
      applyWorkingConsole(() => session.insert_working_console_text(event.key));
    }
    return;
  }
  if (!event.repeat && (event.key === "[" || event.key === "]")) {
    workingMapIndex = event.key === "["
      ? (workingMapIndex + episodeMaps.length - 1) % episodeMaps.length
      : (workingMapIndex + 1) % episodeMaps.length;
    void renderCurrentWorkingMap();
    return;
  }
  if (workingWalkaboutActive) {
    pressedKeys.add(event.code);
    if (["KeyW", "KeyA", "KeyS", "KeyD", "Space", "KeyC", "ShiftLeft", "ShiftRight"].includes(event.code)) event.preventDefault();
  }
});
document.addEventListener("keyup", (event) => pressedKeys.delete(event.code));
addEventListener("blur", () => pressedKeys.clear());
canvas.addEventListener("click", () => {
  if (workingWalkaboutActive && !workingConsoleOpen) void canvas.requestPointerLock();
});
document.addEventListener("mousemove", (event) => {
  if (workingWalkaboutActive && document.pointerLockElement === canvas) {
    mouseDeltaX += event.movementX;
    mouseDeltaY += event.movementY;
  }
});

function animateWorkingWalkabout(frameTime: number): void {
  if (workingWalkaboutActive && !workingMapRendering && !workingConsoleOpen) {
    const forward = Number(pressedKeys.has("KeyW")) - Number(pressedKeys.has("KeyS"));
    const strafe = Number(pressedKeys.has("KeyD")) - Number(pressedKeys.has("KeyA"));
    const vertical = Number(pressedKeys.has("Space")) - Number(pressedKeys.has("KeyC"));
    const hasInput = forward !== 0 || strafe !== 0 || vertical !== 0 || mouseDeltaX !== 0 || mouseDeltaY !== 0;
    if (!hasInput) {
      previousWalkStepTime = frameTime;
    } else if (frameTime >= nextWorkingPresentationTime) {
      const deltaSeconds = Math.min(Math.max((frameTime - previousWalkStepTime) / 1000, 0), 0.25);
      previousWalkStepTime = frameTime;
      const yawDelta = -mouseDeltaX * 0.0025;
      const pitchDelta = -mouseDeltaY * 0.0025;
      mouseDeltaX = 0;
      mouseDeltaY = 0;
      const presentationStart = performance.now();
      try {
        session.step_working_model(
          deltaSeconds,
          forward,
          strafe,
          vertical,
          yawDelta,
          pitchDelta,
          pressedKeys.has("ShiftLeft") || pressedKeys.has("ShiftRight"),
        );
        const presentationMilliseconds = performance.now() - presentationStart;
        // E1M3 and later maps can require several thousand draws for each
        // grouped-parity inspection frame. Coalesce input and leave a bounded
        // recovery interval after each synchronous WASM/WebGPU submission
        // instead of continuously saturating Edge's renderer process.
        nextWorkingPresentationTime = performance.now() + Math.min(Math.max(presentationMilliseconds, 50), 250);
      } catch (error) {
        stopWorkingWalkabout();
        result!.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error), phase: "working-model-walkabout" }, null, 2);
      }
    }
  }
  requestAnimationFrame(animateWorkingWalkabout);
}
requestAnimationFrame(animateWorkingWalkabout);
inspect.addEventListener("click", () => {
  try {
    result.textContent = JSON.stringify(JSON.parse(session.inspect_doom1_wad()), null, 2);
  } catch (error) {
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
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
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
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
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
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
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
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
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
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
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
addEventListener("pagehide", () => {
  unbindPicker();
  unbindDrop();
  disposeIntake(session);
}, { once: true });

const autorunParameters = new URLSearchParams(window.location.search);
if (autorunParameters.get("tokimu_autorun") === "doom-retained-rotation") {
  const packageUrl = autorunParameters.get("tokimu_package");
  if (packageUrl === null) {
    rejectObservedOperation("doom-retained-session-rotation", "tokimu_package is required");
    throw new Error("doom retained-session autorun requires tokimu_package");
  }
  try {
    const response = await fetch(packageUrl, { cache: "no-store" });
    if (!response.ok) throw new Error(`reviewed package fetch failed: ${response.status}`);
    const packageFile = new File([await response.blob()], "doom-shareware-corpus-v1.zip", { type: "application/zip" });
    const outcome = await submitSelectedPackage(packageFile, session);
    receiveIntakeOutcome(outcome);
    if (outcome.kind !== "retained") {
      throw new Error(`reviewed package intake failed: ${JSON.stringify(outcome)}`);
    }
    await runWorkingMapRotation();
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "autorun-rejected", diagnostic: String(error) }, null, 2);
    rejectObservedOperation("doom-retained-session-rotation", error);
  }
}
if (autorunParameters.get("tokimu_autorun") === "doom-browser-console-adr0019") {
  const packageUrl = autorunParameters.get("tokimu_package");
  if (packageUrl === null) {
    rejectObservedOperation("doom-browser-console-adr0019", "tokimu_package is required");
    throw new Error("Doom browser-console autorun requires tokimu_package");
  }
  try {
    const response = await fetch(packageUrl, { cache: "no-store" });
    if (!response.ok) throw new Error(`reviewed package fetch failed: ${response.status}`);
    const packageFile = new File([await response.blob()], "doom-shareware-corpus-v1.zip", { type: "application/zip" });
    const outcome = await submitSelectedPackage(packageFile, session);
    receiveIntakeOutcome(outcome);
    if (outcome.kind !== "retained") {
      throw new Error(`reviewed package intake failed: ${JSON.stringify(outcome)}`);
    }
    await session.render_working_map(canvas, "E1M1");
    workingWalkaboutActive = true;
    updateWorkingMapControls();
    runWorkingConsoleProof();
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "autorun-rejected", diagnostic: String(error) }, null, 2);
    rejectObservedOperation("doom-browser-console-adr0019", error);
  }
}
