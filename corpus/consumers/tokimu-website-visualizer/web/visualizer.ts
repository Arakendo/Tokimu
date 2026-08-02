import init, { WasmVisualizerSession } from "./tokimu_website_visualizer_engine.js";

type Snapshot = {
  schema: number;
  fixture: string;
  frameIndex: number;
  timeSeconds: number;
  paused: boolean;
  bands: Record<string, number>;
  beat: { energy: number; pulse: number; onset: boolean };
  waveform: [number, number][];
  diagnostics: { audioSource: string; microphonePermission: string; presetEvaluator: string; provider: string };
};

const canvas = required<HTMLCanvasElement>("[data-canvas]");
const context = requiredCanvasContext(canvas);
const status = required<HTMLOutputElement>("[data-status]");
const fixture = required<HTMLSelectElement>("[data-fixture]");
const pause = required<HTMLButtonElement>("[data-pause]");
const reset = required<HTMLButtonElement>("[data-reset]");
const fixtureLabel = required<HTMLElement>("[data-fixture-label]");
const frameLabel = required<HTMLElement>("[data-frame]");
const pulseLabel = required<HTMLElement>("[data-pulse]");
const sourceLabel = required<HTMLElement>("[data-source]");
const diagnostic = required<HTMLElement>("[data-diagnostic]");
const frameMsLabel = required<HTMLElement>("[data-frame-ms]");
const drawMsLabel = required<HTMLElement>("[data-draw-ms]");
const startupMsLabel = required<HTMLElement>("[data-startup-ms]");

const startupStartedAt = performance.now();
await init();
const session = new WasmVisualizerSession();
const startupMs = performance.now() - startupStartedAt;
let active = true;
let last = 0;
let released = false;
let animationFrame = 0;

fixture.addEventListener("change", () => {
  session.set_fixture(fixture.value);
  status.textContent = `Rust/WASM fixture selected: ${fixture.value}`;
});
pause.addEventListener("click", () => {
  active = !active;
  session.set_paused(!active);
  pause.textContent = active ? "Pause" : "Resume";
});
reset.addEventListener("click", () => { session.reset(); status.textContent = "Rust/WASM visualizer reset"; });
window.addEventListener("pagehide", release, { once: true });
animationFrame = requestAnimationFrame(frame);

function frame(now: number): void {
  if (released) return;
  if (now - last > 1000 / 60) {
    const frameIntervalMs = last === 0 ? 0 : now - last;
    last = now;
    const drawStartedAt = performance.now();
    const state = JSON.parse(session.step_json(canvas.width, canvas.height)) as Snapshot;
    draw(state);
    const drawMs = performance.now() - drawStartedAt;
    fixtureLabel.textContent = state.fixture;
    frameLabel.textContent = String(state.frameIndex);
    pulseLabel.textContent = state.beat.pulse.toFixed(3);
    sourceLabel.textContent = state.diagnostics.audioSource;
    frameMsLabel.textContent = frameIntervalMs === 0 ? "first frame" : `${frameIntervalMs.toFixed(2)} ms`;
    drawMsLabel.textContent = `${drawMs.toFixed(2)} ms`;
    startupMsLabel.textContent = `${startupMs.toFixed(2)} ms`;
    diagnostic.textContent = `Provider: ${state.diagnostics.provider}. Microphone: ${state.diagnostics.microphonePermission}. Preset evaluator: ${state.diagnostics.presetEvaluator}.`;
  }
  animationFrame = requestAnimationFrame(frame);
}

function release(): void {
  if (released) return;
  released = true;
  cancelAnimationFrame(animationFrame);
  session.free();
}

function draw(state: Snapshot): void {
  const { width, height } = canvas;
  const phase = state.timeSeconds;
  const low = (state.bands.subBass + state.bands.bass) * 0.5;
  const mid = (state.bands.lowMid + state.bands.mid) * 0.5;
  const high = (state.bands.highMid + state.bands.treble) * 0.5;
  const gradient = context.createRadialGradient(width / 2, height / 2, 1, width / 2, height / 2, width * 0.62);
  gradient.addColorStop(0, `rgba(${20 + Math.round(mid * 35)}, ${65 + Math.round(low * 100)}, ${80 + Math.round(high * 100)}, 1)`);
  gradient.addColorStop(1, "#020809");
  context.fillStyle = gradient;
  context.fillRect(0, 0, width, height);
  context.save();
  context.translate(width / 2, height / 2);
  context.strokeStyle = `rgba(101, 244, 217, ${0.28 + state.beat.pulse * 0.55})`;
  context.lineWidth = 1 + state.beat.energy * 4;
  for (let ring = 1; ring <= 4; ring += 1) {
    context.beginPath();
    context.arc(0, 0, ring * width * 0.09 + Math.sin(phase * 2 + ring) * low * 22, 0, Math.PI * 2);
    context.stroke();
  }
  context.restore();
  context.beginPath();
  state.waveform.forEach(([x, y], index) => {
    const px = (x * 0.45 + 0.5) * width;
    const py = (0.5 - y * (0.18 + mid * 0.32)) * height;
    if (index === 0) context.moveTo(px, py); else context.lineTo(px, py);
  });
  context.strokeStyle = "#aef6e8";
  context.lineWidth = 2;
  context.stroke();
  status.textContent = `Rust/WASM active | ${state.fixture} | ${state.paused ? "paused" : "running"}`;
}

function required<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`Visualizer consumer requires ${selector}.`);
  return element;
}

function requiredCanvasContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const context = canvas.getContext("2d");
  if (!context) throw new Error("This browser does not provide a 2D canvas context.");
  return context;
}
