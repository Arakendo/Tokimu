use std::sync::Arc;

use observation_shell::{ObservationShell, ObservationSource, ShellRecord};
use tokimu::Diagnostics;
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::consumer::UiTextRole;
use ui_tools::{
    layout_bitmap_text, lower_resolved_tree_to_draw_list, UiDrawCommand, UiDrawList, UiSurfaceRole,
    UiTheme,
};

mod ui;

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const BACKDROP: MaterialHandle = MaterialHandle(1);
const PANEL: MaterialHandle = MaterialHandle(2);
const ACCENT: MaterialHandle = MaterialHandle(3);
const TEXT: MaterialHandle = MaterialHandle(4);
const MUTED: MaterialHandle = MaterialHandle(5);

const COMMANDS: &[(&str, &str)] = &[
    ("HELP", "help"),
    ("WORLD", "inspect world"),
    ("ENTITIES", "list entities"),
    ("DIAGNOSTICS", "observe diagnostics"),
    ("WATCH WORLD", "watch world 2"),
    ("LIST WATCHES", "list watches"),
    ("BACK", "back"),
    ("JSON", "format json"),
];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Observation Workbench".into(),
            width: 1200,
            height: 760,
        },
        ObservationWorkbench::new(),
    )
}

struct ObservationWorkbench {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    source: ObservationSource,
    shell: ObservationShell,
    selected: usize,
    watch_sequence: u64,
    last_refresh: String,
}

impl ObservationWorkbench {
    fn new() -> Self {
        let mut world = tokimu::World::default();
        let observer = world.spawn();
        let target = world.spawn();
        world.add_relationship::<Follows>(observer, target);
        let mut diagnostics = Diagnostics::default();
        diagnostics.record("observation workbench fixture initialized");
        let source = ObservationSource::from_world_and_diagnostics(&world, &diagnostics);
        let mut shell = ObservationShell::default();
        let _ = shell.execute(&source, "inspect world");
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            source,
            shell,
            selected: 0,
            watch_sequence: 0,
            last_refresh: "WATCH REFRESH: PENDING".into(),
        }
    }

    fn selected_command(&self) -> &'static str {
        COMMANDS[self.selected].1
    }

    fn execute_selected(&mut self) {
        let command = self.selected_command();
        let _ = self.shell.execute(&self.source, command);
    }

    fn refresh_watches(&mut self) {
        self.watch_sequence = self.watch_sequence.saturating_add(1);
        let refreshes = self
            .shell
            .refresh_watches(&self.source, self.watch_sequence);
        self.last_refresh = if refreshes.is_empty() {
            "WATCH REFRESH: NO WATCHES".into()
        } else {
            format!("WATCH REFRESH: {} RESULT(S)", refreshes.len())
        };
    }

    fn update_title(&self) {
        if let Some(window) = &self.window {
            window.set_title(&format!(
                "Tokimu Observation Workbench | command={} | watches={}",
                self.selected_command(),
                self.shell.watches().len()
            ));
        }
    }

    fn submit_draw_list(&mut self, renderer: &mut WgpuBackend, draw_list: &UiDrawList) {
        let mut commands = Vec::new();
        for entry in draw_list.entries() {
            match &entry.command {
                UiDrawCommand::PushClip(_) | UiDrawCommand::PopClip => {}
                UiDrawCommand::Surface(surface) => {
                    commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: QUAD,
                        material: material_for_surface(surface.style.role),
                        pipeline: self.pipeline,
                        instance: Instance2d::new(surface.rect.center, surface.rect.size, 0.0),
                        camera: Some(CAMERA),
                        viewport: None,
                    }))
                }
                UiDrawCommand::Text(text) => commands.extend(
                    layout_bitmap_text(&text.spec, text.style.height)
                        .into_iter()
                        .map(|quad| {
                            RenderCommand::DrawMesh(DrawMeshCommand {
                                mesh: QUAD,
                                material: if matches!(
                                    text.style.role,
                                    UiTextRole::Title | UiTextRole::Heading
                                ) {
                                    TEXT
                                } else {
                                    MUTED
                                },
                                pipeline: self.pipeline,
                                instance: Instance2d::new(quad.center, quad.size, 0.0),
                                camera: Some(CAMERA),
                                viewport: None,
                            })
                        }),
                ),
            }
        }
        renderer.submit(&commands);
    }
}

fn material_for_surface(role: UiSurfaceRole) -> MaterialHandle {
    match role {
        UiSurfaceRole::Panel | UiSurfaceRole::Raised => PANEL,
        UiSurfaceRole::Accent | UiSurfaceRole::Selected => ACCENT,
        _ => BACKDROP,
    }
}

