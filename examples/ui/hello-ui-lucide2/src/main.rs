use std::{fs, path::PathBuf, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    MouseButton, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{flatten_path, parse_path, stroke_paths as tessellate_stroke_paths, window_to_world};

const CAMERA: CameraHandle = CameraHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const LUCIDE_STROKE_HALF_WIDTH: f32 = 1.0 / 32.0;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig { title: "Tokimu Hello UI Lucide 2".into(), width: 900, height: 600 },
        App::default(),
    )
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    meshes: Vec<Option<(MeshHandle, [f32; 2], [f32; 2], f32)>>,
    icon_names: Vec<String>,
    cursor_position: [f32; 2],
    selected: Option<usize>,
}

impl App {
    fn update_window_title(&self) {
        let Some(window) = self.window.as_ref() else { return; };
        let title = self.selected
            .and_then(|index| self.icon_names.get(index))
            .map(|name| format!("Tokimu Hello UI Lucide 2 | {name} | click canvas to return"))
            .unwrap_or_else(|| "Tokimu Hello UI Lucide 2 | 100-icon grid".to_owned());
        window.set_title(&title);
    }
}

fn fan(points: &[[f32; 2]]) -> Mesh {
    let center = points.iter().fold([0.0, 0.0], |sum, point| {
        [sum[0] + point[0] / points.len() as f32, sum[1] + point[1] / points.len() as f32]
    });
    let mut positions = Vec::with_capacity(points.len() * 3);
    for index in 0..points.len() {
        let next = points[(index + 1) % points.len()];
        positions.extend([
            [center[0], center[1], 0.0],
            [points[index][0], points[index][1], 0.0],
            [next[0], next[1], 0.0],
        ]);
    }
    Mesh::uniform_normal(positions, [0.0, 0.0, 1.0])
}

fn star() -> Vec<[f32; 2]> {
    (0..10)
        .map(|index| {
            let angle = -std::f32::consts::FRAC_PI_2 + index as f32 * std::f32::consts::PI / 5.0;
            let radius = if index % 2 == 0 { 0.34 } else { 0.15 };
            [angle.cos() * radius, angle.sin() * radius]
        })
        .collect()
}

fn heart() -> Vec<[f32; 2]> {
    (0..32)
        .map(|index| {
            let t = index as f32 * std::f32::consts::TAU / 32.0;
            let x = 0.24 * (16.0 * t.sin().powi(3)) / 16.0;
            let y = 0.24 * (13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos()) / 16.0;
            [x, y]
        })
        .collect()
}

fn activity() -> Vec<[f32; 2]> {
    vec![[-0.38, 0.0], [-0.22, 0.0], [-0.12, 0.22], [0.0, -0.22], [0.12, 0.18], [0.22, 0.0], [0.38, 0.0]]
}

fn provider_path(name: &str) -> Result<String, String> {
    let relative = format!("target/glyph-corpus/icons/icons/{name}.svg");
    let mut candidates = vec![PathBuf::from(&relative)];
    if let Ok(dir) = std::env::current_dir() {
        candidates.extend(dir.ancestors().map(|path| path.join(&relative)));
    }
    let path = candidates.into_iter().find(|path| path.is_file()).ok_or_else(|| "run prepare-glyph-corpus.ps1 first".to_string())?;
    let svg = fs::read_to_string(path).map_err(|error| error.to_string())?;
    svg.split("d=\"").nth(1).and_then(|value| value.split('\"').next()).map(str::to_owned).ok_or_else(|| format!("{name}.svg has no path data"))
}

fn provider_activity() -> Result<Vec<[f32; 2]>, String> {
    let data = provider_path("activity")?;
    let commands = parse_path(&data)?;
    let points = flatten_path(&commands, 16).into_iter().next().ok_or_else(|| "activity path was empty".to_string())?;
    Ok(points.into_iter().map(|point| [(point[0] - 12.0) / 32.0, (12.0 - point[1]) / 32.0]).collect())
}

