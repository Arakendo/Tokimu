type TokimuIslandMount = {
  root: HTMLElement;
  config: Record<string, unknown>;
  signal: AbortSignal;
};

type TokimuIslandRegistration = {
  release?(): void;
};

interface Window {
  TokimuIslands: {
    register(
      kind: string,
      loader: (mount: TokimuIslandMount) => Promise<TokimuIslandRegistration>,
    ): void;
    unregister(kind: string): void;
    discover(): void;
  };
}