impl PlatformEventHandler for ObservationWorkbench {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        for (handle, name, color) in [
            (
                BACKDROP,
                "observation-workbench-backdrop",
                Color::rgb(0.04, 0.05, 0.07),
            ),
            (
                PANEL,
                "observation-workbench-panel",
                Color::rgb(0.15, 0.19, 0.24),
            ),
            (
                ACCENT,
                "observation-workbench-accent",
                Color::rgb(0.22, 0.55, 0.72),
            ),
            (
                TEXT,
                "observation-workbench-text",
                Color::rgb(0.90, 0.94, 0.98),
            ),
            (
                MUTED,
                "observation-workbench-muted",
                Color::rgb(0.65, 0.72, 0.80),
            ),
        ] {
            renderer.upload_material(handle, &Material::new(name, color))?;
        }
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-observation-workbench-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::ArrowLeft => self.selected = self.selected.saturating_sub(1),
                KeyCode::ArrowRight => self.selected = (self.selected + 1).min(COMMANDS.len() - 1),
                KeyCode::Enter => self.execute_selected(),
                KeyCode::KeyD => {
                    self.selected = 3;
                    self.execute_selected();
                }
                KeyCode::KeyW => {
                    self.selected = 4;
                    self.execute_selected();
                }
                KeyCode::KeyA => {
                    self.selected = 5;
                    self.execute_selected();
                }
                KeyCode::KeyR => {
                    self.selected = 6;
                    self.execute_selected();
                }
                KeyCode::KeyQ => {
                    self.selected = 7;
                    self.execute_selected();
                }
                KeyCode::Space => self.refresh_watches(),
                KeyCode::Escape => {
                    let _ = self.shell.execute(&self.source, "clear");
                }
                _ => {}
            },
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            _ => {}
        }
        self.update_title();
        Ok(())
    }

    fn on_frame(&mut self, _: f64) -> PlatformResult<FrameOutcome> {
        let catalog_lines = COMMANDS
            .iter()
            .enumerate()
            .map(|(index, (label, command))| {
                format!(
                    "{} {}  {}",
                    if index == self.selected { ">" } else { " " },
                    label,
                    command
                )
            })
            .collect();
        let transcript_lines = self
            .shell
            .history()
            .iter()
            .rev()
            .take(6)
            .rev()
            .flat_map(record_lines)
            .collect();
        let watch_lines = if self.shell.watches().is_empty() {
            vec!["NONE".into(), self.last_refresh.clone()]
        } else {
            self.shell
                .watches()
                .iter()
                .map(|watch| format!("#{} {:?} / {}", watch.id, watch.target, watch.interval))
                .chain(std::iter::once(self.last_refresh.clone()))
                .collect()
        };
        let view = ui::WorkbenchView {
            catalog_lines,
            session_lines: vec![
                format!("FORMAT: {:?}", self.shell.format()),
                format!("CONTEXT: {:?}", self.shell.current_context()),
                format!("NAVIGATION: {}", self.shell.navigation_depth()),
                format!("HISTORY: {} RECORDS", self.shell.history().len()),
            ],
            transcript_lines,
            watch_lines,
            control_lines: vec![
                "LEFT/RIGHT SELECT   ENTER EXECUTE".into(),
                "D DIAGNOSTICS   W WATCH WORLD   A WATCHES".into(),
                "R BACK   Q JSON   SPACE REFRESH   ESC CLEAR".into(),
            ],
        };
        let tree = ui::build_scene(self.window_size, &view);
        let resolved = tree
            .resolve(ui_tools::consumer::UiRect::new(
                [self.window_size[0] * 0.5, self.window_size[1] * 0.5],
                self.window_size,
            ))
            .map_err(|error| {
                std::io::Error::other(format!("workbench layout failed: {error:?}"))
            })?;
        let draw_list = lower_resolved_tree_to_draw_list(
            &resolved,
            &UiTheme::default(),
            self.shell.history().len() as u64,
        );
        let Some(mut renderer) = self.renderer.take() else {
            return Ok(FrameOutcome::Continue);
        };
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.04, 0.05, 0.07),
        })]);
        self.submit_draw_list(&mut renderer, &draw_list);
        let _ = renderer.present()?;
        self.renderer = Some(renderer);
        Ok(FrameOutcome::Continue)
    }
}

fn record_lines(record: &ShellRecord) -> Vec<String> {
    let projection = record.projection.replace('\n', " ");
    vec![format!("> {}", record.input), truncate(&projection, 76)]
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() > limit {
        format!(
            "{}...",
            value
                .chars()
                .take(limit.saturating_sub(3))
                .collect::<String>()
        )
    } else {
        value.to_owned()
    }
}

#[derive(Debug)]
struct Follows;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workbench_commands_match_a_fresh_scripted_shell() {
        let mut workbench = ObservationWorkbench::new();
        workbench.selected = 1;
        workbench.execute_selected();
        let expected = ObservationShell::default().execute(&workbench.source, "inspect world");
        assert_eq!(workbench.shell.history().last(), Some(&expected));
    }

    #[test]
    fn control_catalog_contains_only_literal_shell_commands() {
        assert!(COMMANDS
            .iter()
            .all(|(_, command)| !command.trim().is_empty()));
        assert_eq!(COMMANDS[4].1, "watch world 2");
    }
}
