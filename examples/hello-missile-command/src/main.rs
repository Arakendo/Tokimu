use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton,
    NativeWindow, Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler,
    PlatformInputEvent, PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const TILE_MESH: MeshHandle = MeshHandle(1);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const GRID_MATERIAL: MaterialHandle = MaterialHandle(2);
const CITY_MATERIAL: MaterialHandle = MaterialHandle(3);
const HOME_MATERIAL: MaterialHandle = MaterialHandle(4);
const SELECTED_HOME_MATERIAL: MaterialHandle = MaterialHandle(5);
const MISSILE_MATERIAL: MaterialHandle = MaterialHandle(6);
const INTERCEPTOR_MATERIAL: MaterialHandle = MaterialHandle(7);
const EXPLOSION_MATERIAL: MaterialHandle = MaterialHandle(8);
const AIM_MATERIAL: MaterialHandle = MaterialHandle(9);
const MISSILE_TRAIL_MATERIAL: MaterialHandle = MaterialHandle(10);
const INTERCEPTOR_TRAIL_MATERIAL: MaterialHandle = MaterialHandle(11);
const EXPLOSION_CORE_MATERIAL: MaterialHandle = MaterialHandle(12);
const EXPLOSION_RING_MATERIAL: MaterialHandle = MaterialHandle(13);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const GRID_WIDTH: i32 = 18;
const GRID_HEIGHT: i32 = 24;
const STEP_SECONDS: f32 = 0.10;
const MISSILE_SPAWN_INTERVAL: u32 = 7;
const MISSILE_SPEED: f32 = 3.25;
const INTERCEPTOR_SPEED: f32 = 9.0;
const PLAYER_Y: i32 = GRID_HEIGHT - 2;
const TURRET_SCALE: f32 = 0.82;
const CITY_SCALE: f32 = 0.74;
const MISSILE_SCALE: f32 = 0.20;
const INTERCEPTOR_SCALE: f32 = 0.20;
const EXPLOSION_SCALE: f32 = 0.58;
const AIM_SCALE: f32 = 0.20;
const TRAIL_SCALE: f32 = 0.12;
const CITY_COLUMNS: [i32; 4] = [4, 7, 10, 13];
const HOME_COLUMNS: [i32; 2] = [3, GRID_WIDTH - 4];
const MAX_INTERCEPTORS: usize = 1;
const TRAIL_LENGTH: usize = 6;
const EXPLOSION_RING_SCALE: f32 = 0.72;
const EXPLOSION_CORE_SCALE: f32 = 0.42;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Missile Command".into(),
            width: 960,
            height: 1024,
        },
        HelloMissileCommandApp::new(),
    )
}

struct HelloMissileCommandApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    camera: Camera,
    game: MissileCommandGame,
    closing: bool,
}

impl Default for HelloMissileCommandApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            camera: Camera::default(),
            game: MissileCommandGame::new(),
            closing: false,
        }
    }
}

