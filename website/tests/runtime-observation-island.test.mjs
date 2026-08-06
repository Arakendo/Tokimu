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

test("runtime observation island reserves a full-width bounded frame", async () => {
  const loader = await read("website/docs/javascripts/runtime-observation.js");

  assert.match(loader, /mount\.style\.gridColumn = "1 \/ -1"/);
  assert.match(loader, /mount\.style\.width = "100%"/);
  assert.match(loader, /frame\.style\.width = "100%"/);
  assert.match(loader, /frame\.style\.height = "clamp\(44rem, 78vw, 66rem\)"/);
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
