import assert from "node:assert/strict";
import { access, readFile, stat } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const repositoryRoot = path.resolve(import.meta.dirname, "..", "..");

test("kernel UI is a bounded website island with explicit release ownership", async () => {
  const source = await readFile(path.join(repositoryRoot, "website", "interactive", "kernel-ui.ts"), "utf8");
  assert.match(source, /register\(\s*"kernel-ui"/);
  assert.match(source, /Tokimu kernel UI resource workbench/);
  assert.match(source, /waitForFrame\(frame, signal\)/);
  assert.match(source, /frame\.src = "about:blank"/);
  assert.match(source, /fallback\.hidden = false/);
});

test("the browser adapter consumes model observations rather than reproducing resource policy", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "corpus", "consumers", "tokimu-website-kernel-ui", "web", "kernel-ui.ts"),
    "utf8",
  );
  assert.match(source, /new KernelUiSession\(\)/);
  assert.match(source, /active\.apply\(\)/);
  assert.match(source, /active\.request_delete\(\)/);
  assert.match(source, /session\?\.dispose\(\)/);
  assert.doesNotMatch(
    source,
    /resources\.splice|observation\??\.selectedId\s*=|observation\??\.confirmDelete\s*=/,
  );
});

test("the kernel UI page states its ownership boundary before activation", async () => {
  const source = await readFile(path.join(repositoryRoot, "website", "docs", "lab", "kernel-ui.md"), "utf8");
  assert.match(source, /data-tokimu-island="kernel-ui"/);
  assert.match(source, /Open kernel UI workbench/);
  assert.match(source, /Rust owns resource identity/);
  assert.match(source, /not because[\s\S]*belong in `tokimu-core`/);
});

test("the published kernel UI payload remains bounded", async () => {
  const output = path.join(repositoryRoot, "website", "docs", "assets", "islands", "kernel-ui");
  const files = [
    "tokimu_website_kernel_ui_engine_bg.wasm",
    "tokimu_website_kernel_ui_engine.js",
    "kernel-ui.js",
    "index.html",
    "styles.css",
  ];
  const sizes = await Promise.all(files.map((file) => stat(path.join(output, file)).then((entry) => entry.size)));
  assert.ok(sizes[0] <= 512 * 1024, `Kernel UI WASM payload grew to ${sizes[0]} bytes`);
  assert.ok(sizes.reduce((sum, size) => sum + size, 0) <= 640 * 1024, "Kernel UI first-load payload exceeded 640 KiB");
  for (const file of files) await access(path.join(output, file));
});
