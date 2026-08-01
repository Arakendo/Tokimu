import init, { KernelUiSession } from "./tokimu_website_kernel_ui_engine.js";
const filter = need("[data-filter]");
const list = need("[data-resource-list]");
const count = need("[data-count]");
const nameInput = need("[data-name]");
const notesInput = need("[data-notes]");
const title = need("[data-editor-title]");
const kind = need("[data-kind]");
const visibility = need("[data-visibility]");
const hotspot = need("[data-hotspot]");
const applyButton = need("[data-apply]");
const revertButton = need("[data-revert]");
const deleteButton = need("[data-delete]");
const status = need("[data-status]");
const modal = need("[data-modal]");
const deleteName = need("[data-delete-name]");
const confirmDelete = need("[data-confirm-delete]");
const cancelDelete = need("[data-cancel-delete]");
let session = null;
let observation = null;
await init();
session = new KernelUiSession();
render(read(session.observation_json()));
filter.addEventListener("input", () => invoke((active) => active.set_filter(filter.value)));
nameInput.addEventListener("input", () => invoke((active) => active.set_name(nameInput.value), false));
notesInput.addEventListener("input", () => invoke((active) => active.set_notes(notesInput.value), false));
visibility.addEventListener("click", () => invoke((active) => active.toggle_visibility()));
hotspot.addEventListener("click", () => invoke((active) => active.toggle_hotspot()));
applyButton.addEventListener("click", () => invoke((active) => active.apply()));
revertButton.addEventListener("click", () => invoke((active) => active.revert()));
deleteButton.addEventListener("click", () => invoke((active) => active.request_delete()));
confirmDelete.addEventListener("click", () => invoke((active) => active.confirm_delete()));
cancelDelete.addEventListener("click", () => invoke((active) => active.cancel_delete()));
window.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && observation?.confirmDelete) {
        event.preventDefault();
        invoke((active) => active.cancel_delete());
    }
});
window.addEventListener("pagehide", () => {
    session?.dispose();
    session?.free();
    session = null;
}, { once: true });
function invoke(command, syncFields = true) {
    if (!session)
        return;
    render(read(command(session)), syncFields);
}
function render(next, syncFields = true) {
    observation = next;
    count.value = `${next.visibleCount} / ${next.totalCount}`;
    status.value = next.status;
    title.textContent = next.selected.name;
    kind.textContent = next.selected.kind;
    deleteName.textContent = next.selected.name;
    if (document.activeElement !== filter)
        filter.value = next.filter;
    if (syncFields || document.activeElement !== nameInput)
        nameInput.value = next.selected.name;
    if (syncFields || document.activeElement !== notesInput)
        notesInput.value = next.selected.notes;
    setPressed(visibility, next.selected.visible);
    setPressed(hotspot, next.selected.hotspot);
    applyButton.disabled = !next.selected.dirty;
    revertButton.disabled = !next.selected.dirty;
    deleteButton.disabled = !next.canDelete;
    modal.hidden = !next.confirmDelete;
    if (next.confirmDelete)
        confirmDelete.focus({ preventScroll: true });
    list.replaceChildren(...next.resources.map(resourceButton));
}
function resourceButton(resource) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "resource-row";
    button.setAttribute("role", "option");
    button.setAttribute("aria-selected", String(resource.selected));
    button.innerHTML = `
    <span class="resource-name"></span>
    <span class="resource-kind"></span>
    <span class="resource-flags"></span>
  `;
    need(".resource-name", button).textContent = resource.name;
    need(".resource-kind", button).textContent = resource.kind;
    need(".resource-flags", button).textContent = [
        resource.dirty ? "DRAFT" : "",
        resource.hotspot ? "HOT" : "",
        resource.visible ? "" : "HIDDEN",
    ].filter(Boolean).join(" / ");
    button.addEventListener("click", () => invoke((active) => active.select_resource(BigInt(resource.id))));
    return button;
}
function setPressed(button, pressed) {
    button.setAttribute("aria-pressed", String(pressed));
}
function read(json) {
    return JSON.parse(json);
}
function need(selector, root = document) {
    const element = root.querySelector(selector);
    if (!element)
        throw new Error(`Kernel UI workbench requires ${selector}.`);
    return element;
}
