import init, {
  engine_status,
  presentation_scene,
  PresentationSession,
} from "/tokimu/tokimu_presentation_workbench_engine.js";
import {
  colorFromHex,
  colorHex,
  lowerMaterialAuthoring,
  type MaterialAuthoringState,
  type PresentationTargetRef,
} from "./material-authoring.js";

type Rgb = { red: number; green: number; blue: number };
type SourcePresentation = { color: Rgb; opacity: number; visible: boolean };
type Target = PresentationTargetRef & { sourceName: string; source: SourcePresentation };
type Shape = PresentationTargetRef & {
  label: string;
  geometry: { kind: "polygon"; points: [number, number][] } | { kind: "circle"; center: [number, number]; radius: number };
};
type Scene = { schema: number; summary: string; targets: Target[]; shapes: Shape[] };
type ResolvedPresentation = SourcePresentation & { emphasis: "selected" | "warning" | "hotspot" | null };
type Response =
  | { status: "resolved"; resolved: ResolvedPresentation }
  | { status: "rejected"; diagnostic: { code: string; message: string } };

const canvas = required<HTMLCanvasElement>("viewport");
const context = requiredContext(canvas);
const targetSelect = required<HTMLSelectElement>("target");
const tint = required<HTMLInputElement>("tint");
const opacity = required<HTMLInputElement>("opacity");
const visible = required<HTMLInputElement>("visible");
const hotspot = required<HTMLButtonElement>("hotspot");
const reset = required<HTMLButtonElement>("reset");
const status = required<HTMLElement>("status");

let scene: Scene | null = null;
let session: PresentationSession | null = null;
let selectedKey = "";
const resolved = new Map<string, ResolvedPresentation>();

void start();
window.addEventListener("resize", draw);
canvas.addEventListener("click", (event) => selectAt(event));
targetSelect.addEventListener("change", () => {
  selectedKey = targetSelect.value;
  loadControls();
  draw();
});
tint.addEventListener("input", applyAuthoring);
opacity.addEventListener("input", applyAuthoring);
visible.addEventListener("change", applyAuthoring);
hotspot.addEventListener("click", toggleHotspot);
reset.addEventListener("click", resetApplicationLayer);

async function start(): Promise<void> {
  try {
    await init();
    required("engine-status").textContent = engine_status();
    scene = JSON.parse(presentation_scene()) as Scene;
    session = new PresentationSession(JSON.stringify(scene));
    required("summary").textContent = scene.summary;
    for (const target of scene.targets) {
      const option = document.createElement("option");
      option.value = key(target);
      option.textContent = `${target.sourceName} (${target.kind})`;
      targetSelect.append(option);
    }
    selectedKey = targetSelect.value;
    loadControls();
    draw();
  } catch (error) {
    status.textContent = `WASM startup failed: ${message(error)}`;
  }
}

function selectAt(event: MouseEvent): void {
  if (!scene) return;
  const rectangle = canvas.getBoundingClientRect();
  const point: [number, number] = [
    (event.clientX - rectangle.left) / rectangle.width,
    (event.clientY - rectangle.top) / rectangle.height,
  ];
  const shape = [...scene.shapes].reverse().find((candidate) => contains(candidate, point));
  if (!shape) return;
  selectedKey = key(shape);
  targetSelect.value = selectedKey;
  loadControls();
  status.textContent = `Selected ${shape.label}; authoring controls lower into a Tokimu presentation command.`;
  draw();
}

function applyAuthoring(): void {
  const target = selectedTarget();
  if (!target || !session) return;
  const state: MaterialAuthoringState = {
    tint: colorFromHex(tint.value),
    opacity: Number(opacity.value),
    visible: visible.checked,
    emphasis: "selected",
  };
  const response = command(session.set_override(JSON.stringify(lowerMaterialAuthoring(target, state))));
  if (!response) return;
  resolved.set(key(target), response);
  status.textContent = `Tokimu resolved application presentation for ${target.sourceName}.`;
  draw();
}

function toggleHotspot(): void {
  const target = selectedTarget();
  if (!target || !session) return;
  const current = resolvedFor(target);
  const request = { kind: target.kind, key: target.key, layer: "hotspot" };
  const response = current.emphasis === "hotspot"
    ? command(session.clear_override(JSON.stringify(request)))
    : command(session.set_override(JSON.stringify({
      ...request,
      overrideValue: {
        tint: { color: { red: 1, green: 0.56, blue: 0.16 }, mode: "replace" },
        opacityMultiplier: 1,
        visible: true,
        emphasis: "hotspot",
      },
    })));
  if (!response) return;
  resolved.set(key(target), response);
  hotspot.textContent = response.emphasis === "hotspot" ? "Clear hotspot" : "Focus hotspot";
  status.textContent = `Tokimu resolved ${response.emphasis === "hotspot" ? "hotspot" : "restored"} presentation.`;
  loadControls();
  draw();
}

