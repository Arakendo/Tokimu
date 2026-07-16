use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const SHIP_MESH: MeshHandle = MeshHandle(1);
const ASTEROID_MESH: MeshHandle = MeshHandle(2);
const BULLET_MESH: MeshHandle = MeshHandle(3);

const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const SHIP_MATERIAL: MaterialHandle = MaterialHandle(2);
const ASTEROID_LARGE_MATERIAL: MaterialHandle = MaterialHandle(3);
const ASTEROID_MEDIUM_MATERIAL: MaterialHandle = MaterialHandle(4);
const ASTEROID_SMALL_MATERIAL: MaterialHandle = MaterialHandle(5);
const BULLET_MATERIAL: MaterialHandle = MaterialHandle(6);

const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const WORLD_WIDTH: f32 = 20.0;
const WORLD_HEIGHT: f32 = 20.0;
const SHIP_THRUST: f32 = 5.0; // units per second squared
const SHIP_MAX_SPEED: f32 = 8.0;
const BULLET_SPEED: f32 = 12.0;
const BULLET_LIFETIME: f32 = 1.5;
const ASTEROID_LARGE_SPEED: f32 = 2.0;
const ASTEROID_MEDIUM_SPEED: f32 = 3.0;
const ASTEROID_SMALL_SPEED: f32 = 4.0;
const ASTEROID_SPAWN_RATE: f32 = 1.5; // seconds between spawns when < 3 asteroids

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Asteroids".into(),
            width: 960,
            height: 960,
        },
        HelloAsteroidsApp::new(),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn add(&self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }

    fn scale(&self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0 {
            Vec2::new(self.x / len, self.y / len)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn radius(&self) -> f32 {
        match self {
            AsteroidSize::Large => 1.0,
            AsteroidSize::Medium => 0.6,
            AsteroidSize::Small => 0.35,
        }
    }

    fn material(&self) -> MaterialHandle {
        match self {
            AsteroidSize::Large => ASTEROID_LARGE_MATERIAL,
            AsteroidSize::Medium => ASTEROID_MEDIUM_MATERIAL,
            AsteroidSize::Small => ASTEROID_SMALL_MATERIAL,
        }
    }

    fn speed(&self) -> f32 {
        match self {
            AsteroidSize::Large => ASTEROID_LARGE_SPEED,
            AsteroidSize::Medium => ASTEROID_MEDIUM_SPEED,
            AsteroidSize::Small => ASTEROID_SMALL_SPEED,
        }
    }

    fn split(&self) -> Option<[AsteroidSize; 2]> {
        match self {
            AsteroidSize::Large => Some([AsteroidSize::Medium, AsteroidSize::Medium]),
            AsteroidSize::Medium => Some([AsteroidSize::Small, AsteroidSize::Small]),
            AsteroidSize::Small => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Entity {
    pos: Vec2,
    vel: Vec2,
    angle: f32, // radians
}

#[derive(Clone, Copy, Debug)]
struct Asteroid {
    entity: Entity,
    size: AsteroidSize,
    rotation_speed: f32,
}

#[derive(Clone, Copy, Debug)]
struct Bullet {
    entity: Entity,
    lifetime: f32,
}

#[derive(Clone, Copy, Debug)]
struct Ship {
    entity: Entity,
    lives: u32,
    invulnerable_until: f32,
}

impl Ship {
    fn new() -> Self {
        Self {
            entity: Entity {
                pos: Vec2::new(0.0, 0.0),
                vel: Vec2::new(0.0, 0.0),
                angle: 0.0,
            },
            lives: 3,
            invulnerable_until: 2.0,
        }
    }

    fn respawn(&mut self) {
        self.entity.pos = Vec2::new(0.0, 0.0);
        self.entity.vel = Vec2::new(0.0, 0.0);
        self.entity.angle = 0.0;
        self.invulnerable_until = 2.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

struct AsteroidsGame {
    ship: Ship,
    asteroids: Vec<Asteroid>,
    bullets: Vec<Bullet>,
    score: u32,
    state: GameState,
    time_since_last_spawn: f32,
    input_thrust_forward: bool,
    input_thrust_backward: bool,
    input_strafe_left: bool,
    input_strafe_right: bool,
    cursor_target: Option<Vec2>,
    fire_cooldown: f32,
    rng: SimpleRng,
}

impl AsteroidsGame {
    fn new() -> Self {
        let mut game = Self {
            ship: Ship::new(),
            asteroids: Vec::new(),
            bullets: Vec::new(),
            score: 0,
            state: GameState::Playing,
            time_since_last_spawn: 0.0,
            input_thrust_forward: false,
            input_thrust_backward: false,
            input_strafe_left: false,
            input_strafe_right: false,
            cursor_target: None,
            fire_cooldown: 0.0,
            rng: SimpleRng::new(),
        };
        game.spawn_initial_asteroids();
        game
    }

    fn spawn_initial_asteroids(&mut self) {
        let count = 4;
        for _ in 0..count {
            self.spawn_asteroid(AsteroidSize::Large);
        }
    }

    fn spawn_asteroid(&mut self, size: AsteroidSize) {
        // Spawn at random edge position
        let (x, y) = if self.rng.next_bool() {
            let x = if self.rng.next_bool() {
                -WORLD_WIDTH * 0.5
            } else {
                WORLD_WIDTH * 0.5
            };
            let y = self.rng.next_float(-WORLD_HEIGHT * 0.5, WORLD_HEIGHT * 0.5);
            (x, y)
        } else {
            let x = self.rng.next_float(-WORLD_WIDTH * 0.5, WORLD_WIDTH * 0.5);
            let y = if self.rng.next_bool() {
                -WORLD_HEIGHT * 0.5
            } else {
                WORLD_HEIGHT * 0.5
            };
            (x, y)
        };

        let angle = self.rng.next_float(0.0, std::f32::consts::TAU);
        let speed = size.speed();
        let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        self.asteroids.push(Asteroid {
            entity: Entity {
                pos: Vec2::new(x, y),
                vel,
                angle: self.rng.next_float(0.0, std::f32::consts::TAU),
            },
            size,
            rotation_speed: self.rng.next_float(-1.5, 1.5),
        });
    }

    fn wrap_position(&self, pos: Vec2) -> Vec2 {
        let x = if pos.x < -WORLD_WIDTH * 0.5 {
            WORLD_WIDTH * 0.5 + (pos.x + WORLD_WIDTH * 0.5)
        } else if pos.x > WORLD_WIDTH * 0.5 {
            -WORLD_WIDTH * 0.5 + (pos.x - WORLD_WIDTH * 0.5)
        } else {
            pos.x
        };

        let y = if pos.y < -WORLD_HEIGHT * 0.5 {
            WORLD_HEIGHT * 0.5 + (pos.y + WORLD_HEIGHT * 0.5)
        } else if pos.y > WORLD_HEIGHT * 0.5 {
            -WORLD_HEIGHT * 0.5 + (pos.y - WORLD_HEIGHT * 0.5)
        } else {
            pos.y
        };

        Vec2::new(x, y)
    }

    fn update_ship(&mut self, dt: f32) {
        if self.state != GameState::Playing {
            return;
        }

        if let Some(cursor_target) = self.cursor_target {
            let aim_delta = Vec2::new(
                cursor_target.x - self.ship.entity.pos.x,
                cursor_target.y - self.ship.entity.pos.y,
            );
            if aim_delta.length() > 0.001 {
                self.ship.entity.angle =
                    aim_delta.y.atan2(aim_delta.x) - std::f32::consts::FRAC_PI_2;
            }
        }

        let forward = self.ship_forward_vector();
        let right = Vec2::new(forward.y, -forward.x);
        let thrust_forward = if self.input_thrust_forward { 1.0 } else { 0.0 };
        let thrust_backward = if self.input_thrust_backward { 1.0 } else { 0.0 };
        let strafe_right = if self.input_strafe_right { 1.0 } else { 0.0 };
        let strafe_left = if self.input_strafe_left { 1.0 } else { 0.0 };
        let thrust = forward
            .scale(thrust_forward - thrust_backward)
            .add(right.scale(strafe_right - strafe_left));

        if thrust.length() > 0.0 {
            self.ship.entity.vel = self.ship.entity.vel.add(thrust.scale(SHIP_THRUST * dt));

            // Clamp max speed
            if self.ship.entity.vel.length() > SHIP_MAX_SPEED {
                self.ship.entity.vel = self.ship.entity.vel.normalize().scale(SHIP_MAX_SPEED);
            }
        }

        // Update position
        self.ship.entity.pos = self.ship.entity.pos.add(self.ship.entity.vel.scale(dt));
        self.ship.entity.pos = self.wrap_position(self.ship.entity.pos);

        // Update invulnerability
        if self.ship.invulnerable_until > 0.0 {
            self.ship.invulnerable_until -= dt;
        }

        // Fire cooldown
        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= dt;
        }
    }

    fn fire_bullet(&mut self) {
        let ship_pos = self.ship.entity.pos;
        let ship_angle = self.ship.entity.angle;
        let forward = self.ship_forward_vector();

        // Bullet spawns at the tip of the ship
        let tip_offset = forward.scale(0.6);
        let bullet_pos = ship_pos.add(tip_offset);

        let bullet_vel = forward.scale(BULLET_SPEED);

        self.bullets.push(Bullet {
            entity: Entity {
                pos: bullet_pos,
                vel: bullet_vel,
                angle: ship_angle,
            },
            lifetime: BULLET_LIFETIME,
        });
    }

    fn try_fire_bullet(&mut self) {
        if self.state == GameState::Playing && self.fire_cooldown <= 0.0 {
            self.fire_bullet();
            self.fire_cooldown = 0.25;
        }
    }

    fn ship_forward_vector(&self) -> Vec2 {
        Vec2::new(-self.ship.entity.angle.sin(), self.ship.entity.angle.cos())
    }

    fn update_bullets(&mut self, dt: f32) {
        let mut new_bullets = Vec::new();

        for bullet in &self.bullets {
            let new_pos = bullet.entity.pos.add(bullet.entity.vel.scale(dt));
            let wrapped_pos = self.wrap_position(new_pos);

            let mut updated_bullet = *bullet;
            updated_bullet.entity.pos = wrapped_pos;
            updated_bullet.lifetime -= dt;

            if updated_bullet.lifetime > 0.0 {
                new_bullets.push(updated_bullet);
            }
        }

        self.bullets = new_bullets;
    }

    fn update_asteroids(&mut self, dt: f32) {
        let mut new_asteroids = Vec::new();

        for asteroid in &self.asteroids {
            // Update position
            let new_pos = asteroid.entity.pos.add(asteroid.entity.vel.scale(dt));
            let wrapped_pos = self.wrap_position(new_pos);

            // Update rotation
            let updated_angle = asteroid.entity.angle + asteroid.rotation_speed * dt;

            let mut updated_asteroid = *asteroid;
            updated_asteroid.entity.pos = wrapped_pos;
            updated_asteroid.entity.angle = updated_angle;

            new_asteroids.push(updated_asteroid);
        }

        self.asteroids = new_asteroids;
    }

    fn check_collisions(&mut self) {
        // Bullet-Asteroid collisions - use new collections to avoid borrow issues
        let mut new_bullets = Vec::new();
        let mut new_asteroids = Vec::new();
        let mut split_spawns = Vec::new();
        let mut score_added = 0;

        for bullet in &self.bullets {
            let mut bullet_hit = false;

            for asteroid in &self.asteroids {
                let dist = ((bullet.entity.pos.x - asteroid.entity.pos.x).powi(2)
                    + (bullet.entity.pos.y - asteroid.entity.pos.y).powi(2))
                .sqrt();

                if dist < bullet.entity.vel.length() * 0.1 + asteroid.size.radius() {
                    // Collision detected
                    bullet_hit = true;

                    // Add score
                    let points = match asteroid.size {
                        AsteroidSize::Large => 20,
                        AsteroidSize::Medium => 50,
                        AsteroidSize::Small => 100,
                    };
                    score_added += points;

                    // Split asteroid if possible
                    if let Some(sizes) = asteroid.size.split() {
                        for size in sizes {
                            split_spawns.push((size, asteroid.entity));
                        }
                    }

                    break; // Bullet hits only one asteroid
                }
            }

            if !bullet_hit {
                new_bullets.push(*bullet);
            }
        }

        for asteroid in &self.asteroids {
            let mut keep_asteroid = true;
            for bullet in &self.bullets {
                let dist = ((bullet.entity.pos.x - asteroid.entity.pos.x).powi(2)
                    + (bullet.entity.pos.y - asteroid.entity.pos.y).powi(2))
                .sqrt();

                if dist < bullet.entity.vel.length() * 0.1 + asteroid.size.radius() {
                    keep_asteroid = false;
                    break;
                }
            }

            if keep_asteroid {
                new_asteroids.push(*asteroid);
            }
        }

        self.bullets = new_bullets;
        self.asteroids = new_asteroids;
        self.score += score_added;

        let mut spawned_asteroids = Vec::new();
        for (size, entity) in split_spawns {
            self.spawn_asteroid_split_into_new(size, &entity, &mut spawned_asteroids);
        }
        self.asteroids.extend(spawned_asteroids);

        // Ship-Asteroid collisions
        if self.ship.invulnerable_until <= 0.0 && self.state == GameState::Playing {
            for asteroid in &self.asteroids {
                let dist = ((self.ship.entity.pos.x - asteroid.entity.pos.x).powi(2)
                    + (self.ship.entity.pos.y - asteroid.entity.pos.y).powi(2))
                .sqrt();

                if dist < self.ship.entity.vel.length() * 0.1 + asteroid.size.radius() {
                    // Ship hit by asteroid
                    self.ship_hit();
                    break;
                }
            }
        }

        // Check win condition (spawn more asteroids if all destroyed)
        if self.asteroids.is_empty() && self.state == GameState::Playing {
            self.spawn_initial_asteroids();
        }
    }

    fn spawn_asteroid_split_into_new(
        &mut self,
        size: AsteroidSize,
        parent_entity: &Entity,
        target_asteroids: &mut Vec<Asteroid>,
    ) {
        let angle = self.rng.next_float(0.0, std::f32::consts::TAU);
        let speed = size.speed() * 1.2;
        let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        target_asteroids.push(Asteroid {
            entity: Entity {
                pos: parent_entity.pos,
                vel,
                angle: self.rng.next_float(0.0, std::f32::consts::TAU),
            },
            size,
            rotation_speed: self.rng.next_float(-1.5, 1.5),
        });
    }

    fn ship_hit(&mut self) {
        if self.ship.lives > 0 {
            self.ship.lives -= 1;
            if self.ship.lives == 0 {
                self.state = GameState::GameOver;
            } else {
                self.ship.respawn();
            }
        }
    }

    fn spawn_timer(&mut self, dt: f32) {
        if self.state != GameState::Playing {
            return;
        }

        self.time_since_last_spawn += dt;

        // Spawn more asteroids when < 3 remain
        if self.asteroids.len() < 3 && self.time_since_last_spawn >= ASTEROID_SPAWN_RATE {
            let mut rng = SimpleRng::new();

            let size = if self.asteroids.is_empty() {
                AsteroidSize::Large
            } else if rng.next_bool() {
                AsteroidSize::Medium
            } else {
                AsteroidSize::Large
            };
            self.spawn_asteroid(size);
            self.time_since_last_spawn = 0.0;
        }
    }

    fn advance(&mut self, dt: f32) {
        self.update_ship(dt);
        self.update_bullets(dt);
        self.update_asteroids(dt);
        self.check_collisions();
        self.spawn_timer(dt);
    }

    fn reset(&mut self) {
        *self = AsteroidsGame::new();
    }

    fn set_cursor_target(&mut self, cursor_target: Option<Vec2>) {
        self.cursor_target = cursor_target;
    }

    fn set_input_thrust_forward(&mut self, pressed: bool) {
        self.input_thrust_forward = pressed;
    }

    fn set_input_thrust_backward(&mut self, pressed: bool) {
        self.input_thrust_backward = pressed;
    }

    fn set_input_strafe_left(&mut self, pressed: bool) {
        self.input_strafe_left = pressed;
    }

    fn set_input_strafe_right(&mut self, pressed: bool) {
        self.input_strafe_right = pressed;
    }
}

// Simple random helpers using a basic LCG for demo purposes
struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new() -> Self {
        Self { state: 12345 }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    fn next_float(&mut self, min: f32, max: f32) -> f32 {
        let r = (self.next_u32() as f32) / 4294967295.0;
        min + r * (max - min)
    }

    fn next_bool(&mut self) -> bool {
        self.next_u32() % 2 == 0
    }
}

struct HelloAsteroidsApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    camera: Camera,
    game: AsteroidsGame,
    closing: bool,
}

impl Default for HelloAsteroidsApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            camera: Camera::default(),
            game: AsteroidsGame::new(),
            closing: false,
        }
    }
}

impl HelloAsteroidsApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let status = match self.game.state {
                GameState::Playing => {
                    format!("lives={} | score={}", self.game.ship.lives, self.game.score)
                }
                GameState::GameOver => "game over - press R or Space to restart".to_string(),
            };
            window.set_title(&format!(
                "Tokimu Hello Asteroids | {} | use WASD to move, mouse to aim, left click to fire",
                status,
            ));
        }
    }

    fn update_camera(&mut self) {
        self.camera = Camera::orthographic_2d_with_height(
            self.window_size[0],
            self.window_size[1],
            WORLD_HEIGHT + 4.0,
        );
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.upload_camera(CAMERA_HANDLE, self.camera);
            renderer.set_active_camera(CAMERA_HANDLE);
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        let mut commands = Vec::new();
        commands.push(RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.07, 0.08, 0.12),
        }));

        // Render asteroids
        for asteroid in &self.game.asteroids {
            let scale = [asteroid.size.radius() * 2.0, asteroid.size.radius() * 2.0];
            commands.push(draw_entity(
                asteroid.entity,
                asteroid.size.material(),
                scale,
                self.pipeline,
            ));
        }

        // Render bullets
        for bullet in &self.game.bullets {
            let scale = [0.24, 0.24];
            commands.push(draw_entity(
                bullet.entity,
                BULLET_MATERIAL,
                scale,
                self.pipeline,
            ));
        }

        // Render ship if not game over and not invulnerable
        if self.game.state == GameState::Playing
            && (self.game.ship.invulnerable_until <= 0.0
                || (self.game.ship.invulnerable_until as i32 % 10) >= 5)
        {
            let scale = [0.6, 0.6];
            commands.push(draw_entity(
                self.game.ship.entity,
                SHIP_MATERIAL,
                scale,
                self.pipeline,
            ));
        }

        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloAsteroidsApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;

        // Create ship mesh (triangle pointing up)
        let ship_positions: Vec<[f32; 3]> = vec![
            [0.0, 0.5, 0.0],   // top
            [-0.4, -0.5, 0.0], // bottom left
            [0.4, -0.5, 0.0],  // bottom right
        ];
        renderer.upload_mesh(
            SHIP_MESH,
            &Mesh::uniform_normal(ship_positions, [0.0, 0.0, 1.0]),
        );

        // Create asteroid mesh as a triangle fan so the final wraparound is explicit.
        let asteroid_outline = [
            [0.82, 0.10],
            [0.72, 0.43],
            [0.45, 0.72],
            [0.10, 0.84],
            [-0.24, 0.80],
            [-0.56, 0.60],
            [-0.82, 0.18],
            [-0.76, -0.18],
            [-0.62, -0.56],
            [-0.18, -0.84],
            [0.18, -0.78],
            [0.60, -0.62],
            [0.84, -0.18],
        ];
        let asteroid_positions = triangle_fan_2d(&asteroid_outline);
        renderer.upload_mesh(
            ASTEROID_MESH,
            &Mesh::uniform_normal(asteroid_positions, [0.0, 0.0, 1.0]),
        );

        // Create bullet mesh as a tiny diamond so it stays visible at small scales.
        let bullet_positions: Vec<[f32; 3]> = vec![
            [0.0, 0.35, 0.0],
            [-0.22, 0.0, 0.0],
            [0.0, -0.35, 0.0],
            [0.0, 0.35, 0.0],
            [0.0, -0.35, 0.0],
            [0.22, 0.0, 0.0],
        ];
        renderer.upload_mesh(
            BULLET_MESH,
            &Mesh::uniform_normal(bullet_positions, [0.0, 0.0, 1.0]),
        );

        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("asteroids-background", Color::rgb(0.07, 0.08, 0.12)),
        )?;
        renderer.upload_material(
            SHIP_MATERIAL,
            &Material::new("asteroids-ship", Color::rgb(0.86, 0.94, 1.0)),
        )?;
        renderer.upload_material(
            ASTEROID_LARGE_MATERIAL,
            &Material::new("asteroid-large", Color::rgb(0.88, 0.88, 0.94)),
        )?;
        renderer.upload_material(
            ASTEROID_MEDIUM_MATERIAL,
            &Material::new("asteroid-medium", Color::rgb(0.76, 0.84, 0.98)),
        )?;
        renderer.upload_material(
            ASTEROID_SMALL_MATERIAL,
            &Material::new("asteroid-small", Color::rgb(0.98, 0.84, 0.56)),
        )?;
        renderer.upload_material(
            BULLET_MATERIAL,
            &Material::new("asteroids-bullet", Color::rgb(1.0, 0.98, 0.72)),
        )?;

        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "asteroids-pipeline",
            PipelineKind::SolidColor2d,
        ))?;

        self.renderer = Some(renderer);
        self.update_camera();
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            self.closing = true;
            return Ok(());
        }

        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.game
                .set_cursor_target(Some(self.cursor_to_world_position(x, y)));
            self.update_window_title();
        }

        if let PlatformInputEvent::MouseInput { button, pressed } = event {
            if button == tokimu::MouseButton::Left && pressed {
                self.game.try_fire_bullet();
                self.update_window_title();
            }
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            match key {
                KeyCode::KeyW | KeyCode::ArrowUp => self.game.set_input_thrust_forward(pressed),
                KeyCode::KeyS | KeyCode::ArrowDown => self.game.set_input_thrust_backward(pressed),
                KeyCode::KeyA | KeyCode::ArrowLeft => self.game.set_input_strafe_left(pressed),
                KeyCode::KeyD | KeyCode::ArrowRight => self.game.set_input_strafe_right(pressed),
                KeyCode::Space => {
                    if pressed && self.game.state == GameState::GameOver {
                        self.game.reset();
                    }
                }

                KeyCode::Escape => {
                    if pressed {
                        self.closing = true;
                    }
                }
            }

            self.update_window_title();
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
            self.update_camera();
        }

        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        if self.closing {
            return Ok(FrameOutcome::Exit);
        }

        self.game.advance(delta_seconds as f32);
        self.update_window_title();
        self.render_scene()
    }
}

