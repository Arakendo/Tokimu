import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const pageUrl = new URL(
  "../docs/architecture/rust-and-typescript.md",
  import.meta.url,
);

test("the public authoring page preserves the Rust and TypeScript boundary", async () => {
  const page = await readFile(pageUrl, "utf8");

  assert.match(page, /Tokimu is implemented primarily in Rust/);
  assert.match(
    page,
    /TypeScript supplies syntax, types, and tooling\. Tokimu owns the semantics\./,
  );
  assert.match(page, /does \*\*not\*\* make TypeScript a second engine/);
  assert.match(page, /TypeScript-first authoring direction/);
  assert.match(page, /Website TypeScript is a different role/);
  assert.match(page, /does not yet demonstrate authored scenes or rules/);
});
