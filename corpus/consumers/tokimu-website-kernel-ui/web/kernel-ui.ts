import init, { KernelUiSession } from "./tokimu_website_kernel_ui_engine.js";

type ResourceObservation = {
  id: number;
  kind: string;
  name: string;
  notes: string;
  visible: boolean;
  hotspot: boolean;
  dirty: boolean;
  selected: boolean;
};

type WorkbenchObservation = {
  schema: number;
  status: string;
  filter: string;
  selectedId: number;
  totalCount: number;
  visibleCount: number;
  confirmDelete: boolean;
  canDelete: boolean;
  selected: ResourceObservation;
  resources: ResourceObservation[];
};

const filter = need<HTMLInputElement>("[data-filter]");
const list = need<HTMLElement>("[data-resource-list]");
const count = need<HTMLOutputElement>("[data-count]");
const nameInput = need<HTMLInputElement>("[data-name]");
const notesInput = need<HTMLTextAreaElement>("[data-notes]");
const title = need<HTMLElement>("[data-editor-title]");
const kind = need<HTMLElement>("[data-kind]");
const visibility = need<HTMLButtonElement>("[data-visibility]");
const hotspot = need<HTMLButtonElement>("[data-hotspot]");
const applyButton = need<HTMLButtonElement>("[data-apply]");
const revertButton = need<HTMLButtonElement>("[data-revert]");
const deleteButton = need<HTMLButtonElement>("[data-delete]");
const status = need<HTMLOutputElement>("[data-status]");
const modal = need<HTMLElement>("[data-modal]");
const deleteName = need<HTMLElement>("[data-delete-name]");
const confirmDelete = need<HTMLButtonElement>("[data-confirm-delete]");
const cancelDelete = need<HTMLButtonElement>("[data-cancel-delete]");

let session: KernelUiSession | null = null;
let observation: WorkbenchObservation | null = null;

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

function invoke(command: (active: KernelUiSession) => string, syncFields = true): void {
  if (!session) return;
  render(read(command(session)), syncFields);
}

function render(next: WorkbenchObservation, syncFields = true): void {
  observation = next;
  count.value = `${next.visibleCount} / ${next.totalCount}`;
  status.value = next.status;
  title.textContent = next.selected.name;
  kind.textContent = next.selected.kind;
  deleteName.textContent = next.selected.name;

  if (document.activeElement !== filter) filter.value = next.filter;
  if (syncFields || document.activeElement !== nameInput) nameInput.value = next.selected.name;
  if (syncFields || document.activeElement !== notesInput) notesInput.value = next.selected.notes;

  setPressed(visibility, next.selected.visible);
  setPressed(hotspot, next.selected.hotspot);
  applyButton.disabled = !next.selected.dirty;
  revertButton.disabled = !next.selected.dirty;
  deleteButton.disabled = !next.canDelete;
  modal.hidden = !next.confirmDelete;
  if (next.confirmDelete) confirmDelete.focus({ preventScroll: true });

  list.replaceChildren(...next.resources.map(resourceButton));
}

function resourceButton(resource: ResourceObservation): HTMLButtonElement {
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
  need<HTMLElement>(".resource-name", button).textContent = resource.name;
  need<HTMLElement>(".resource-kind", button).textContent = resource.kind;
  need<HTMLElement>(".resource-flags", button).textContent = [
    resource.dirty ? "DRAFT" : "",
    resource.hotspot ? "HOT" : "",
    resource.visible ? "" : "HIDDEN",
  ].filter(Boolean).join(" / ");
  button.addEventListener("click", () => invoke((active) => active.select_resource(BigInt(resource.id))));
  return button;
}

function setPressed(button: HTMLButtonElement, pressed: boolean): void {
  button.setAttribute("aria-pressed", String(pressed));
}

function read(json: string): WorkbenchObservation {
  return JSON.parse(json) as WorkbenchObservation;
}

function need<T extends Element>(selector: string, root: ParentNode = document): T {
  const element = root.querySelector<T>(selector);
  if (!element) throw new Error(`Kernel UI workbench requires ${selector}.`);
  return element;
}
