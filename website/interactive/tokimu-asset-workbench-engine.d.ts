declare module "*tokimu_asset_workbench_engine.js" {
  export default function init(moduleOrPath?: RequestInfo | URL): Promise<unknown>;
  export function engine_status(): string;
  export function inspect_asset(fileName: string, bytes: Uint8Array): string;
}
