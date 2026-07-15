use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const TILE_MESH: MeshHandle = MeshHandle(1);
const WALL_MATERIAL: MaterialHandle = MaterialHandle(1);
const PELLET_MATERIAL: MaterialHandle = MaterialHandle(2);
const PLAYER_MATERIAL: MaterialHandle = MaterialHandle(3);
const GHOST_MATERIAL: MaterialHandle = MaterialHandle(4);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const GRID_WIDTH: i32 = 15;
const GRID_HEIGHT: i32 = 15;
const STEP_SECONDS: f32 = 0.16;
const TILE_SCALE: f32 = 0.92;
const PELLET_SCALE: f32 = 0.22;
const PLAYER_SCALE: f32 = 0.82;
const GHOST_SCALE: f32 = 0.82;
const GHOST_TURN_INTERVAL: u32 = 4;
const PLAYER_START: (i32, i32) = (1, 1);
const GHOST_STARTS: [(i32, i32); 2] = [(GRID_WIDTH - 2, 1), (GRID_WIDTH - 2, GRID_HEIGHT - 2)];
const SCATTER_TARGETS: [(i32, i32); 4] = [
    (GRID_WIDTH - 2, 1),
    (GRID_WIDTH - 2, GRID_HEIGHT - 2),
    (1, GRID_HEIGHT - 2),
    (1, 1),
];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Pac-Man".into(),
            width: 960,
            height: 960,
        },
        HelloPacmanApp::new(),
    )
}

struct HelloPacmanApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    camera: Camera,
    game: PacmanGame,
}

impl Default for HelloPacmanApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            camera: Camera::default(),
            game: PacmanGame::new(),
        }
    }
}

