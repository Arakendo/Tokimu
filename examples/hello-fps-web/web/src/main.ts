import { createHudElements, renderHud } from "./hud.js";
import type { FpsFrameSnapshot } from "./protocol.js";

const demoFrame: FpsFrameSnapshot = {
  frame: 0,
  elapsedSeconds: 0,
  player: { x: 0, y: 0, z: 0, yaw: 0, pitch: 0 },
  hud: {
    score: 0,
    wave: 1,
    targets: 0,
    projectiles: 0,
    status: "booting",
  },
};

let animationFrameHandle = 0;
let pollHandle = 0;
let startTime = 0;
let usingExternalFeed = false;
let currentFrame: FpsFrameSnapshot = cloneFrame(demoFrame);

export function bootstrapHelloFpsWeb(): void {
  if (typeof document === "undefined") {
    return;
  }

  const hud = createHudElements(document);
  const viewport = createViewportElements(document);
  startTime = performance.now();
  currentFrame = cloneFrame(demoFrame);

  const publishFrame = (snapshot: FpsFrameSnapshot): void => {
    usingExternalFeed = true;
    currentFrame = cloneFrame(snapshot);
    renderHud(hud, currentFrame);
    renderViewport(viewport, currentFrame, true);
  };

  window.tokimuHelloFpsWebPushFrame = publishFrame;
  window.addEventListener("tokimu:fps-frame", (event) => {
    publishFrame(event.detail);
  });

  pollHandle = window.setInterval(async () => {
    if (usingExternalFeed) {
      return;
    }

    try {
      const response = await fetch("./live-frame.json?cacheBust=" + Date.now(), {
        cache: "no-store",
      });
      if (!response.ok) {
        return;
      }

      const snapshot = (await response.json()) as FpsFrameSnapshot;
      publishFrame(snapshot);
    } catch {
      // Keep the demo preview running until the Rust process publishes a frame.
    }
  }, 100);

  const tick = (timestamp: number): void => {
    if (usingExternalFeed) {
      return;
    }

    const elapsedSeconds = (timestamp - startTime) / 1000;
    currentFrame.frame += 1;
    currentFrame.elapsedSeconds = elapsedSeconds;
    currentFrame.player.x = Math.sin(elapsedSeconds * 0.7) * 2.2;
    currentFrame.player.y = 1.6 + Math.sin(elapsedSeconds * 1.6) * 0.1;
    currentFrame.player.z = Math.cos(elapsedSeconds * 0.7) * 2.2;
    currentFrame.player.yaw = Math.sin(elapsedSeconds * 0.45) * 1.2;
    currentFrame.player.pitch = Math.cos(elapsedSeconds * 0.85) * 0.22;
    currentFrame.hud.score = Math.floor(elapsedSeconds * 2.5) % 99;
    currentFrame.hud.wave = 1 + Math.floor(elapsedSeconds / 18);
    currentFrame.hud.targets = 8 - (Math.floor(elapsedSeconds * 0.9) % 5);
    currentFrame.hud.projectiles = Math.floor((elapsedSeconds * 6) % 12);
    currentFrame.hud.status = elapsedSeconds < 1.0 ? "booting browser HUD" : "browser shell previewing demo feed";
    renderHud(hud, currentFrame);
    renderViewport(viewport, currentFrame, false);
    animationFrameHandle = window.requestAnimationFrame(tick);
  };

  if (typeof window !== "undefined") {
    currentFrame.hud.status = "waiting for Rust frame feed";
    renderHud(hud, currentFrame);
    renderViewport(viewport, currentFrame, false);
    window.setTimeout(() => {
      if (!usingExternalFeed) {
        currentFrame.hud.status = "TypeScript shell ready";
        renderHud(hud, currentFrame);
        renderViewport(viewport, currentFrame, false);
        animationFrameHandle = window.requestAnimationFrame(tick);
      }
    }, 0);
  }
}

function createViewportElements(document: Document): {
  canvas: HTMLCanvasElement;
  context: CanvasRenderingContext2D;
} {
  const canvas = requireElement(document, "tokimu-canvas") as HTMLCanvasElement;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Unable to create 2D canvas context for tokimu-canvas");
  }

  return { canvas, context };
}

