import init, { WasmPaintSession } from "./tokimu_website_paint_engine.js";

type Observation = {
  document?: { width: number; height: number; revision: number; dirty: boolean };
  history?: { undoDepth: number; redoDepth: number };
};
type CanvasPoint = { x: number; y: number };
type Tool = "pencil" | "erase" | "fill" | "sample";

const canvas = need<HTMLCanvasElement>("[data-canvas]");
const context = needCanvasContext(canvas);

const canvasShell = need<HTMLElement>("[data-canvas-shell]");
const canvasLabel = need<HTMLElement>("[data-canvas-label]");
const status = need<HTMLOutputElement>("[data-status]");
const details = need<HTMLElement>("[data-details]");
const colorInput = need<HTMLInputElement>("[data-color]");
const brushSizeInput = need<HTMLInputElement>("[data-brush-size]");
const brushSizeValue = need<HTMLOutputElement>("[data-brush-size-value]");
const undoButton = need<HTMLButtonElement>("[data-undo]");
const redoButton = need<HTMLButtonElement>("[data-redo]");
const zoomInput = need<HTMLInputElement>("[data-zoom]");
const zoomValue = need<HTMLOutputElement>("[data-zoom-value]");
const observationList = need<HTMLOListElement>("[data-observations]");

let session: WasmPaintSession;
let tool: Tool = "pencil";
let strokePoints: CanvasPoint[] = [];
let activePointerId: number | undefined;
let brushDiameter = Number(brushSizeInput.value);
let zoomPercent = Number(zoomInput.value);
const startupStartedAt = performance.now();

await boot();

async function boot() {
  await init();
  session = blankSession();
  bindControls();
  const startupMilliseconds = performance.now() - startupStartedAt;
  await refresh(`Blank document ready (${startupMilliseconds.toFixed(1)} ms startup)`);
  recordObservation(
    `WASM startup observed at ${startupMilliseconds.toFixed(1)} ms; timing is diagnostic evidence, not a cross-browser guarantee`,
  );
}

function bindControls() {
  document.querySelectorAll<HTMLButtonElement>("[data-tool]").forEach((button) => {
    button.addEventListener("click", () => selectTool(button.dataset.tool as Tool));
  });
  selectTool("pencil");
  setBrushDiameter(brushDiameter);

  need<HTMLButtonElement>("[data-new]").addEventListener("click", async () => {
    replaceSession(blankSession());
    await refresh("Blank document ready");
  });
  undoButton.addEventListener("click", () => void undo());
  redoButton.addEventListener("click", () => void redo());
  need<HTMLButtonElement>("[data-reset]").addEventListener("click", () => void reset());
  need<HTMLButtonElement>("[data-export]").addEventListener("click", downloadPng);
  need<HTMLButtonElement>("[data-zoom-in]").addEventListener("click", () => setZoom(zoomPercent + 25));
  need<HTMLButtonElement>("[data-zoom-out]").addEventListener("click", () => setZoom(zoomPercent - 25));
  need<HTMLButtonElement>("[data-fit]").addEventListener("click", fitPreview);
  brushSizeInput.addEventListener("input", () => setBrushDiameter(Number(brushSizeInput.value)));
  zoomInput.addEventListener("input", () => setZoom(Number(zoomInput.value)));
  need<HTMLInputElement>("[data-open]").addEventListener("change", (event) => {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (file) void openFile(file);
  });

  canvas.addEventListener("pointerdown", (event) => void beginPointerEdit(event));
  canvas.addEventListener("pointermove", collectPointerEdit);
  canvas.addEventListener("pointerup", (event) => void finishPointerEdit(event));
  canvas.addEventListener("pointercancel", cancelPointerEdit);
  canvas.addEventListener("lostpointercapture", cancelLostPointerEdit);

  canvasShell.addEventListener("dragover", (event) => event.preventDefault());
  canvasShell.addEventListener("drop", (event) => {
    event.preventDefault();
    const file = event.dataTransfer?.files[0];
    if (file) void openFile(file);
  });

  document.addEventListener("keydown", (event) => void handleKey(event));
  window.addEventListener(
    "pagehide",
    () => {
      session.dispose();
    },
    { once: true },
  );
}

