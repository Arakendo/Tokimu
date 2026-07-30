import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import test from "node:test";

const sourceUrl = new URL(
  "../interactive/asset-observation.ts",
  import.meta.url,
);
const stylesUrl = new URL(
  "../docs/stylesheets/tokimu.css",
  import.meta.url,
);
const designUrl = new URL(
  "../../corpus/consumers/tokimu-website/DESIGN.md",
  import.meta.url,
);

test("the asset island keeps semantic evidence accessible beside Canvas", async () => {
  const source = await readFile(sourceUrl, "utf8");

  assert.match(source, /role="img"/);
  assert.match(source, /aria-describedby="\$\{summaryId\} \$\{reportId\}"/);
  assert.match(source, /role="status"/);
  assert.match(source, /aria-live="polite"/);
  assert.match(source, /authoritative observation as text/);
});

test("local files are bounded before their bytes cross the WASM boundary", async () => {
  const source = await readFile(sourceUrl, "utf8");
  const fileHandler = source.slice(
    source.indexOf("const onFile ="),
    source.indexOf("const onResize ="),
  );
  const bytePresenter = source.slice(
    source.indexOf("const presentBytes ="),
    source.indexOf("const loadKnownFixture ="),
  );
  const sizeCheck = fileHandler.indexOf("if (file.size > maxBytes)");
  const byteRead = fileHandler.indexOf("file.arrayBuffer()");
  const byteLengthCheck = bytePresenter.indexOf("if (bytes.byteLength > maxBytes)");
  const wasmCall = bytePresenter.indexOf("engine.inspect_asset(fileName, bytes)");

  assert.ok(sizeCheck >= 0, "missing local-file size check");
  assert.ok(byteRead > sizeCheck, "file bytes are read before the size check");
  assert.ok(byteLengthCheck >= 0, "missing byte-buffer size check");
  assert.ok(wasmCall > byteLengthCheck, "WASM inspection appears before the byte-size check");
  assert.match(source, /does not upload selected bytes/);
  assert.match(source, /const MAX_DIAGNOSTICS = 8/);
});

test("the island avoids continuous and hidden presentation work", async () => {
  const source = await readFile(sourceUrl, "utf8");

  assert.doesNotMatch(source, /requestAnimationFrame|setInterval/);
  assert.match(source, /document\.hidden \|\| !isIntersecting/);
  assert.match(source, /IntersectionObserver/);
  assert.match(source, /visibilitychange/);
});

test("the visual shell declares reduced-motion and forced-color behavior", async () => {
  const styles = await readFile(stylesUrl, "utf8");

  assert.match(styles, /@media \(prefers-reduced-motion: reduce\)/);
  assert.match(styles, /@media \(forced-colors: active\)/);
  assert.match(styles, /\.visually-hidden/);
});

test("the consumer contract records local-only and textual evidence", async () => {
  const design = await readFile(designUrl, "utf8");

  assert.match(design, /User-selected files remain local to the browser tab/);
  assert.match(design, /observation summary, properties, verdict, and diagnostics/);
  assert.match(design, /event-driven\s+only/);
});

test("the published island payload remains inside its recorded launch budget", async () => {
  const payloads = [
    "../docs/assets/islands/asset-observation/tokimu_asset_workbench_engine_bg.wasm",
    "../docs/assets/islands/asset-observation/tokimu_asset_workbench_engine.js",
    "../docs/javascripts/asset-observation.js",
    "../docs/javascripts/islands.js",
    "../docs/assets/islands/asset-observation/shapes-rect-01-geometry.svg",
  ];
  const sizes = await Promise.all(
    payloads.map((path) => stat(new URL(path, import.meta.url)).then((entry) => entry.size)),
  );

  // Rust toolchain revisions vary the uncompressed component size, while the
  // complete first-load contract remains bounded by the total 1 MiB limit.
  assert.ok(sizes[0] <= 1024 * 1024, `WASM payload grew to ${sizes[0]} bytes`);
  assert.ok(sizes[1] <= 24 * 1024, `WASM binding grew to ${sizes[1]} bytes`);
  assert.ok(sizes[2] <= 24 * 1024, `island adapter grew to ${sizes[2]} bytes`);
  assert.ok(sizes[3] <= 12 * 1024, `island lifecycle grew to ${sizes[3]} bytes`);
  assert.ok(
    sizes.reduce((total, size) => total + size, 0) <= 1024 * 1024,
    `published island payload grew to ${sizes.reduce((total, size) => total + size, 0)} bytes`,
  );
});

test("timing labels distinguish startup, inspection, and presentation observations", async () => {
  const source = await readFile(sourceUrl, "utf8");

  assert.match(source, /"WASM startup"/);
  assert.match(source, /"Inspection"/);
  assert.match(source, /"First evidence"/);
  assert.match(source, /"Canvas presentation"/);
  assert.match(source, /ms observed/);
});
