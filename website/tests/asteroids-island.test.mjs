import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const repositoryRoot = path.resolve(import.meta.dirname, "..", "..");

test("Asteroids is a bounded website island with explicit lifecycle ownership", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "website", "interactive", "asteroids-game.ts"),
    "utf8",
  );

  assert.match(source, /register\(\s*"asteroids-game"/);
  assert.match(source, /Playable Tokimu Asteroid field/);
  assert.match(source, /waitForFrame\(frame, signal\)/);
  assert.match(source, /frame\.tabIndex = 0/);
  assert.match(source, /frame\.focus\(\{ preventScroll: true \}\)/);
  assert.match(source, /frame\.contentWindow\?\.focus\(\)/);
  assert.match(source, /signal\.addEventListener\("abort"/);
  assert.match(source, /frame\.src = "about:blank"/);
  assert.match(source, /frame\.remove\(\)/);
  assert.match(source, /fallback\.hidden = false/);
  assert.doesNotMatch(source, /\b(score|collision|wave|particle)\s*=/i);
});

test("the Asteroids canvas explicitly owns keyboard focus", async () => {
  const source = await readFile(
    path.join(
      repositoryRoot,
      "corpus",
      "consumers",
      "tokimu-website-asteroids",
      "web",
      "asteroids.ts",
    ),
    "utf8",
  );

  assert.match(source, /canvas\.tabIndex = 0/);
  assert.match(source, /canvas\.focus\(\{ preventScroll: true \}\)/);
  assert.match(source, /keys\.has\("KeyW"\)/);
  assert.match(source, /keys\.has\("KeyA"\)/);
  assert.match(source, /keys\.has\("KeyS"\)/);
  assert.match(source, /keys\.has\("KeyD"\)/);
});

test("the homepage keeps the playable claim useful before activation", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "website", "docs", "index.md"),
    "utf8",
  );

  assert.match(source, /Enter the asteroid field/);
  assert.match(source, /Rust owns the field; the browser owns\s+input and pixels/);
  assert.match(source, /data-tokimu-island="asteroids-game"/);
  assert.match(source, /data-island-action="activate"/);
  assert.match(source, /data-island-mount hidden/);
  assert.match(source, /"activation": "explicit"/);
});

test("the built website contains the complete standalone Asteroids payload", async () => {
  const output = path.join(
    repositoryRoot,
    "website",
    "docs",
    "assets",
    "islands",
    "asteroids-game",
  );

  for (const file of [
    "index.html",
    "styles.css",
    "asteroids.js",
    "tokimu_website_asteroids_engine.js",
    "tokimu_website_asteroids_engine_bg.wasm",
  ]) {
    await access(path.join(output, file));
  }

  const document = await readFile(path.join(output, "index.html"), "utf8");
  assert.match(document, /<html lang="en">/);
  assert.match(document, /<meta name="viewport"/);
  assert.match(document, /<main class="game-shell">/);
  assert.match(document, /<h1>Asteroid field<\/h1>/);
  assert.match(document, /<script type="module" src="\.\/asteroids\.js"><\/script>/);
});