function renderViewport(
  viewport: { canvas: HTMLCanvasElement; context: CanvasRenderingContext2D },
  snapshot: FpsFrameSnapshot,
  liveFeed: boolean,
): void {
  const { canvas, context } = viewport;
  const dpr = window.devicePixelRatio || 1;
  const width = Math.max(1, Math.floor(canvas.clientWidth || canvas.width));
  const height = Math.max(1, Math.floor(canvas.clientHeight || canvas.height));
  const targetWidth = Math.max(1, Math.floor(width * dpr));
  const targetHeight = Math.max(1, Math.floor(height * dpr));

  if (canvas.width !== targetWidth || canvas.height !== targetHeight) {
    canvas.width = targetWidth;
    canvas.height = targetHeight;
  }

  context.save();
  context.scale(dpr, dpr);
  context.clearRect(0, 0, width, height);

  const frameHue = liveFeed ? 192 : 212;
  const accentHue = snapshot.hud.score * 2.1 % 360;
  const background = context.createLinearGradient(0, 0, width, height);
  background.addColorStop(0, `hsla(${frameHue}, 45%, 7%, 1)`);
  background.addColorStop(1, "hsla(222, 36%, 4%, 1)");
  context.fillStyle = background;
  context.fillRect(0, 0, width, height);

  drawGrid(context, width, height, snapshot.elapsedSeconds, liveFeed);
  drawArena(context, width, height, snapshot, accentHue, liveFeed);
  drawHudOverlay(context, width, height, snapshot, liveFeed);

  context.restore();
}

function drawGrid(
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
  elapsedSeconds: number,
  liveFeed: boolean,
): void {
  const spacing = 48;
  const shimmer = 0.18 + (liveFeed ? 0.1 : 0.04) * Math.sin(elapsedSeconds * 2.0);
  context.strokeStyle = `rgba(143, 226, 255, ${shimmer})`;
  context.lineWidth = 1;

  for (let x = 0; x <= width; x += spacing) {
    context.beginPath();
    context.moveTo(x + 0.5, 0);
    context.lineTo(x + 0.5, height);
    context.stroke();
  }

  for (let y = 0; y <= height; y += spacing) {
    context.beginPath();
    context.moveTo(0, y + 0.5);
    context.lineTo(width, y + 0.5);
    context.stroke();
  }

  const centerX = width * 0.5;
  const centerY = height * 0.56;
  context.strokeStyle = liveFeed ? "rgba(255, 195, 106, 0.35)" : "rgba(143, 226, 255, 0.22)";
  context.beginPath();
  context.arc(centerX, centerY, Math.min(width, height) * 0.14, 0, Math.PI * 2);
  context.stroke();
  context.beginPath();
  context.moveTo(centerX - 16, centerY);
  context.lineTo(centerX + 16, centerY);
  context.moveTo(centerX, centerY - 16);
  context.lineTo(centerX, centerY + 16);
  context.stroke();
}

