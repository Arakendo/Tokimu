import type { FpsFrameSnapshot } from "./protocol.js";

export interface HudElements {
  frame: HTMLElement;
  elapsed: HTMLElement;
  player: HTMLElement;
  score: HTMLElement;
  wave: HTMLElement;
  targets: HTMLElement;
  projectiles: HTMLElement;
  status: HTMLElement;
}

export function createHudElements(document: Document): HudElements {
  return {
    frame: requireElement(document, "hud-frame"),
    elapsed: requireElement(document, "hud-elapsed"),
    player: requireElement(document, "hud-player"),
    score: requireElement(document, "hud-score"),
    wave: requireElement(document, "hud-wave"),
    targets: requireElement(document, "hud-targets"),
    projectiles: requireElement(document, "hud-projectiles"),
    status: requireElement(document, "status"),
  };
}

export function renderHud(elements: HudElements, snapshot: FpsFrameSnapshot): void {
  elements.frame.textContent = String(snapshot.frame);
  elements.elapsed.textContent = `${snapshot.elapsedSeconds.toFixed(1)}s`;
  elements.player.textContent = `pos ${snapshot.player.x.toFixed(1)}, ${snapshot.player.y.toFixed(1)}, ${snapshot.player.z.toFixed(1)} | look ${snapshot.player.yaw.toFixed(2)}, ${snapshot.player.pitch.toFixed(2)}`;
  elements.score.textContent = String(snapshot.hud.score);
  elements.wave.textContent = String(snapshot.hud.wave);
  elements.targets.textContent = String(snapshot.hud.targets);
  elements.projectiles.textContent = String(snapshot.hud.projectiles);
  elements.status.textContent = snapshot.hud.status;
}

function requireElement(document: Document, id: string): HTMLElement {
  const element = document.getElementById(id);
  if (!element) {
    throw new Error(`Missing required HUD element: ${id}`);
  }
  return element;
}