impl HelloMissileCommandApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if self.closing {
            return;
        }

        if let Some(window) = self.window.as_ref() {
            let status = match self.game.state {
                GameState::Playing => "A/D swap home sites, move the cursor to aim, left click to fire",
                GameState::Won => "city defended - left click to restart",
                GameState::Lost => "defenses breached - left click to restart",
            };
            window.set_title(&format!(
                "Tokimu Hello Missile Command | score={} | missiles_left={} | home={} | {}",
                self.game.score,
                self.game.missiles.len(),
                self.game.selected_home_index + 1,
                status,
            ));
        }
    }

    fn update_camera(&mut self) {
        self.camera = Camera::orthographic_2d_with_height(
            self.window_size[0],
            self.window_size[1],
            GRID_HEIGHT as f32 + 4.0,
        );
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.upload_camera(CAMERA_HANDLE, self.camera);
            renderer.set_active_camera(CAMERA_HANDLE);
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        if self.closing {
            return Ok(FrameOutcome::Exit);
        }

        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        let mut commands = Vec::new();
        commands.push(RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.03, 0.04, 0.06),
        }));
        commands.push(draw_tile(
            (GRID_WIDTH / 2, GRID_HEIGHT / 2),
            BACKGROUND_MATERIAL,
            [GRID_WIDTH as f32, GRID_HEIGHT as f32],
            self.pipeline,
        ));
        commands.push(draw_tile(
            (GRID_WIDTH / 2, GRID_HEIGHT / 2),
            GRID_MATERIAL,
            [GRID_WIDTH as f32 + 1.0, GRID_HEIGHT as f32 + 1.0],
            self.pipeline,
        ));

        for city in &self.game.cities {
            if !city.destroyed {
                commands.push(draw_tile(city.position, CITY_MATERIAL, [CITY_SCALE, CITY_SCALE], self.pipeline));
            }
        }

        for (index, home) in self.game.home_positions.iter().enumerate() {
            let material = if index == self.game.selected_home_index {
                SELECTED_HOME_MATERIAL
            } else {
                HOME_MATERIAL
            };
            commands.push(draw_tile(*home, material, [TURRET_SCALE, TURRET_SCALE], self.pipeline));
        }

        commands.push(draw_world_tile(
            self.game.cursor_target,
            AIM_MATERIAL,
            [AIM_SCALE, AIM_SCALE],
            self.pipeline,
        ));

        for missile in &self.game.missiles {
            for trail in &missile.trail {
                commands.push(draw_world_tile(*trail, MISSILE_TRAIL_MATERIAL, [TRAIL_SCALE, TRAIL_SCALE], self.pipeline));
            }
            commands.push(draw_world_tile(missile.position, MISSILE_MATERIAL, [MISSILE_SCALE, MISSILE_SCALE], self.pipeline));
        }

        for interceptor in &self.game.interceptors {
            for trail in &interceptor.trail {
                commands.push(draw_world_tile(*trail, INTERCEPTOR_TRAIL_MATERIAL, [TRAIL_SCALE, TRAIL_SCALE], self.pipeline));
            }
            commands.push(draw_world_tile(interceptor.position, INTERCEPTOR_MATERIAL, [INTERCEPTOR_SCALE, INTERCEPTOR_SCALE], self.pipeline));
        }

        for explosion in &self.game.explosions {
            commands.push(draw_world_tile(
                explosion.center,
                EXPLOSION_MATERIAL,
                [EXPLOSION_SCALE, EXPLOSION_SCALE],
                self.pipeline,
            ));
            for cell in explosion_cells(explosion.center, explosion.radius) {
                commands.push(draw_tile(cell, EXPLOSION_RING_MATERIAL, [EXPLOSION_RING_SCALE, EXPLOSION_RING_SCALE], self.pipeline));
            }
            for cell in explosion_cells(explosion.center, (explosion.radius * 0.6).max(0.5)) {
                commands.push(draw_tile(cell, EXPLOSION_CORE_MATERIAL, [EXPLOSION_CORE_SCALE, EXPLOSION_CORE_SCALE], self.pipeline));
            }
        }

        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloMissileCommandApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(TILE_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("missile-background", Color::rgb(0.05, 0.06, 0.09)),
        )?;
        renderer.upload_material(
            GRID_MATERIAL,
            &Material::new("missile-grid", Color::rgb(0.16, 0.20, 0.26)),
        )?;
        renderer.upload_material(
            CITY_MATERIAL,
            &Material::new("missile-city", Color::rgb(0.40, 0.84, 0.98)),
        )?;
        renderer.upload_material(
            HOME_MATERIAL,
            &Material::new("missile-home", Color::rgb(0.50, 0.98, 0.70)),
        )?;
        renderer.upload_material(
            SELECTED_HOME_MATERIAL,
            &Material::new("missile-home-selected", Color::rgb(0.76, 0.98, 0.86)),
        )?;
        renderer.upload_material(
            MISSILE_MATERIAL,
            &Material::new("missile-enemy", Color::rgb(0.96, 0.46, 0.44)),
        )?;
        renderer.upload_material(
            INTERCEPTOR_MATERIAL,
            &Material::new("missile-interceptor", Color::rgb(0.98, 0.88, 0.42)),
        )?;
        renderer.upload_material(
            MISSILE_TRAIL_MATERIAL,
            &Material::new("missile-trail", Color::rgba(0.96, 0.46, 0.44, 0.35)),
        )?;
        renderer.upload_material(
            INTERCEPTOR_TRAIL_MATERIAL,
            &Material::new("missile-interceptor-trail", Color::rgba(0.98, 0.88, 0.42, 0.40)),
        )?;
        renderer.upload_material(
            EXPLOSION_CORE_MATERIAL,
            &Material::new("missile-explosion-core", Color::rgba(0.98, 0.90, 0.42, 0.92)),
        )?;
        renderer.upload_material(
            EXPLOSION_RING_MATERIAL,
            &Material::new("missile-explosion-ring", Color::rgba(0.98, 0.72, 0.36, 0.45)),
        )?;
        renderer.upload_material(
            EXPLOSION_MATERIAL,
            &Material::new("missile-explosion", Color::rgba(0.98, 0.82, 0.44, 0.75)),
        )?;
        renderer.upload_material(
            AIM_MATERIAL,
            &Material::new("missile-aim", Color::rgb(0.98, 0.96, 0.68)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "missile-command-pipeline",
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
            self.renderer = None;
            self.window = None;
            return Ok(());
        }

        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.game.cursor_target = self.cursor_to_world_position(x, y);
            self.update_window_title();
        }

        if let PlatformInputEvent::MouseInput { button, pressed } = event {
            if pressed && button == MouseButton::Left {
                if self.game.state == GameState::Playing {
                    self.game.fire_interceptor();
                } else {
                    self.game.reset();
                }
                self.update_window_title();
            }
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if pressed {
                match key {
                    tokimu::KeyCode::KeyA => self.game.select_home(-1),
                    tokimu::KeyCode::KeyD => self.game.select_home(1),
                    tokimu::KeyCode::Space => {
                        if self.game.state != GameState::Playing {
                            self.game.reset();
                        }
                    }
                    _ => {}
                }
                self.update_window_title();
            }
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

impl HelloMissileCommandApp {
    fn cursor_to_world_position(&self, cursor_x: f32, cursor_y: f32) -> (f32, f32) {
        let safe_width = self.window_size[0].max(1.0);
        let safe_height = self.window_size[1].max(1.0);
        let normalized_x = (cursor_x / safe_width).clamp(0.0, 1.0);
        let normalized_y = (cursor_y / safe_height).clamp(0.0, 1.0);
        let world_height = GRID_HEIGHT as f32 + 4.0;
        let world_width = world_height * (safe_width / safe_height);
        let world_x = (normalized_x - 0.5) * world_width;
        let world_y = (0.5 - normalized_y) * world_height;
        (world_x, world_y)
    }
}

fn draw_tile(
    cell: (i32, i32),
    material: MaterialHandle,
    scale: [f32; 2],
    pipeline: PipelineHandle,
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh: TILE_MESH,
        material,
        pipeline,
        instance: tile_instance(cell, scale),
        camera: Some(CAMERA_HANDLE),
        viewport: None,
    })
}

