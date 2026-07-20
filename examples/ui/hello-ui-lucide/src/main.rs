use std::{fs, path::PathBuf, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const CAMERA: CameraHandle = CameraHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const ICONS: [&str; 5] = ["minus", "plus", "x", "check", "arrow-right"];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Lucide".into(),
            width: 900,
            height: 600,
        },
        App::default(),
    )
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    strokes: Vec<(MeshHandle, [f32; 2])>,
}

fn icon_path(name: &str) -> Result<String, String> {
    let relative = format!("target/glyph-corpus/icons/icons/{name}.svg");
    let mut paths = vec![PathBuf::from(&relative)];
    if let Ok(dir) = std::env::current_dir() {
        paths.extend(dir.ancestors().map(|path| path.join(&relative)));
    }
    if let Ok(exe) = std::env::current_exe() {
        paths.extend(exe.ancestors().map(|path| path.join(&relative)));
    }
    let path = paths
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| "run prepare-glyph-corpus.ps1 first".to_string())?;
    let svg = fs::read_to_string(path).map_err(|error| error.to_string())?;
    Ok(svg
        .split("d=\"")
        .skip(1)
        .filter_map(|value| value.split('\"').next())
        .collect::<Vec<_>>()
        .join(";"))
}

fn path_segments(path: &str) -> Vec<([f32; 2], [f32; 2])> {
    let mut all_segments = Vec::new();
    for subpath in path.split(';') {
        all_segments.extend(path_segments_single(subpath));
    }
    all_segments
}

fn path_segments_single(path: &str) -> Vec<([f32; 2], [f32; 2])> {
    let mut tokens = Vec::new();
    let mut number = String::new();
    for character in path.chars() {
        if character.is_ascii_alphabetic() {
            if !number.is_empty() {
                tokens.push(number.parse::<f32>().ok());
                number.clear();
            }
            tokens.push(None);
            tokens.push(Some(character as u32 as f32));
        } else if character.is_ascii_digit() || matches!(character, '.' | 'e' | 'E') {
            number.push(character);
        } else if matches!(character, '-' | '+') {
            if !number.is_empty() {
                tokens.push(number.parse::<f32>().ok());
                number.clear();
            }
            number.push(character);
        } else if !number.is_empty() {
            tokens.push(number.parse::<f32>().ok());
            number.clear();
        }
    }
    if !number.is_empty() {
        tokens.push(number.parse::<f32>().ok());
    }

    let mut result = Vec::new();
    let mut command = 'M';
    let mut current = [0.0, 0.0];
    let mut index = 0;
    while index < tokens.len() {
        while index < tokens.len() && tokens[index].is_none() {
            index += 1;
        }
        if index >= tokens.len() {
            break;
        }
        if let Some(value) = tokens[index] {
            if value > 0.0 && value < 128.0 && (value as u8 as char).is_ascii_alphabetic() {
                command = value as u8 as char;
                index += 1;
                continue;
            }
        }
        let relative = command.is_ascii_lowercase();
        let normalized = command.to_ascii_uppercase();
        let read = |tokens: &[Option<f32>], index: &mut usize| -> Option<f32> {
            while *index < tokens.len() && tokens[*index].is_none() {
                *index += 1;
            }
            let value = tokens.get(*index).copied().flatten()?;
            *index += 1;
            Some(value)
        };
        let Some(x) = read(&tokens, &mut index) else { break };
        let y = if matches!(normalized, 'H' | 'V') {
            x
        } else {
            let Some(y) = read(&tokens, &mut index) else { break };
            y
        };
        let next = if normalized == 'H' {
            [if relative { current[0] + x } else { x }, current[1]]
        } else if normalized == 'V' {
            [current[0], if relative { current[1] + x } else { x }]
        } else {
            [if relative { current[0] + x } else { x }, if relative { current[1] + y } else { y }]
        };
        if normalized == 'M' {
            current = next;
            command = if relative { 'l' } else { 'L' };
        } else if matches!(normalized, 'L' | 'H' | 'V') {
            result.push((current, next));
            current = next;
        } else {
            break;
        }
    }
    result
}

fn capsule_mesh(start: [f32; 2], end: [f32; 2], radius: f32) -> Mesh {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    let direction = [dx / length, dy / length];
    let normal = [-direction[1], direction[0]];
    let segments = 12;
    let mut perimeter = Vec::with_capacity((segments + 1) * 2);
    for index in 0..=segments {
        let angle = std::f32::consts::FRAC_PI_2
            - index as f32 * std::f32::consts::PI / segments as f32;
        perimeter.push([
            end[0] + radius * (direction[0] * angle.cos() + normal[0] * angle.sin()),
            end[1] + radius * (direction[1] * angle.cos() + normal[1] * angle.sin()),
            0.0,
        ]);
    }
    for index in 0..=segments {
        let angle = 3.0 * std::f32::consts::FRAC_PI_2
            - index as f32 * std::f32::consts::PI / segments as f32;
        perimeter.push([
            start[0] + radius * (direction[0] * angle.cos() + normal[0] * angle.sin()),
            start[1] + radius * (direction[1] * angle.cos() + normal[1] * angle.sin()),
            0.0,
        ]);
    }
    let center = [(start[0] + end[0]) * 0.5, (start[1] + end[1]) * 0.5, 0.0];
    let mut positions = Vec::with_capacity(perimeter.len() * 3);
    for index in 0..perimeter.len() {
        positions.extend([center, perimeter[index], perimeter[(index + 1) % perimeter.len()]]);
    }
    Mesh::uniform_normal(positions, [0.0, 0.0, 1.0])
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(MATERIAL, &Material::new("lucide", Color::rgb(0.45, 0.68, 0.92)))?;

        for (icon_index, name) in ICONS.iter().enumerate() {
            let paths = path_segments(&icon_path(name)?);
            let to_world = |point: [f32; 2]| {
                [
                    (point[0] - 12.0) / 40.0,
                    (12.0 - point[1]) / 40.0,
                ]
            };
            let center_x = (icon_index as f32 - 2.0) * 0.62;
            for (segment_index, (start, end)) in paths.into_iter().enumerate() {
                let a = to_world(start);
                let b = to_world(end);
                let handle = MeshHandle(10 + (icon_index * 8 + segment_index) as u64);
                renderer.upload_mesh(
                    handle,
                    &capsule_mesh([a[0] + center_x, a[1]], [b[0] + center_x, b[1]], 0.09),
                );
                self.strokes.push((handle, [center_x, 0.0]));
            }
        }
        self.pipeline = renderer.register_pipeline(&Pipeline::new("lucide", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
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
        let commands = self
            .strokes
            .iter()
            .map(|(mesh, _)| RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: *mesh,
                material: MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::new([0.0, 0.0], [0.55, 0.55], 0.0),
                camera: Some(CAMERA),
                viewport: None,
            }))
            .collect::<Vec<_>>();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
