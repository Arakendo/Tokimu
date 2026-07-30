import init, { AsteroidsSession, } from "./tokimu_website_asteroids_engine.js";
const canvas = required("[data-game-canvas]");
const context = canvasContext(canvas);
const status = required("[data-status]");
const score = required("[data-score]");
const wave = required("[data-wave]");
const lives = required("[data-lives]");
const combo = required("[data-combo]");
const comboWrap = required("[data-combo-wrap]");
const curtain = required("[data-curtain]");
const curtainKicker = required("[data-curtain-kicker]");
const curtainTitle = required("[data-curtain-title]");
const curtainDetail = required("[data-curtain-detail]");
const restart = required("[data-restart]");
const FIXED_STEP = 1 / 120;
const MAX_STEPS = 8;
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
const keys = new Set();
const pulses = { pause: false, restart: false };
const pointer = { x: 0, y: 0, active: false, firing: false, id: null };
const stars = Array.from({ length: 90 }, (_, index) => ({
    x: hash01(index * 13 + 1),
    y: hash01(index * 29 + 7),
    size: 0.5 + hash01(index * 41 + 9) * 1.4,
    phase: hash01(index * 71 + 5) * Math.PI * 2,
}));
let session = null;
let snapshot = null;
let animationFrame = 0;
let previousTime = performance.now();
let accumulator = 0;
let seed = 0x746f_6b69;
let inputAbort = null;
await boot();
async function boot() {
    try {
        await init();
        session = new AsteroidsSession(seed);
        resize();
        snapshot = parseSnapshot(session.snapshot());
        status.textContent = "Rust/WASM simulation ready";
        installInput();
        animationFrame = requestAnimationFrame(frame);
    }
    catch (error) {
        status.textContent = `Unable to start: ${message(error)}`;
        curtain.hidden = false;
        curtainKicker.textContent = "STARTUP DIAGNOSTIC";
        curtainTitle.textContent = "Unavailable";
        curtainDetail.textContent =
            "The static corpus description remains valid; the WASM consumer did not start.";
        throw error;
    }
}
function frame(now) {
    if (!session)
        return;
    accumulator = Math.min(accumulator + Math.min((now - previousTime) / 1000, 0.1), FIXED_STEP * MAX_STEPS);
    previousTime = now;
    let steps = 0;
    while (accumulator >= FIXED_STEP && steps < MAX_STEPS) {
        snapshot = parseSnapshot(session.step(JSON.stringify(consumeInput()), FIXED_STEP));
        accumulator -= FIXED_STEP;
        steps += 1;
    }
    if (snapshot) {
        draw(snapshot, now / 1000);
        updateHud(snapshot);
    }
    animationFrame = requestAnimationFrame(frame);
}
function consumeInput() {
    const aim = pointer.active ? canvasToWorld(pointer) : null;
    const input = {
        thrust: keys.has("KeyW") || keys.has("ArrowUp"),
        brake: keys.has("KeyS") || keys.has("ArrowDown"),
        turnLeft: keys.has("KeyA") || keys.has("ArrowLeft"),
        turnRight: keys.has("KeyD") || keys.has("ArrowRight"),
        fire: keys.has("Space") || pointer.firing,
        pausePressed: pulses.pause,
        restartPressed: pulses.restart,
    };
    if (aim) {
        input.aimX = aim.x;
        input.aimY = aim.y;
    }
    pulses.pause = false;
    pulses.restart = false;
    return input;
}
function draw(state, time) {
    const dpr = devicePixelRatio || 1;
    context.save();
    context.setTransform(dpr, 0, 0, dpr, 0, 0);
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    context.clearRect(0, 0, width, height);
    const shake = reducedMotion.matches ? 0 : state.screenShake * 4;
    context.translate(Math.sin(time * 83) * shake, Math.cos(time * 71) * shake);
    drawField(width, height, time);
    drawParticles(state);
    drawAsteroids(state);
    drawProjectiles(state);
    drawShip(state, time);
    context.restore();
}
function drawField(width, height, time) {
    const gradient = context.createRadialGradient(width * 0.52, height * 0.48, 0, width * 0.52, height * 0.48, Math.max(width, height) * 0.7);
    gradient.addColorStop(0, "#0b2023");
    gradient.addColorStop(0.55, "#061113");
    gradient.addColorStop(1, "#020708");
    context.fillStyle = gradient;
    context.fillRect(-12, -12, width + 24, height + 24);
    for (const star of stars) {
        context.globalAlpha = 0.3 + Math.sin(time + star.phase) * 0.18;
        context.fillStyle = "#b9fff4";
        context.fillRect(star.x * width, star.y * height, star.size, star.size);
    }
    context.globalAlpha = 1;
}
function drawAsteroids(state) {
    context.lineWidth = 1.5;
    for (const asteroid of state.asteroids) {
        const position = worldToCanvas(asteroid.position, state);
        const radius = worldScale(asteroid.radius, state);
        context.beginPath();
        for (let index = 0; index < 10; index += 1) {
            const angle = asteroid.angle + (index / 10) * Math.PI * 2;
            const noise = 0.79 + hash01(asteroid.id * 31 + index * 17) * 0.28;
            const x = position.x + Math.cos(angle) * radius * noise;
            const y = position.y + Math.sin(angle) * radius * noise;
            if (index === 0)
                context.moveTo(x, y);
            else
                context.lineTo(x, y);
        }
        context.closePath();
        context.fillStyle =
            asteroid.size === "large"
                ? "rgba(84, 121, 125, 0.34)"
                : "rgba(122, 171, 170, 0.4)";
        context.strokeStyle = "#9bc9c5";
        context.fill();
        context.stroke();
    }
}
function drawProjectiles(state) {
    context.save();
    context.globalCompositeOperation = "lighter";
    context.strokeStyle = "#f7c45b";
    context.shadowColor = "#f7c45b";
    context.shadowBlur = 10;
    context.lineWidth = 2;
    for (const projectile of state.projectiles) {
        const position = worldToCanvas(projectile.position, state);
        context.beginPath();
        context.moveTo(position.x, position.y);
        context.lineTo(position.x - Math.cos(projectile.angle) * 8, position.y - Math.sin(projectile.angle) * 8);
        context.stroke();
    }
    context.restore();
}
function drawParticles(state) {
    context.save();
    const prefersReducedMotion = reducedMotion.matches;
    context.globalCompositeOperation = prefersReducedMotion ? "source-over" : "lighter";
    for (let index = 0; index < state.particles.length; index += 1) {
        if (prefersReducedMotion && index % 3 !== 0)
            continue;
        const particle = state.particles[index];
        const position = worldToCanvas(particle.position, state);
        const life = Math.max(0, 1 - particle.normalizedAge);
        const motionScale = prefersReducedMotion ? 0.65 : 1;
        const radius = Math.max(0.6, worldScale(particle.size, state) * motionScale);
        const rgb = particle.kind === "thrust"
            ? "76, 231, 219"
            : particle.kind === "ship"
                ? "255, 119, 84"
                : particle.kind === "muzzle"
                    ? "255, 231, 154"
                    : particle.kind === "wave"
                        ? "132, 174, 255"
                        : particle.kind === "score"
                            ? "113, 255, 176"
                            : "247, 196, 91";
        context.fillStyle = `rgba(${rgb}, ${life})`;
        context.beginPath();
        context.arc(position.x, position.y, radius, 0, Math.PI * 2);
        context.fill();
    }
    context.restore();
}
function drawShip(state, time) {
    if (state.mode === "gameOver" ||
        (state.ship.invulnerable > 0 && Math.floor(time * 9) % 2 === 0)) {
        return;
    }
    const position = worldToCanvas(state.ship.position, state);
    const radius = worldScale(state.ship.radius, state);
    context.save();
    context.translate(position.x, position.y);
    context.rotate(state.ship.angle);
    context.strokeStyle = "#72e6d5";
    context.fillStyle = "rgba(68, 220, 205, 0.16)";
    context.lineWidth = 2;
    context.beginPath();
    context.moveTo(radius * 1.3, 0);
    context.lineTo(-radius, radius * 0.78);
    context.lineTo(-radius * 0.55, 0);
    context.lineTo(-radius, -radius * 0.78);
    context.closePath();
    context.fill();
    context.stroke();
    context.restore();
}
function updateHud(state) {
    score.textContent = state.score.toString().padStart(6, "0");
    wave.textContent = state.wave.toString().padStart(2, "0");
    lives.textContent = state.lives.toString();
    combo.textContent = `x${state.combo}`;
    comboWrap.hidden = state.combo <= 1 || state.comboRemaining <= 0;
    curtain.hidden = state.mode === "playing";
    if (state.mode === "playing") {
        status.textContent = "Simulation active";
    }
    else if (state.mode === "paused") {
        curtainKicker.textContent = "FIELD STATUS";
        curtainTitle.textContent = "Paused";
        curtainDetail.textContent = "Press P to continue.";
        status.textContent = "Simulation paused";
    }
    else {
        curtainKicker.textContent = `SCORE ${state.score.toString().padStart(6, "0")}`;
        curtainTitle.textContent = "Field lost";
        curtainDetail.textContent = "Press Enter or use Restart field.";
        status.textContent = `Game over · high score ${state.highScore}`;
    }
}
function installInput() {
    inputAbort?.abort();
    inputAbort = new AbortController();
    const options = { signal: inputAbort.signal };
    addEventListener("keydown", (event) => {
        if (["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(event.code)) {
            event.preventDefault();
        }
        if (!event.repeat && event.code === "KeyP")
            pulses.pause = true;
        if (!event.repeat && event.code === "Enter")
            pulses.restart = true;
        keys.add(event.code);
    }, options);
    addEventListener("keyup", (event) => keys.delete(event.code), options);
    addEventListener("blur", clearInput, options);
    canvas.addEventListener("pointermove", updatePointer, options);
    canvas.addEventListener("pointerenter", (event) => {
        pointer.active = true;
        updatePointer(event);
    }, options);
    canvas.addEventListener("pointerleave", (event) => {
        if (event.pointerType === "mouse" && pointer.id === null) {
            pointer.active = false;
            pointer.firing = false;
        }
    }, options);
    canvas.addEventListener("pointerdown", (event) => {
        event.preventDefault();
        pointer.id = event.pointerId;
        pointer.active = true;
        pointer.firing = true;
        canvas.setPointerCapture(event.pointerId);
        updatePointer(event);
    }, options);
    canvas.addEventListener("pointerup", releasePointer, options);
    canvas.addEventListener("pointercancel", releasePointer, options);
    restart.addEventListener("click", () => {
        if (!session)
            return;
        seed = (seed + 1) >>> 0;
        snapshot = parseSnapshot(session.reset(seed));
    }, options);
    addEventListener("resize", resize, options);
    addEventListener("pagehide", release, { once: true, signal: inputAbort.signal });
}
function clearInput() {
    keys.clear();
    pulses.pause = false;
    pulses.restart = false;
    pointer.firing = false;
}
function releasePointer(event) {
    if (pointer.id !== event.pointerId)
        return;
    pointer.firing = false;
    pointer.id = null;
    if (canvas.hasPointerCapture(event.pointerId)) {
        canvas.releasePointerCapture(event.pointerId);
    }
    if (event.pointerType !== "mouse") {
        pointer.active = false;
    }
}
function updatePointer(event) {
    const bounds = canvas.getBoundingClientRect();
    pointer.x = event.clientX - bounds.left;
    pointer.y = event.clientY - bounds.top;
}
function resize() {
    const dpr = Math.min(devicePixelRatio || 1, 2);
    const width = Math.max(320, canvas.clientWidth);
    const height = Math.max(180, canvas.clientHeight);
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    session?.set_viewport(100, 100 * (height / width));
}
function worldToCanvas(position, state) {
    return {
        x: ((position.x + state.width * 0.5) / state.width) * canvas.clientWidth,
        y: ((position.y + state.height * 0.5) / state.height) * canvas.clientHeight,
    };
}
function canvasToWorld(position) {
    if (!snapshot)
        return null;
    return {
        x: (position.x / canvas.clientWidth) * snapshot.width - snapshot.width * 0.5,
        y: (position.y / canvas.clientHeight) * snapshot.height -
            snapshot.height * 0.5,
    };
}
function worldScale(value, state) {
    return value * (canvas.clientWidth / state.width);
}
function parseSnapshot(json) {
    const value = JSON.parse(json);
    if (value.schema !== 1)
        throw new Error(`Unsupported snapshot schema ${value.schema}`);
    return value;
}
function hash01(value) {
    const mixed = Math.sin(value * 12.9898) * 43758.5453;
    return mixed - Math.floor(mixed);
}
function required(selector) {
    const element = document.querySelector(selector);
    if (!element)
        throw new Error(`Missing required element: ${selector}`);
    return element;
}
function canvasContext(element) {
    const value = element.getContext("2d");
    if (!value)
        throw new Error("Canvas 2D is unavailable.");
    return value;
}
function message(error) {
    return error instanceof Error ? error.message : String(error);
}
export function disposeAsteroids() {
    release();
}
function release() {
    inputAbort?.abort();
    inputAbort = null;
    clearInput();
    if (pointer.id !== null && canvas.hasPointerCapture(pointer.id)) {
        canvas.releasePointerCapture(pointer.id);
    }
    pointer.active = false;
    pointer.id = null;
    cancelAnimationFrame(animationFrame);
    animationFrame = 0;
    session?.free();
    session = null;
}
