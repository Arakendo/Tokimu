import assert from "node:assert/strict";
import { access, readFile, stat } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const repositoryRoot = path.resolve(import.meta.dirname, "..", "..");

test("Tokimu Paint is a bounded website island with explicit release ownership", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "website", "interactive", "tokimu-paint.ts"),
    "utf8",
  );

  assert.match(source, /register\(\s*"tokimu-paint"/);
  assert.match(source, /Tokimu Paint raster editing workbench/);
  assert.match(source, /waitForFrame\(frame, signal\)/);
  assert.match(source, /signal\.addEventListener\("abort"/);
  assert.match(source, /frame\.src = "about:blank"/);
  assert.match(source, /frame\.remove\(\)/);
  assert.match(source, /fallback\.hidden = false/);
  assert.doesNotMatch(source, /\b(floodFill|preview_bytes|export_png)\s*\(/);
});

test("the standalone Paint bundle releases its Rust/WASM session on page teardown", async () => {
  const source = await readFile(
    path.join(
      repositoryRoot,
      "corpus",
      "consumers",
      "tokimu-website-paint",
      "web",
      "paint.ts",
    ),
    "utf8",
  );

  assert.match(source, /window\.addEventListener\(\s*"pagehide"/);
  assert.match(source, /session\.dispose\(\)/);
  assert.match(source, /WASM startup observed at/);
  assert.match(source, /performance\.now\(\)/);
  assert.match(source, /canvas\.setPointerCapture\(event\.pointerId\)/);
  assert.match(source, /event\.preventDefault\(\)/);
  assert.match(source, /drawLivePixelLine/);
  assert.match(source, /drawLiveBrushStamp/);
  assert.match(source, /kind: "brushStroke"/);
  assert.match(source, /data-brush-size/);
  assert.match(source, /updateHistoryControls\(observation\)/);
  assert.match(source, /undoButton\.disabled = undoDepth === 0/);
  assert.match(source, /redoButton\.disabled = redoDepth === 0/);
  assert.match(source, /context\.globalCompositeOperation/);
  assert.match(source, /Rust receives the complete point list on/);
});

test("the published Paint payload remains within its recorded launch budget", async () => {
  const output = path.join(
    repositoryRoot,
    "website",
    "docs",
    "assets",
    "islands",
    "tokimu-paint",
  );
  const files = [
    "tokimu_website_paint_engine_bg.wasm",
    "tokimu_website_paint_engine.js",
    "paint.js",
    "index.html",
    "styles.css",
    "lucide.svg",
  ];
  const sizes = await Promise.all(files.map((file) => stat(path.join(output, file)).then((entry) => entry.size)));

  // These are compressed-hosting-independent limits for the reviewed first-load
  // files. Browser startup duration is recorded by the workbench itself instead.
  assert.ok(sizes[0] <= 768 * 1024, `Paint WASM payload grew to ${sizes[0]} bytes`);
  assert.ok(sizes[1] <= 32 * 1024, `Paint WASM binding grew to ${sizes[1]} bytes`);
  assert.ok(sizes[2] <= 32 * 1024, `Paint workbench adapter grew to ${sizes[2]} bytes`);
  assert.ok(
    sizes.reduce((total, size) => total + size, 0) <= 832 * 1024,
    `Paint first-load payload grew to ${sizes.reduce((total, size) => total + size, 0)} bytes`,
  );
});

test("the Paint page keeps its bounded ownership claim available before activation", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "website", "docs", "lab", "paint.md"),
    "utf8",
  );

  assert.match(source, /data-tokimu-island="tokimu-paint"/);
  assert.match(source, /Open raster workbench/);
  assert.match(source, /data-island-action="activate"/);
  assert.match(source, /data-island-mount hidden/);
  assert.match(source, /"activation": "explicit"/);
  assert.match(source, /Canvas is a presentation target only/);
});

test("the website build publishes the standalone Paint payload", async () => {
  const output = path.join(
    repositoryRoot,
    "website",
    "docs",
    "assets",
    "islands",
    "tokimu-paint",
  );

  for (const file of [
    "index.html",
    "styles.css",
    "paint.js",
    "tokimu_website_paint_engine.js",
    "tokimu_website_paint_engine_bg.wasm",
  ]) {
    await access(path.join(output, file));
  }

  const document = await readFile(path.join(output, "index.html"), "utf8");
  assert.match(document, /<main class="workbench">/);
  assert.match(document, /data-canvas-label/);
  assert.match(document, /data-brush-size/);
  assert.match(document, /lucide\.svg#pencil/);
  assert.match(document, /<canvas data-canvas/);
  assert.match(document, /<script type="module" src="\.\/paint\.js"><\/script>/);
});