impl HelloPacmanApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let status = match self.game.state {
                GameState::Playing => "use arrows or WASD",
                GameState::Won => "maze cleared - press space to restart",
                GameState::Lost => "caught - press space to restart",
            };
            window.set_title(&format!(
                "Tokimu Hello Pac-Man | score={} | pellets_left={} | {}",
                self.game.score,
                self.game.level.pellets.len(),
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
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        let mut commands = Vec::new();
        commands.push(RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.03, 0.04, 0.06),
        }));

        for &wall in &self.game.level.walls {
            commands.push(draw_tile(wall, WALL_MATERIAL, TILE_SCALE, self.pipeline));
        }

        for &pellet in &self.game.level.pellets {
            commands.push(draw_tile(
                pellet,
                PELLET_MATERIAL,
                PELLET_SCALE,
                self.pipeline,
            ));
        }

        for &ghost in &self.game.ghosts {
            commands.push(draw_tile(ghost, GHOST_MATERIAL, GHOST_SCALE, self.pipeline));
        }

        commands.push(draw_tile(
            self.game.player,
            PLAYER_MATERIAL,
            PLAYER_SCALE,
            self.pipeline,
        ));

        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloPacmanApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(TILE_MESH, &Mesh::quad());
        renderer.upload_material(
            WALL_MATERIAL,
            &Material::new("pacman-wall", Color::rgb(0.15, 0.20, 0.26)),
        )?;
        renderer.upload_material(
            PELLET_MATERIAL,
            &Material::new("pacman-pellet", Color::rgb(0.98, 0.86, 0.42)),
        )?;
        renderer.upload_material(
            PLAYER_MATERIAL,
            &Material::new("pacman-player", Color::rgb(0.35, 0.94, 0.80)),
        )?;
        renderer.upload_material(
            GHOST_MATERIAL,
            &Material::new("pacman-ghost", Color::rgb(0.95, 0.38, 0.42)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "pacman-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_camera();
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if pressed {
                match key {
                    KeyCode::ArrowUp | KeyCode::KeyW => self.game.queue_direction(Direction::Up),
                    KeyCode::ArrowDown | KeyCode::KeyS => {
                        self.game.queue_direction(Direction::Down)
                    }
                    KeyCode::ArrowLeft | KeyCode::KeyA => {
                        self.game.queue_direction(Direction::Left)
                    }
                    KeyCode::ArrowRight | KeyCode::KeyD => {
                        self.game.queue_direction(Direction::Right)
                    }
                    KeyCode::Space => {
                        if self.game.state != GameState::Playing {
                            self.game.reset();
                        }
                    }
                    KeyCode::Escape => {}
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
        self.game.advance(delta_seconds as f32);
        self.update_window_title();
        self.render_scene()
    }
}

fn draw_tile(
    cell: (i32, i32),
    material: MaterialHandle,
    scale: f32,
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

fn tile_instance(cell: (i32, i32), scale: f32) -> Instance2d {
    let (x, y) = cell_to_world(cell);
    Instance2d::new([x, y], [scale, scale], 0.0)
}

fn cell_to_world(cell: (i32, i32)) -> (f32, f32) {
    let world_x = cell.0 as f32 - GRID_WIDTH as f32 * 0.5 + 0.5;
    let world_y = GRID_HEIGHT as f32 * 0.5 - cell.1 as f32 - 0.5;
    (world_x, world_y)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GameState {
    Playing,
    Won,
    Lost,
}

struct Level {
    walls: HashSet<(i32, i32)>,
    pellets: HashSet<(i32, i32)>,
}

impl Level {
    fn new() -> Self {
        let mut walls = HashSet::new();

        for x in 0..GRID_WIDTH {
            walls.insert((x, 0));
            walls.insert((x, GRID_HEIGHT - 1));
        }

        for y in 0..GRID_HEIGHT {
            walls.insert((0, y));
            walls.insert((GRID_WIDTH - 1, y));
        }

        for y in 2..GRID_HEIGHT - 2 {
            if y != 3 && y != 11 {
                walls.insert((4, y));
            }
            if y != 5 && y != 9 {
                walls.insert((10, y));
            }
        }

        for x in 2..GRID_WIDTH - 2 {
            if x != 5 && x != 9 {
                walls.insert((x, 4));
            }
            if x != 3 && x != 11 {
                walls.insert((x, 10));
            }
        }

        let starts = [PLAYER_START, GHOST_STARTS[0], GHOST_STARTS[1]];
        let mut pellets = HashSet::new();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = (x, y);
                if !walls.contains(&cell) && !starts.contains(&cell) {
                    pellets.insert(cell);
                }
            }
        }

        Self { walls, pellets }
    }

    fn is_wall(&self, cell: (i32, i32)) -> bool {
        self.walls.contains(&cell)
    }

    fn consume_pellet(&mut self, cell: (i32, i32)) -> bool {
        self.pellets.remove(&cell)
    }

    fn is_walkable(&self, cell: (i32, i32)) -> bool {
        in_bounds(cell) && !self.is_wall(cell)
    }
}

struct PacmanGame {
    level: Level,
    player: (i32, i32),
    direction: Direction,
    queued_direction: Direction,
    ghosts: Vec<(i32, i32)>,
    score: u32,
    state: GameState,
    step_accumulator: f32,
    ghost_update_count: u32,
    scatter_phase: usize,
}

impl PacmanGame {
    fn new() -> Self {
        let mut game = Self {
            level: Level::new(),
            player: PLAYER_START,
            direction: Direction::Right,
            queued_direction: Direction::Right,
            ghosts: GHOST_STARTS.into_iter().collect(),
            score: 0,
            state: GameState::Playing,
            step_accumulator: 0.0,
            ghost_update_count: 0,
            scatter_phase: 0,
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        self.level = Level::new();
        self.player = PLAYER_START;
        self.direction = Direction::Right;
        self.queued_direction = Direction::Right;
        self.ghosts = GHOST_STARTS.into_iter().collect();
        self.score = 0;
        self.state = GameState::Playing;
        self.step_accumulator = 0.0;
        self.ghost_update_count = 0;
        self.scatter_phase = 0;
    }

    fn queue_direction(&mut self, direction: Direction) {
        if direction != self.direction.opposite() {
            self.queued_direction = direction;
        }
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
        if self.queued_direction != self.direction.opposite() {
            self.direction = self.queued_direction;
        }

        let next_player = step_from(self.player, self.direction);
        if self.level.is_walkable(next_player) {
            self.player = next_player;
        }

        if self.level.consume_pellet(self.player) {
            self.score += 1;
        }

        if self.ghosts.contains(&self.player) {
            self.state = GameState::Lost;
            return;
        }

        self.ghost_update_count += 1;
        if self.ghost_update_count.is_multiple_of(GHOST_TURN_INTERVAL) {
            self.scatter_phase = (self.scatter_phase + 1) % SCATTER_TARGETS.len();
        }

        for index in 0..self.ghosts.len() {
            let target = self.ghost_target(index);
            if let Some(next) = next_step_toward(self.ghosts[index], target, &self.level) {
                self.ghosts[index] = next;
            }

            if self.ghosts[index] == self.player {
                self.state = GameState::Lost;
                return;
            }
        }

        if self.level.pellets.is_empty() {
            self.state = GameState::Won;
        }
    }

    fn ghost_target(&self, index: usize) -> (i32, i32) {
        match index {
            0 => self.player,
            _ => SCATTER_TARGETS[self.scatter_phase],
        }
    }
}

fn step_from(cell: (i32, i32), direction: Direction) -> (i32, i32) {
    let delta = direction.delta();
    (cell.0 + delta.0, cell.1 + delta.1)
}

fn in_bounds(cell: (i32, i32)) -> bool {
    cell.0 >= 0 && cell.0 < GRID_WIDTH && cell.1 >= 0 && cell.1 < GRID_HEIGHT
}

fn next_step_toward(start: (i32, i32), goal: (i32, i32), level: &Level) -> Option<(i32, i32)> {
    if start == goal {
        return Some(start);
    }

    let mut queue = VecDeque::new();
    let mut came_from = HashMap::new();
    queue.push_back(start);
    came_from.insert(start, start);

    while let Some(cell) = queue.pop_front() {
        for neighbor in neighbors(cell) {
            if !level.is_walkable(neighbor) || came_from.contains_key(&neighbor) {
                continue;
            }

            came_from.insert(neighbor, cell);
            if neighbor == goal {
                return reconstruct_first_step(start, goal, &came_from);
            }

            queue.push_back(neighbor);
        }
    }

    None
}

fn reconstruct_first_step(
    start: (i32, i32),
    goal: (i32, i32),
    came_from: &HashMap<(i32, i32), (i32, i32)>,
) -> Option<(i32, i32)> {
    let mut current = goal;

    while let Some(previous) = came_from.get(&current) {
        if *previous == start {
            return Some(current);
        }

        if *previous == current {
            break;
        }

        current = *previous;
    }

    None
}

fn neighbors(cell: (i32, i32)) -> [(i32, i32); 4] {
    [
        (cell.0, cell.1 - 1),
        (cell.0 - 1, cell.1),
        (cell.0, cell.1 + 1),
        (cell.0 + 1, cell.1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pathfinding_routes_around_a_blocking_wall() {
        let level = Level {
            walls: [(2, 2)].into_iter().collect(),
            pellets: HashSet::new(),
        };

        assert_eq!(next_step_toward((1, 2), (3, 2), &level), Some((1, 1)));
    }

    #[test]
    fn player_collects_pellets() {
        let mut game = PacmanGame::new();
        game.level.walls.clear();
        game.level.pellets.clear();
        game.player = (1, 1);
        game.direction = Direction::Right;
        game.queued_direction = Direction::Right;
        game.ghosts = vec![(10, 10), (11, 11)];
        game.level.pellets.insert((2, 1));
        game.level.pellets.insert((12, 12));

        game.advance(STEP_SECONDS);

        assert_eq!(game.player, (2, 1));
        assert_eq!(game.score, 1);
        assert_eq!(game.state, GameState::Playing);
    }

    #[test]
    fn ghost_chases_the_player_with_pathfinding() {
        let mut game = PacmanGame::new();
        game.level.walls = [
            (1, 4),
            (2, 4),
            (3, 4),
            (4, 4),
            (5, 4),
            (6, 4),
            (1, 6),
            (2, 6),
            (3, 6),
            (4, 6),
            (5, 6),
            (6, 6),
        ]
        .into_iter()
        .collect();
        game.level.pellets.clear();
        game.player = (5, 5);
        game.direction = Direction::Right;
        game.queued_direction = Direction::Right;
        game.ghosts = vec![(1, 5), (13, 13)];
        game.level.pellets.insert((12, 12));

        game.advance(STEP_SECONDS);

        assert_eq!(game.ghosts[0], (2, 5));
        assert_eq!(game.state, GameState::Playing);
    }
}
