import test from "node:test";
import assert from "node:assert/strict";

import {
  isFrontFacing,
  orbitFromDrag,
  projectMeshPoint,
  screenSpaceWinding,
} from "../wwwroot/app/mesh-preview.js";

const identityView = {
  center: [0, 0, 0],
  distance: 5,
  focal: 100,
  viewportWidth: 200,
  viewportHeight: 200,
  yaw: 0,
  pitch: 0,
};

test("canvas projection retains the camera-facing GLB winding", () => {
  // The right-handed cross product is -Z, toward the identity camera at -Z.
  const front = [
    [-1, -1, -1],
    [-1, 1, -1],
    [1, 1, -1],
  ].map((point) => projectMeshPoint(point, identityView));

  assert.ok(screenSpaceWinding(front) > 0);
  assert.equal(isFrontFacing(front), true);
  assert.equal(isFrontFacing([front[0], front[2], front[1]]), false);
});

test("drag orbit moves the virtual camera opposite the pointer", () => {
  const origin = { yaw: 0, pitch: 0 };
  const right = orbitFromDrag(origin, 10, 0);
  const down = orbitFromDrag(origin, 0, 10);

  assert.equal(right.yaw, -0.12);
  assert.equal(right.pitch, 0);
  assert.equal(down.yaw, 0);
  assert.equal(down.pitch, -0.12);
});

test("vertical orbit remains bounded", () => {
  assert.equal(orbitFromDrag({ yaw: 0, pitch: 1.3 }, 0, -100).pitch, 1.35);
  assert.equal(orbitFromDrag({ yaw: 0, pitch: -1.3 }, 0, 100).pitch, -1.35);
});