impl HelloAsteroidsApp {
    fn cursor_to_world_position(&self, cursor_x: f32, cursor_y: f32) -> Vec2 {
        let safe_width = self.window_size[0].max(1.0);
        let safe_height = self.window_size[1].max(1.0);
        let world_height = WORLD_HEIGHT + 4.0;
        let world_width = world_height * (safe_width / safe_height);
        let normalized_x = (cursor_x / safe_width).clamp(0.0, 1.0);
        let normalized_y = (cursor_y / safe_height).clamp(0.0, 1.0);
        Vec2::new(
            (normalized_x - 0.5) * world_width,
            (0.5 - normalized_y) * world_height,
        )
    }
}

fn draw_entity(
    entity: Entity,
    material: MaterialHandle,
    scale: [f32; 2],
    pipeline: PipelineHandle,
) -> RenderCommand {
    let mesh = if material == BULLET_MATERIAL || material == SHIP_MATERIAL {
        if material == BULLET_MATERIAL {
            BULLET_MESH
        } else {
            SHIP_MESH
        }
    } else {
        ASTEROID_MESH
    };

    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh,
        material,
        pipeline,
        instance: entity_instance(entity, scale),
        camera: Some(CAMERA_HANDLE),
        viewport: None,
    })
}

fn triangle_fan_2d(points: &[[f32; 2]]) -> Vec<[f32; 3]> {
    assert!(points.len() >= 3);

    let center = [0.0, 0.0, 0.0];
    let mut vertices = Vec::with_capacity(points.len() * 3);

    for index in 0..points.len() {
        let next = (index + 1) % points.len();

        vertices.push(center);
        vertices.push([points[index][0], points[index][1], 0.0]);
        vertices.push([points[next][0], points[next][1], 0.0]);
    }

    vertices
}