fn sample_manifest() -> Result<Vec<String>, String> {
    let relative = "target/lucide-corpus-100/manifest.txt";
    let mut candidates = vec![PathBuf::from(relative)];
    if let Ok(dir) = std::env::current_dir() { candidates.extend(dir.ancestors().map(|path| path.join(relative))); }
    let path = candidates.into_iter().find(|path| path.is_file()).ok_or_else(|| "run prepare-lucide-sample.ps1 first".to_string())?;
    Ok(fs::read_to_string(path).map_err(|error| error.to_string())?.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_owned).collect())
}

fn sample_svg_paths(name: &str) -> Result<Vec<Vec<[f32; 2]>>, String> {
    let relative = format!("target/lucide-corpus-100/{name}");
    let mut candidates = vec![PathBuf::from(&relative)];
    if let Ok(dir) = std::env::current_dir() { candidates.extend(dir.ancestors().map(|path| path.join(&relative))); }
    let path = candidates.into_iter().find(|path| path.is_file()).ok_or_else(|| format!("missing sampled icon {name}"))?;
    let svg = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut paths = Vec::new();
    for data in svg.split("d=\"").skip(1).filter_map(|value| value.split('\"').next()) {
        let commands = parse_path(data)?;
        for points in flatten_path(&commands, 32) {
            if points.len() > 1 {
                paths.push(points.into_iter().map(|point| [(point[0] - 12.0) / 32.0, (12.0 - point[1]) / 32.0]).collect());
            }
        }
    }
    for element in ["circle", "rect", "line", "polyline", "polygon"] {
        let mut rest = svg.as_str();
        while let Some(start) = rest.find(&format!("<{element}")) {
            rest = &rest[start..];
            let Some(end) = rest.find('>') else { break };
            let tag = &rest[..=end];
            if element == "circle" {
                if let (Some(cx), Some(cy), Some(radius)) = (attribute(tag, "cx"), attribute(tag, "cy"), attribute(tag, "r")) {
                    paths.push((0..=32).map(|index| {
                        let angle = index as f32 * std::f32::consts::TAU / 32.0;
                        [((cx + radius * angle.cos()) - 12.0) / 32.0, (12.0 - (cy + radius * angle.sin())) / 32.0]
                    }).collect());
                }
            } else if element == "line" {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (attribute(tag, "x1"), attribute(tag, "y1"), attribute(tag, "x2"), attribute(tag, "y2")) {
                    paths.push(vec![[(x1 - 12.0) / 32.0, (12.0 - y1) / 32.0], [(x2 - 12.0) / 32.0, (12.0 - y2) / 32.0]]);
                }
            } else if matches!(element, "polyline" | "polygon") {
                if let Some(points) = attribute_text(tag, "points") {
                    let values = points.split(|character: char| character == ',' || character.is_ascii_whitespace()).filter(|value| !value.is_empty()).filter_map(|value| value.parse::<f32>().ok()).collect::<Vec<_>>();
                    let mut polygon = values.chunks_exact(2).map(|pair| [(pair[0] - 12.0) / 32.0, (12.0 - pair[1]) / 32.0]).collect::<Vec<_>>();
                    if element == "polygon" && polygon.first() != polygon.last() {
                        if let Some(first) = polygon.first().copied() { polygon.push(first); }
                    }
                    if polygon.len() > 1 { paths.push(polygon); }
                }
            } else if let (Some(x), Some(y), Some(width), Some(height)) = (attribute(tag, "x"), attribute(tag, "y"), attribute(tag, "width"), attribute(tag, "height")) {
                let radius_x = attribute(tag, "rx").unwrap_or(0.0).min(width * 0.5);
                let radius_y = attribute(tag, "ry").unwrap_or(radius_x).min(height * 0.5);
                let rectangle = rounded_rectangle(x, y, width, height, radius_x, radius_y);
                paths.push(rectangle.iter().copied().chain(rectangle.first().copied()).collect());
            }
            rest = &rest[end + 1..];
        }
    }
    Ok(paths)
}

fn attribute(tag: &str, name: &str) -> Option<f32> {
    let prefix = format!("{name}=\"");
    let start = tag.find(&prefix)? + prefix.len();
    let end = tag[start..].find('"')? + start;
    tag[start..end].parse().ok()
}

fn attribute_text(tag: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=\"");
    let start = tag.find(&prefix)? + prefix.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_owned())
}

