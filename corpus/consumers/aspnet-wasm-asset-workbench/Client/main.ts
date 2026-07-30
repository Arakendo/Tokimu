import init, {
  engine_status,
  inspect_asset,
} from "/tokimu/tokimu_asset_workbench_engine.js";

type Property = { label: string; value: string };
type PreviewContour = { points: [number, number][]; closed: boolean };
type PreviewPath = {
  contours: PreviewContour[];
  fill: boolean;
  stroke: boolean;
  color: [number, number, number, number];
  stroke_width: number;
};
type PreviewTriangle = { points: [[number, number, number], [number, number, number], [number, number, number]] };
type Vec3 = [number, number, number];
type ProjectedPoint = { x: number; y: number; depth: number };
type AssetObservation = {
  schema: number;
  fileName: string;
  format: string;
  status: string;
  byteLength: number;
  summary: string;
  properties: Property[];
  diagnostics: string[];
  preview: { kind: string; paths: PreviewPath[]; triangles: PreviewTriangle[] } | null;
};

const canvas = required<HTMLCanvasElement>("preview");
const context = canvasContext(canvas);

const dropZone = required<HTMLElement>("drop-zone");
const emptyState = required<HTMLElement>("empty-state");
const fileInput = required<HTMLInputElement>("file-input");
const chooseFile = required<HTMLButtonElement>("choose-file");
let engineReady = false;
let currentObservation: AssetObservation | null = null;
const meshView = {
  yaw: -0.72,
  pitch: 0.38,
  zoom: 1,
  dragging: false,
  pointerX: 0,
  pointerY: 0,
};

async function start(): Promise<void> {
  try {
    await init();
    engineReady = true;
    required("engine-status").textContent = engine_status();
    drawIdle();
  } catch (error) {
    required("engine-status").textContent = `WASM startup failed: ${message(error)}`;
  }
}

chooseFile.addEventListener("click", () => fileInput.click());
fileInput.addEventListener("change", () => {
  const file = fileInput.files?.[0];
  if (file) void inspectFile(file);
});

for (const eventName of ["dragenter", "dragover"]) {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.add("dragging");
  });
}
for (const eventName of ["dragleave", "drop"]) {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.remove("dragging");
  });
}
dropZone.addEventListener("drop", (event) => {
  const file = event.dataTransfer?.files[0];
  if (file) void inspectFile(file);
});
window.addEventListener("resize", () => drawObservation(currentObservation));
canvas.addEventListener("pointerdown", (event) => {
  if (!isMeshObservation(currentObservation)) return;
  meshView.dragging = true;
  meshView.pointerX = event.clientX;
  meshView.pointerY = event.clientY;
  canvas.setPointerCapture(event.pointerId);
  canvas.classList.add("orbiting");
});
canvas.addEventListener("pointermove", (event) => {
  if (!meshView.dragging) return;
  // Orbit the diagnostic model in the same direction as the drag gesture.
  meshView.yaw -= (event.clientX - meshView.pointerX) * 0.012;
  meshView.pitch = clamp(meshView.pitch - (event.clientY - meshView.pointerY) * 0.012, -1.35, 1.35);
  meshView.pointerX = event.clientX;
  meshView.pointerY = event.clientY;
  drawObservation(currentObservation);
});
canvas.addEventListener("pointerup", stopOrbit);
canvas.addEventListener("pointercancel", stopOrbit);
canvas.addEventListener("wheel", (event) => {
  if (!isMeshObservation(currentObservation)) return;
  event.preventDefault();
  meshView.zoom = clamp(meshView.zoom * Math.exp(-event.deltaY * 0.001), 0.45, 3.5);
  drawObservation(currentObservation);
}, { passive: false });
window.addEventListener("keydown", (event) => {
  if (event.key.toLowerCase() !== "r" || !isMeshObservation(currentObservation)) return;
  resetMeshView();
  drawObservation(currentObservation);
});

async function inspectFile(file: File): Promise<void> {
  if (!engineReady) return;
  emptyState.hidden = true;
  required("asset-name").textContent = file.name;
  required("summary").textContent = "Transferring bytes into Tokimu WASM...";

  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const observation = JSON.parse(inspect_asset(file.name, bytes)) as AssetObservation;
    currentObservation = observation;
    resetMeshView();
    presentObservation(observation);
  } catch (error) {
    currentObservation = null;
    presentError(file.name, message(error));
  }
}

