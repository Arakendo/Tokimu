export type Rgb = { red: number; green: number; blue: number };

export type PresentationTargetRef = { kind: string; key: string };

export type MaterialAuthoringState = {
  tint: Rgb;
  opacity: number;
  visible: boolean;
  emphasis: "selected" | "warning" | "hotspot" | null;
};

export type WasmPresentationRequest = {
  kind: string;
  key: string;
  layer: "application" | "hotspot";
  overrideValue: {
    tint: { color: Rgb; mode: "replace" };
    opacityMultiplier: number;
    visible: boolean;
    emphasis: "selected" | "warning" | "hotspot" | null;
  };
};

/**
 * The local precursor to an eventual @tokimu/shader authoring package.
 * It is deliberately data-only: it cannot emit WGSL or access browser state.
 */
export function lowerMaterialAuthoring(
  target: PresentationTargetRef,
  state: MaterialAuthoringState,
): WasmPresentationRequest {
  return {
    kind: target.kind,
    key: target.key,
    layer: "application",
    overrideValue: {
      tint: { color: state.tint, mode: "replace" },
      opacityMultiplier: clamp(state.opacity, 0, 1),
      visible: state.visible,
      emphasis: state.emphasis,
    },
  };
}

export function colorFromHex(value: string): Rgb {
  const number = Number.parseInt(value.slice(1), 16);
  return {
    red: ((number >> 16) & 0xff) / 255,
    green: ((number >> 8) & 0xff) / 255,
    blue: (number & 0xff) / 255,
  };
}

export function colorHex(color: Rgb): string {
  const component = (value: number) => Math.round(clamp(value, 0, 1) * 255).toString(16).padStart(2, "0");
  return `#${component(color.red)}${component(color.green)}${component(color.blue)}`;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value));
}
