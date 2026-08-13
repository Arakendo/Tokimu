import init, { run_fixture } from "./pkg/hello-render-resource-identity-web.js";

const run = document.querySelector("#run");
const status = document.querySelector("#status");
const canvas = document.querySelector("#scene");

await init();

run.addEventListener("click", async () => {
  run.disabled = true;
  status.textContent = "running";
  try {
    status.textContent = await run_fixture(canvas);
  } catch (error) {
    status.textContent = `failed | ${error?.stack ?? error}`;
  } finally {
    run.disabled = false;
  }
});
