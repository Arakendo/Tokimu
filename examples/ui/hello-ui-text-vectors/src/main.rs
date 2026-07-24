use std::{sync::Arc, time::Instant};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{UiFontFormat, UiFontRasterizer, UiFontSource};

const CAMERA: CameraHandle = CameraHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const GRID_MATERIAL: MaterialHandle = MaterialHandle(2);
const GRID_MESH: MeshHandle = MeshHandle(2);
const COLUMNS: usize = 16;
const ROWS: usize = 6;
const GRID_ORIGIN: [f32; 2] = [-0.82, 0.72];
const CELL_STEP: [f32; 2] = [0.11, 0.22];
const CELL_SIZE: [f32; 2] = [0.10, 0.19];
const FONT_PIXELS: f32 = 52.0;
const SELECTED_SCALE: f32 = 5.0;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Text Vectors | printable glyph grid".into(),
            width: 900,
            height: 600,
        },
        App::default(),
    )
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    glyphs: Vec<Option<(MeshHandle, char)>>,
    cursor_position: [f32; 2],
    selected: Option<usize>,
}

impl App {
    fn update_title(&self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let title = match self.selected.and_then(|index| self.glyphs.get(index)) {
            Some(Some((_, character))) => format!(
                "Tokimu Hello UI Text Vectors | U+{:04X} {:?} | click canvas to return",
                *character as u32, character
            ),
            _ => "Tokimu Hello UI Text Vectors | printable glyph grid".to_owned(),
        };
        window.set_title(&title);
    }

    fn cell_center(index: usize) -> [f32; 2] {
        let row = index / COLUMNS;
        let column = index % COLUMNS;
        [
            GRID_ORIGIN[0] + column as f32 * CELL_STEP[0],
            GRID_ORIGIN[1] - row as f32 * CELL_STEP[1],
        ]
    }

    fn printable_character(index: usize) -> Option<char> {
        let codepoint = 0x20 + index;
        (codepoint <= 0x7e).then(|| char::from_u32(codepoint as u32).unwrap())
    }
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        self.window = Some(window.clone());
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        let source = UiFontSource::from_prepared_corpus("inter", UiFontFormat::Ttf)
            .map_err(|error| error.to_string())?;
        let font = UiFontRasterizer::from_bytes(source.bytes).map_err(|error| error.to_string())?;
        self.pipeline = renderer
            .register_pipeline(&Pipeline::new("text-vectors", PipelineKind::SolidColor2d))?;
        renderer.upload_material(
            MATERIAL,
            &Material::new("text-vectors-glyph", Color::rgb(0.72, 0.84, 0.96)),
        )?;
        renderer.upload_material(
            GRID_MATERIAL,
            &Material::new("text-vectors-grid", Color::rgb(0.11, 0.14, 0.18)),
        )?;
        renderer.upload_mesh(
            GRID_MESH,
            &Mesh::uniform_normal(cell_border_positions(), [0.0, 0.0, 1.0]),
        );

