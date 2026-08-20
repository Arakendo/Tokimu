const parameters = new URLSearchParams(window.location.search);
const endpoint = parameters.get("tokimu_observer");
const runId = parameters.get("tokimu_run");
let sequence = 0;
const subjectId = crypto.randomUUID();
let currentOperation = "doom-browser-workbench";
export const terminalObservationEnabled = endpoint !== null && runId !== null;
function emit(event, operation = currentOperation, detail = "") {
    if (!terminalObservationEnabled)
        return;
    sequence += 1;
    const body = JSON.stringify({
        schemaVersion: 1,
        runId,
        subjectId,
        sequence,
        event,
        operation: String(operation).slice(0, 4096),
        detail: String(detail).slice(0, 4096),
    });
    void fetch(endpoint, {
        method: "POST",
        mode: "no-cors",
        cache: "no-store",
        keepalive: true,
        headers: { "Content-Type": "text/plain;charset=UTF-8" },
        body,
    }).catch(() => {
        // The in-page subject cannot classify loss of its external observer.
    });
}
export function beginObservedOperation(operation) {
    currentOperation = operation;
    emit("operation-started", operation);
}
export function completeObservedOperation(operation = currentOperation) {
    emit("operation-completed", operation);
}
export function rejectObservedOperation(operation, detail) {
    emit("structured-rejection", operation, String(detail));
}
export function operatorCompleted(operation = currentOperation) {
    emit("operator-completed", operation);
}
emit("subject-started");
window.setInterval(() => emit("heartbeat"), 5000);
window.addEventListener("error", (event) => {
    emit("page-error", currentOperation, event.message);
});
window.addEventListener("unhandledrejection", (event) => {
    emit("page-error", currentOperation, String(event.reason));
});
