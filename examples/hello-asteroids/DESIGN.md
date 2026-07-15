# Hello Asteroids - Example Design Document

## Purpose

`hello-asteroids` is a 2D space shooter example that proves continuous physics/movement, rotation and momentum-based movement, asteroid fragmentation/spawning, player shooting mechanics with projectiles, and collision detection between projectiles/asteroids and player/asteroids.

Unlike `hello-snake` (grid-based) or `hello-space-invaders` (discrete wave movement), `hello-asteroids` demonstrates:

- Continuous 2D vector movement
- Rotation and thrust-based physics
- Object fragmentation (asteroid splitting into smaller pieces)
- Screen wrapping for objects
- Collision detection between moving entities

## Architecture Boundaries

This example must remain strictly within the boundaries defined in [ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation state and scheduling concepts
- Rendering must not mutate simulation state
- The game logic lives entirely in the runtime/application layer
- No external assets are required for MVP (use procedural geometry)

## Acceptance Criteria

A milestone flip to `[x]` when its deliverables are done and exercised by this example:

- [x] Engine boots, opens a native window, and maintains fixed-step simulation loop
- [x] Player ship can rotate left/right using keyboard input
- [x] Player ship can thrust forward using continuous physics/movement
- [x] Projectiles fire from the player ship in the direction it's facing
- [x] Asteroids spawn at random positions with random velocities
- [x] Asteroids move continuously and wrap around screen edges
- [x] Asteroid collision with projectiles destroys both
- [x] Large asteroids split into smaller asteroids when hit
- [x] Player collision with asteroid ends the game or resets state
- [x] Game state (score, remaining asteroids, lives) is visible in UI

## Core Entities & State

The simulation will track:

1. **Player Ship**
   - Position (2D vector)
   - Velocity (2D vector)
   - Rotation angle
   - Facing direction (derived from rotation)

2. **Projectiles/Bullets**
   - Position (2D vector)
   - Velocity (2D vector)
   - Lifetime/duration

3. **Asteroids**
   - Position (2D vector)
   - Velocity (2D vector)
   - Size/level (large, medium, small)
   - Rotation angle and rotation speed

4. **Game State**
   - Score
   - Remaining asteroids count
   - Player lives or game-over state

## Collision Detection Strategy

For MVP, use simple circle-based collision detection:

- Each entity has a position and a radius/collision size
- Two entities collide if the distance between their centers is less than the sum of their radii

This avoids complex polygon collision while still proving the concept.

## Input Mapping

- **Left/Right Arrow or A/D**: Rotate ship counterclockwise/clockwise
- **Up Arrow or W**: Apply thrust in the direction the ship is facing
- **Space**: Fire projectile
- **Escape**: Exit game
- **R or Space (when game over)**: Restart game

## Rendering Strategy

Use Tokimu's 2D rendering pipeline with procedural geometry:

1. **Ship Mesh**: A simple triangle or custom polygon pointing up
2. **Asteroid Mesh**: Irregular polygons or circular shapes with varied edges
3. **Projectile Mesh**: Small line segments or filled circles
4. **Background**: Solid dark color or subtle stars

All meshes and materials should be uploaded and managed through the `WgpuBackend`.

## Implementation Preferences

- Favor small, compileable increments over large speculative rewrites
- Keep early implementations boring and concrete
- Rendering must not mutate simulation state
- Determinism-related behavior should be explicit: fixed timestep policy, clear ownership of time progression
- Acceptance-criteria-complete work over speculative completeness

## Next Steps After MVP

Once the core mechanics are proven:

1. Add visual effects (explosions, thrust flames)
2. Add sound effects (if audio infrastructure is available)
3. Implement difficulty scaling (faster asteroids, more spawn rate)
4. Add power-ups or special weapons

## References

- [Tokimu Software Design Document](../../docs/Tokimu Software Design Document.md)
- [ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md)
- [Roadmap](../../docs/roadmap.md)
