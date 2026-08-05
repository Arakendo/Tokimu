import assert from "node:assert/strict";
import { access, readFile, stat } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const repositoryRoot = path.resolve(import.meta.dirname, "..", "..");

test("the Ratatui lab keeps layout and rasterization inside Rust/WASM", async () => {
  const source = await readFile(
    path.join(
      repositoryRoot,
      "corpus",
      "consumers",
      "tokimu-website-ratatui-lab",
      "web",
      "ratatui-lab.ts",
    ),
    "utf8",
  );

  assert.match(source, /template_frame_rgba/);
  assert.match(source, /putImageData/);
  assert.match(source, /Ratatui widgets -> TokimuBackend -> Tokimu text raster/);
  assert.doesNotMatch(source, /template_snapshot|fillText|strokeText/);
});

test("the Ratatui lab page states the provider and browser boundaries", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "website", "docs", "lab", "ratatui.md"),
    "utf8",
  );

  assert.match(source, /retained TokimuBackend surface/);
  assert.match(source, /Tokimu's font\s+provider rasterizes/);
  assert.match(source, /blits the resulting RGBA frame/);
  assert.match(source, /does not interpret Ratatui cells/);
});

test("the published Ratatui lab contains the Tokimu-rendered payload", async () => {
  const output = path.join(
    repositoryRoot,
    "website",
    "docs",
    "assets",
    "islands",
    "ratatui-lab",
  );
  const files = [
    "tokimu_website_ratatui_lab_engine_bg.wasm",
    "tokimu_website_ratatui_lab_engine.js",
    "ratatui-lab.js",
    "index.html",
    "styles.css",
  ];
  const sizes = await Promise.all(
    files.map((file) => stat(path.join(output, file)).then((entry) => entry.size)),
  );

  for (const file of files) await access(path.join(output, file));
  assert.ok(sizes[0] <= 640 * 1024, `Ratatui lab WASM payload grew to ${sizes[0]} bytes`);
  assert.ok(
    sizes.reduce((sum, size) => sum + size, 0) <= 768 * 1024,
    "Ratatui lab first-load payload exceeded 768 KiB",
  );

  const browserSource = await readFile(path.join(output, "ratatui-lab.js"), "utf8");
  assert.match(browserSource, /template_frame_rgba/);
  assert.match(browserSource, /putImageData/);
  assert.doesNotMatch(browserSource, /fillText|template_snapshot/);
});
