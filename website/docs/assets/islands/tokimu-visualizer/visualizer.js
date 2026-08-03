import init, { WasmVisualizerSession } from "./tokimu_website_visualizer_engine.js";
const canvas = required("[data-canvas]");
const context = requiredCanvasContext(canvas);
const status = required("[data-status]");
const fixture = required("[data-fixture]");
const mode = required("[data-mode]");
const pause = required("[data-pause]");
const reset = required("[data-reset]");
const fixtureLabel = required("[data-fixture-label]");
const frameLabel = required("[data-frame]");
const pulseLabel = required("[data-pulse]");
const sourceLabel = required("[data-source]");
const diagnostic = required("[data-diagnostic]");
const frameMsLabel = required("[data-frame-ms]");
const drawMsLabel = required("[data-draw-ms]");
const startupMsLabel = required("[data-startup-ms]");
const executionModeLabel = required("[data-execution-mode]");
const presetSourceLabel = required("[data-preset-source]");
const literalGeometryLabel = required("[data-literal-geometry]");
const shaderHandlingLabel = required("[data-shader-handling]");
const textureHandlingLabel = required("[data-texture-handling]");
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
mode.addEventListener("change", () => {
    session.set_mode(mode.value);
    status.textContent = `Rust/WASM execution mode selected: ${mode.value}`;
});
pause.addEventListener("click", () => {
    active = !active;
    session.set_paused(!active);
    pause.textContent = active ? "Pause" : "Resume";
});
reset.addEventListener("click", () => { session.reset(); status.textContent = "Rust/WASM visualizer reset"; });
window.addEventListener("pagehide", release, { once: true });
animationFrame = requestAnimationFrame(frame);
function frame(now) {
    if (released)
        return;
    if (now - last > 1000 / 60) {
        const frameIntervalMs = last === 0 ? 0 : now - last;
        last = now;
        const drawStartedAt = performance.now();
        const state = JSON.parse(session.step_json(canvas.width, canvas.height));
        draw(state);
        const drawMs = performance.now() - drawStartedAt;
        fixtureLabel.textContent = state.fixture;
        frameLabel.textContent = String(state.frameIndex);
        pulseLabel.textContent = state.beat.pulse.toFixed(3);
        sourceLabel.textContent = state.diagnostics.audioSource;
        frameMsLabel.textContent = frameIntervalMs === 0 ? "first frame" : `${frameIntervalMs.toFixed(2)} ms`;
        drawMsLabel.textContent = `${drawMs.toFixed(2)} ms`;
        startupMsLabel.textContent = `${startupMs.toFixed(2)} ms`;
        updateMilkDropEvidence(state);
        const customWaveDiagnostic = state.milkdrop && state.milkdrop.customWaveCount > 0
            ? ` Selected literal custom waves: ${state.milkdrop.customWaveCount}; Canvas draws Rust/WASM-lowered audio samples and does not execute custom-wave code.`
            : "";
        const customShapeDiagnostic = state.milkdrop && state.milkdrop.customShapeCount > 0
            ? ` Selected custom shapes: ${state.milkdrop.customShapeCount}; bounded per-frame geometry is evaluated by Rust/WASM, while Canvas only draws returned points. General shape code and textures remain unsupported.`
            : "";
        diagnostic.textContent = `Provider: ${state.diagnostics.provider}. Microphone: ${state.diagnostics.microphonePermission}. Preset evaluator: ${state.diagnostics.presetEvaluator}.${state.milkdrop ? " Canvas is a browser observation view, not native feedback-shader equivalence." : ""}${customWaveDiagnostic}${customShapeDiagnostic}`;
    }
    animationFrame = requestAnimationFrame(frame);
}
function updateMilkDropEvidence(state) {
    if (!state.milkdrop) {
        executionModeLabel.textContent = "original signal field";
        presetSourceLabel.textContent = "not selected";
        literalGeometryLabel.textContent = "not selected";
        shaderHandlingLabel.textContent = "not requested";
        textureHandlingLabel.textContent = "not requested";
        return;
    }
    executionModeLabel.textContent = "Rust/WASM selected subset";
    presetSourceLabel.textContent = "Tokimu-authored selected fixture";
    literalGeometryLabel.textContent = `${state.milkdrop.customWaveCount} literal wave(s), ${state.milkdrop.customShapeCount} bound shape(s)`;
    const inspection = state.milkdrop.shaderInspection;
    shaderHandlingLabel.textContent = `${inspection.entries} source entry(s), ${inspection.blockers} blocker(s); not translated`;
    textureHandlingLabel.textContent = `${inspection.textureSamplingEntries} texture source(s); not resolved`;
}
function release() {
    if (released)
        return;
    released = true;
    cancelAnimationFrame(animationFrame);
    session.free();
}
function draw(state) {
    const { width, height } = canvas;
    const controls = state.milkdrop;
    const phase = controls?.phase ?? state.timeSeconds;
    const low = averageBand(state.bands.subBass, state.bands.bass, "low");
    const mid = averageBand(state.bands.lowMid, state.bands.mid, "mid");
    const high = averageBand(state.bands.highMid, state.bands.treble, "high");
    const zoom = controls?.zoom ?? 1;
    const energy = controls?.audioEnergy ?? state.beat.energy;
    const decay = controls?.decay ?? 1;
    const gradient = context.createRadialGradient(width / 2, height / 2, 1, width / 2, height / 2, width * 0.62);
    gradient.addColorStop(0, `rgba(${20 + Math.round(mid * 35)}, ${65 + Math.round(low * 100)}, ${80 + Math.round(high * 100)}, 1)`);
    gradient.addColorStop(1, "#020809");
    context.fillStyle = gradient;
    context.fillRect(0, 0, width, height);
    context.save();
    context.translate(width / 2, height / 2);
    context.strokeStyle = `rgba(101, 244, 217, ${(0.28 + state.beat.pulse * 0.55) * decay})`;
    context.lineWidth = 1 + energy * 4;
    for (let ring = 1; ring <= 4; ring += 1) {
        context.beginPath();
        context.arc(0, 0, (ring * width * 0.09 + Math.sin(phase * 2 + ring) * low * 22) * zoom, 0, Math.PI * 2);
        context.stroke();
    }
    context.restore();
    context.beginPath();
    state.waveform.forEach(([x, y], index) => {
        const px = (x * 0.45 + 0.5) * width;
        const py = (0.5 - y * (0.18 + mid * 0.32)) * height;
        if (index === 0)
            context.moveTo(px, py);
        else
            context.lineTo(px, py);
    });
    context.strokeStyle = "#aef6e8";
    context.lineWidth = 2;
    context.stroke();
    for (const wave of controls?.customWaves ?? []) {
        const [red, green, blue, alpha] = wave.color;
        context.save();
        context.globalCompositeOperation = wave.additive ? "lighter" : "source-over";
        context.strokeStyle = `rgba(${Math.round(red * 255)}, ${Math.round(green * 255)}, ${Math.round(blue * 255)}, ${alpha})`;
        context.fillStyle = context.strokeStyle;
        context.lineWidth = wave.thick ? 3 : 1.5;
        context.beginPath();
        wave.points.forEach(([x, y], index) => {
            const px = x * width;
            const py = y * height;
            if (index === 0)
                context.moveTo(px, py);
            else
                context.lineTo(px, py);
        });
        if (wave.dots) {
            for (const [x, y] of wave.points) {
                context.beginPath();
                context.arc(x * width, y * height, wave.thick ? 2.2 : 1.4, 0, Math.PI * 2);
                context.fill();
            }
        }
        else {
            context.stroke();
        }
        context.restore();
    }
    for (const shape of controls?.customShapes ?? []) {
        if (shape.points.length < 3)
            continue;
        const [red, green, blue, alpha] = shape.color;
        context.save();
        context.globalCompositeOperation = shape.additive ? "lighter" : "source-over";
        context.strokeStyle = `rgba(${Math.round(red * 255)}, ${Math.round(green * 255)}, ${Math.round(blue * 255)}, ${alpha})`;
        context.fillStyle = `rgba(${Math.round(red * 255)}, ${Math.round(green * 255)}, ${Math.round(blue * 255)}, ${alpha * 0.22})`;
        context.lineWidth = shape.thickOutline ? 3 : 1.5;
        context.beginPath();
        shape.points.forEach(([x, y], index) => {
            const px = x * width;
            const py = y * height;
            if (index === 0)
                context.moveTo(px, py);
            else
                context.lineTo(px, py);
        });
        context.closePath();
        context.fill();
        context.stroke();
        context.restore();
    }
    status.textContent = `Rust/WASM active | ${state.mode} | ${state.fixture} | ${state.paused ? "paused" : "running"}`;
}
function averageBand(first, second, label) {
    if (!Number.isFinite(first) || !Number.isFinite(second)) {
        throw new Error(`Rust/WASM snapshot supplied non-finite ${label} audio-band data.`);
    }
    return (first + second) * 0.5;
}
function required(selector) {
    const element = document.querySelector(selector);
    if (!element)
        throw new Error(`Visualizer consumer requires ${selector}.`);
    return element;
}
function requiredCanvasContext(canvas) {
    const context = canvas.getContext("2d");
    if (!context)
        throw new Error("This browser does not provide a 2D canvas context.");
    return context;
}
