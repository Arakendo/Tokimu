import {
  mapRatatuiKeyboardInput,
  mapRatatuiWheelDelta,
  type BrowserRatatuiInput,
} from "./ratatui-input.js";

const equal = (actual: BrowserRatatuiInput | undefined, expected: BrowserRatatuiInput | undefined) => {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`expected ${JSON.stringify(expected)}, received ${JSON.stringify(actual)}`);
  }
};

equal(mapRatatuiKeyboardInput({ key: "Enter" }), { kind: "submit" });
equal(mapRatatuiKeyboardInput({ key: "ArrowUp" }), { kind: "history_up" });
equal(mapRatatuiKeyboardInput({ key: "PageDown" }), { kind: "scroll", lines: 8 });
equal(mapRatatuiKeyboardInput({ key: "x" }), { kind: "append_text", text: "x" });
equal(mapRatatuiKeyboardInput({ key: "x", ctrlKey: true }), undefined);
equal(mapRatatuiKeyboardInput({ key: "Tab" }), undefined);
equal(mapRatatuiWheelDelta(12), { kind: "scroll", lines: 3 });
equal(mapRatatuiWheelDelta(-12), { kind: "scroll", lines: -3 });
equal(mapRatatuiWheelDelta(0), undefined);

console.log("runtime-observation-workbench Ratatui browser input mapping: ok");
