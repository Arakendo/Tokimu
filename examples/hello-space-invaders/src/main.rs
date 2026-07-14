use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle,
    NativeWindow, Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler,
    PlatformInputEvent, PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const TILE_MESH: MeshHandle = MeshHandle(1);
const BORDER_MATERIAL: MaterialHandle = MaterialHandle(1);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(2);
const PLAYER_MATERIAL: MaterialHandle = MaterialHandle(3);
const INVADER_MATERIAL: MaterialHandle = MaterialHandle(4);
const BULLET_MATERIAL: MaterialHandle = MaterialHandle(5);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const GRID_WIDTH: i32 = 18;
const GRID_HEIGHT: i32 = 22;
const STEP_SECONDS: f32 = 0.12;
const PLAYER_MOVE_COOLDOWN_SECONDS: f32 = 0.14;
const PLAYER_Y: i32 = GRID_HEIGHT - 2;
const INVADER_ROWS: i32 = 4;
const INVADER_COLUMNS: i32 = 8;
const INVADER_START_X: i32 = 4;
const INVADER_START_Y: i32 = 3;
const INVADER_MOVE_INTERVAL: u32 = 4;
const PLAYER_SCALE: f32 = 0.82;
const INVADER_SCALE: f32 = 0.82;
const BULLET_SCALE: f32 = 0.22;
const BULLET_LIMIT: usize = 1;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Space Invaders".into(),
            width: 960,
            height: 960,
        },
        HelloSpaceInvadersApp::new(),
    )
}

struct HelloSpaceInvadersApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    camera: Camera,
    game: SpaceInvadersGame,
}

impl Default for HelloSpaceInvadersApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            camera: Camera::default(),
            game: SpaceInvadersGame::new(),
        }
    }
}