fn entity_instance(entity: Entity, scale: [f32; 2]) -> Instance2d {
    let x = entity.pos.x;
    let y = entity.pos.y;
    // Instance2d rotation is in radians
    Instance2d::new([x, y], scale, entity.angle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successive_asteroid_spawns_use_advancing_random_values() {
        let mut game = AsteroidsGame::new();

        let before_len = game.asteroids.len();
        let first = {
            game.spawn_asteroid(AsteroidSize::Large);
            *game.asteroids.last().expect("first asteroid spawn")
        };
        let second = {
            game.spawn_asteroid(AsteroidSize::Large);
            *game.asteroids.last().expect("second asteroid spawn")
        };

        assert_eq!(before_len + 2, game.asteroids.len());
        assert_ne!(first.entity.pos, second.entity.pos);
        assert_ne!(first.entity.vel, second.entity.vel);
    }

    #[test]
    fn escape_closes_the_game_loop() {
        let mut app = HelloAsteroidsApp::new();
        app.closing = true;

        let outcome = app.on_frame(0.016).expect("frame should succeed");

        assert_eq!(outcome, FrameOutcome::Exit);
    }

    #[test]
    fn cursor_target_turns_the_ship_toward_the_mouse() {
        let mut game = AsteroidsGame::new();
        game.set_cursor_target(Some(Vec2::new(10.0, 0.0)));

        game.advance(0.0);

        assert!((game.ship.entity.angle + std::f32::consts::FRAC_PI_2).abs() < 0.001);
    }

    #[test]
    fn wasd_thrust_applies_momentum_in_the_facing_frame() {
        let mut game = AsteroidsGame::new();
        game.ship.entity.angle = 0.0;
        game.set_input_thrust_forward(true);

        game.advance(1.0);

        assert!(game.ship.entity.vel.y > 0.0);
        assert!(game.ship.entity.pos.y > 0.0);
    }

    #[test]
    fn triangle_fan_wraps_back_to_the_first_point() {
        let fan = triangle_fan_2d(&[[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]]);

        assert_eq!(fan.len(), 9);
        assert_eq!(fan[0], [0.0, 0.0, 0.0]);
        assert_eq!(fan[1], [1.0, 0.0, 0.0]);
        assert_eq!(fan[2], [0.0, 1.0, 0.0]);
        assert_eq!(fan[6], [0.0, 0.0, 0.0]);
        assert_eq!(fan[7], [-1.0, 0.0, 0.0]);
        assert_eq!(fan[8], [1.0, 0.0, 0.0]);
    }

    #[test]
    fn left_click_fires_a_bullet_when_ready() {
        let mut game = AsteroidsGame::new();

        game.try_fire_bullet();

        assert_eq!(game.bullets.len(), 1);
    }
}
