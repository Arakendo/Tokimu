import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import init, {
  inspect_asset,
} from "../docs/assets/islands/asset-observation/tokimu_asset_workbench_engine.js";

const assetRoot = new URL(
  "../docs/assets/islands/asset-observation/",
  import.meta.url,
);
const wasm = await readFile(
  new URL("tokimu_asset_workbench_engine_bg.wasm", assetRoot),
);
await init({ module_or_path: wasm });

test("the known SVG fixture produces stable provider-neutral evidence", async () => {
  const bytes = new Uint8Array(
    await readFile(new URL("shapes-rect-01-geometry.svg", assetRoot)),
  );
  const observation = JSON.parse(
    inspect_asset("shapes-rect-01-geometry.svg", bytes),
  );
  const paths = observation.preview?.paths ?? [];
  const contours = paths.reduce(
    (total, path) => total + path.contours.length,
    0,
  );
  const points = paths.reduce(
    (total, path) =>
      total +
      path.contours.reduce(
        (pathTotal, contour) => pathTotal + contour.points.length,
        0,
      ),
    0,
  );

  assert.equal(observation.status, "renderable");
  assert.equal(paths.length, 4);
  assert.equal(contours, 4);
  assert.equal(points, 48);
  assert.deepEqual(observation.diagnostics, []);
});

test("malformed SVG becomes a bounded observation diagnostic", () => {
  const bytes = new TextEncoder().encode("<svg><");
  const observation = JSON.parse(inspect_asset("malformed.svg", bytes));

  assert.equal(observation.status, "error");
  assert.equal(observation.preview, null);
  assert.equal(observation.diagnostics.length, 1);
  assert.match(observation.diagnostics[0], /xml|svg|document|element/i);
});

test("empty and binary-corrupted SVG inputs fail with bounded diagnostics", () => {
  const fixtures = [
    ["empty.svg", new Uint8Array()],
    ["binary-corrupted.svg", new Uint8Array([0, 255, 0, 255, 60, 115, 118, 103])],
  ];

  for (const [fileName, bytes] of fixtures) {
    const observation = JSON.parse(inspect_asset(fileName, bytes));

    assert.equal(observation.status, "error", fileName);
    assert.equal(observation.preview, null, fileName);
    assert.ok(observation.diagnostics.length >= 1, fileName);
    assert.ok(observation.diagnostics.length <= 8, fileName);
  }
});

test("an entity-bearing SVG never becomes active browser content", () => {
  const bytes = new TextEncoder().encode(
    '<!DOCTYPE svg [<!ENTITY probe "TOKIMU_ENTITY_PROBE">]>' +
      '<svg xmlns="http://www.w3.org/2000/svg"><text>&probe;</text></svg>',
  );
  const observation = JSON.parse(inspect_asset("entity-probe.svg", bytes));

  assert.ok(["error", "inspected", "previewable", "renderable"].includes(observation.status));
  assert.ok(observation.diagnostics.length <= 8);
  assert.doesNotMatch(JSON.stringify(observation), /TOKIMU_ENTITY_PROBE/);
});