function selectTool(nextTool: Tool) {
  tool = nextTool;
  document.querySelectorAll<HTMLButtonElement>("[data-tool]").forEach((button) => {
    button.dataset.active = String(button.dataset.tool === tool);
  });
  status.value = `${tool} selected`;
  recordObservation(`${tool} selected`);
}

function blankSession() {
  return new WasmPaintSession(640, 400, 0, 0, 0, 0);
}

function replaceSession(nextSession: WasmPaintSession) {
  session.dispose();
  session = nextSession;
  strokePoints = [];
}

async function openFile(file: File) {
  try {
    // Do not discard the current document until Rust accepts the new source.
    const nextSession = WasmPaintSession.open(
      new Uint8Array(await file.arrayBuffer()),
      file.type || fileExtension(file.name),
    );
    replaceSession(nextSession);
    await refresh(`Opened ${file.name}`);
  } catch (error) {
    reportError("Open failed", error);
  }
}

async function beginPointerEdit(event: PointerEvent) {
  if (!event.isPrimary || event.button !== 0) return;
  event.preventDefault();
  canvas.setPointerCapture(event.pointerId);
  activePointerId = event.pointerId;
  const point = canvasPoint(event);
  if (tool === "fill") {
    await applyCommand({ kind: "floodFill", origin: point, replacement: rgba() });
    releasePointer(event);
  } else if (tool === "sample") {
    const sampled = session.sample_rgba(point.x, point.y);
    colorInput.value = `#${[sampled[0], sampled[1], sampled[2]]
      .map((channel) => channel.toString(16).padStart(2, "0"))
      .join("")}`;
    status.value = "Color sampled";
    recordObservation("Color sampled from the Rust-owned document");
    releasePointer(event);
  } else {
    strokePoints = [point];
    drawLivePixelLine(point, point);
    status.value = "Drawing preview";
  }
}

function collectPointerEdit(event: PointerEvent) {
  if (!isActiveStroke(event)) return;
  event.preventDefault();

  const next = canvasPoint(event);
  const previous = strokePoints.at(-1);
  if (!previous || samePoint(previous, next)) return;

  strokePoints.push(next);
  drawLivePixelLine(previous, next);
}

async function finishPointerEdit(event: PointerEvent) {
  if (!isActiveStroke(event)) return;
  event.preventDefault();

  const finalPoint = canvasPoint(event);
  const previous = strokePoints.at(-1);
  if (previous && !samePoint(previous, finalPoint)) {
    strokePoints.push(finalPoint);
    drawLivePixelLine(previous, finalPoint);
  }

  const points = strokePoints;
  strokePoints = [];
  releasePointer(event);
  await applyCommand(strokeCommand(points));
}

function cancelPointerEdit(event: PointerEvent) {
  if (activePointerId !== event.pointerId) return;
  event.preventDefault();
  strokePoints = [];
  releasePointer(event);
  status.value = "Stroke cancelled";
  recordObservation("Pointer stroke cancelled");
}

function cancelLostPointerEdit(event: PointerEvent) {
  if (activePointerId !== event.pointerId) return;
  activePointerId = undefined;
  strokePoints = [];
}

function releasePointer(event: PointerEvent) {
  if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId);
  if (activePointerId === event.pointerId) activePointerId = undefined;
}

function isActiveStroke(event: PointerEvent) {
  return (
    activePointerId === event.pointerId &&
    strokePoints.length > 0 &&
    canvas.hasPointerCapture(event.pointerId)
  );
}