function resetApplicationLayer(): void {
  const target = selectedTarget();
  if (!target || !session) return;
  const response = command(session.clear_override(JSON.stringify({ kind: target.kind, key: target.key, layer: "application" })));
  if (!response) return;
  resolved.set(key(target), response);
  status.textContent = `Restored source presentation for ${target.sourceName}.`;
  loadControls();
  draw();
}

function loadControls(): void {
  const target = selectedTarget();
  if (!target) return;
  const value = resolvedFor(target);
  tint.value = colorHex(value.color);
  opacity.value = value.opacity.toString();
  visible.checked = value.visible;
  hotspot.textContent = value.emphasis === "hotspot" ? "Clear hotspot" : "Focus hotspot";
}

function draw(): void {
  resizeCanvas();
  context.fillStyle = "#071014";
  context.fillRect(0, 0, canvas.width, canvas.height);
  if (!scene) return;
  for (const shape of scene.shapes) drawShape(shape);
}

function drawShape(shape: Shape): void {
  const value = resolvedFor(shape);
  if (!value.visible) return;
  context.save();
  context.globalAlpha = value.opacity;
  context.fillStyle = toCss(value.color);
  context.strokeStyle = value.emphasis === "hotspot" ? "#ffad3e" : value.emphasis === "selected" ? "#d9f7ff" : "#19313a";
  context.lineWidth = value.emphasis ? 5 : 2;
  context.beginPath();
  if (shape.geometry.kind === "polygon") {
    shape.geometry.points.forEach(([x, y], index) => {
      const point = toCanvas([x, y]);
      if (index === 0) context.moveTo(point[0], point[1]); else context.lineTo(point[0], point[1]);
    });
    context.closePath();
  } else {
    const center = toCanvas(shape.geometry.center);
    context.arc(center[0], center[1], shape.geometry.radius * Math.min(canvas.width, canvas.height), 0, Math.PI * 2);
  }
  context.fill();
  context.stroke();
  context.restore();
}

function contains(shape: Shape, point: [number, number]): boolean {
  if (shape.geometry.kind === "circle") {
    const [x, y] = shape.geometry.center;
    return Math.hypot(point[0] - x, point[1] - y) <= shape.geometry.radius;
  }
  let inside = false;
  const points = shape.geometry.points;
  for (let index = 0, previous = points.length - 1; index < points.length; previous = index++) {
    const [x, y] = points[index];
    const [previousX, previousY] = points[previous];
    const crosses = (y > point[1]) !== (previousY > point[1]) && point[0] < ((previousX - x) * (point[1] - y)) / (previousY - y) + x;
    if (crosses) inside = !inside;
  }
  return inside;
}

function resolvedFor(target: PresentationTargetRef): ResolvedPresentation {
  const source = scene?.targets.find((candidate) => key(candidate) === key(target))?.source;
  if (!source) throw new Error(`Unknown presentation target ${key(target)}.`);
  return resolved.get(key(target)) ?? { ...source, emphasis: null };
}

function selectedTarget(): Target | undefined { return scene?.targets.find((target) => key(target) === selectedKey); }
function key(target: PresentationTargetRef): string { return `${target.kind}:${target.key}`; }
function command(json: string): ResolvedPresentation | undefined {
  const response = JSON.parse(json) as Response;
  if (response.status === "resolved") return response.resolved;
  status.textContent = `${response.diagnostic.code}: ${response.diagnostic.message}`;
  return undefined;
}
function resizeCanvas(): void {
  const rectangle = canvas.getBoundingClientRect();
  const scale = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.round(rectangle.width * scale));
  canvas.height = Math.max(1, Math.round(rectangle.height * scale));
  context.setTransform(scale, 0, 0, scale, 0, 0);
}
function toCanvas(point: [number, number]): [number, number] { return [point[0] * canvas.clientWidth, point[1] * canvas.clientHeight]; }
function toCss(color: Rgb): string { return `rgb(${Math.round(color.red * 255)} ${Math.round(color.green * 255)} ${Math.round(color.blue * 255)})`; }
function message(error: unknown): string { return error instanceof Error ? error.message : String(error); }
function required<T extends HTMLElement>(id: string): T { const element = document.getElementById(id); if (!element) throw new Error(`Missing #${id}.`); return element as T; }
function requiredContext(element: HTMLCanvasElement): CanvasRenderingContext2D {
  const canvasContext = element.getContext("2d");
  if (!canvasContext) throw new Error("2D canvas support is required for this corpus consumer.");
  return canvasContext;
}
