export const mapRatatuiKeyboardInput = (input) => {
    if (input.ctrlKey || input.altKey || input.metaKey)
        return undefined;
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
export const mapRatatuiWheelDelta = (deltaY) => {
    if (!Number.isFinite(deltaY) || deltaY === 0)
        return undefined;
    return { kind: "scroll", lines: deltaY > 0 ? 3 : -3 };
};