// This is presentation-only feedback. Rust receives the complete point list on
// pointer release and remains the sole owner of committed document pixels.
function drawLivePixelLine(start: CanvasPoint, end: CanvasPoint) {
  context.save();
  context.globalCompositeOperation = tool === "erase" ? "destination-out" : "source-over";
  context.fillStyle = colorInput.value;

  let x = start.x;
  let y = start.y;
  const deltaX = Math.abs(end.x - x);
  const stepX = x < end.x ? 1 : -1;
  const deltaY = -Math.abs(end.y - y);
  const stepY = y < end.y ? 1 : -1;
  let error = deltaX + deltaY;

  while (true) {
    drawLiveBrushStamp(x, y);
    if (x === end.x && y === end.y) break;
    const doubledError = 2 * error;
    if (doubledError >= deltaY) {
      error += deltaY;
      x += stepX;
    }
    if (doubledError <= deltaX) {
      error += deltaX;
      y += stepY;
    }
  }

  context.restore();
}

function drawLiveBrushStamp(centerX: number, centerY: number) {
  const radius = Math.floor(brushDiameter / 2);
  for (let y = centerY - radius; y <= centerY + radius; y += 1) {
    for (let x = centerX - radius; x <= centerX + radius; x += 1) {
      const deltaX = x - centerX;
      const deltaY = y - centerY;
      if (deltaX * deltaX + deltaY * deltaY <= radius * radius) {
        context.fillRect(x, y, 1, 1);
      }
    }
  }
}

function samePoint(left: CanvasPoint, right: CanvasPoint) {
  return left.x === right.x && left.y === right.y;
}

function strokeCommand(points: CanvasPoint[]) {
  if (brushDiameter === 1) {
    return tool === "erase"
      ? { kind: "eraseStroke", points }
      : { kind: "pencilStroke", points, color: rgba() };
  }

  return {
    kind: "brushStroke",
    points,
    color: tool === "erase" ? transparentRgba() : rgba(),
    diameter: brushDiameter,
  };
}

function setBrushDiameter(nextDiameter: number) {
  brushDiameter = clamp(
    Math.round(nextDiameter),
    Number(brushSizeInput.min),
    Number(brushSizeInput.max),
  );
  brushSizeInput.value = String(brushDiameter);
  brushSizeValue.value = `${brushDiameter} px`;
}

async function handleKey(event: KeyboardEvent) {
  if (event.ctrlKey || event.metaKey) {
    if (event.key.toLowerCase() === "z") {
      event.preventDefault();
      if (event.shiftKey) await redo(); else await undo();
      return;
    }
    if (event.key.toLowerCase() === "y") {
      event.preventDefault();
      await redo();
      return;
    }
    if (event.key.toLowerCase() === "s") {
      event.preventDefault();
      downloadPng();
      return;
    }
  }

  if (event.target instanceof HTMLInputElement) return;
  const shortcut: Record<string, Tool> = { b: "pencil", e: "erase", f: "fill", i: "sample" };
  const nextTool = shortcut[event.key.toLowerCase()];
  if (nextTool) {
    event.preventDefault();
    selectTool(nextTool);
  }
}

async function undo() {
  try {
    await refresh("Undo", session.undo_json());
  } catch (error) {
    reportError("Undo failed", error);
  }
}

async function redo() {
  try {
    await refresh("Redo", session.redo_json());
  } catch (error) {
    reportError("Redo failed", error);
  }
}

async function reset() {
  try {
    await refresh("Reset", session.reset_json());
  } catch (error) {
    reportError("Reset failed", error);
  }
}

async function applyCommand(command: unknown) {
  try {
    await refresh("Edited", session.apply_json(JSON.stringify(command)));
  } catch (error) {
    reportError("Edit failed", error);
  }
}

async function refresh(message: string, controlJson?: string) {
  if (controlJson) JSON.parse(controlJson);
  const preview = JSON.parse(session.preview_observation_json()) as {
    width: number;
    height: number;
  };
  canvas.width = preview.width;
  canvas.height = preview.height;
  canvasLabel.textContent = `Document canvas / ${preview.width} x ${preview.height}`;
  context.putImageData(
    new ImageData(new Uint8ClampedArray(session.preview_bytes()), preview.width, preview.height),
    0,
    0,
  );
  applyZoom();

  const observation = JSON.parse(session.observation_json()) as Observation;
  updateHistoryControls(observation);
  status.value = message;
  details.textContent = `${observation.document?.width} x ${observation.document?.height} | revision ${observation.document?.revision} | undo ${observation.history?.undoDepth} | redo ${observation.history?.redoDepth}`;
  recordObservation(message);
}

