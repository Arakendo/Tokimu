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

const FPS_FRAME_SCHEMA_ID = "tokimu.example.fps-frame-snapshot";
const FPS_FRAME_SCHEMA_VERSION = 1;
const PROTOCOL_VERSION = 1;

interface ObservationEnvelope {
  protocol_version: number;
  schema_id: string;
  schema_version: number;
  sequence: number;
  message_kind: string;
  payload: number[];
}

/** Decodes the native file bridge without making wire framing part of HUD state. */
export function decodeFpsFrameEnvelope(value: unknown): FpsFrameSnapshot {
  if (!isRecord(value)) {
    throw new Error("FPS frame envelope must be an object");
  }

  const envelope = value as Partial<ObservationEnvelope>;
  if (envelope.protocol_version !== PROTOCOL_VERSION) {
    throw new Error("unsupported FPS frame protocol version");
  }
  if (
    envelope.schema_id !== FPS_FRAME_SCHEMA_ID ||
    envelope.schema_version !== FPS_FRAME_SCHEMA_VERSION
  ) {
    throw new Error("unsupported FPS frame schema");
  }
  if (envelope.message_kind !== "observation_snapshot") {
    throw new Error("unsupported FPS frame message kind");
  }
  if (!isSequence(envelope.sequence)) {
    throw new Error("FPS frame sequence must be a non-negative integer");
  }
  if (!Array.isArray(envelope.payload) || envelope.payload.some((byte) => !isByte(byte))) {
    throw new Error("FPS frame payload must be a byte array");
  }

  const payload = JSON.parse(new TextDecoder().decode(new Uint8Array(envelope.payload))) as unknown;
  if (!isFpsFrameSnapshot(payload)) {
    throw new Error("FPS frame payload does not match the snapshot contract");
  }
  if (payload.frame !== envelope.sequence) {
    throw new Error("FPS frame sequence does not match its application frame");
  }
  return payload;
}

function isFpsFrameSnapshot(value: unknown): value is FpsFrameSnapshot {
  if (!isRecord(value) || !isFiniteNumber(value.frame) || !isFiniteNumber(value.elapsedSeconds)) {
    return false;
  }
  const player = value.player;
  const hud = value.hud;
  return (
    isRecord(player) &&
    isFiniteNumber(player.x) &&
    isFiniteNumber(player.y) &&
    isFiniteNumber(player.z) &&
    isFiniteNumber(player.yaw) &&
    isFiniteNumber(player.pitch) &&
    isRecord(hud) &&
    isFiniteNumber(hud.score) &&
    isFiniteNumber(hud.wave) &&
    isFiniteNumber(hud.targets) &&
    isFiniteNumber(hud.projectiles) &&
    typeof hud.status === "string"
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function isByte(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 255;
}

function isSequence(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
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
