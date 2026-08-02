export default function init(moduleOrPath?: RequestInfo | URL): Promise<unknown>;

export class WasmVisualizerSession {
  constructor();
  free(): void;
  set_fixture(fixture: string): void;
  set_paused(paused: boolean): void;
  reset(): void;
  step_json(width: number, height: number): string;
}