fn rounded_rectangle(x: f32, y: f32, width: f32, height: f32, radius_x: f32, radius_y: f32) -> Vec<[f32; 2]> {
    if radius_x <= f32::EPSILON || radius_y <= f32::EPSILON {
        return vec![
            [(x - 12.0) / 32.0, (12.0 - y) / 32.0],
            [(x + width - 12.0) / 32.0, (12.0 - y) / 32.0],
            [(x + width - 12.0) / 32.0, (12.0 - y - height) / 32.0],
            [(x - 12.0) / 32.0, (12.0 - y - height) / 32.0],
        ];
    }
    let mut points = Vec::with_capacity(20);
    // Walk the perimeter continuously: top-left, top-right, bottom-right,
    // bottom-left. The straight edges are formed by the joins between arcs.
    for (center_x, center_y, start) in [
        (x + radius_x, y + radius_y, std::f32::consts::PI),
        (x + width - radius_x, y + radius_y, -std::f32::consts::FRAC_PI_2),
        (x + width - radius_x, y + height - radius_y, 0.0_f32),
        (x + radius_x, y + height - radius_y, std::f32::consts::FRAC_PI_2),
    ] {
        for step in 0..=4 {
            let angle = start + step as f32 * std::f32::consts::FRAC_PI_2 / 4.0;
            points.push([
                (center_x + radius_x * angle.cos() - 12.0) / 32.0,
                (12.0 - (center_y + radius_y * angle.sin())) / 32.0,
            ]);
        }
    }
    points
}

fn circle() -> Vec<[f32; 2]> {
    (0..33)
        .map(|index| {
            let angle = index as f32 * std::f32::consts::TAU / 32.0;
            [angle.cos() * 0.25, angle.sin() * 0.25]
        })
        .collect()
}

fn diamond() -> Vec<[f32; 2]> {
    vec![[0.0, 0.34], [0.26, 0.0], [0.0, -0.34], [-0.26, 0.0]]
}

fn zap() -> Vec<[f32; 2]> {
    vec![
        [-0.22, 0.34], [0.02, 0.08], [-0.08, 0.08], [0.22, -0.34],
        [-0.02, -0.03], [0.08, -0.03],
    ]
}

fn triangle() -> Vec<[f32; 2]> {
    vec![[0.0, 0.34], [0.30, -0.25], [-0.30, -0.25]]
}

fn square() -> Vec<[f32; 2]> {
    vec![[-0.27, -0.27], [0.27, -0.27], [0.27, 0.27], [-0.27, 0.27]]
}

fn hexagon() -> Vec<[f32; 2]> {
    (0..6)
        .map(|index| {
            let angle = index as f32 * std::f32::consts::TAU / 6.0;
            [angle.cos() * 0.30, angle.sin() * 0.30]
        })
        .collect()
}

fn arrow() -> Vec<[f32; 2]> {
    vec![
        [-0.34, -0.07], [0.08, -0.07], [0.08, -0.16],
        [0.34, 0.0], [0.08, 0.16], [0.08, 0.07], [-0.34, 0.07],
    ]
}