function drawArena(
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
  snapshot: FpsFrameSnapshot,
  accentHue: number,
  liveFeed: boolean,
): void {
  const centerX = width * 0.5;
  const centerY = height * 0.56;
  const orbit = Math.min(width, height) * 0.23;
  const playerAngle = snapshot.player.yaw;
  const playerDistance = 30 + snapshot.player.pitch * 35;

  // Player marker
  context.save();
  context.translate(centerX, centerY);
  context.rotate(playerAngle);
  context.fillStyle = liveFeed ? "rgba(255, 195, 106, 0.95)" : "rgba(143, 226, 255, 0.95)";
  context.beginPath();
  context.moveTo(0, -playerDistance);
  context.lineTo(8, -playerDistance + 18);
  context.lineTo(-8, -playerDistance + 18);
  context.closePath();
  context.fill();
  context.restore();

  const targetCount = Math.max(1, snapshot.hud.targets);
  for (let index = 0; index < targetCount; index += 1) {
    const angle = (index / targetCount) * Math.PI * 2 + snapshot.elapsedSeconds * 0.45;
    const radius = orbit * (0.68 + 0.08 * Math.sin(snapshot.elapsedSeconds + index));
    const x = centerX + Math.cos(angle) * radius;
    const y = centerY + Math.sin(angle) * radius * 0.74;
    context.fillStyle = `hsla(${(accentHue + index * 18) % 360}, 92%, 66%, 0.9)`;
    context.beginPath();
    context.arc(x, y, 10, 0, Math.PI * 2);
    context.fill();
  }

  const projectileCount = Math.max(0, snapshot.hud.projectiles);
  for (let index = 0; index < projectileCount; index += 1) {
    const offset = snapshot.elapsedSeconds * 1.8 + index * 0.42;
    const x = centerX + Math.cos(offset) * orbit * 0.45;
    const y = centerY + Math.sin(offset * 1.7) * orbit * 0.28 - index * 4;
    context.fillStyle = "rgba(255, 242, 184, 0.95)";
    context.beginPath();
    context.arc(x, y, 4 + (index % 3), 0, Math.PI * 2);
    context.fill();
  }
}

function drawHudOverlay(
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
  snapshot: FpsFrameSnapshot,
  liveFeed: boolean,
): void {
  const label = liveFeed ? "LIVE RUST FRAME" : "DEMO VIEW";
  const status = snapshot.hud.status;
  context.fillStyle = "rgba(6, 10, 18, 0.68)";
  context.fillRect(16, 16, Math.min(width - 32, 360), 76);
  context.strokeStyle = "rgba(143, 226, 255, 0.22)";
  context.strokeRect(16.5, 16.5, Math.min(width - 33, 360), 76);

  context.fillStyle = liveFeed ? "#ffc36a" : "#8fe2ff";
  context.font = "600 13px Segoe UI, Trebuchet MS, sans-serif";
  context.fillText(label, 30, 40);

  context.fillStyle = "#f5f7fb";
  context.font = "500 14px Segoe UI, Trebuchet MS, sans-serif";
  context.fillText(`frame ${snapshot.frame}  score ${snapshot.hud.score}  wave ${snapshot.hud.wave}`, 30, 60);
  context.fillStyle = "#a5b1c8";
  context.font = "12px Segoe UI, Trebuchet MS, sans-serif";
  context.fillText(status, 30, 80);

  const bottomLabel = liveFeed ? "Rust snapshot bridge active" : "Waiting for Rust frame feed";
  context.fillStyle = "rgba(6, 10, 18, 0.60)";
  context.fillRect(16, height - 56, Math.min(width - 32, 360), 40);
  context.strokeRect(16.5, height - 55.5, Math.min(width - 33, 360), 39);
  context.fillStyle = "#f5f7fb";
  context.fillText(bottomLabel, 30, height - 31);
}

function cloneFrame(snapshot: FpsFrameSnapshot): FpsFrameSnapshot {
  return {
    frame: snapshot.frame,
    elapsedSeconds: snapshot.elapsedSeconds,
    player: {
      x: snapshot.player.x,
      y: snapshot.player.y,
      z: snapshot.player.z,
      yaw: snapshot.player.yaw,
      pitch: snapshot.player.pitch,
    },
    hud: {
      score: snapshot.hud.score,
      wave: snapshot.hud.wave,
      targets: snapshot.hud.targets,
      projectiles: snapshot.hud.projectiles,
      status: snapshot.hud.status,
    },
  };
}

function requireElement(document: Document, id: string): HTMLElement {
  const element = document.getElementById(id);
  if (!element) {
    throw new Error(`Missing required element: ${id}`);
  }
  return element;
}

if (typeof window !== "undefined") {
  window.addEventListener("DOMContentLoaded", bootstrapHelloFpsWeb);
  window.addEventListener("beforeunload", () => {
    if (animationFrameHandle !== 0) {
      window.cancelAnimationFrame(animationFrameHandle);
    }
    if (pollHandle !== 0) {
      window.clearInterval(pollHandle);
    }
  });
}
