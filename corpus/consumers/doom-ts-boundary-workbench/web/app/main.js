import init, { BrowserIntakeSession } from "../pkg/doom_ts_boundary_workbench_engine.js";
import { bindLocalPackagePicker, disposeIntake } from "./intake.js";
const button = document.querySelector("#select");
const inspect = document.querySelector("#inspect");
const render = document.querySelector("#render");
const renderCutouts = document.querySelector("#render-cutouts");
const download = document.querySelector("#download");
const clear = document.querySelector("#clear");
const input = document.querySelector("#package");
const result = document.querySelector("#result");
const canvas = document.querySelector("#scene");
if (button === null || inspect === null || render === null || renderCutouts === null || download === null || clear === null || input === null || result === null || canvas === null)
    throw new Error("intake DOM is incomplete");
await init();
const session = new BrowserIntakeSession();
const unbind = bindLocalPackagePicker(button, input, session, (outcome) => {
    result.textContent = JSON.stringify(outcome, null, 2);
    inspect.disabled = outcome.kind !== "retained";
    render.disabled = outcome.kind !== "retained";
    renderCutouts.disabled = outcome.kind !== "retained";
    download.disabled = true;
    clear.disabled = outcome.kind !== "retained";
});
clear.addEventListener("click", () => {
    disposeIntake(session);
    inspect.disabled = true;
    render.disabled = true;
    renderCutouts.disabled = true;
    download.disabled = true;
    clear.disabled = true;
    result.textContent = JSON.stringify({ kind: "disposed", retainedResources: 0, retainedBytes: 0 }, null, 2);
});
inspect.addEventListener("click", () => {
    try {
        result.textContent = JSON.stringify(JSON.parse(session.inspect_doom1_wad()), null, 2);
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
});
render.addEventListener("click", async () => {
    render.disabled = true;
    result.textContent = "Preparing and presenting the retained E1M1 package...";
    try {
        result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_static_e1m1(canvas) }, null, 2);
        download.disabled = false;
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
    finally {
        render.disabled = false;
    }
});
renderCutouts.addEventListener("click", async () => {
    renderCutouts.disabled = true;
    result.textContent = "Preparing and presenting retained E1M1 with corpus-local masked cutouts...";
    try {
        result.textContent = JSON.stringify({ kind: "presented", observation: await session.render_static_e1m1_masked_cutouts(canvas) }, null, 2);
        download.disabled = false;
    }
    catch (error) {
        result.textContent = JSON.stringify({ kind: "rejected", diagnostic: String(error) }, null, 2);
    }
    finally {
        renderCutouts.disabled = false;
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