fn stroke(points: &[[f32; 2]], width: f32) -> Mesh {
    Mesh::uniform_normal(tessellate_stroke_paths(&[points.to_vec()], width), [0.0, 0.0, 1.0])
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        self.window = Some(window.clone());
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(MATERIAL, &Material::new("lucide2", Color::rgb(0.45, 0.68, 0.92)))?;
        for (index, (points, center)) in [
            (star(), [-0.50, 0.25]),
            (heart(), [0.50, 0.25]),
            (activity(), [-0.50, -0.30]),
            (diamond(), [0.50, -0.30]),
        ]
        .into_iter()
        .enumerate()
        {
            let handle = MeshHandle(10 + index as u64);
            let points = if index == 2 { provider_activity().map_err(|error| error.to_string())? } else { points };
            let mesh = if index == 2 { stroke(&points, LUCIDE_STROKE_HALF_WIDTH) } else { fan(&points) };
            renderer.upload_mesh(handle, &mesh);
            self.meshes.push(Some((handle, center, [0.8, 0.8], 0.0)));
        }
        let circle_handle = MeshHandle(20);
        renderer.upload_mesh(circle_handle, &stroke(&circle(), 0.025));
        self.meshes.push(Some((circle_handle, [-0.50, -0.82], [0.8, 0.8], 0.0)));
        let zap_handle = MeshHandle(21);
        renderer.upload_mesh(zap_handle, &fan(&zap()));
        self.meshes.push(Some((zap_handle, [0.50, -0.82], [0.8, 0.8], 0.0)));
        for (handle, points) in [
            (MeshHandle(22), triangle()),
            (MeshHandle(23), square()),
            (MeshHandle(24), hexagon()),
            (MeshHandle(25), arrow()),
        ] {
            renderer.upload_mesh(handle, &fan(&points));
        }

        let base_meshes = [
            MeshHandle(10), MeshHandle(11), MeshHandle(12),
            MeshHandle(13), MeshHandle(20), MeshHandle(21),
            MeshHandle(22), MeshHandle(23), MeshHandle(24), MeshHandle(25),
        ];
        self.meshes.clear();
        for row in 0..10 {
            for column in 0..10 {
                let seed = (row * 37 + column * 61 + row * column * 11) % 101;
                let mesh = base_meshes[seed % base_meshes.len()];
                let x = -0.86 + column as f32 * 0.19;
                let y = 0.84 - row as f32 * 0.19;
                self.meshes.push(Some((mesh, [x, y], [0.24, 0.24], 0.0)));
            }
        }
        self.meshes.clear();
        let sampled = sample_manifest().map_err(|error| error.to_string())?.into_iter().take(100).collect::<Vec<_>>();
        self.icon_names = sampled.clone();
        for (index, name) in sampled.into_iter().take(100).enumerate() {
            let paths = sample_svg_paths(&name).map_err(|error| error.to_string())?;
            let row = index / 10;
            let column = index % 10;
            let cell = [-0.86 + column as f32 * 0.19, 0.84 - row as f32 * 0.19];
            if paths.is_empty() {
                self.meshes.push(None);
                continue;
            }
            let handle = MeshHandle(1000 + index as u64);
            renderer.upload_mesh(handle, &Mesh::uniform_normal(tessellate_stroke_paths(&paths, LUCIDE_STROKE_HALF_WIDTH), [0.0, 0.0, 1.0]));
            self.meshes.push(Some((handle, cell, [0.24, 0.24], 0.0)));
        }
        self.pipeline = renderer.register_pipeline(&Pipeline::new("lucide2", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.cursor_position = [x, y];
        }
        if let PlatformInputEvent::MouseInput { button: MouseButton::Left, pressed: true } = event {
            if self.selected.is_some() {
                self.selected = None;
                self.update_window_title();
                return Ok(());
            }
            let [world_x, world_y] = window_to_world(self.size, self.cursor_position);
            let column = ((world_x + 0.86 + 0.095) / 0.19).floor() as i32;
            let row = ((0.84 - world_y + 0.095) / 0.19).floor() as i32;
            self.selected = if (0..10).contains(&column) && (0..10).contains(&row)
                && world_x >= -0.86 + column as f32 * 0.19 - 0.095
                && world_x <= -0.86 + column as f32 * 0.19 + 0.095
                && world_y <= 0.84 - row as f32 * 0.19 + 0.095
                && world_y >= 0.84 - row as f32 * 0.19 - 0.095
            {
                Some(row as usize * 10 + column as usize)
            } else {
                None
            };
            self.update_window_title();
        }
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() { renderer.resize_surface(width, height); }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else { return Ok(FrameOutcome::Continue); };
        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: Color::rgb(0.05, 0.06, 0.08) })]);
        let commands = self.meshes.iter().enumerate().filter_map(|(index, cell)| {
            if let Some(selected) = self.selected {
                if selected != index { return None; }
            }
            let (mesh, center, scale, rotation) = cell.as_ref()?;
            let draw_scale = if self.selected == Some(index) { [0.8, 0.8] } else { *scale };
            Some(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: *mesh, material: MATERIAL, pipeline: self.pipeline,
            instance: Instance2d::new(if self.selected == Some(index) { [0.0, 0.0] } else { *center }, draw_scale, *rotation), camera: Some(CAMERA), viewport: None,
            }))
        }).collect::<Vec<_>>();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
