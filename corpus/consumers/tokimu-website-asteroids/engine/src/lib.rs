use particle_tools::{
    lower_particle_instances_2d, ParticleEmitter2d, ParticlePresentationRole, ParticleSpawn2d,
    ParticleSystem2d, ParticleSystemConfig, ParticleVec2, ParticleView2d, ScalarRange,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const FIXED_STEP: f32 = 1.0 / 120.0;
const MAX_STEP: f32 = FIXED_STEP * 8.0;
const SHIP_RADIUS: f32 = 1.0;
const SHIP_THRUST: f32 = 24.0;
const SHIP_DRAG: f32 = 0.16;
const SHIP_MAX_SPEED: f32 = 18.0;
const TURN_SPEED: f32 = 4.2;
const PROJECTILE_SPEED: f32 = 34.0;
const PROJECTILE_LIFETIME: f32 = 1.3;
const FIRE_INTERVAL: f32 = 0.14;
const RESPAWN_INVULNERABILITY: f32 = 2.0;
const COMBO_WINDOW: f32 = 1.8;
const MAX_ASTEROIDS: usize = 64;
const MAX_PROJECTILES: usize = 48;
const MAX_PARTICLES: usize = 320;
const PARTICLE_ROLE_THRUST: ParticlePresentationRole = ParticlePresentationRole(1);
const PARTICLE_ROLE_IMPACT: ParticlePresentationRole = ParticlePresentationRole(2);
const PARTICLE_ROLE_SHIP: ParticlePresentationRole = ParticlePresentationRole(3);
const PARTICLE_ROLE_MUZZLE: ParticlePresentationRole = ParticlePresentationRole(4);
const PARTICLE_ROLE_WAVE: ParticlePresentationRole = ParticlePresentationRole(5);
const PARTICLE_ROLE_SCORE: ParticlePresentationRole = ParticlePresentationRole(6);

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    const ZERO: Self = Self { x: 0.0, y: 0.0 };

    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn from_angle(angle: f32) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    fn scale(self, amount: f32) -> Self {
        Self::new(self.x * amount, self.y * amount)
    }

    fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    fn normalized(self) -> Self {
        let length = self.length();
        if length > f32::EPSILON {
            self.scale(1.0 / length)
        } else {
            Self::ZERO
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum Mode {
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn radius(self) -> f32 {
        match self {
            Self::Large => 3.2,
            Self::Medium => 2.0,
            Self::Small => 1.1,
        }
    }

    fn score(self) -> u32 {
        match self {
            Self::Large => 100,
            Self::Medium => 250,
            Self::Small => 500,
        }
    }

    fn child(self) -> Option<Self> {
        match self {
            Self::Large => Some(Self::Medium),
            Self::Medium => Some(Self::Small),
            Self::Small => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Ship {
    position: Vec2,
    velocity: Vec2,
    angle: f32,
    radius: f32,
    invulnerable: f32,
    thrusting: bool,
}

impl Ship {
    fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            angle: -std::f32::consts::FRAC_PI_2,
            radius: SHIP_RADIUS,
            invulnerable: RESPAWN_INVULNERABILITY,
            thrusting: false,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Asteroid {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    angle: f32,
    spin: f32,
    radius: f32,
    size: AsteroidSize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Projectile {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    angle: f32,
    lifetime: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum ParticleKind {
    Thrust,
    Impact,
    Ship,
    Muzzle,
    Wave,
    Score,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParticleSnapshot {
    id: u64,
    position: ParticleVec2,
    normalized_age: f32,
    size: f32,
    kind: ParticleKind,
}

fn new_particle_system(seed: u32) -> ParticleSystem2d {
    ParticleSystem2d::new(
        ParticleSystemConfig {
            capacity: MAX_PARTICLES,
            maximum_burst: 32,
            maximum_lifetime: 1.0,
            maximum_step_seconds: FIXED_STEP,
        },
        seed ^ 0x7061_7274,
    )
    .expect("static Asteroids particle configuration must be valid")
}

fn thrust_request() -> ParticleSpawn2d {
    ParticleSpawn2d {
        count: 0,
        origin: ParticleVec2::ZERO,
        inherited_velocity: ParticleVec2::ZERO,
        direction_radians: ScalarRange::constant(0.0),
        speed: ScalarRange::new(5.0, 10.0, "thrust.speed")
            .expect("static thrust speed range must be valid"),
        lifetime: ScalarRange::new(0.18, 0.42, "thrust.lifetime")
            .expect("static thrust lifetime range must be valid"),
        initial_size: ScalarRange::new(0.12, 0.34, "thrust.initial_size")
            .expect("static thrust size range must be valid"),
        final_size: ScalarRange::constant(0.0),
        initial_rotation: ScalarRange::new(0.0, std::f32::consts::TAU, "thrust.rotation")
            .expect("static thrust rotation range must be valid"),
        angular_velocity: ScalarRange::new(-3.0, 3.0, "thrust.angular_velocity")
            .expect("static thrust angular velocity range must be valid"),
        acceleration: ParticleVec2::ZERO,
        drag: 0.7,
        presentation_role: PARTICLE_ROLE_THRUST,
    }
}

fn particle_vec(value: Vec2) -> ParticleVec2 {
    ParticleVec2::new(value.x, value.y)
}

fn particle_role(kind: ParticleKind) -> ParticlePresentationRole {
    match kind {
        ParticleKind::Thrust => PARTICLE_ROLE_THRUST,
        ParticleKind::Impact => PARTICLE_ROLE_IMPACT,
        ParticleKind::Ship => PARTICLE_ROLE_SHIP,
        ParticleKind::Muzzle => PARTICLE_ROLE_MUZZLE,
        ParticleKind::Wave => PARTICLE_ROLE_WAVE,
        ParticleKind::Score => PARTICLE_ROLE_SCORE,
    }
}

fn particle_kind(role: ParticlePresentationRole) -> ParticleKind {
    match role {
        PARTICLE_ROLE_THRUST => ParticleKind::Thrust,
        PARTICLE_ROLE_IMPACT => ParticleKind::Impact,
        PARTICLE_ROLE_SHIP => ParticleKind::Ship,
        PARTICLE_ROLE_MUZZLE => ParticleKind::Muzzle,
        PARTICLE_ROLE_WAVE => ParticleKind::Wave,
        PARTICLE_ROLE_SCORE => ParticleKind::Score,
        _ => panic!("Asteroids received an unknown particle presentation role"),
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct InputFrame {
    thrust: bool,
    brake: bool,
    turn_left: bool,
    turn_right: bool,
    fire: bool,
    pause_pressed: bool,
    restart_pressed: bool,
    aim_x: Option<f32>,
    aim_y: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot<'a> {
    schema: u32,
    mode: Mode,
    width: f32,
    height: f32,
    elapsed: f32,
    score: u32,
    high_score: u32,
    lives: u32,
    wave: u32,
    combo: u32,
    combo_remaining: f32,
    screen_shake: f32,
    ship: &'a Ship,
    asteroids: &'a [Asteroid],
    projectiles: &'a [Projectile],
    particles: Vec<ParticleSnapshot>,
}

#[wasm_bindgen]
pub struct AsteroidsSession {
    seed: u32,
    rng: Rng,
    width: f32,
    height: f32,
    elapsed: f32,
    mode: Mode,
    score: u32,
    high_score: u32,
    lives: u32,
    wave: u32,
    combo: u32,
    combo_remaining: f32,
    screen_shake: f32,
    fire_cooldown: f32,
    next_id: u32,
    ship: Ship,
    asteroids: Vec<Asteroid>,
    projectiles: Vec<Projectile>,
    particles: ParticleSystem2d,
    thrust_emitter: ParticleEmitter2d,
}

#[wasm_bindgen]
impl AsteroidsSession {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        let particles = new_particle_system(seed);
        let thrust_emitter = ParticleEmitter2d::new(thrust_request(), 58.0)
            .expect("static Asteroids thrust emitter must be valid");
        let mut session = Self {
            seed,
            rng: Rng::new(seed),
            width: 100.0,
            height: 56.25,
            elapsed: 0.0,
            mode: Mode::Playing,
            score: 0,
            high_score: 0,
            lives: 3,
            wave: 1,
            combo: 1,
            combo_remaining: 0.0,
            screen_shake: 0.0,
            fire_cooldown: 0.0,
            next_id: 1,
            ship: Ship::new(),
            asteroids: Vec::new(),
            projectiles: Vec::new(),
            particles,
            thrust_emitter,
        };
        session.spawn_wave();
        session
    }

    pub fn set_viewport(&mut self, width: f32, height: f32) -> Result<(), JsValue> {
        if !width.is_finite() || !height.is_finite() || width < 16.0 || height < 9.0 {
            return Err(JsValue::from_str(
                "viewport dimensions must be finite and at least 16 by 9",
            ));
        }
        self.width = width.min(400.0);
        self.height = height.min(225.0);
        self.ship.position = self.wrap(self.ship.position);
        Ok(())
    }

    pub fn step(&mut self, input_json: &str, delta_seconds: f32) -> Result<String, JsValue> {
        let input = parse_input(input_json).map_err(|message| JsValue::from_str(&message))?;
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return Err(JsValue::from_str(
                "delta seconds must be finite and non-negative",
            ));
        }
        self.apply_frame(input, delta_seconds.min(MAX_STEP));
        self.snapshot()
    }

    pub fn snapshot(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.snapshot_value())
            .map_err(|error| JsValue::from_str(&format!("snapshot failed: {error}")))
    }

    pub fn reset(&mut self, seed: u32) -> Result<String, JsValue> {
        let high_score = self.high_score.max(self.score);
        *self = Self::new(seed);
        self.high_score = high_score;
        self.snapshot()
    }
}

impl AsteroidsSession {
    fn snapshot_value(&self) -> Snapshot<'_> {
        let view = ParticleView2d::new(
            ParticleVec2::new(-self.width * 0.5, -self.height * 0.5),
            ParticleVec2::new(self.width * 0.5, self.height * 0.5),
        )
        .expect("validated viewport must produce valid particle bounds");
        let particles =
            lower_particle_instances_2d(self.particles.particles(), view, MAX_PARTICLES)
                .instances
                .into_iter()
                .map(|particle| ParticleSnapshot {
                    id: particle.id.0,
                    position: particle.position,
                    normalized_age: particle.normalized_age,
                    size: particle.size,
                    kind: particle_kind(particle.presentation_role),
                })
                .collect();
        Snapshot {
            schema: 1,
            mode: self.mode,
            width: self.width,
            height: self.height,
            elapsed: self.elapsed,
            score: self.score,
            high_score: self.high_score.max(self.score),
            lives: self.lives,
            wave: self.wave,
            combo: self.combo,
            combo_remaining: self.combo_remaining,
            screen_shake: self.screen_shake,
            ship: &self.ship,
            asteroids: &self.asteroids,
            projectiles: &self.projectiles,
            particles,
        }
    }

    fn apply_frame(&mut self, input: InputFrame, delta_seconds: f32) {
        if input.restart_pressed && self.mode == Mode::GameOver {
            let high_score = self.high_score.max(self.score);
            let seed = self.seed.wrapping_add(1);
            *self = Self::new(seed);
            self.high_score = high_score;
            return;
        }
        if input.pause_pressed && self.mode != Mode::GameOver {
            self.mode = if self.mode == Mode::Paused {
                Mode::Playing
            } else {
                Mode::Paused
            };
        }
        if self.mode != Mode::Playing {
            return;
        }

        let mut remaining = delta_seconds.min(MAX_STEP);
        while remaining > 0.0 {
            let dt = remaining.min(FIXED_STEP);
            self.update(&input, dt);
            remaining -= dt;
        }
    }

    fn update(&mut self, input: &InputFrame, dt: f32) {
        self.elapsed += dt;
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        self.combo_remaining = (self.combo_remaining - dt).max(0.0);
        if self.combo_remaining == 0.0 {
            self.combo = 1;
        }
        self.screen_shake = (self.screen_shake - dt * 2.8).max(0.0);
        self.ship.invulnerable = (self.ship.invulnerable - dt).max(0.0);

        self.update_ship(input, dt);
        self.update_projectiles(dt);
        self.update_asteroids(dt);
        self.particles
            .step(dt)
            .expect("fixed Asteroids step must satisfy particle bounds");
        self.resolve_projectile_collisions();
        self.resolve_ship_collision();

        if self.asteroids.is_empty() {
            self.wave += 1;
            self.spawn_wave();
        }
    }

    fn update_ship(&mut self, input: &InputFrame, dt: f32) {
        if let (Some(aim_x), Some(aim_y)) = (input.aim_x, input.aim_y) {
            let delta = Vec2::new(aim_x, aim_y).sub(self.ship.position);
            if delta.length_squared() > 0.01 {
                self.ship.angle = delta.y.atan2(delta.x);
            }
        } else {
            let turn = (input.turn_right as u8 as f32) - (input.turn_left as u8 as f32);
            self.ship.angle += turn * TURN_SPEED * dt;
        }

        let forward = Vec2::from_angle(self.ship.angle);
        let thrust = (input.thrust as u8 as f32) - (input.brake as u8 as f32) * 0.55;
        self.ship.thrusting = thrust.abs() > 0.01;
        if self.ship.thrusting {
            self.ship.velocity = self
                .ship
                .velocity
                .add(forward.scale(thrust * SHIP_THRUST * dt));
            if input.thrust {
                self.emit_thrust(forward, dt);
            }
        }

        let drag = (1.0 - SHIP_DRAG * dt).max(0.0);
        self.ship.velocity = self.ship.velocity.scale(drag);
        let speed = self.ship.velocity.length();
        if speed > SHIP_MAX_SPEED {
            self.ship.velocity = self.ship.velocity.normalized().scale(SHIP_MAX_SPEED);
        }
        self.ship.position = self.wrap(self.ship.position.add(self.ship.velocity.scale(dt)));

        if input.fire && self.fire_cooldown == 0.0 && self.projectiles.len() < MAX_PROJECTILES {
            self.fire_projectile(forward);
            self.fire_cooldown = FIRE_INTERVAL;
        }
    }

    fn update_projectiles(&mut self, dt: f32) {
        let (width, height) = (self.width, self.height);
        for projectile in &mut self.projectiles {
            projectile.position = wrap_in(
                projectile.position.add(projectile.velocity.scale(dt)),
                width,
                height,
            );
            projectile.lifetime -= dt;
        }
        self.projectiles
            .retain(|projectile| projectile.lifetime > 0.0);
    }

    fn update_asteroids(&mut self, dt: f32) {
        let (width, height) = (self.width, self.height);
        for asteroid in &mut self.asteroids {
            asteroid.position = wrap_in(
                asteroid.position.add(asteroid.velocity.scale(dt)),
                width,
                height,
            );
            asteroid.angle += asteroid.spin * dt;
        }
    }

    fn fire_projectile(&mut self, forward: Vec2) {
        let id = self.take_id();
        let position = self.ship.position.add(forward.scale(SHIP_RADIUS + 0.35));
        self.projectiles.push(Projectile {
            id,
            position,
            velocity: self.ship.velocity.add(forward.scale(PROJECTILE_SPEED)),
            angle: self.ship.angle,
            lifetime: PROJECTILE_LIFETIME,
        });
        self.emit_muzzle_flash(position, forward);
    }

    fn resolve_projectile_collisions(&mut self) {
        let mut projectile_hits = vec![false; self.projectiles.len()];
        let mut asteroid_hits = vec![false; self.asteroids.len()];

        for (projectile_index, projectile) in self.projectiles.iter().enumerate() {
            for (asteroid_index, asteroid) in self.asteroids.iter().enumerate() {
                if asteroid_hits[asteroid_index] {
                    continue;
                }
                let radius = asteroid.radius + 0.22;
                if toroidal_distance_squared(
                    projectile.position,
                    asteroid.position,
                    self.width,
                    self.height,
                ) <= radius * radius
                {
                    projectile_hits[projectile_index] = true;
                    asteroid_hits[asteroid_index] = true;
                    break;
                }
            }
        }

        let mut children = Vec::new();
        let mut impacts = Vec::new();
        for (index, asteroid) in self.asteroids.iter().enumerate() {
            if !asteroid_hits[index] {
                continue;
            }
            self.score = self
                .score
                .saturating_add(asteroid.size.score().saturating_mul(self.combo.max(1)));
            self.high_score = self.high_score.max(self.score);
            self.combo = if self.combo_remaining > 0.0 {
                (self.combo + 1).min(8)
            } else {
                1
            };
            self.combo_remaining = COMBO_WINDOW;
            self.screen_shake = (self.screen_shake + asteroid.radius * 0.08).min(1.0);
            impacts.push((
                asteroid.position,
                asteroid.velocity,
                asteroid.radius,
                self.combo,
            ));

            if let Some(child_size) = asteroid.size.child() {
                for side in [-1.0_f32, 1.0] {
                    let angle = asteroid.velocity.y.atan2(asteroid.velocity.x) + side * 0.72;
                    let speed = asteroid.velocity.length() * 1.18 + 1.5;
                    children.push((
                        asteroid.position,
                        Vec2::from_angle(angle).scale(speed),
                        child_size,
                    ));
                }
            }
        }

        self.projectiles = self
            .projectiles
            .drain(..)
            .enumerate()
            .filter_map(|(index, projectile)| (!projectile_hits[index]).then_some(projectile))
            .collect();
        self.asteroids = self
            .asteroids
            .drain(..)
            .enumerate()
            .filter_map(|(index, asteroid)| (!asteroid_hits[index]).then_some(asteroid))
            .collect();

        for (position, velocity, radius, combo) in impacts {
            self.emit_impact(position, velocity, radius, ParticleKind::Impact);
            self.emit_score_sparks(position, velocity, combo);
        }
        for (position, velocity, size) in children {
            self.push_asteroid(position, velocity, size);
        }
    }

    fn resolve_ship_collision(&mut self) {
        if self.ship.invulnerable > 0.0 {
            return;
        }
        let collided = self.asteroids.iter().any(|asteroid| {
            let radius = asteroid.radius + self.ship.radius * 0.72;
            toroidal_distance_squared(
                self.ship.position,
                asteroid.position,
                self.width,
                self.height,
            ) <= radius * radius
        });
        if !collided {
            return;
        }

        self.emit_impact(
            self.ship.position,
            self.ship.velocity,
            3.0,
            ParticleKind::Ship,
        );
        self.emit_ship_debris_ring(self.ship.position, self.ship.velocity);
        self.screen_shake = 1.0;
        self.lives = self.lives.saturating_sub(1);
        self.combo = 1;
        self.combo_remaining = 0.0;
        if self.lives == 0 {
            self.mode = Mode::GameOver;
            self.high_score = self.high_score.max(self.score);
        } else {
            self.ship = Ship::new();
        }
    }

    fn spawn_wave(&mut self) {
        let count = (3 + self.wave as usize).min(10);
        for _ in 0..count {
            if self.asteroids.len() >= MAX_ASTEROIDS {
                break;
            }
            let edge = self.rng.range_u32(0, 4);
            let half_width = self.width * 0.5;
            let half_height = self.height * 0.5;
            let position = match edge {
                0 => Vec2::new(-half_width, self.rng.range(-half_height, half_height)),
                1 => Vec2::new(half_width, self.rng.range(-half_height, half_height)),
                2 => Vec2::new(self.rng.range(-half_width, half_width), -half_height),
                _ => Vec2::new(self.rng.range(-half_width, half_width), half_height),
            };
            let toward_center = position.scale(-1.0).normalized();
            let drift = Vec2::from_angle(self.rng.range(0.0, std::f32::consts::TAU));
            let speed = self.rng.range(3.0, 6.0) + self.wave as f32 * 0.18;
            let velocity = toward_center
                .scale(0.62)
                .add(drift.scale(0.38))
                .normalized()
                .scale(speed);
            self.push_asteroid(position, velocity, AsteroidSize::Large);
        }
        self.emit_wave_pulse();
    }

    fn push_asteroid(&mut self, position: Vec2, velocity: Vec2, size: AsteroidSize) {
        if self.asteroids.len() >= MAX_ASTEROIDS {
            return;
        }
        let id = self.take_id();
        let angle = self.rng.range(0.0, std::f32::consts::TAU);
        let spin = self.rng.range(-1.1, 1.1);
        self.asteroids.push(Asteroid {
            id,
            position: self.wrap(position),
            velocity,
            angle,
            spin,
            radius: size.radius(),
            size,
        });
    }

    fn emit_thrust(&mut self, forward: Vec2, dt: f32) {
        let direction = self.ship.angle + std::f32::consts::PI;
        let speed_fraction = (self.ship.velocity.length() / SHIP_MAX_SPEED).clamp(0.0, 1.0);
        let length_scale = 1.0 + speed_fraction * 0.9;
        self.thrust_emitter
            .set_particles_per_second(52.0 + speed_fraction * 26.0)
            .expect("static Asteroids thrust rate must be valid");
        self.thrust_emitter.set_template(ParticleSpawn2d {
            origin: particle_vec(self.ship.position.sub(forward.scale(0.9))),
            inherited_velocity: particle_vec(self.ship.velocity),
            direction_radians: ScalarRange::new(
                direction - 0.11,
                direction + 0.11,
                "thrust.direction",
            )
            .expect("static thrust direction range must be valid"),
            initial_size: ScalarRange::new(
                0.12 * length_scale,
                0.34 * length_scale,
                "thrust.initial_size",
            )
            .expect("static thrust size range must be valid"),
            ..thrust_request()
        });
        self.thrust_emitter
            .emit(&mut self.particles, dt)
            .expect("fixed Asteroids thrust emission must satisfy particle bounds");
    }

    fn emit_impact(
        &mut self,
        position: Vec2,
        inherited_velocity: Vec2,
        radius: f32,
        kind: ParticleKind,
    ) {
        let count = ((radius * 6.0) as usize).clamp(8, 28);
        let scale = radius.sqrt();
        self.particles
            .spawn(ParticleSpawn2d {
                count,
                origin: particle_vec(position),
                inherited_velocity: particle_vec(inherited_velocity.scale(0.2)),
                direction_radians: ScalarRange::new(0.0, std::f32::consts::TAU, "impact.direction")
                    .expect("static impact direction range must be valid"),
                speed: ScalarRange::new(2.0 * scale, 11.0 * scale, "impact.speed")
                    .expect("static impact speed range must be valid"),
                lifetime: ScalarRange::new(0.28, 0.86, "impact.lifetime")
                    .expect("static impact lifetime range must be valid"),
                initial_size: ScalarRange::new(0.12 * scale, 0.5 * scale, "impact.initial_size")
                    .expect("static impact size range must be valid"),
                final_size: ScalarRange::constant(0.0),
                initial_rotation: ScalarRange::new(0.0, std::f32::consts::TAU, "impact.rotation")
                    .expect("static impact rotation range must be valid"),
                angular_velocity: ScalarRange::new(-4.0, 4.0, "impact.angular_velocity")
                    .expect("static impact angular velocity range must be valid"),
                acceleration: ParticleVec2::ZERO,
                drag: 0.7,
                presentation_role: particle_role(kind),
            })
            .expect("bounded Asteroids impact request must be valid");
    }

    fn emit_muzzle_flash(&mut self, position: Vec2, forward: Vec2) {
        let direction = self.ship.angle;
        self.particles
            .spawn(ParticleSpawn2d {
                count: 7,
                origin: particle_vec(position),
                inherited_velocity: particle_vec(self.ship.velocity.add(forward.scale(5.0))),
                direction_radians: ScalarRange::new(
                    direction - 0.18,
                    direction + 0.18,
                    "muzzle.direction",
                )
                .expect("static muzzle direction range must be valid"),
                speed: ScalarRange::new(6.0, 14.0, "muzzle.speed")
                    .expect("static muzzle speed range must be valid"),
                lifetime: ScalarRange::new(0.06, 0.18, "muzzle.lifetime")
                    .expect("static muzzle lifetime range must be valid"),
                initial_size: ScalarRange::new(0.08, 0.22, "muzzle.initial_size")
                    .expect("static muzzle size range must be valid"),
                final_size: ScalarRange::constant(0.0),
                initial_rotation: ScalarRange::constant(0.0),
                angular_velocity: ScalarRange::constant(0.0),
                acceleration: ParticleVec2::ZERO,
                drag: 0.45,
                presentation_role: PARTICLE_ROLE_MUZZLE,
            })
            .expect("bounded Asteroids muzzle request must be valid");
    }

    fn emit_wave_pulse(&mut self) {
        self.particles
            .spawn(ParticleSpawn2d {
                count: 24,
                origin: ParticleVec2::ZERO,
                inherited_velocity: ParticleVec2::ZERO,
                direction_radians: ScalarRange::new(0.0, std::f32::consts::TAU, "wave.direction")
                    .expect("static wave direction range must be valid"),
                speed: ScalarRange::new(7.0, 13.0, "wave.speed")
                    .expect("static wave speed range must be valid"),
                lifetime: ScalarRange::new(0.42, 0.78, "wave.lifetime")
                    .expect("static wave lifetime range must be valid"),
                initial_size: ScalarRange::new(0.06, 0.16, "wave.initial_size")
                    .expect("static wave size range must be valid"),
                final_size: ScalarRange::constant(0.02),
                initial_rotation: ScalarRange::constant(0.0),
                angular_velocity: ScalarRange::constant(0.0),
                acceleration: ParticleVec2::ZERO,
                drag: 0.3,
                presentation_role: PARTICLE_ROLE_WAVE,
            })
            .expect("bounded Asteroids wave request must be valid");
    }

    fn emit_ship_debris_ring(&mut self, position: Vec2, inherited_velocity: Vec2) {
        self.particles
            .spawn(ParticleSpawn2d {
                count: 14,
                origin: particle_vec(position),
                inherited_velocity: particle_vec(inherited_velocity.scale(0.15)),
                direction_radians: ScalarRange::new(0.0, std::f32::consts::TAU, "ship.direction")
                    .expect("static ship direction range must be valid"),
                speed: ScalarRange::new(4.0, 10.0, "ship.speed")
                    .expect("static ship speed range must be valid"),
                lifetime: ScalarRange::new(0.48, 0.96, "ship.lifetime")
                    .expect("static ship lifetime range must be valid"),
                initial_size: ScalarRange::new(0.08, 0.28, "ship.initial_size")
                    .expect("static ship size range must be valid"),
                final_size: ScalarRange::constant(0.0),
                initial_rotation: ScalarRange::new(0.0, std::f32::consts::TAU, "ship.rotation")
                    .expect("static ship rotation range must be valid"),
                angular_velocity: ScalarRange::new(-6.0, 6.0, "ship.angular_velocity")
                    .expect("static ship angular velocity range must be valid"),
                acceleration: ParticleVec2::ZERO,
                drag: 0.55,
                presentation_role: PARTICLE_ROLE_SHIP,
            })
            .expect("bounded Asteroids ship-debris request must be valid");
    }

    fn emit_score_sparks(&mut self, position: Vec2, inherited_velocity: Vec2, combo: u32) {
        let bounded_combo = combo.clamp(1, 8);
        let intensity = bounded_combo as f32;
        self.particles
            .spawn(ParticleSpawn2d {
                count: 4 + bounded_combo as usize,
                origin: particle_vec(position),
                inherited_velocity: particle_vec(inherited_velocity.scale(0.1)),
                direction_radians: ScalarRange::new(0.0, std::f32::consts::TAU, "score.direction")
                    .expect("static score direction range must be valid"),
                speed: ScalarRange::new(1.8, 3.2 + intensity, "score.speed")
                    .expect("static score speed range must be valid"),
                lifetime: ScalarRange::new(0.18, 0.42, "score.lifetime")
                    .expect("static score lifetime range must be valid"),
                initial_size: ScalarRange::new(0.06, 0.1 + intensity * 0.018, "score.initial_size")
                    .expect("static score size range must be valid"),
                final_size: ScalarRange::constant(0.0),
                initial_rotation: ScalarRange::constant(0.0),
                angular_velocity: ScalarRange::constant(0.0),
                acceleration: ParticleVec2::ZERO,
                drag: 0.5,
                presentation_role: PARTICLE_ROLE_SCORE,
            })
            .expect("bounded Asteroids score request must be valid");
    }

    fn wrap(&self, position: Vec2) -> Vec2 {
        wrap_in(position, self.width, self.height)
    }

    fn take_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }
}

fn wrap_in(position: Vec2, width: f32, height: f32) -> Vec2 {
    let half_width = width * 0.5;
    let half_height = height * 0.5;
    Vec2::new(
        (position.x + half_width).rem_euclid(width) - half_width,
        (position.y + half_height).rem_euclid(height) - half_height,
    )
}

fn toroidal_distance_squared(a: Vec2, b: Vec2, width: f32, height: f32) -> f32 {
    let mut dx = (a.x - b.x).abs();
    let mut dy = (a.y - b.y).abs();
    dx = dx.min(width - dx);
    dy = dy.min(height - dy);
    dx * dx + dy * dy
}

fn parse_input(input_json: &str) -> Result<InputFrame, String> {
    serde_json::from_str(input_json).map_err(|error| format!("invalid input frame: {error}"))
}

#[derive(Clone, Debug)]
struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x6d2b_79f5 } else { seed },
        }
    }

    fn next_u32(&mut self) -> u32 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        self.state = value;
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn range(&mut self, minimum: f32, maximum: f32) -> f32 {
        minimum + (maximum - minimum) * self.next_f32()
    }

    fn range_u32(&mut self, minimum: u32, maximum: u32) -> u32 {
        minimum + self.next_u32() % (maximum - minimum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_sequence(session: &mut AsteroidsSession) {
        for frame in 0..240 {
            session.apply_frame(
                InputFrame {
                    thrust: frame < 120,
                    turn_right: (40..100).contains(&frame),
                    fire: frame % 18 == 0,
                    ..InputFrame::default()
                },
                1.0 / 60.0,
            );
        }
    }

    #[test]
    fn equal_seed_and_input_produce_equal_snapshots() {
        let mut first = AsteroidsSession::new(42);
        let mut second = AsteroidsSession::new(42);
        apply_sequence(&mut first);
        apply_sequence(&mut second);

        let first = serde_json::to_string(&first.snapshot_value()).unwrap();
        let second = serde_json::to_string(&second.snapshot_value()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn frame_delta_is_bounded() {
        let mut session = AsteroidsSession::new(7);
        session.apply_frame(InputFrame::default(), 10.0);
        assert!((session.elapsed - MAX_STEP).abs() < 0.000_01);
    }

    #[test]
    fn malformed_input_is_rejected_at_the_wasm_boundary() {
        assert!(parse_input("{not-json}").is_err());
    }

    #[test]
    fn pause_stops_time_progression() {
        let mut session = AsteroidsSession::new(19);
        session.apply_frame(
            InputFrame {
                pause_pressed: true,
                ..InputFrame::default()
            },
            FIXED_STEP,
        );
        let elapsed = session.elapsed;
        session.apply_frame(InputFrame::default(), 1.0);
        assert_eq!(session.mode, Mode::Paused);
        assert_eq!(session.elapsed, elapsed);
    }

    #[test]
    fn thrust_uses_shared_bounded_particle_state() {
        let mut session = AsteroidsSession::new(31);
        for _ in 0..240 {
            session.apply_frame(
                InputFrame {
                    thrust: true,
                    ..InputFrame::default()
                },
                FIXED_STEP,
            );
        }

        assert!(session.particles.active_count() > 0);
        assert!(session.particles.active_count() <= MAX_PARTICLES);
        assert!(session
            .particles
            .particles()
            .iter()
            .all(|particle| particle.presentation_role == PARTICLE_ROLE_THRUST));
    }

    #[test]
    fn snapshot_maps_provider_neutral_roles_to_game_meaning() {
        let mut session = AsteroidsSession::new(37);
        session.emit_impact(Vec2::ZERO, Vec2::ZERO, 2.0, ParticleKind::Impact);

        let snapshot = session.snapshot_value();
        assert!(!snapshot.particles.is_empty());
        assert!(snapshot
            .particles
            .iter()
            .any(|particle| particle.kind == ParticleKind::Wave));
        assert!(snapshot
            .particles
            .iter()
            .any(|particle| particle.kind == ParticleKind::Impact));
        assert!(snapshot
            .particles
            .iter()
            .all(|particle| (0.0..=1.0).contains(&particle.normalized_age)));
    }

    #[test]
    fn gameplay_effects_map_to_bounded_shared_particle_requests() {
        let mut session = AsteroidsSession::new(43);
        let forward = Vec2::from_angle(session.ship.angle);
        session.fire_projectile(forward);
        session.emit_ship_debris_ring(session.ship.position, session.ship.velocity);
        session.emit_score_sparks(session.ship.position, session.ship.velocity, 3);

        assert!(session.particles.active_count() <= MAX_PARTICLES);
        assert!(session
            .particles
            .particles()
            .iter()
            .any(|particle| particle.presentation_role == PARTICLE_ROLE_MUZZLE));
        assert!(session
            .particles
            .particles()
            .iter()
            .any(|particle| particle.presentation_role == PARTICLE_ROLE_SHIP));
        assert!(session
            .particles
            .particles()
            .iter()
            .any(|particle| particle.presentation_role == PARTICLE_ROLE_SCORE));
    }

    #[test]
    fn particle_snapshot_stays_within_the_consumer_budget() {
        let mut session = AsteroidsSession::new(41);
        for _ in 0..720 {
            session.apply_frame(
                InputFrame {
                    thrust: true,
                    ..InputFrame::default()
                },
                FIXED_STEP,
            );
        }

        let bytes = serde_json::to_vec(&session.snapshot_value()).unwrap();
        assert!(session.particles.active_count() <= MAX_PARTICLES);
        assert!(bytes.len() < 256 * 1024);
    }
}