function updateHistoryControls(observation: Observation) {
  const undoDepth = observation.history?.undoDepth ?? 0;
  const redoDepth = observation.history?.redoDepth ?? 0;

  undoButton.disabled = undoDepth === 0;
  redoButton.disabled = redoDepth === 0;
  undoButton.title = undoDepth === 0 ? "Nothing to undo" : `Undo (${undoDepth} available)`;
  redoButton.title = redoDepth === 0 ? "Nothing to redo" : `Redo (${redoDepth} available)`;
}

function setZoom(nextPercent: number) {
  zoomPercent = clamp(nextPercent, Number(zoomInput.min), Number(zoomInput.max));
  zoomInput.value = String(zoomPercent);
  applyZoom();
  zoomValue.value = `${zoomPercent}%`;
}

function applyZoom() {
  canvas.style.width = `${Math.round((canvas.width * zoomPercent) / 100)}px`;
  canvas.style.height = `${Math.round((canvas.height * zoomPercent) / 100)}px`;
  zoomValue.value = `${zoomPercent}%`;
}

function fitPreview() {
  const availableWidth = Math.max(1, canvasShell.clientWidth - 32);
  const availableHeight = Math.max(1, canvasShell.clientHeight - 32);
  setZoom(Math.floor(Math.min((availableWidth / canvas.width) * 100, (availableHeight / canvas.height) * 100) / 25) * 25);
  status.value = "Preview fitted";
  recordObservation("Preview fitted without changing document pixels");
}

function canvasPoint(event: PointerEvent): CanvasPoint {
  const bounds = canvas.getBoundingClientRect();
  return {
    x: clamp(Math.floor(((event.clientX - bounds.left) * canvas.width) / bounds.width), 0, canvas.width - 1),
    y: clamp(Math.floor(((event.clientY - bounds.top) * canvas.height) / bounds.height), 0, canvas.height - 1),
  };
}

function rgba() {
  const value = colorInput.value.slice(1);
  return {
    red: Number.parseInt(value.slice(0, 2), 16),
    green: Number.parseInt(value.slice(2, 4), 16),
    blue: Number.parseInt(value.slice(4, 6), 16),
    alpha: 255,
  };
}

function transparentRgba() {
  return { red: 0, green: 0, blue: 0, alpha: 0 };
}

function downloadPng() {
  try {
    // Copy from WASM memory before the browser takes ownership of the Blob.
    const wasmBytes = session.export_png_bytes();
    const bytes = new Uint8Array(wasmBytes.length);
    bytes.set(wasmBytes);
    const link = document.createElement("a");
    link.href = URL.createObjectURL(new Blob([bytes.buffer], { type: "image/png" }));
    link.download = "tokimu-paint.png";
    link.click();
    setTimeout(() => URL.revokeObjectURL(link.href), 0);
    status.value = "PNG exported";
    recordObservation("Deterministic PNG export copied from WASM memory");
  } catch (error) {
    reportError("Export failed", error);
  }
}

function reportError(prefix: string, error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  status.value = `${prefix}: ${message}`;
  recordObservation(`${prefix}: ${message}`);
}

function recordObservation(message: string) {
  const item = document.createElement("li");
  item.textContent = message;
  observationList.prepend(item);
  while (observationList.children.length > 8) observationList.lastElementChild?.remove();
}

function fileExtension(name: string) {
  return name.split(".").pop() || "";
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(maximum, Math.max(minimum, value));
}

function need<T extends Element>(selector: string) {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`Missing ${selector}`);
  return element;
}

function needCanvasContext(target: HTMLCanvasElement) {
  const value = target.getContext("2d", { alpha: true });
  if (!value) throw new Error("Canvas 2D context is unavailable");
  return value;
}
