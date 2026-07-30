import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("the homepage keeps useful evidence content without the interactive bundle", async () => {
  const source = await readFile(
    new URL("../docs/index.md", import.meta.url),
    "utf8",
  );

  assert.match(source, /Asset observation workbench/);
  assert.match(source, /Rust owns parsing and vector lowering/);
  assert.match(source, /Asteroid field/);
  assert.match(source, /Rust owns the field; the browser owns/);
  assert.match(source, /data-island-action="activate"/);
  assert.match(source, /data-island-mount hidden/);
  assert.match(source, /No engine payload loaded/);
  assert.match(source, /No game payload loaded/);
});