function presentObservation(observation: AssetObservation): void {
  required("asset-name").textContent = observation.fileName;
  required("summary").textContent = observation.summary;
  required("format-badge").textContent = `${observation.format} / ${observation.status}`.toUpperCase();

  const properties = required<HTMLDListElement>("properties");
  properties.replaceChildren();
  appendProperty(properties, "Bytes", observation.byteLength.toLocaleString());
  for (const property of observation.properties) {
    appendProperty(properties, property.label, property.value);
  }

  const diagnostics = required<HTMLUListElement>("diagnostics");
  diagnostics.replaceChildren();
  const messages = observation.diagnostics.length
    ? observation.diagnostics
    : ["No importer diagnostics were emitted."];
  for (const diagnostic of messages) {
    const item = document.createElement("li");
    item.textContent = diagnostic;
    diagnostics.append(item);
  }
  drawObservation(observation);
}

function drawObservation(observation: AssetObservation | null): void {
  resizeCanvas();
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = "#080d0f";
  context.fillRect(0, 0, canvas.width, canvas.height);

  if (!observation?.preview) {
    drawPending(observation);
    return;
  }

  if (observation.preview.kind === "mesh-triangles") {
    drawMeshPreview(observation.preview.triangles);
    return;
  }

  if (!observation.preview.paths.length) {
    drawPending(observation);
    return;
  }

  const points = observation.preview.paths.flatMap((path) =>
    path.contours.flatMap((contour) => contour.points)
  );
  const xs = points.map((point) => point[0]);
  const ys = points.map((point) => point[1]);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);
  const width = Math.max(maxX - minX, 0.001);
  const height = Math.max(maxY - minY, 0.001);
  const scale = Math.min((canvas.width - 100) / width, (canvas.height - 100) / height);
  const offsetX = (canvas.width - width * scale) / 2;
  const offsetY = (canvas.height - height * scale) / 2;

  for (const path of observation.preview.paths) {
    const [red, green, blue, alpha] = path.color;
    context.beginPath();
    for (const contour of path.contours) {
      contour.points.forEach(([x, y], index) => {
        const px = offsetX + (x - minX) * scale;
        const py = offsetY + (y - minY) * scale;
        if (index === 0) context.moveTo(px, py);
        else context.lineTo(px, py);
      });
      if (contour.closed) context.closePath();
    }
    const color = `rgba(${red * 255}, ${green * 255}, ${blue * 255}, ${alpha})`;
    if (path.fill) {
      context.fillStyle = color;
      context.fill("evenodd");
    }
    if (path.stroke) {
      context.strokeStyle = color;
      context.lineWidth = Math.max(1.5, path.stroke_width * Math.max(scale, 1));
      context.lineJoin = "round";
      context.lineCap = "round";
      context.stroke();
    }
  }
}

function drawMeshPreview(triangles: PreviewTriangle[]): void {
  if (!triangles.length) {
    drawPending(currentObservation);
    return;
  }

  const bounds = meshBounds(triangles.flatMap((triangle) => triangle.points));
  const view = meshCamera(bounds);
  const projected = triangles.flatMap((triangle) => {
    const points = triangle.points.map((point) => projectMeshPoint(point, view));
    const [a, b, c] = points;
    // Canvas has a downward-facing Y axis, so front-facing GLB triangles
    // arrive with positive screen-space winding after projection.
    const normalZ = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    if (normalZ <= 0) return [];
    const depth = (a.depth + b.depth + c.depth) / 3;
    const brightness = clamp(0.48 + Math.abs(normalZ) * 0.00002, 0.48, 0.9);
    return [{ points, depth, brightness }];
  }).sort((left, right) => right.depth - left.depth);

  context.lineJoin = "round";
  for (const triangle of projected) {
    const [a, b, c] = triangle.points;
    context.beginPath();
    context.moveTo(a.x, a.y);
    context.lineTo(b.x, b.y);
    context.lineTo(c.x, c.y);
    context.closePath();
    const red = Math.round(68 + triangle.brightness * 90);
    const green = Math.round(116 + triangle.brightness * 110);
    const blue = Math.round(107 + triangle.brightness * 80);
    context.fillStyle = `rgb(${red}, ${green}, ${blue})`;
    context.globalAlpha = 1;
    context.fill();
    context.strokeStyle = "#0b1114";
    context.lineWidth = Math.max(1, window.devicePixelRatio || 1);
    context.stroke();
  }
  drawMeshHelp();
}

