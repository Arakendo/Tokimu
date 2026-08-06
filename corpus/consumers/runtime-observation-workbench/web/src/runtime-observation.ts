/**
 * The author-facing boundary for the runtime-observation WASM corpus.
 *
 * These are owned JSON copies. They are observations and requests, not a
 * browser-side world model.
 */
export interface RuntimeObservationWasm {
  observation_json(sequence: number, selectedEntity?: number): string;
  latest_observation_diff_json(): string;
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

/** A raw-text Observation Shell boundary. TypeScript never parses commands. */
export interface ObservationShellWasm extends RuntimeObservationWasm {
  execute_json(input: string, sequence: number): string;
  command_catalog_json(): string;
  ratatui_append_text(text: string): void;
  ratatui_backspace(): void;
  ratatui_clear_prompt(): void;
  ratatui_submit(): string;
  ratatui_history_up(): void;
  ratatui_history_down(): void;
  ratatui_scroll_by(lines: number): void;
  ratatui_frame_rgba(width: number, height: number): Uint8Array;
  ratatui_frame_width(): number;
  ratatui_frame_height(): number;
}

export class ObservationShellClient {
  private sequence = 0;

  constructor(private readonly wasm: ObservationShellWasm) {}

  execute(input: string): unknown {
    return JSON.parse(this.wasm.execute_json(input, this.sequence++));
  }

  catalog(): unknown {
    return JSON.parse(this.wasm.command_catalog_json());
  }
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

  latestObservationDiff(): unknown {
    return JSON.parse(this.wasm.latest_observation_diff_json());
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
