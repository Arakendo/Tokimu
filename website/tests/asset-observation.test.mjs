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
