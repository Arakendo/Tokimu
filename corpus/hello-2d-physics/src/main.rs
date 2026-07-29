use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const BOARD_MESH: MeshHandle = MeshHandle(1);
const PEG_MESH: MeshHandle = MeshHandle(2);
const MARBLE_MESH: MeshHandle = MeshHandle(3);
const BOARD_MATERIAL: MaterialHandle = MaterialHandle(1);
const PEG_MATERIAL: MaterialHandle = MaterialHandle(2);
const MARBLE_MATERIAL: MaterialHandle = MaterialHandle(3);
const SETTLED_MATERIAL: MaterialHandle = MaterialHandle(4);
const HIGHLIGHT_MATERIAL: MaterialHandle = MaterialHandle(5);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BOARD_WIDTH: f32 = 12.0;
const BOARD_HEIGHT: f32 = 16.0;
const SIDE_WALL_X: f32 = 5.8;
const FLOOR_Y: f32 = -6.9;
const PEG_RADIUS: f32 = 0.18;
const MARBLE_RADIUS: f32 = 0.16;
const GRAVITY: f32 = -11.5;
const RESTITUTION: f32 = 0.62;
const SIDE_DAMPING: f32 = 0.78;
const PEG_COUNT_PER_ROW: usize = 6;
const PEG_ROWS: usize = 7;
const DROP_COOLDOWN: f32 = 0.65;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello 2D Physics".into(),
            width: 1280,
            height: 960,
        },
        Hello2dPhysicsApp::new(),
    )
}

struct Hello2dPhysicsApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    camera: Camera,
    pipeline: PipelineHandle,
    simulation: MarbleBoard,
}

impl Default for Hello2dPhysicsApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            camera: Camera::default(),
            pipeline: PipelineHandle(0),
            simulation: MarbleBoard::new(),
        }
    }
}

impl Hello2dPhysicsApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello 2D Physics | score={} | active={} | settled={} | press space to drop",
                self.simulation.score,
                self.simulation.active_marble_count(),
                self.simulation.settled_marble_count(),
            ));
        }
    }

    fn update_camera(&mut self) {
        self.camera = Camera::orthographic_2d_with_height(
            self.window_size[0],
            self.window_size[1],
            BOARD_HEIGHT,
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

        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.03, 0.04, 0.06),
        })];

        commands.push(draw_board_segment(
            [0.0, FLOOR_Y - 0.65],
            [BOARD_WIDTH * 0.70, 0.26],
            BOARD_MATERIAL,
            self.pipeline,
        ));
        commands.push(draw_board_segment(
            [-BOARD_WIDTH * 0.34, 5.8],
            [0.20, 11.8],
            BOARD_MATERIAL,
            self.pipeline,
        ));
        commands.push(draw_board_segment(
            [BOARD_WIDTH * 0.34, 5.8],
            [0.20, 11.8],
            BOARD_MATERIAL,
            self.pipeline,
        ));

        for peg in &self.simulation.pegs {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: PEG_MESH,
                material: PEG_MATERIAL,
                pipeline: self.pipeline,
                instance: peg.instance(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }));
        }

        for bin in &self.simulation.bins {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: BOARD_MESH,
                material: if bin.is_highlighted {
                    HIGHLIGHT_MATERIAL
                } else {
                    BOARD_MATERIAL
                },
                pipeline: self.pipeline,
                instance: bin.instance(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }));
        }

        for marble in &self.simulation.marbles {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MARBLE_MESH,
                material: if marble.settled {
                    SETTLED_MATERIAL
                } else {
                    MARBLE_MATERIAL
                },
                pipeline: self.pipeline,
                instance: marble.instance(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }));
        }

        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for Hello2dPhysicsApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(BOARD_MESH, &Mesh::quad());
        renderer.upload_mesh(PEG_MESH, &Mesh::diamond());
        renderer.upload_mesh(MARBLE_MESH, &Mesh::diamond());
        renderer.upload_material(
            BOARD_MATERIAL,
            &Material::new("physics-board", Color::rgb(0.14, 0.17, 0.22)),
        )?;
        renderer.upload_material(
            PEG_MATERIAL,
            &Material::new("physics-peg", Color::rgb(0.54, 0.62, 0.92)),
        )?;
        renderer.upload_material(
            MARBLE_MATERIAL,
            &Material::new("physics-marble", Color::rgb(0.98, 0.82, 0.32)),
        )?;
        renderer.upload_material(
            SETTLED_MATERIAL,
            &Material::new("physics-marble-settled", Color::rgb(0.76, 0.86, 0.96)),
        )?;
        renderer.upload_material(
            HIGHLIGHT_MATERIAL,
            &Material::new("physics-highlight", Color::rgb(0.95, 0.45, 0.55)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "physics-pipeline",
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
                    KeyCode::Space => self.simulation.drop_marble(),
                    KeyCode::ArrowDown => self.simulation.reset(),
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
        self.simulation.advance(delta_seconds as f32);
        self.update_window_title();
        self.render_scene()
    }
}

