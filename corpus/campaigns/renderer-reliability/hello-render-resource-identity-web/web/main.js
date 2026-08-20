import init, { BrowserReplacementPressure, run_fixture } from "./pkg/hello-render-resource-identity-web.js";
import {
  beginObservedOperation,
  completeObservedOperation,
  rejectObservedOperation,
} from "./terminal-observer.js";

const run = document.querySelector("#run");
const runPressure = document.querySelector("#run-pressure");
const runRetainedPressure = document.querySelector("#run-retained-pressure");
const probeRetainedAliasing = document.querySelector("#probe-retained-aliasing");
const probeRetainedAtomicity = document.querySelector("#probe-retained-atomicity");
const status = document.querySelector("#status");
const canvas = document.querySelector("#scene");

await init();
const replacementPressure = new BrowserReplacementPressure();
const retainedPressure = new BrowserReplacementPressure();

function setControlsDisabled(disabled) {
  run.disabled = disabled;
  runPressure.disabled = disabled;
  runRetainedPressure.disabled = disabled;
  probeRetainedAliasing.disabled = disabled;
  probeRetainedAtomicity.disabled = disabled;
}

async function runReplacementSequence(pressure, replacementMethod, alternative) {
  const operation = `resource-lifetime-${alternative}`;
  beginObservedOperation(operation);
  setControlsDisabled(true);
  const records = [];
  const started = performance.now();
  try {
    for (let sequence = 1; sequence <= 27; sequence += 1) {
      status.textContent = JSON.stringify({
        status: "running",
        alternative,
        sequence,
        total: 27,
        retainedRecords: records.length,
      }, null, 2);
      await new Promise((resolve) => requestAnimationFrame(resolve));
      const replacementStarted = performance.now();
      const observation = await replacementMethod.call(pressure, canvas, sequence);
      records.push({
        sequence,
        elapsedMilliseconds: performance.now() - replacementStarted,
        observation,
      });
      await new Promise((resolve) => requestAnimationFrame(resolve));
    }
    status.textContent = JSON.stringify({
      status: "complete",
      alternative,
      replacements: records.length,
      elapsedMilliseconds: performance.now() - started,
      physicalGpuReclamation: "unobserved",
      records,
    }, null, 2);
    if (alternative === "A-whole-backend") {
      completeObservedOperation(operation);
    }
  } catch (error) {
    status.textContent = JSON.stringify({
      status: "failed",
      alternative,
      completedReplacements: records.length,
      elapsedMilliseconds: performance.now() - started,
      diagnostic: String(error?.stack ?? error),
      records,
    }, null, 2);
    rejectObservedOperation(operation, error?.stack ?? error);
  } finally {
    setControlsDisabled(false);
  }
}

run.addEventListener("click", async () => {
  const operation = "resource-identity-fixture";
  beginObservedOperation(operation);
  run.disabled = true;
  status.textContent = "running";
  try {
    status.textContent = await run_fixture(canvas);
    completeObservedOperation(operation);
  } catch (error) {
    status.textContent = `failed | ${error?.stack ?? error}`;
    rejectObservedOperation(operation, error?.stack ?? error);
  } finally {
    run.disabled = false;
  }
});

runPressure.addEventListener("click", async () => {
  await runReplacementSequence(
    replacementPressure,
    replacementPressure.replace_scene,
    "A-whole-backend",
  );
});

runRetainedPressure.addEventListener("click", async () => {
  await runReplacementSequence(
    retainedPressure,
    retainedPressure.replace_scene_retained,
    "B-adapter-private-reset",
  );
});

probeRetainedAliasing.addEventListener("click", () => {
  setControlsDisabled(true);
  try {
    status.textContent = retainedPressure.probe_retained_cross_set_aliasing();
  } catch (error) {
    status.textContent = `failed | ${error?.stack ?? error}`;
  } finally {
    setControlsDisabled(false);
  }
});

probeRetainedAtomicity.addEventListener("click", () => {
  setControlsDisabled(true);
  try {
    status.textContent = retainedPressure.probe_retained_reset_atomicity();
    completeObservedOperation("resource-lifetime-B-adapter-private-reset");
  } catch (error) {
    status.textContent = `failed | ${error?.stack ?? error}`;
    rejectObservedOperation("resource-lifetime-B-adapter-private-reset", error?.stack ?? error);
  } finally {
    setControlsDisabled(false);
  }
});
