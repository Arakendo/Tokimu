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
let startTime = 0;
let usingExternalFeed = false;
let currentFrame: FpsFrameSnapshot = cloneFrame(demoFrame);

export function bootstrapHelloFpsWeb(): void {
  if (typeof document === "undefined") {
    return;
  }

  const hud = createHudElements(document);
  startTime = performance.now();
  currentFrame = cloneFrame(demoFrame);

  const publishFrame = (snapshot: FpsFrameSnapshot): void => {
    usingExternalFeed = true;
    currentFrame = cloneFrame(snapshot);
    renderHud(hud, currentFrame);
  };

  window.tokimuHelloFpsWebPushFrame = publishFrame;
  window.addEventListener("tokimu:fps-frame", (event) => {
    publishFrame(event.detail);
  });

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
    animationFrameHandle = window.requestAnimationFrame(tick);
  };

  if (typeof window !== "undefined") {
    currentFrame.hud.status = "waiting for Rust frame feed";
    renderHud(hud, currentFrame);
    window.setTimeout(() => {
      if (!usingExternalFeed) {
        currentFrame.hud.status = "TypeScript shell ready";
        renderHud(hud, currentFrame);
        animationFrameHandle = window.requestAnimationFrame(tick);
      }
    }, 0);
  }
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

if (typeof window !== "undefined") {
  window.addEventListener("DOMContentLoaded", bootstrapHelloFpsWeb);
  window.addEventListener("beforeunload", () => {
    if (animationFrameHandle !== 0) {
      window.cancelAnimationFrame(animationFrameHandle);
    }
  });
}
