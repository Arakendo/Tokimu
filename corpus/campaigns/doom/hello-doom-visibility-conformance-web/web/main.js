import init, { run_fixture } from "./pkg/hello-doom-visibility-conformance-web.js";

const status = document.querySelector("#status");
const canvas = document.querySelector("#scene");
const controls = [...document.querySelectorAll("button[data-fixture]")];

await init();

for (const control of controls) {
  control.addEventListener("click", async () => {
    controls.forEach((button) => (button.disabled = true));
    status.textContent = `running ${control.dataset.fixture}`;
    try {
      status.textContent = await run_fixture(canvas, control.dataset.fixture);
    } catch (error) {
      status.textContent = `failed | ${error?.stack ?? error}`;
    } finally {
      controls.forEach((button) => (button.disabled = false));
    }
  });
}
