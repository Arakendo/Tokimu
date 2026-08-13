import init, { BrowserIntakeSession } from "../pkg/doom_ts_boundary_workbench_engine.js";
import { bindLocalPackagePicker, disposeIntake } from "./intake.js";

const button = document.querySelector<HTMLButtonElement>("#select");
const inspect = document.querySelector<HTMLButtonElement>("#inspect");
const render = document.querySelector<HTMLButtonElement>("#render");
const renderCutouts = document.querySelector<HTMLButtonElement>("#render-cutouts");
const renderSelected = document.querySelector<HTMLButtonElement>("#render-selected");
const renderDiagnosticSky = document.querySelector<HTMLButtonElement>("#render-diagnostic-sky");
const renderExitsign = document.querySelector<HTMLButtonElement>("#render-exitsign");
const download = document.querySelector<HTMLButtonElement>("#download");
const clear = document.querySelector<HTMLButtonElement>("#clear");
const input = document.querySelector<HTMLInputElement>("#package");
const result = document.querySelector<HTMLElement>("#result");
const canvas = document.querySelector<HTMLCanvasElement>("#scene");
if (button === null || inspect === null || render === null || renderCutouts === null || renderSelected === null || renderDiagnosticSky === null || renderExitsign === null || download === null || clear === null || input === null || result === null || canvas === null) throw new Error("intake DOM is incomplete");

await init();
const session = new BrowserIntakeSession();
const unbind = bindLocalPackagePicker(button, input, session, (outcome) => {
  result.textContent = JSON.stringify(outcome, null, 2);
  inspect.disabled = outcome.kind !== "retained";
  render.disabled = outcome.kind !== "retained";
  renderCutouts.disabled = outcome.kind !== "retained";
  renderSelected.disabled = outcome.kind !== "retained";
  renderDiagnosticSky.disabled = outcome.kind !== "retained";
  renderExitsign.disabled = outcome.kind !== "retained";
  download.disabled = true;
  clear.disabled = outcome.kind !== "retained";
});
clear.addEventListener("click", () => {
  disposeIntake(session);
  inspect.disabled = true;
  render.disabled = true;
  renderCutouts.disabled = true;
  renderSelected.disabled = true;
  renderDiagnosticSky.disabled = true;
  renderExitsign.disabled = true;
  download.disabled = true;
  clear.disabled = true;
  result.textContent = JSON.stringify({ kind: "disposed", retainedResources: 0, retainedBytes: 0 }, null, 2);
});
inspect.addEventListener("click", () => {
  try {
    result.textContent = JSON.stringify(JSON.parse(session.inspect_doom1_wad()), null, 2);
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  }
});
render.addEventListener("click", async () => {
  render.disabled = true;
  result.textContent = "Preparing and presenting the retained E1M1 package...";
  try {
    result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_static_e1m1(canvas) }, null, 2);
    download.disabled = false;
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
    render.disabled = false;
  }
});
renderCutouts.addEventListener("click", async () => {
  renderCutouts.disabled = true;
  result.textContent = "Preparing and presenting retained E1M1 with corpus-local masked cutouts...";
  try {
    result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_static_e1m1_masked_cutouts(canvas) }, null, 2);
    download.disabled = false;
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
    renderCutouts.disabled = false;
  }
});
renderSelected.addEventListener("click", async () => {
  renderSelected.disabled = true;
  result.textContent = "Preparing source-spawn E1M1 with corpus-local AABB/frustum selection...";
  try {
    result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_static_e1m1_selected_cutouts(canvas) }, null, 2);
    download.disabled = false;
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
    renderSelected.disabled = false;
  }
});
renderDiagnosticSky.addEventListener("click", async () => {
  renderDiagnosticSky.disabled = true;
  result.textContent = "Preparing retained E1M1 sky omissions with the opt-in Purple diagnostic stand-in...";
  try {
    result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_e1m1_diagnostic_sky_omissions(canvas) }, null, 2);
    download.disabled = false;
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
    renderDiagnosticSky.disabled = false;
  }
});
renderExitsign.addEventListener("click", async () => {
  renderExitsign.disabled = true;
  result.textContent = "Preparing the canonical E1M1 EXITSIGN orientation view...";
  try {
    result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_e1m1_exitsign(canvas) }, null, 2);
    download.disabled = false;
  } catch (error) {
    result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
  } finally {
    renderExitsign.disabled = false;
  }
});
download.addEventListener("click", () => {
  canvas.toBlob((blob) => {
    if (blob === null) {
      result.textContent = JSON.stringify({ kind: "rejected", diagnostic: "browser did not encode the canvas as PNG" }, null, 2);
      return;
    }
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "doom-e1m1-browser-first-frame.png";
    document.body.append(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 0);
  }, "image/png");
});
addEventListener("pagehide", () => { unbind(); disposeIntake(session); }, { once: true });
