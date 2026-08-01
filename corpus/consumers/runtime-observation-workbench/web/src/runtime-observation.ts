/**
 * The author-facing boundary for the runtime-observation WASM corpus.
 *
 * These are owned JSON copies. They are observations and requests, not a
 * browser-side world model.
 */
export interface RuntimeObservationWasm {
  observation_json(sequence: number, selectedEntity?: number): string;
  ui_snapshot_json(width: number, height: number, sequence: number, selectedEntity?: number): string;
  enqueue_json(requestJson: string): string;
  apply_json(tick: number): string;
  presentation_json(): string;
  select_arm_presentation_json(): string;
  animation_catalog_json(): string;
  playback_json(): string;
  playback_command_json(commandJson: string): string;
  advance_animation_fixed_step(): string;
}

export interface Position {
  x: number;
  y: number;
  z: number;
}

export type RuntimeCommand =
  | { command: "move_by"; delta: Position }
  | { command: "set_enabled"; enabled: boolean };

export interface CommandRequest {
  id: number;
  target: number;
  authority: "observer" | "operator";
  expected_revision?: number;
  command: RuntimeCommand;
}

export type PlaybackCommand =
  | { command: "play"; clip: number }
  | { command: "pause" }
  | { command: "resume" }
  | { command: "stop" }
  | { command: "seek"; seconds: number }
  | { command: "set_speed"; speed: number }
  | { command: "set_looping"; looping: boolean }
  | { command: "next_step" }
  | { command: "reset" };

/** Keeps transport details out of UI components and sequences observations. */
export class RuntimeObservationClient {
  private sequence = 0;

  constructor(private readonly wasm: RuntimeObservationWasm) {}

  observe(selectedEntity?: number): unknown {
    return JSON.parse(this.wasm.observation_json(this.sequence++, selectedEntity));
  }

  observeUi(width: number, height: number, selectedEntity?: number): unknown {
    return JSON.parse(this.wasm.ui_snapshot_json(width, height, this.sequence++, selectedEntity));
  }

  enqueue(request: CommandRequest): unknown {
    return JSON.parse(this.wasm.enqueue_json(JSON.stringify(request)));
  }

  apply(tick: number): unknown {
    return JSON.parse(this.wasm.apply_json(tick));
  }

  playback(command: PlaybackCommand): unknown {
    return JSON.parse(this.wasm.playback_command_json(JSON.stringify(command)));
  }
}
