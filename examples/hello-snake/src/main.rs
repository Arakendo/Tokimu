use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const BOARD_MESH: MeshHandle = MeshHandle(1);
const BOARD_BORDER_MATERIAL: MaterialHandle = MaterialHandle(1);
const BOARD_MATERIAL: MaterialHandle = MaterialHandle(2);
const SNAKE_HEAD_MATERIAL: MaterialHandle = MaterialHandle(3);
const SNAKE_BODY_MATERIAL: MaterialHandle = MaterialHandle(4);
const FOOD_MATERIAL: MaterialHandle = MaterialHandle(5);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 20;
const STEP_SECONDS: f32 = 0.14;
const SEGMENT_SCALE: f32 = 0.88;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Snake".into(),
            width: 960,
            height: 960,
        },
        HelloSnakeApp::new(),
    )
}

struct HelloSnakeApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    camera: Camera,
    game: SnakeGame,
}

impl Default for HelloSnakeApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            camera: Camera::default(),
            game: SnakeGame::new(),
        }
    }
}

impl HelloSnakeApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let status = if self.game.game_over {
                "game over - press space to restart"
            } else {
                "use arrows or WASD"
            };
            window.set_title(&format!(
                "Tokimu Hello Snake | score={} | length={} | {}",
                self.game.score,
                self.game.snake.len(),
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

    fn render_board(&mut self) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.04, 0.05, 0.07),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: BOARD_MESH,
                material: BOARD_BORDER_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::new(
                    [0.0, 0.0],
                    [GRID_WIDTH as f32 + 1.0, GRID_HEIGHT as f32 + 1.0],
                    0.0,
                ),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: BOARD_MESH,
                material: BOARD_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::new([0.0, 0.0], [GRID_WIDTH as f32, GRID_HEIGHT as f32], 0.0),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: BOARD_MESH,
                material: FOOD_MATERIAL,
                pipeline: self.pipeline,
                instance: cell_instance(self.game.food, 0.82),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
        ]);

        for (index, segment) in self.game.snake.iter().enumerate() {
            let material = if index == 0 {
                SNAKE_HEAD_MATERIAL
            } else {
                SNAKE_BODY_MATERIAL
            };
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: BOARD_MESH,
                material,
                pipeline: self.pipeline,
                instance: cell_instance(*segment, SEGMENT_SCALE),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            })]);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloSnakeApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(BOARD_MESH, &Mesh::quad());
        renderer.upload_material(
            BOARD_BORDER_MATERIAL,
            &Material::new("board-border", Color::rgb(0.15, 0.20, 0.24)),
        )?;
        renderer.upload_material(
            BOARD_MATERIAL,
            &Material::new("board", Color::rgb(0.08, 0.10, 0.13)),
        )?;
        renderer.upload_material(
            SNAKE_HEAD_MATERIAL,
            &Material::new("snake-head", Color::rgb(0.52, 0.98, 0.70)),
        )?;
        renderer.upload_material(
            SNAKE_BODY_MATERIAL,
            &Material::new("snake-body", Color::rgb(0.20, 0.76, 0.44)),
        )?;
        renderer.upload_material(
            FOOD_MATERIAL,
            &Material::new("snake-food", Color::rgb(0.96, 0.34, 0.36)),
        )?;
        self.pipeline = renderer
            .register_pipeline(&Pipeline::new("snake-pipeline", PipelineKind::SolidColor2d))?;
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
                        if self.game.game_over {
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
        self.render_board()
    }
}

