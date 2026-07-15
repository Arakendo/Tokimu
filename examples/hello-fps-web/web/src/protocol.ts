export interface FpsHudSnapshot {
  score: number;
  wave: number;
  targets: number;
  projectiles: number;
  status: string;
}

export interface FpsPlayerSnapshot {
  x: number;
  y: number;
  z: number;
  yaw: number;
  pitch: number;
}

export interface FpsFrameSnapshot {
  frame: number;
  elapsedSeconds: number;
  player: FpsPlayerSnapshot;
  hud: FpsHudSnapshot;
}

declare global {
  interface Window {
    tokimuHelloFpsWebPushFrame?: (snapshot: FpsFrameSnapshot) => void;
  }

  interface WindowEventMap {
    "tokimu:fps-frame": CustomEvent<FpsFrameSnapshot>;
  }
}

export {};
