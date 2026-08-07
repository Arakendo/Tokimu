import assert from "node:assert/strict";
import { access, readFile, stat } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const read = (relativePath) => readFile(path.join(repoRoot, relativePath), "utf8");

test("runtime observation browser consumer retains one Rust-owned shell session", async () => {
  const app = await read("corpus/consumers/runtime-observation-workbench/web/app.ts");
  const engine = await read("corpus/consumers/runtime-observation-workbench/engine/src/lib.rs");

  assert.match(app, /new WasmObservationShellSession\(\)/);
  assert.match(app, /const runtime = shellWasm as RuntimeObservationWasm/);
  assert.match(app, /renderRatatuiShell/);
  assert.doesNotMatch(app, /new WasmRuntimeObservationSession\(\)/);
  assert.match(engine, /semantic_controls_and_ratatui_share_one_rust_owned_runtime_session/);
});

test("runtime observation lab documents the bounded shared-session claim", async () => {
  const page = await read("website/docs/lab/runtime-observation.md");

  assert.match(page, /same retained/);
  assert.match(page, /WasmObservationShellSession/);
  assert.match(page, /does not parse commands/);
  assert.match(page, /does not claim native\/browser session\s+handoff/);
  assert.match(page, /`AR-0013` remains incubating/);
});

test("runtime observation island fills its width and follows the child document height", async () => {
  const loader = await read("website/docs/javascripts/runtime-observation.js");

  assert.match(loader, /mount\.style\.gridColumn = "1 \/ -1"/);
  assert.match(loader, /mount\.style\.width = "100%"/);
  assert.match(loader, /frame\.style\.width = "100%"/);
  assert.match(loader, /installFrameHeightSync/);
  assert.match(loader, /tokimu-runtime-observation-height/);
  assert.doesNotMatch(loader, /clamp\(44rem, 78vw, 66rem\)/);
});

test("runtime observation island presents Ratatui before expandable semantic evidence", async () => {
  const page = await read("corpus/consumers/runtime-observation-workbench/web/index.html");
  const styles = await read("corpus/consumers/runtime-observation-workbench/web/styles.css");
  const app = await read("corpus/consumers/runtime-observation-workbench/web/app.ts");

  assert.ok(page.indexOf('id="ratatui-shell"') < page.indexOf('id="output"'));
  assert.match(styles, /white-space: pre-wrap/);
  assert.doesNotMatch(styles, /overflow-y: auto/);
  assert.match(app, /scheduleDocumentHeight/);
  assert.match(app, /document\.documentElement\.scrollHeight/);
});

test("runtime observation readiness follows Rust/WASM startup rather than iframe load", async () => {
  const child = await read("corpus/consumers/runtime-observation-workbench/web/app.ts");
  const page = await read("corpus/consumers/runtime-observation-workbench/web/index.html");
  const loader = await read("website/docs/javascripts/runtime-observation.js");
  const islands = await read("website/docs/javascripts/islands.js");

  assert.match(child, /tokimu-runtime-observation-state/);
  assert.match(child, /state: "startup_failed"/);
  assert.match(child, /void start\(\)\.catch\(reportStartupFailure\)/);
  assert.match(page, /id="startup-error"/);
  assert.match(page, /role="alert"/);
  assert.match(loader, /waitForRuntime/);
  assert.match(loader, /event\.data\.state === "ready"/);
  assert.match(loader, /event\.data\.state === "error"/);
  assert.doesNotMatch(loader, /const onLoad = .*resolve/);
  assert.match(islands, /error\?\.message/);
});

test("runtime observation island publishes the bounded WASM artifact set", async () => {
  const assetRoot = path.join(repoRoot, "website", "docs", "assets", "islands", "runtime-observation");
  const requiredAssets = [
    "runtime_observation_workbench_engine_bg.wasm",
    "runtime_observation_workbench_engine.js",
    "app.js",
    "index.html",
    "styles.css",
  ];

  await Promise.all(requiredAssets.map((asset) => access(path.join(assetRoot, asset))));

  const wasm = await stat(path.join(assetRoot, "runtime_observation_workbench_engine_bg.wasm"));
  assert.ok(wasm.size <= 2 * 1024 * 1024, `WASM artifact is unexpectedly large: ${wasm.size} bytes`);

  const totalSize = (await Promise.all(requiredAssets.map(async (asset) => (await stat(path.join(assetRoot, asset))).size)))
    .reduce((total, size) => total + size, 0);
  assert.ok(totalSize <= 3 * 1024 * 1024, `Published island is unexpectedly large: ${totalSize} bytes`);

  const app = await read("website/docs/assets/islands/runtime-observation/app.js");
  assert.match(app, /WasmObservationShellSession/);
  assert.match(app, /putImageData/);
  assert.doesNotMatch(app, /new WasmRuntimeObservationSession\(\)/);
});