fn cell_instance(cell: (i32, i32), scale: f32) -> Instance2d {
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

struct SnakeGame {
    snake: Vec<(i32, i32)>,
    direction: Direction,
    queued_direction: Direction,
    food: (i32, i32),
    score: u32,
    game_over: bool,
    step_accumulator: f32,
    rng_state: u32,
}

impl SnakeGame {
    fn new() -> Self {
        Self::with_seed(0xA5A5_1F1F)
    }

    fn with_seed(seed: u32) -> Self {
        let mut game = Self {
            snake: Vec::new(),
            direction: Direction::Right,
            queued_direction: Direction::Right,
            food: (0, 0),
            score: 0,
            game_over: false,
            step_accumulator: 0.0,
            rng_state: seed,
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        self.snake = vec![
            (GRID_WIDTH / 2, GRID_HEIGHT / 2),
            (GRID_WIDTH / 2 - 1, GRID_HEIGHT / 2),
            (GRID_WIDTH / 2 - 2, GRID_HEIGHT / 2),
        ];
        self.direction = Direction::Right;
        self.queued_direction = Direction::Right;
        self.score = 0;
        self.game_over = false;
        self.step_accumulator = 0.0;
        self.spawn_food();
    }

    fn queue_direction(&mut self, direction: Direction) {
        if direction != self.direction.opposite() {
            self.queued_direction = direction;
        }
    }

    fn advance(&mut self, delta_seconds: f32) {
        if self.game_over {
            return;
        }

        self.step_accumulator += delta_seconds;
        while self.step_accumulator >= STEP_SECONDS && !self.game_over {
            self.step_accumulator -= STEP_SECONDS;
            self.step_once();
        }
    }

    fn step_once(&mut self) {
        if self.queued_direction != self.direction.opposite() {
            self.direction = self.queued_direction;
        }

        let head = self.snake[0];
        let delta = self.direction.delta();
        let next_head = (head.0 + delta.0, head.1 + delta.1);

        if !self.in_bounds(next_head) || self.snake.contains(&next_head) {
            self.game_over = true;
            return;
        }

        self.snake.insert(0, next_head);

        if next_head == self.food {
            self.score += 1;
            if !self.spawn_food() {
                self.game_over = true;
            }
        } else {
            let _ = self.snake.pop();
        }
    }

    fn spawn_food(&mut self) -> bool {
        let total_cells = GRID_WIDTH * GRID_HEIGHT;
        let start = self.next_random(total_cells as u32) as i32;

        for offset in 0..total_cells {
            let index = (start + offset) % total_cells;
            let candidate = (index % GRID_WIDTH, index / GRID_WIDTH);
            if !self.snake.contains(&candidate) {
                self.food = candidate;
                return true;
            }
        }

        false
    }

    fn in_bounds(&self, cell: (i32, i32)) -> bool {
        cell.0 >= 0 && cell.0 < GRID_WIDTH && cell.1 >= 0 && cell.1 < GRID_HEIGHT
    }

    fn next_random(&mut self, modulus: u32) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        if modulus == 0 {
            0
        } else {
            self.rng_state % modulus
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_and_grows_when_food_is_eaten() {
        let mut game = SnakeGame::with_seed(1);
        game.snake = vec![(4, 4), (3, 4), (2, 4)];
        game.direction = Direction::Right;
        game.queued_direction = Direction::Right;
        game.food = (5, 4);

        game.advance(STEP_SECONDS);

        assert_eq!(game.snake[0], (5, 4));
        assert_eq!(game.score, 1);
        assert_eq!(game.snake.len(), 4);
        assert!(!game.game_over);
    }

    #[test]
    fn stops_on_wall_collision() {
        let mut game = SnakeGame::with_seed(2);
        game.snake = vec![
            (GRID_WIDTH - 1, 4),
            (GRID_WIDTH - 2, 4),
            (GRID_WIDTH - 3, 4),
        ];
        game.direction = Direction::Right;
        game.queued_direction = Direction::Right;

        game.advance(STEP_SECONDS);

        assert!(game.game_over);
    }

    #[test]
    fn ignores_reverse_turns() {
        let mut game = SnakeGame::with_seed(3);
        game.direction = Direction::Right;
        game.queued_direction = Direction::Right;

        game.queue_direction(Direction::Left);

        assert_eq!(game.queued_direction, Direction::Right);
    }
}
