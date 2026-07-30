export default function init(moduleOrPath?: RequestInfo | URL): Promise<unknown>;

export class AsteroidsSession {
  constructor(seed: number);
  free(): void;
  set_viewport(width: number, height: number): void;
  step(inputJson: string, deltaSeconds: number): string;
  snapshot(): string;
  reset(seed: number): string;
}
