export type Vec3 = [number, number, number];

export type ProjectedPoint = { x: number; y: number; depth: number };

export type MeshView = {
  yaw: number;
  pitch: number;
};

export type MeshProjection = MeshView & {
  center: Vec3;
  distance: number;
  focal: number;
  viewportWidth: number;
  viewportHeight: number;
};

export function orbitFromDrag(view: MeshView, deltaX: number, deltaY: number): MeshView {
  return {
    // The preview is an orbit view: dragging right/down moves the virtual
    // camera right/down around the fixed model, rather than rotating the
    // model with the pointer.
    yaw: view.yaw - deltaX * 0.012,
    pitch: clamp(view.pitch - deltaY * 0.012, -1.35, 1.35),
  };
}

export function projectMeshPoint(point: Vec3, view: MeshProjection): ProjectedPoint {
  const x = point[0] - view.center[0];
  const y = point[1] - view.center[1];
  const z = point[2] - view.center[2];
  const cosYaw = Math.cos(view.yaw);
  const sinYaw = Math.sin(view.yaw);
  const yawX = cosYaw * x + sinYaw * z;
  const yawZ = -sinYaw * x + cosYaw * z;
  const cosPitch = Math.cos(view.pitch);
  const sinPitch = Math.sin(view.pitch);
  const pitchY = cosPitch * y - sinPitch * yawZ;
  const depth = sinPitch * y + cosPitch * yawZ + view.distance;
  const scale = view.focal / Math.max(depth, 0.001);
  return {
    x: view.viewportWidth / 2 + yawX * scale,
    y: view.viewportHeight / 2 - pitchY * scale,
    depth,
  };
}

export function screenSpaceWinding(points: readonly ProjectedPoint[]): number {
  const [a, b, c] = points;
  if (!a || !b || !c) return 0;
  return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
}

export function isFrontFacing(points: readonly ProjectedPoint[]): boolean {
  // Canvas Y increases downward. A source triangle facing the identity camera
  // therefore has positive screen-space winding after projection.
  return screenSpaceWinding(points) > 0;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value));
}
