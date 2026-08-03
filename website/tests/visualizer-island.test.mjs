import assert from "node:assert/strict";
import { access, readFile, stat } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const repositoryRoot = path.resolve(import.meta.dirname, "..", "..");

test("the visualizer is a bounded website island with explicit release ownership", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "website", "interactive", "tokimu-visualizer.ts"),
    "utf8",
  );

  assert.match(source, /register\(\s*"tokimu-visualizer"/);
  assert.match(source, /Tokimu Signal field visualizer/);
  assert.match(source, /waitForFrame\(frame, signal\)/);
  assert.match(source, /signal\.addEventListener\("abort"/);
  assert.match(source, /frame\.src = "about:blank"/);
  assert.match(source, /fallback\.hidden = false/);
});

test("the browser adapter renders Rust/WASM observations without owning audio analysis", async () => {
  const source = await readFile(
    path.join(
      repositoryRoot,
      "corpus",
      "consumers",
      "tokimu-website-visualizer",
      "web",
      "visualizer.ts",
    ),
    "utf8",
  );

  assert.match(source, /new WasmVisualizerSession\(\)/);
  assert.match(source, /session\.step_json/);
  assert.match(source, /session\.set_fixture/);
  assert.match(source, /session\.set_mode/);
  assert.match(source, /session\.set_paused/);
  assert.match(source, /window\.addEventListener\(\s*"pagehide"/);
  assert.match(source, /session\.free\(\)/);
  assert.match(source, /const startupStartedAt = performance\.now\(\)/);
  assert.match(source, /const frameIntervalMs =/);
  assert.match(source, /const drawStartedAt = performance\.now\(\)/);
  assert.match(source, /state\.milkdrop/);
  assert.match(source, /controls\?\.customWaves/);
  assert.match(source, /does not execute custom-wave code/);
  assert.doesNotMatch(source, /AudioContext|navigator\.mediaDevices|getUserMedia|projectM/i);
});

test("the selected MilkDrop mode is evaluated inside the bounded Rust/WASM session", async () => {
  const source = await readFile(
    path.join(
      repositoryRoot,
      "corpus",
      "consumers",
      "tokimu-website-visualizer",
      "engine",
      "src",
      "lib.rs",
    ),
    "utf8",
  );

  assert.match(source, /MilkDropSelectedRuntime/);
  assert.match(source, /include_str!\(.*tokimu-selected-fixture\.milk/s);
  assert.match(source, /MilkDropSelected/);
  assert.match(source, /milkdrop-tools\/selected-first-party-subset/);
  assert.match(source, /step_with_audio/);
  assert.match(source, /MilkDropBrowserCustomWave/);
  assert.match(source, /MilkDropBrowserCustomShape/);
  assert.match(source, /inspect_shader_entries/);
  assert.match(source, /MilkDropBrowserShaderInspection/);
  assert.match(source, /TextureRequirementsUnderReview/);
  assert.match(source, /"projectm"\)\.is_err\(\)/);
  assert.doesNotMatch(source, /projectM[-_ ]visualizer|extern\s+crate\s+projectm/i);
});

test("the visualizer page states its synthetic-input and deferred-compatibility boundary", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "website", "docs", "lab", "visualizer.md"),
    "utf8",
  );

  assert.match(source, /data-tokimu-island="tokimu-visualizer"/);
  assert.match(source, /Open visualizer/);
  assert.match(source, /Synthetic sources are intentional/);
  assert.match(source, /bounded first-party MilkDrop scalar subset/);
  assert.match(source, /untextured convex custom shape/);
  assert.match(source, /Canvas renders that returned data/);
});

test("the published visualizer payload remains bounded", async () => {
  const output = path.join(
    repositoryRoot,
    "website",
    "docs",
    "assets",
    "islands",
    "tokimu-visualizer",
  );
  const files = [
    "tokimu_website_visualizer_engine_bg.wasm",
    "tokimu_website_visualizer_engine.js",
    "visualizer.js",
    "index.html",
    "styles.css",
  ];
  const sizes = await Promise.all(files.map((file) => stat(path.join(output, file)).then((entry) => entry.size)));

  for (const file of files) await access(path.join(output, file));
  assert.ok(sizes[0] <= 512 * 1024, `Visualizer WASM payload grew to ${sizes[0]} bytes`);
  assert.ok(
    sizes.reduce((sum, size) => sum + size, 0) <= 640 * 1024,
    "Visualizer first-load payload exceeded 640 KiB",
  );
});

test("the published visualizer entrypoint keeps synthetic controls and a canvas output", async () => {
  const source = await readFile(
    path.join(
      repositoryRoot,
      "website",
      "docs",
      "assets",
      "islands",
      "tokimu-visualizer",
      "index.html",
    ),
    "utf8",
  );

  assert.match(source, /data-fixture/);
  assert.match(source, /data-mode/);
  assert.match(source, /data-pause/);
  assert.match(source, /data-reset/);
  assert.match(source, /data-frame-ms/);
  assert.match(source, /data-draw-ms/);
  assert.match(source, /data-startup-ms/);
  assert.match(source, /data-execution-mode/);
  assert.match(source, /data-preset-source/);
  assert.match(source, /data-literal-geometry/);
  assert.match(source, /data-shader-handling/);
  assert.match(source, /data-texture-handling/);
  assert.match(source, /<canvas data-canvas/);
  assert.match(source, /<script type="module" src="\.\/visualizer\.js"><\/script>/);
});
