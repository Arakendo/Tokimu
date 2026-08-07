// Browser event translation stays in this consumer. The terminal surface only
// receives semantic actions and never imports DOM event types.
export type BrowserKeyInput = Readonly<{
  key: string;
  ctrlKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
}>;

export type BrowserRatatuiInput =
  | Readonly<{ kind: "submit" }>
  | Readonly<{ kind: "backspace" }>
  | Readonly<{ kind: "clear_prompt" }>
  | Readonly<{ kind: "history_up" }>
  | Readonly<{ kind: "history_down" }>
  | Readonly<{ kind: "scroll"; lines: number }>
  | Readonly<{ kind: "append_text"; text: string }>;

export const mapRatatuiKeyboardInput = (
  input: BrowserKeyInput,
): BrowserRatatuiInput | undefined => {
  if (input.ctrlKey || input.altKey || input.metaKey) return undefined;

  switch (input.key) {
    case "Enter": return { kind: "submit" };
    case "Backspace": return { kind: "backspace" };
    case "Escape": return { kind: "clear_prompt" };
    case "ArrowUp": return { kind: "history_up" };
    case "ArrowDown": return { kind: "history_down" };
    case "PageUp": return { kind: "scroll", lines: -8 };
    case "PageDown": return { kind: "scroll", lines: 8 };
    case "Home": return { kind: "scroll", lines: -1000 };
    case "End": return { kind: "scroll", lines: 1000 };
    default:
      return input.key.length === 1
        ? { kind: "append_text", text: input.key }
        : undefined;
  }
};

export const mapRatatuiWheelDelta = (deltaY: number): BrowserRatatuiInput | undefined => {
  if (!Number.isFinite(deltaY) || deltaY === 0) return undefined;
  return { kind: "scroll", lines: deltaY > 0 ? 3 : -3 };
};