fn tile_instance(cell: (i32, i32), scale: [f32; 2]) -> Instance2d {
    let (x, y) = cell_to_world(cell);
    Instance2d::new([x, y], scale, 0.0)
}

fn draw_world_tile(
    position: (f32, f32),
    material: MaterialHandle,
    scale: [f32; 2],
    pipeline: PipelineHandle,
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh: TILE_MESH,
        material,
        pipeline,
        instance: Instance2d::new([position.0, position.1], scale, 0.0),
        camera: Some(CAMERA_HANDLE),
        viewport: None,
    })
}

fn explosion_cells(center: (f32, f32), radius: f32) -> Vec<(i32, i32)> {
    let min_x = (center.0 - radius).floor() as i32;
    let max_x = (center.0 + radius).ceil() as i32;
    let min_y = (center.1 - radius).floor() as i32;
    let max_y = (center.1 + radius).ceil() as i32;
    let mut cells = Vec::new();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if x < 0 || x >= GRID_WIDTH || y < 0 || y >= GRID_HEIGHT {
                continue;
            }

            let cell_center = cell_to_world((x, y));
            let dx = cell_center.0 - center.0;
            let dy = cell_center.1 - center.1;
            if (dx * dx + dy * dy).sqrt() <= radius {
                cells.push((x, y));
            }
        }
    }

    cells
}

fn cell_to_world(cell: (i32, i32)) -> (f32, f32) {
    let world_x = cell.0 as f32 - GRID_WIDTH as f32 * 0.5 + 0.5;
    let world_y = GRID_HEIGHT as f32 * 0.5 - cell.1 as f32 - 0.5;
    (world_x, world_y)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GameState {
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug)]
struct City {
    position: (i32, i32),
    destroyed: bool,
}

#[derive(Clone, Debug)]
struct Missile {
    position: (f32, f32),
    target: (f32, f32),
    trail: Vec<(f32, f32)>,
}

#[derive(Clone, Debug)]
struct Interceptor {
    position: (f32, f32),
    target: (f32, f32),
    trail: Vec<(f32, f32)>,
}

#[derive(Clone, Copy, Debug)]
struct Explosion {
    center: (f32, f32),
    radius: f32,
    max_radius: f32,
}

struct MissileCommandGame {
    home_positions: [(i32, i32); 2],
    selected_home_index: usize,
    cursor_target: (f32, f32),
    missiles: Vec<Missile>,
    interceptors: Vec<Interceptor>,
    explosions: Vec<Explosion>,
    cities: Vec<City>,
    score: u32,
    state: GameState,
    step_accumulator: f32,
    missile_step_count: u32,
    rng_state: u32,
}