fn draw_board_segment(
    center: [f32; 2],
    scale: [f32; 2],
    material: MaterialHandle,
    pipeline: PipelineHandle,
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh: BOARD_MESH,
        material,
        pipeline,
        instance: Instance2d::new(center, scale, 0.0),
        camera: Some(CAMERA_HANDLE),
        viewport: None,
    })
}

#[derive(Clone, Copy, Debug)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }

    fn scale(self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }

    fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(self) -> Vec2 {
        let length = self.length();
        if length > 0.0001 {
            self.scale(1.0 / length)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

#[derive(Clone, Debug)]
struct Marble {
    pos: Vec2,
    vel: Vec2,
    settled: bool,
    bin_index: usize,
}

impl Marble {
    fn instance(&self) -> Instance2d {
        Instance2d::identity()
            .with_translation([self.pos.x, self.pos.y])
            .with_scale([MARBLE_RADIUS * 2.0, MARBLE_RADIUS * 2.0])
            .with_rotation(self.vel.x * 0.12)
    }
}

#[derive(Clone, Debug)]
struct Peg {
    pos: Vec2,
}

impl Peg {
    fn instance(&self) -> Instance2d {
        Instance2d::identity()
            .with_translation([self.pos.x, self.pos.y])
            .with_scale([PEG_RADIUS * 2.0, PEG_RADIUS * 2.0])
            .with_rotation(0.5)
    }
}

#[derive(Clone, Debug)]
struct Bin {
    center_x: f32,
    width: f32,
    is_highlighted: bool,
}

impl Bin {
    fn instance(&self) -> Instance2d {
        Instance2d::identity()
            .with_translation([self.center_x, FLOOR_Y - 0.2])
            .with_scale([self.width, 0.28])
    }
}

struct MarbleBoard {
    marbles: Vec<Marble>,
    pegs: Vec<Peg>,
    bins: Vec<Bin>,
    score: u32,
    elapsed_seconds: f32,
    spawn_cooldown: f32,
    next_spawn_offset: f32,
}

impl MarbleBoard {
    fn new() -> Self {
        let mut board = Self {
            marbles: Vec::new(),
            pegs: build_pegs(),
            bins: build_bins(),
            score: 0,
            elapsed_seconds: 0.0,
            spawn_cooldown: 0.0,
            next_spawn_offset: 0.0,
        };
        board.drop_marble();
        board
    }

    fn reset(&mut self) {
        *self = MarbleBoard::new();
    }

    fn drop_marble(&mut self) {
        if self.spawn_cooldown > 0.0 {
            return;
        }

        let offset = self.next_spawn_offset * 0.32;
        self.next_spawn_offset = (self.next_spawn_offset + 1.0) % 5.0;
        self.spawn_cooldown = DROP_COOLDOWN;
        self.marbles.push(Marble {
            pos: Vec2::new(offset, BOARD_HEIGHT * 0.5 - 1.2),
            vel: Vec2::new(0.0, 0.0),
            settled: false,
            bin_index: 0,
        });
    }

    fn advance(&mut self, dt: f32) {
        self.elapsed_seconds += dt;
        self.spawn_cooldown = (self.spawn_cooldown - dt).max(0.0);

        if self.elapsed_seconds.rem_euclid(2.2) < dt && self.active_marble_count() < 6 {
            self.drop_marble();
        }

        let pegs = self.pegs.clone();
        let bin_count = self.bins.len();

        for marble in &mut self.marbles {
            if marble.settled {
                continue;
            }

            marble.vel.y += GRAVITY * dt;
            marble.pos = marble.pos.add(marble.vel.scale(dt));

            resolve_walls(marble);
            resolve_pegs(marble, &pegs);
            resolve_floor(marble, bin_count);
        }

        self.update_bins();
    }

    fn active_marble_count(&self) -> usize {
        self.marbles.iter().filter(|marble| !marble.settled).count()
    }

    fn settled_marble_count(&self) -> usize {
        self.marbles.iter().filter(|marble| marble.settled).count()
    }

    fn update_bins(&mut self) {
        for (index, bin) in self.bins.iter_mut().enumerate() {
            bin.is_highlighted = self
                .marbles
                .iter()
                .any(|marble| marble.settled && marble.bin_index == index);
        }

        self.score = self
            .marbles
            .iter()
            .filter(|marble| marble.settled)
            .map(|marble| bin_score(self.bins.len(), marble.bin_index))
            .sum();
    }
}

fn resolve_walls(marble: &mut Marble) {
    if marble.pos.x < -SIDE_WALL_X + MARBLE_RADIUS {
        marble.pos.x = -SIDE_WALL_X + MARBLE_RADIUS;
        marble.vel.x = marble.vel.x.abs() * SIDE_DAMPING;
    } else if marble.pos.x > SIDE_WALL_X - MARBLE_RADIUS {
        marble.pos.x = SIDE_WALL_X - MARBLE_RADIUS;
        marble.vel.x = -marble.vel.x.abs() * SIDE_DAMPING;
    }

    if marble.pos.y > BOARD_HEIGHT * 0.5 - MARBLE_RADIUS {
        marble.pos.y = BOARD_HEIGHT * 0.5 - MARBLE_RADIUS;
        marble.vel.y = -marble.vel.y.abs() * RESTITUTION;
    }
}

fn resolve_pegs(marble: &mut Marble, pegs: &[Peg]) {
    for peg in pegs {
        let delta = Vec2::new(marble.pos.x - peg.pos.x, marble.pos.y - peg.pos.y);
        let distance = delta.length();
        let min_distance = MARBLE_RADIUS + PEG_RADIUS;
        if distance > 0.0001 && distance < min_distance {
            let normal = delta.normalize();
            marble.pos = peg.pos.add(normal.scale(min_distance + 0.001));
            let velocity_into_normal = marble.vel.dot(normal);
            if velocity_into_normal < 0.0 {
                let bounce = normal.scale(-(1.0 + RESTITUTION) * velocity_into_normal);
                marble.vel = marble.vel.add(bounce);
                marble.vel.x *= 0.99;
            }
        }
    }
}

fn resolve_floor(marble: &mut Marble, bin_count: usize) {
    if marble.pos.y <= FLOOR_Y + MARBLE_RADIUS {
        marble.pos.y = FLOOR_Y + MARBLE_RADIUS;
        marble.vel.y = 0.0;
        marble.vel.x *= 0.94;

        if marble.vel.length() < 0.25 {
            marble.settled = true;
            marble.bin_index = bin_for_x(marble.pos.x, bin_count);
        }
    }
}

fn bin_for_x(x: f32, bin_count: usize) -> usize {
    let normalized = ((x + SIDE_WALL_X) / (SIDE_WALL_X * 2.0)).clamp(0.0, 0.9999);
    let index = (normalized * bin_count as f32).floor() as usize;
    index.min(bin_count - 1)
}

fn build_pegs() -> Vec<Peg> {
    let mut pegs = Vec::new();
    let row_spacing = 1.18;
    let column_spacing = 1.72;
    let start_y = BOARD_HEIGHT * 0.5 - 2.0;

    for row in 0..PEG_ROWS {
        let y = start_y - row as f32 * row_spacing;
        let offset = if row % 2 == 0 {
            0.0
        } else {
            column_spacing * 0.5
        };
        for column in 0..PEG_COUNT_PER_ROW {
            let x = -((PEG_COUNT_PER_ROW - 1) as f32 * column_spacing * 0.5)
                + column as f32 * column_spacing
                + offset;
            pegs.push(Peg {
                pos: Vec2::new(x, y),
            });
        }
    }

    pegs
}

fn build_bins() -> Vec<Bin> {
    let bin_count = 6;
    let width = (SIDE_WALL_X * 2.0) / bin_count as f32;
    (0..bin_count)
        .map(|index| {
            let center_x = -SIDE_WALL_X + width * 0.5 + index as f32 * width;
            Bin {
                center_x,
                width: width * 0.92,
                is_highlighted: false,
            }
        })
        .collect()
}

fn bin_score(bin_count: usize, index: usize) -> u32 {
    let center = (bin_count / 2) as i32;
    let distance = (index as i32 - center).abs() as u32;
    10 + (center as u32 - distance) * 10
}
