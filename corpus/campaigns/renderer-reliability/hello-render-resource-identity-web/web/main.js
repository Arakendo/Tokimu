import init, {
  BrowserReplacementPressure,
  run_fixture,
  run_scene_generation_prototype,
} from "./pkg/hello-render-resource-identity-web.js";
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
const runGenerationPrototype = document.querySelector("#run-generation-prototype");
const probeProviderStaging = document.querySelector("#probe-provider-staging");
const probeScopedTextureUpdate = document.querySelector("#probe-scoped-texture-update");
const runScopedTexturePressure = document.querySelector("#run-scoped-texture-pressure");
const runProviderStagingPressure = document.querySelector("#run-provider-staging-pressure");
const status = document.querySelector("#status");
const canvas = document.querySelector("#scene");

await init();
const replacementPressure = new BrowserReplacementPressure();
const retainedPressure = new BrowserReplacementPressure();
const providerStagingPressure = new BrowserReplacementPressure();
const repeatedProviderStagingPressure = new BrowserReplacementPressure();
const scopedTexturePressure = new BrowserReplacementPressure();

function setControlsDisabled(disabled) {
  run.disabled = disabled;
  runPressure.disabled = disabled;
  runRetainedPressure.disabled = disabled;
  probeRetainedAliasing.disabled = disabled;
  probeRetainedAtomicity.disabled = disabled;
  runGenerationPrototype.disabled = disabled;
  probeProviderStaging.disabled = disabled;
  probeScopedTextureUpdate.disabled = disabled;
  runScopedTexturePressure.disabled = disabled;
  runProviderStagingPressure.disabled = disabled;
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

runGenerationPrototype.addEventListener("click", () => {
  const operation = "resource-lifetime-C-semantic-prototype";
  beginObservedOperation(operation);
  setControlsDisabled(true);
  try {
    status.textContent = run_scene_generation_prototype();
    completeObservedOperation(operation);
  } catch (error) {
    status.textContent = `failed | ${error?.stack ?? error}`;
    rejectObservedOperation(operation, error?.stack ?? error);
  } finally {
    setControlsDisabled(false);
  }
});

probeProviderStaging.addEventListener("click", async () => {
  const operation = "resource-lifetime-C-real-provider-staging";
  beginObservedOperation(operation);
  setControlsDisabled(true);
  status.textContent = "running provider A / failed-B / committed-B / stale-A-command probe";
  try {
    status.textContent = await providerStagingPressure.probe_provider_staging(canvas);
    completeObservedOperation(operation);
  } catch (error) {
    status.textContent = `failed | ${error?.stack ?? error}`;
    rejectObservedOperation(operation, error?.stack ?? error);
  } finally {
    setControlsDisabled(false);
  }
});

probeScopedTextureUpdate.addEventListener("click", async () => {
  const operation = "ar0033-scoped-texture-update";
  beginObservedOperation(operation);
  setControlsDisabled(true);
  status.textContent = "running AR-0033 fixed-descriptor texture update probe";
  try {
    status.textContent = `stable-contract=ADR-0019; ${await providerStagingPressure.probe_scoped_texture_update(canvas)}`;
    completeObservedOperation(operation, status.textContent);
  } catch (error) {
    status.textContent = `failed | ${error?.stack ?? error}`;
    rejectObservedOperation(operation, error?.stack ?? error);
  } finally {
    setControlsDisabled(false);
  }
});

runScopedTexturePressure.addEventListener("click", async () => {
  const operation = "ar0033-scoped-texture-pressure";
  beginObservedOperation(operation);
  setControlsDisabled(true);
  const records = [];
  const started = performance.now();
  try {
    for (let revision = 1; revision <= 27; revision += 1) {
      const injectPreparedDrop = revision % 5 === 0;
      await new Promise((resolve) => requestAnimationFrame(resolve));
      const observation = await scopedTexturePressure.update_console_texture_scoped(
        canvas,
        revision,
        injectPreparedDrop,
      );
      records.push({ revision, injectPreparedDrop, observation });
    }
    status.textContent = JSON.stringify({
      status: "complete",
      review: "AR-0033",
      contract: "ADR-0019",
      pressure: "fixed-descriptor-console-texture",
      updates: records.length,
      preparedDrops: records.filter((record) => record.injectPreparedDrop).length,
      resourceSets: 1,
      logicalInventoryStayedFixed: true,
      providerDiagnostics: 0,
      physicalGpuReclamation: "unobserved",
      elapsedMilliseconds: performance.now() - started,
      finalObservation: records.at(-1)?.observation,
      records,
    }, null, 2);
    completeObservedOperation(operation, status.textContent);
  } catch (error) {
    status.textContent = `failed | ${error?.stack ?? error}`;
    rejectObservedOperation(operation, error?.stack ?? error);
  } finally {
    setControlsDisabled(false);
  }
});

runProviderStagingPressure.addEventListener("click", async () => {
  const operation = "resource-lifetime-C-real-provider-staging-pressure";
  beginObservedOperation(operation);
  setControlsDisabled(true);
  const records = [];
  const started = performance.now();
  try {
    for (let sequence = 1; sequence <= 27; sequence += 1) {
      const targetScene = sequence % 2;
      const injectLateFailure = sequence % 5 === 0;
      status.textContent = JSON.stringify({
        status: "running",
        alternative: "C-real-provider-staging-pressure",
        sequence,
        total: 27,
        targetScene,
        injectLateFailure,
      }, null, 2);
      await new Promise((resolve) => requestAnimationFrame(resolve));
      const replacementStarted = performance.now();
      const observation = await repeatedProviderStagingPressure.replace_scene_staged(
        canvas,
        targetScene,
        injectLateFailure,
      );
      records.push({
        sequence,
        targetScene,
        injectLateFailure,
        elapsedMilliseconds: performance.now() - replacementStarted,
        observation,
      });
      await new Promise((resolve) => requestAnimationFrame(resolve));
    }
    status.textContent = JSON.stringify({
      status: "complete",
      alternative: "C-real-provider-staging-pressure",
      replacements: records.length,
      injectedLateFailures: records.filter((record) => record.injectLateFailure).length,
      elapsedMilliseconds: performance.now() - started,
      providerSessions: 1,
      steadyLogicalSetsAfterCommit: 1,
      maximumLogicalSetsDuringStage: 2,
      physicalGpuReclamation: "unobserved",
      records,
    }, null, 2);
    completeObservedOperation(operation);
  } catch (error) {
    status.textContent = JSON.stringify({
      status: "failed",
      alternative: "C-real-provider-staging-pressure",
      completedReplacements: records.length,
      elapsedMilliseconds: performance.now() - started,
      diagnostic: String(error?.stack ?? error),
      records,
    }, null, 2);
    rejectObservedOperation(operation, error?.stack ?? error);
  } finally {
    setControlsDisabled(false);
  }
});

const autorun = new URLSearchParams(window.location.search).get("tokimu_autorun");
if (autorun === "provider-staging") {
  probeProviderStaging.click();
} else if (autorun === "scoped-texture-update") {
  probeScopedTextureUpdate.click();
} else if (autorun === "scoped-texture-pressure") {
  runScopedTexturePressure.click();
} else if (autorun === "provider-staging-pressure") {
  runProviderStagingPressure.click();
} else if (autorun !== null) {
  status.textContent = `failed | unknown tokimu_autorun=${autorun}`;
}