impl HelloSpaceInvadersApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let status = match self.game.state {
                GameState::Playing => "move with arrows or WASD, space to fire",
                GameState::Won => "wave cleared - press space to restart",
                GameState::Lost => "base breached - press space to restart",
            };
            window.set_title(&format!(
                "Tokimu Hello Space Invaders | score={} | invaders_left={} | {}",
                self.game.score,
                self.game.invaders.len(),
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
        commands.push(draw_tile(
            (GRID_WIDTH / 2, GRID_HEIGHT / 2),
            BACKGROUND_MATERIAL,
            [GRID_WIDTH as f32, GRID_HEIGHT as f32],
            self.pipeline,
        ));
        commands.push(draw_tile(
            (GRID_WIDTH / 2, GRID_HEIGHT / 2),
            BORDER_MATERIAL,
            [GRID_WIDTH as f32 + 1.0, GRID_HEIGHT as f32 + 1.0],
            self.pipeline,
        ));

        for invader in &self.game.invaders {
            commands.push(draw_tile(*invader, INVADER_MATERIAL, [INVADER_SCALE, INVADER_SCALE], self.pipeline));
        }

        for bullet in &self.game.bullets {
            commands.push(draw_tile(*bullet, BULLET_MATERIAL, [BULLET_SCALE, BULLET_SCALE], self.pipeline));
        }

        commands.push(draw_tile(
            self.game.player,
            PLAYER_MATERIAL,
            [PLAYER_SCALE, PLAYER_SCALE],
            self.pipeline,
        ));

        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloSpaceInvadersApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(TILE_MESH, &Mesh::quad());
        renderer.upload_material(
            BORDER_MATERIAL,
            &Material::new("space-border", Color::rgb(0.18, 0.22, 0.28)),
        )?;
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("space-background", Color::rgb(0.05, 0.06, 0.09)),
        )?;
        renderer.upload_material(
            PLAYER_MATERIAL,
            &Material::new("space-player", Color::rgb(0.45, 0.96, 0.80)),
        )?;
        renderer.upload_material(
            INVADER_MATERIAL,
            &Material::new("space-invader", Color::rgb(0.95, 0.42, 0.48)),
        )?;
        renderer.upload_material(
            BULLET_MATERIAL,
            &Material::new("space-bullet", Color::rgb(0.98, 0.90, 0.44)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "space-invaders-pipeline",
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
                    KeyCode::ArrowLeft | KeyCode::KeyA => self.game.move_player(-1),
                    KeyCode::ArrowRight | KeyCode::KeyD => self.game.move_player(1),
                    KeyCode::Space => {
                        if self.game.state == GameState::Playing {
                            self.game.fire();
                        } else {
                            self.game.reset();
                        }
                    }
                    KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::KeyW | KeyCode::KeyS => {}
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InvaderDirection {
    Left,
    Right,
}

impl InvaderDirection {
    fn delta(self) -> i32 {
        match self {
            InvaderDirection::Left => -1,
            InvaderDirection::Right => 1,
        }
    }

    fn opposite(self) -> Self {
        match self {
            InvaderDirection::Left => InvaderDirection::Right,
            InvaderDirection::Right => InvaderDirection::Left,
        }
    }
}

struct SpaceInvadersGame {
    player: (i32, i32),
    bullets: Vec<(i32, i32)>,
    invaders: Vec<(i32, i32)>,
    invader_direction: InvaderDirection,
    score: u32,
    state: GameState,
    step_accumulator: f32,
    player_move_cooldown: f32,
    invader_step_count: u32,
}

impl SpaceInvadersGame {
    fn new() -> Self {
        let mut game = Self {
            player: (GRID_WIDTH / 2, PLAYER_Y),
            bullets: Vec::new(),
            invaders: Vec::new(),
            invader_direction: InvaderDirection::Right,
            score: 0,
            state: GameState::Playing,
            step_accumulator: 0.0,
            player_move_cooldown: 0.0,
            invader_step_count: 0,
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        self.player = (GRID_WIDTH / 2, PLAYER_Y);
        self.bullets.clear();
        self.invaders.clear();
        self.invader_direction = InvaderDirection::Right;
        self.score = 0;
        self.state = GameState::Playing;
        self.step_accumulator = 0.0;
        self.player_move_cooldown = 0.0;
        self.invader_step_count = 0;

        for row in 0..INVADER_ROWS {
            for column in 0..INVADER_COLUMNS {
                self.invaders.push((INVADER_START_X + column * 2, INVADER_START_Y + row * 2));
            }
        }
    }

    fn move_player(&mut self, delta_x: i32) {
        if self.state != GameState::Playing {
            return;
        }

        if self.player_move_cooldown > 0.0 {
            return;
        }

        let next_x = (self.player.0 + delta_x).clamp(1, GRID_WIDTH - 2);
        self.player.0 = next_x;
        self.player_move_cooldown = PLAYER_MOVE_COOLDOWN_SECONDS;
    }

    fn fire(&mut self) {
        if self.state != GameState::Playing || self.bullets.len() >= BULLET_LIMIT {
            return;
        }

        self.bullets.push((self.player.0, self.player.1 - 1));
    }

    fn advance(&mut self, delta_seconds: f32) {
        if self.state != GameState::Playing {
            return;
        }

        self.step_accumulator += delta_seconds;
        self.player_move_cooldown = (self.player_move_cooldown - delta_seconds).max(0.0);
        while self.step_accumulator >= STEP_SECONDS && self.state == GameState::Playing {
            self.step_accumulator -= STEP_SECONDS;
            self.step_once();
        }
    }

    fn step_once(&mut self) {
        self.move_bullets();
        self.invader_step_count += 1;
        if self.invader_step_count % INVADER_MOVE_INTERVAL == 0 {
            self.move_invaders();
        }
        self.resolve_bullet_hits();
        self.check_player_contact();

        if self.invaders.is_empty() {
            self.state = GameState::Won;
        }
    }

    fn move_bullets(&mut self) {
        for bullet in &mut self.bullets {
            bullet.1 -= 1;
        }
        self.bullets.retain(|bullet| bullet.1 > 0);
    }

    fn move_invaders(&mut self) {
        let edge_hit = self.invaders.iter().any(|(x, _)| match self.invader_direction {
            InvaderDirection::Right => *x >= GRID_WIDTH - 2,
            InvaderDirection::Left => *x <= 1,
        });

        if edge_hit {
            for invader in &mut self.invaders {
                invader.1 += 1;
            }
            self.invader_direction = self.invader_direction.opposite();
        } else {
            let delta_x = self.invader_direction.delta();
            for invader in &mut self.invaders {
                invader.0 += delta_x;
            }
        }
    }

    fn resolve_bullet_hits(&mut self) {
        let mut remaining_bullets = Vec::new();
        for bullet in self.bullets.drain(..) {
            if let Some(hit_index) = self.invaders.iter().position(|invader| *invader == bullet) {
                self.invaders.remove(hit_index);
                self.score += 10;
            } else if bullet.1 > 0 {
                remaining_bullets.push(bullet);
            }
        }
        self.bullets = remaining_bullets;
    }

    fn check_player_contact(&mut self) {
        if self.invaders.iter().any(|invader| *invader == self.player || invader.1 >= self.player.1) {
            self.state = GameState::Lost;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_moves_within_bounds_and_fires_one_bullet() {
        let mut game = SpaceInvadersGame::new();
        game.move_player(-1);
        game.advance(PLAYER_MOVE_COOLDOWN_SECONDS as f32);
        game.move_player(-1);
        game.fire();
        game.fire();

        assert_eq!(game.player.0, GRID_WIDTH / 2 - 2);
        assert_eq!(game.bullets.len(), 1);
        assert_eq!(game.bullets[0], (GRID_WIDTH / 2 - 2, PLAYER_Y - 1));
    }

    #[test]
    fn bullet_hits_invader_and_scores() {
        let mut game = SpaceInvadersGame::new();
        game.invaders = vec![(5, 4)];
        game.bullets = vec![(5, 5)];
        game.invader_direction = InvaderDirection::Right;

        game.step_once();

        assert!(game.invaders.is_empty());
        assert_eq!(game.score, 10);
        assert_eq!(game.state, GameState::Won);
    }

    #[test]
    fn invaders_reverse_and_drop_at_the_edge() {
        let mut game = SpaceInvadersGame::new();
        game.invaders = vec![(GRID_WIDTH - 2, 4), (GRID_WIDTH - 4, 4)];
        game.invader_direction = InvaderDirection::Right;

        game.step_once();
        game.step_once();
        game.step_once();
        game.step_once();

        assert_eq!(game.invader_direction, InvaderDirection::Left);
        assert_eq!(game.invaders[0].1, 5);
        assert_eq!(game.invaders[1].1, 5);
    }
}
