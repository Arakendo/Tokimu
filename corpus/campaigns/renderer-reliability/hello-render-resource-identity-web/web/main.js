import init, { BrowserReplacementPressure, run_fixture } from "./pkg/hello-render-resource-identity-web.js";

const run = document.querySelector("#run");
const runPressure = document.querySelector("#run-pressure");
const status = document.querySelector("#status");
const canvas = document.querySelector("#scene");

await init();
const replacementPressure = new BrowserReplacementPressure();

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

runPressure.addEventListener("click", async () => {
  runPressure.disabled = true;
  run.disabled = true;
  const records = [];
  const started = performance.now();
  try {
    for (let sequence = 1; sequence <= 27; sequence += 1) {
      status.textContent = JSON.stringify({
        status: "running",
        sequence,
        total: 27,
        retainedRecords: records.length,
      }, null, 2);
      await new Promise((resolve) => requestAnimationFrame(resolve));
      const replacementStarted = performance.now();
      const observation = await replacementPressure.replace_scene(canvas, sequence);
      records.push({
        sequence,
        elapsedMilliseconds: performance.now() - replacementStarted,
        observation,
      });
      await new Promise((resolve) => requestAnimationFrame(resolve));
    }
    status.textContent = JSON.stringify({
      status: "complete",
      replacements: records.length,
      elapsedMilliseconds: performance.now() - started,
      physicalGpuReclamation: "unobserved",
      records,
    }, null, 2);
  } catch (error) {
    status.textContent = JSON.stringify({
      status: "failed",
      completedReplacements: records.length,
      elapsedMilliseconds: performance.now() - started,
      diagnostic: String(error?.stack ?? error),
      records,
    }, null, 2);
  } finally {
    runPressure.disabled = false;
    run.disabled = false;
  }
});