        let pixel_scale = 1.0 / self.size[1];
        let build_start = Instant::now();
        self.glyphs.clear();
        let mut triangles = 0usize;
        for index in 0..(COLUMNS * ROWS) {
            let Some(character) = Self::printable_character(index) else {
                self.glyphs.push(None);
                continue;
            };
            // Space has advance metrics but no outline; it remains an empty cell.
            if character.is_whitespace() {
                self.glyphs.push(None);
                continue;
            }
            let layout = font.layout(&character.to_string(), FONT_PIXELS);
            let Some(positioned) = layout.glyphs.first() else {
                self.glyphs.push(None);
                continue;
            };
            let mesh_positions = font
                .tessellate_positioned_glyph(
                    positioned,
                    FONT_PIXELS,
                    pixel_scale,
                    [0.0, 0.0],
                    pixel_scale * 0.18,
                )
                .map_err(|error| format!("glyph {character:?} failed: {}", error.message))?;
            if mesh_positions.is_empty() {
                self.glyphs.push(None);
                continue;
            }
            triangles += mesh_positions.len() / 3;
            let handle = MeshHandle(100 + index as u64);
            let positions = center_glyph_in_cell(mesh_positions)
                .into_iter()
                .map(|[x, y]| [x, y, 0.0])
                .collect();
            renderer.upload_mesh(handle, &Mesh::uniform_normal(positions, [0.0, 0.0, 1.0]));
            self.glyphs.push(Some((handle, character)));
        }
        println!(
            "hello-ui-text-vectors: printable glyphs={}, triangles={}, build_ms={:.3}",
            self.glyphs.iter().filter(|glyph| glyph.is_some()).count(),
            triangles,
            build_start.elapsed().as_secs_f64() * 1000.0
        );
        self.renderer = Some(renderer);
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::CursorMoved { x, y } => self.cursor_position = [x, y],
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
            } => {
                if self.selected.is_some() {
                    self.selected = None;
                } else {
                    let [world_x, world_y] =
                        ui_tools::window_to_world(self.size, self.cursor_position);
                    let column = ((world_x - GRID_ORIGIN[0] + CELL_STEP[0] * 0.5) / CELL_STEP[0])
                        .floor() as i32;
                    let row = ((GRID_ORIGIN[1] - world_y + CELL_STEP[1] * 0.5) / CELL_STEP[1])
                        .floor() as i32;
                    self.selected = if (0..COLUMNS as i32).contains(&column)
                        && (0..ROWS as i32).contains(&row)
                    {
                        Some(row as usize * COLUMNS + column as usize)
                    } else {
                        None
                    };
                }
                self.update_title();
            }
            PlatformInputEvent::Resized { width, height } => {
                self.size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.06, 0.08),
        })]);
        if self.selected.is_none() {
            let grid_commands = (0..COLUMNS * ROWS)
                .map(|index| {
                    RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: GRID_MESH,
                        material: GRID_MATERIAL,
                        pipeline: self.pipeline,
                        instance: Instance2d::new(Self::cell_center(index), [1.0, 1.0], 0.0),
                        camera: Some(CAMERA),
                        viewport: None,
                    })
                })
                .collect::<Vec<_>>();
            renderer.submit(&grid_commands);
        }
        let commands = self
            .glyphs
            .iter()
            .enumerate()
            .filter_map(|(index, glyph)| {
                let (mesh, _) = glyph.as_ref()?;
                if self.selected.is_some() && self.selected != Some(index) {
                    return None;
                }
                let selected = self.selected == Some(index);
                Some(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: *mesh,
                    material: MATERIAL,
                    pipeline: self.pipeline,
                    instance: Instance2d::new(
                        if selected {
                            [0.0, 0.0]
                        } else {
                            Self::cell_center(index)
                        },
                        if selected {
                            [SELECTED_SCALE, SELECTED_SCALE]
                        } else {
                            [1.0, 1.0]
                        },
                        0.0,
                    ),
                    camera: Some(CAMERA),
                    viewport: None,
                }))
            })
            .collect::<Vec<_>>();
        renderer.submit(&commands);
        renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

fn center_glyph_in_cell(mut positions: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    let bounds = positions.iter().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |bounds, point| {
            [
                bounds[0].min(point[0]),
                bounds[1].min(point[1]),
                bounds[2].max(point[0]),
                bounds[3].max(point[1]),
            ]
        },
    );
    let center = [(bounds[0] + bounds[2]) * 0.5, (bounds[1] + bounds[3]) * 0.5];
    for point in &mut positions {
        point[0] -= center[0];
        point[1] -= center[1];
    }
    positions
}

fn cell_border_positions() -> Vec<[f32; 3]> {
    let half = [CELL_SIZE[0] * 0.5, CELL_SIZE[1] * 0.5];
    let thickness = 0.0012;
    let mut positions = Vec::with_capacity(24);
    append_quad(
        &mut positions,
        [-half[0], half[1] - thickness],
        [half[0], half[1]],
    );
    append_quad(
        &mut positions,
        [-half[0], -half[1]],
        [half[0], -half[1] + thickness],
    );
    append_quad(
        &mut positions,
        [-half[0], -half[1]],
        [-half[0] + thickness, half[1]],
    );
    append_quad(
        &mut positions,
        [half[0] - thickness, -half[1]],
        [half[0], half[1]],
    );
    positions
}

fn append_quad(positions: &mut Vec<[f32; 3]>, min: [f32; 2], max: [f32; 2]) {
    positions.extend_from_slice(&[
        [min[0], min[1], 0.0],
        [max[0], min[1], 0.0],
        [max[0], max[1], 0.0],
        [min[0], min[1], 0.0],
        [max[0], max[1], 0.0],
        [min[0], max[1], 0.0],
    ]);
}
