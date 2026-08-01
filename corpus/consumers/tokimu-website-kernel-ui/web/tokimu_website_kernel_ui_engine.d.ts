export default function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<unknown>;

export class KernelUiSession {
  constructor();
  observation_json(): string;
  set_filter(value: string): string;
  select_resource(resourceId: bigint): string;
  set_name(value: string): string;
  set_notes(value: string): string;
  toggle_visibility(): string;
  toggle_hotspot(): string;
  apply(): string;
  revert(): string;
  request_delete(): string;
  cancel_delete(): string;
  confirm_delete(): string;
  dispose(): void;
  free(): void;
}