impl MissileCommandGame {
    fn new() -> Self {
        let mut game = Self {
            home_positions: [
                (HOME_COLUMNS[0], PLAYER_Y),
                (HOME_COLUMNS[1], PLAYER_Y),
            ],
            selected_home_index: 0,
            cursor_target: (0.0, 0.0),
            missiles: Vec::new(),
            interceptors: Vec::new(),
            explosions: Vec::new(),
            cities: Vec::new(),
            score: 0,
            state: GameState::Playing,
            step_accumulator: 0.0,
            missile_step_count: 0,
            rng_state: 0xC0FFEE11,
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        self.selected_home_index = 0;
        self.cursor_target = (0.0, 0.0);
        self.missiles.clear();
        self.interceptors.clear();
        self.explosions.clear();
        self.score = 0;
        self.state = GameState::Playing;
        self.step_accumulator = 0.0;
        self.missile_step_count = 0;
        self.rng_state = 0xC0FFEE11;
        self.cities = CITY_COLUMNS
            .into_iter()
            .map(|column| City {
                position: (column, PLAYER_Y),
                destroyed: false,
            })
            .collect();
    }

    fn select_home(&mut self, delta: i32) {
        if self.state == GameState::Playing {
            let next = (self.selected_home_index as i32 + delta).rem_euclid(self.home_positions.len() as i32);
            self.selected_home_index = next as usize;
        }
    }

    fn fire_interceptor(&mut self) {
        if self.state != GameState::Playing || self.interceptors.len() >= MAX_INTERCEPTORS {
            return;
        }

        let home = self.home_positions[self.selected_home_index];
        let (home_x, home_y) = cell_to_world(home);
        let target = self.cursor_target;
        self.interceptors.push(Interceptor {
            position: (home_x, home_y - 0.4),
            target,
            trail: Vec::new(),
        });
    }

    fn advance(&mut self, delta_seconds: f32) {
        if self.state != GameState::Playing {
            return;
        }

        self.step_accumulator += delta_seconds;
        while self.step_accumulator >= STEP_SECONDS && self.state == GameState::Playing {
            self.step_accumulator -= STEP_SECONDS;
            self.step_once();
        }
    }

    fn step_once(&mut self) {
        self.missile_step_count += 1;
        if self.missile_step_count % MISSILE_SPAWN_INTERVAL == 0 {
            self.spawn_missiles();
        }

        self.move_missiles();
        self.move_interceptors();
        self.resolve_explosions();
        self.resolve_collisions();
        self.check_end_state();
    }

    fn spawn_missiles(&mut self) {
        let spawn_x = self.next_random_range(1, GRID_WIDTH - 2) as f32;
        let target_x = self.next_random_range(1, GRID_WIDTH - 2) as f32;
        self.missiles.push(Missile {
            position: cell_to_world((spawn_x as i32, 1)),
            target: cell_to_world((target_x as i32, PLAYER_Y)),
            trail: Vec::new(),
        });
    }

    fn move_missiles(&mut self) {
        for missile in &mut self.missiles {
            missile.trail.push(missile.position);
            if missile.trail.len() > TRAIL_LENGTH {
                missile.trail.remove(0);
            }

            let delta_x = missile.target.0 - missile.position.0;
            let delta_y = missile.target.1 - missile.position.1;
            let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

            if distance <= MISSILE_SPEED * STEP_SECONDS {
                missile.position = missile.target;
            } else if distance > 0.0 {
                let step = MISSILE_SPEED * STEP_SECONDS;
                missile.position.0 += delta_x / distance * step;
                missile.position.1 += delta_y / distance * step;
            }
        }
    }

    fn move_interceptors(&mut self) {
        for interceptor in &mut self.interceptors {
            interceptor.trail.push(interceptor.position);
            if interceptor.trail.len() > TRAIL_LENGTH {
                interceptor.trail.remove(0);
            }

            let delta_x = interceptor.target.0 - interceptor.position.0;
            let delta_y = interceptor.target.1 - interceptor.position.1;
            let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

            if distance <= INTERCEPTOR_SPEED * STEP_SECONDS {
                interceptor.position = interceptor.target;
            } else if distance > 0.0 {
                let step = INTERCEPTOR_SPEED * STEP_SECONDS;
                interceptor.position.0 += delta_x / distance * step;
                interceptor.position.1 += delta_y / distance * step;
            }
        }
    }

    fn resolve_explosions(&mut self) {
        for explosion in &mut self.explosions {
            explosion.radius = (explosion.radius + 0.6).min(explosion.max_radius);
        }
        self.explosions.retain(|explosion| explosion.radius < explosion.max_radius);
    }

    fn resolve_collisions(&mut self) {
        let mut detonations = Vec::new();
        self.interceptors.retain(|interceptor| {
            let dx = interceptor.position.0 - interceptor.target.0;
            let dy = interceptor.position.1 - interceptor.target.1;
            let reached_target = (dx * dx + dy * dy).sqrt() <= INTERCEPTOR_SPEED * STEP_SECONDS;
            if reached_target {
                detonations.push(interceptor.target);
            }
            !reached_target
        });

        for target in detonations {
            self.explosions.push(Explosion {
                center: target,
                    radius: 0.3,
                    max_radius: 3.0,
            });
        }

        let mut next_missiles = Vec::new();
        for missile in self.missiles.drain(..) {
            let missile_world = missile.position;
            let reached_target = {
                let dx = missile_world.0 - missile.target.0;
                let dy = missile_world.1 - missile.target.1;
                (dx * dx + dy * dy).sqrt() <= MISSILE_SPEED * STEP_SECONDS
            };

            if self.explosions.iter().any(|explosion| {
                let dx = missile_world.0 - explosion.center.0;
                let dy = missile_world.1 - explosion.center.1;
                (dx * dx + dy * dy).sqrt() <= explosion.radius
            }) {
                self.score += 10;
                self.explosions.push(Explosion {
                    center: missile_world,
                    radius: 0.3,
                    max_radius: 2.6,
                });
            } else if let Some(city) = self
                .cities
                .iter_mut()
                .find(|city| {
                    !city.destroyed
                        && (cell_to_world(city.position).0 - missile_world.0).abs() <= 0.6
                        && (cell_to_world(city.position).1 - missile_world.1).abs() <= 0.8
                })
            {
                city.destroyed = true;
            } else if reached_target {
                self.state = GameState::Lost;
            } else {
                next_missiles.push(missile);
            }
        }
        self.missiles = next_missiles;
    }

    fn check_end_state(&mut self) {
        if self.cities.iter().all(|city| city.destroyed) {
            self.state = GameState::Lost;
            return;
        }

        if self.missiles.is_empty() && self.score >= 50 {
            self.state = GameState::Won;
        }
    }

    fn next_random_range(&mut self, min: i32, max: i32) -> i32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        let span = (max - min + 1).max(1) as u32;
        min + (self.rng_state % span) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turret_moves_and_fires_one_interceptor() {
        let mut game = MissileCommandGame::new();
        game.select_home(1);
        game.cursor_target = (10.0, 8.0);
        game.fire_interceptor();
        game.fire_interceptor();

        assert_eq!(game.selected_home_index, 1);
        assert_eq!(game.interceptors.len(), 1);
        assert_eq!(game.interceptors[0].target, (10.0, 8.0));
    }

    #[test]
    fn interceptor_detonates_into_an_orb() {
        let mut game = MissileCommandGame::new();
        game.cities.clear();
        game.cursor_target = cell_to_world((8, 6));
        game.interceptors = vec![Interceptor {
            position: cell_to_world((8, 6)),
            target: cell_to_world((8, 6)),
            trail: Vec::new(),
        }];

        game.resolve_collisions();
        game.resolve_explosions();

        assert_eq!(game.explosions.len(), 1);
        assert_eq!(game.explosions[0].center, cell_to_world((8, 6)));
        assert!(game.explosions[0].radius > 0.0);
    }

    #[test]
    fn orb_destroys_missiles_in_range() {
        let mut game = MissileCommandGame::new();
        game.cities.clear();
        game.missiles = vec![Missile {
            position: cell_to_world((8, 6)),
            target: cell_to_world((8, PLAYER_Y)),
            trail: Vec::new(),
        }];
        game.explosions = vec![Explosion {
            center: cell_to_world((8, 6)),
            radius: 1.0,
            max_radius: 2.0,
        }];

        game.resolve_collisions();
        game.check_end_state();

        assert_eq!(game.score, 10);
        assert!(game.missiles.is_empty());
    }

    #[test]
    fn missiles_destroy_cities_and_can_end_the_game() {
        let mut game = MissileCommandGame::new();
        game.cities = vec![City {
            position: (4, PLAYER_Y),
            destroyed: false,
        }];
        game.missiles = vec![Missile {
            position: cell_to_world((4, PLAYER_Y)),
            target: cell_to_world((4, PLAYER_Y)),
            trail: Vec::new(),
        }];

        game.resolve_collisions();
        game.check_end_state();

        assert!(game.cities[0].destroyed);
        assert_eq!(game.state, GameState::Lost);
    }
}
