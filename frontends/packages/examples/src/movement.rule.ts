// Lowered example.
//
// This rule declares `execution: "lowered"` and uses only recognized,
// deterministic constructs (a query, a fixed-step loop, arithmetic). It is
// shaped to match what the Rust host (crates/tokimu-ts-frontend) parses, so it
// lowers into the same runtime-system plan as a hand-written Rust rule.
import { rule, query } from "tokimu";

interface Vec2 {
  x: number;
  y: number;
}

export const applyVelocity = rule("apply-velocity", {
  execution: "lowered",
  inputs: ["Transform", "Velocity"],
  outputs: ["Transform"],
  signals: ["velocity-applied"],
  run(ctx) {
    for (const entity of query("Transform", "Velocity")) {
      const transform = entity.get<Vec2>("Transform");
      const velocity = entity.get<Vec2>("Velocity");
      transform.x += velocity.x * ctx.fixedDelta;
      transform.y += velocity.y * ctx.fixedDelta;
      entity.set("Transform", transform);
    }
    ctx.emit("velocity-applied");
  },
});
