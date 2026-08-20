/**
 * Browser mechanism only. It neither examines bytes nor keeps them after the
 * Rust/WASM request returns. Call this from an input/change event caused by an
 * explicit user gesture.
 */
export async function submitSelectedPackage(file, session) {
    if (file === undefined)
        return { kind: "cancelled" };
    try {
        const bytes = new Uint8Array(await file.arrayBuffer());
        const observation = JSON.parse(session.import_selected_package(file.name, file.type || "application/octet-stream", bytes));
        return { kind: "retained", observation };
    }
    catch (error) {
        return { kind: "rejected", diagnostic: error instanceof Error ? error.message : String(error) };
    }
}
/** Releases Rust-owned selected bytes when the workbench is disposed. */
export function disposeIntake(session) {
    session.dispose();
}
/**
 * Binds a visible user gesture to local-file selection. The click handler is
 * intentionally the only code path that opens the picker; the change handler
 * only transports the resulting selection to Rust/WASM.
 */
export function bindLocalPackagePicker(button, input, session, report) {
    if (input.type !== "file") {
        throw new Error("the DOOM intake input must have type=file");
    }
    const openPicker = () => input.click();
    const receiveSelection = async () => {
        const result = await submitSelectedPackage(input.files?.item(0) ?? undefined, session);
        report(result);
        // Do not retain the browser's File selection after the Rust/WASM call.
        input.value = "";
    };
    button.addEventListener("click", openPicker);
    input.addEventListener("change", receiveSelection);
    return () => {
        button.removeEventListener("click", openPicker);
        input.removeEventListener("change", receiveSelection);
    };
}
/**
 * Binds an explicit local drag/drop gesture to the same Rust-owned intake
 * boundary as the file picker. TypeScript transports one File's bytes and
 * never interprets WAD/package contents.
 */
export function bindLocalPackageDrop(target, session, report) {
    const setActive = (active) => {
        target.dataset.dragActive = String(active);
        target.setAttribute("aria-label", active
            ? "Release one reviewed Doom package to inspect it"
            : "Drop one reviewed Doom package here");
    };
    const acceptDrag = (event) => {
        event.preventDefault();
        if (event.dataTransfer !== null)
            event.dataTransfer.dropEffect = "copy";
        setActive(true);
    };
    const leaveDrag = (event) => {
        event.preventDefault();
        if (event.relatedTarget instanceof Node && target.contains(event.relatedTarget))
            return;
        setActive(false);
    };
    const receiveDrop = async (event) => {
        event.preventDefault();
        setActive(false);
        const files = event.dataTransfer?.files;
        if (files === undefined || files.length === 0) {
            report({ kind: "cancelled" });
            return;
        }
        if (files.length !== 1) {
            report({
                kind: "rejected",
                diagnostic: `drop exactly one reviewed package; received ${files.length}`,
            });
            return;
        }
        report(await submitSelectedPackage(files.item(0) ?? undefined, session));
    };
    target.addEventListener("dragenter", acceptDrag);
    target.addEventListener("dragover", acceptDrag);
    target.addEventListener("dragleave", leaveDrag);
    target.addEventListener("drop", receiveDrop);
    setActive(false);
    return () => {
        target.removeEventListener("dragenter", acceptDrag);
        target.removeEventListener("dragover", acceptDrag);
        target.removeEventListener("dragleave", leaveDrag);
        target.removeEventListener("drop", receiveDrop);
    };
}