function meshBounds(points: Vec3[]): { center: Vec3; radius: number } {
  const min: Vec3 = [Infinity, Infinity, Infinity];
  const max: Vec3 = [-Infinity, -Infinity, -Infinity];
  for (const [x, y, z] of points) {
    min[0] = Math.min(min[0], x); min[1] = Math.min(min[1], y); min[2] = Math.min(min[2], z);
    max[0] = Math.max(max[0], x); max[1] = Math.max(max[1], y); max[2] = Math.max(max[2], z);
  }
  const center: Vec3 = [(min[0] + max[0]) / 2, (min[1] + max[1]) / 2, (min[2] + max[2]) / 2];
  return { center, radius: Math.max(0.001, Math.hypot(max[0] - min[0], max[1] - min[1], max[2] - min[2]) / 2) };
}

function meshCamera(bounds: { center: Vec3; radius: number }): { center: Vec3; distance: number; focal: number } {
  return {
    center: bounds.center,
    distance: (bounds.radius * 3.2) / meshView.zoom,
    focal: Math.min(canvas.width, canvas.height) * 0.72,
  };
}

function projectMeshPoint(point: Vec3, view: { center: Vec3; distance: number; focal: number }): ProjectedPoint {
  const x = point[0] - view.center[0];
  const y = point[1] - view.center[1];
  const z = point[2] - view.center[2];
  const cosYaw = Math.cos(meshView.yaw);
  const sinYaw = Math.sin(meshView.yaw);
  const yawX = cosYaw * x + sinYaw * z;
  const yawZ = -sinYaw * x + cosYaw * z;
  const cosPitch = Math.cos(meshView.pitch);
  const sinPitch = Math.sin(meshView.pitch);
  const pitchY = cosPitch * y - sinPitch * yawZ;
  const depth = sinPitch * y + cosPitch * yawZ + view.distance;
  const scale = view.focal / Math.max(depth, 0.001);
  return { x: canvas.width / 2 + yawX * scale, y: canvas.height / 2 - pitchY * scale, depth };
}

function drawMeshHelp(): void {
  context.save();
  context.textAlign = "center";
  context.fillStyle = "#9da7a5";
  context.font = "12px Bahnschrift";
  context.fillText("DRAG TO ORBIT  |  WHEEL TO ZOOM  |  R TO RESET", canvas.width / 2, canvas.height - 24);
  context.restore();
}

function isMeshObservation(observation: AssetObservation | null): boolean {
  return observation?.preview?.kind === "mesh-triangles";
}

function resetMeshView(): void {
  meshView.yaw = -0.72;
  meshView.pitch = 0.38;
  meshView.zoom = 1;
  meshView.dragging = false;
}

function stopOrbit(event: PointerEvent): void {
  meshView.dragging = false;
  canvas.classList.remove("orbiting");
  if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId);
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value));
}

function drawIdle(): void {
  resizeCanvas();
  context.fillStyle = "#080d0f";
  context.fillRect(0, 0, canvas.width, canvas.height);
}

function drawPending(observation: AssetObservation | null): void {
  if (!observation) return;
  context.textAlign = "center";
  context.fillStyle = "#91e0c7";
  context.font = "600 16px Bahnschrift";
  context.fillText(`${observation.format.toUpperCase()} INSPECTED`, canvas.width / 2, canvas.height / 2 - 12);
  context.fillStyle = "#9da7a5";
  context.font = "14px Bahnschrift";
  context.fillText("Render lowering is explicitly pending.", canvas.width / 2, canvas.height / 2 + 18);
}

function presentError(fileName: string, error: string): void {
  required("asset-name").textContent = fileName;
  required("format-badge").textContent = "ERROR";
  required("summary").textContent = "Tokimu rejected the source input.";
  required("properties").replaceChildren();
  const diagnostics = required<HTMLUListElement>("diagnostics");
  diagnostics.replaceChildren();
  const item = document.createElement("li");
  item.textContent = error;
  diagnostics.append(item);
  drawIdle();
}

function resizeCanvas(): void {
  const rect = dropZone.getBoundingClientRect();
  const ratio = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.round(rect.width * ratio));
  canvas.height = Math.max(1, Math.round(rect.height * ratio));
  context.setTransform(1, 0, 0, 1, 0, 0);
}

function appendProperty(list: HTMLDListElement, label: string, value: string): void {
  const term = document.createElement("dt");
  term.textContent = label;
  const detail = document.createElement("dd");
  detail.textContent = value;
  list.append(term, detail);
}

function required<T extends HTMLElement = HTMLElement>(id: string): T {
  const element = document.getElementById(id);
  if (!element) throw new Error(`missing element #${id}`);
  return element as T;
}

function canvasContext(element: HTMLCanvasElement): CanvasRenderingContext2D {
  const value = element.getContext("2d");
  if (!value) throw new Error("Canvas 2D is unavailable");
  return value;
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

void start();
