use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, RenderFrameStats, Renderer, Texture, TextureHandle, WgpuBackend, WindowConfig,
};
use tokimu_input::{InputState, KeyCode};
use ui_tools::{alpha_to_rgba8, UiFontRasterizer, UiFontSource, UiRect, UiTextInputOperation};

use tokimu_console_command_window::{
    native_interaction::ConsoleInteractionState, tosumu_session::TosumuSession,
};

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const BACKDROP: MaterialHandle = MaterialHandle(1);
const CONSOLE: MaterialHandle = MaterialHandle(2);
const PROMPT: MaterialHandle = MaterialHandle(3);
const HEADER: MaterialHandle = MaterialHandle(4);
const TEXT_TINT: [u8; 3] = [190, 235, 222];
const MUTED_TINT: [u8; 3] = [129, 163, 157];
const ACCENT_TINT: [u8; 3] = [115, 230, 198];
const MAX_TRANSCRIPT_LINES: usize = 15;
const TRANSCRIPT_CAPACITY: usize = 8;
const TRANSCRIPT_START_Y: f32 = 0.38;
const TRANSCRIPT_LINE_HEIGHT: f32 = 0.115;
const TEXT_RESOURCE_BASE: u64 = 100;
const HEADER_TITLE_SLOT: u64 = 0;
const HEADER_METADATA_SLOT: u64 = 1;
const TRANSCRIPT_SLOT_BASE: u64 = 2;
const SCROLL_STATUS_SLOT: u64 = TRANSCRIPT_SLOT_BASE + TRANSCRIPT_CAPACITY as u64;
const PROMPT_SLOT: u64 = SCROLL_STATUS_SLOT + 1;
const PROMPT_HELP_SLOT: u64 = PROMPT_SLOT + 1;
const WARM_FRAME_INTERVAL: u64 = 120;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Console Command Window | Departure Mono".into(),
            width: 1180,
            height: 760,
        },
        ConsoleCorpus::default(),
    )
}

struct TextDraw {
    material: MaterialHandle,
    center: [f32; 2],
    size: [f32; 2],
}

struct ConsoleCorpus {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    input_events: InputState,
    interaction: ConsoleInteractionState,
    tosumu: Option<TosumuSession>,
    font: Option<UiFontRasterizer>,
    text_draws: Vec<TextDraw>,
    frame_index: u64,
    warm_frames_without_text_change: u64,
    needs_text_rebuild: bool,
}

impl Default for ConsoleCorpus {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            input_events: InputState::default(),
            interaction: ConsoleInteractionState::new(
                vec![
                "[system] console fixture initialized".into(),
                "[font] Departure Mono / OTF provider resolved".into(),
                "[boundary] local window controls only until Tosumu fixture initialization".into(),
                "[hint] type HELP, STATUS, CHECK, DESCRIBE demo/message, or CLEAR then press ENTER"
                    .into(),
                ],
                MAX_TRANSCRIPT_LINES,
            ),
            tosumu: None,
            font: None,
            text_draws: Vec::new(),
            frame_index: 0,
            warm_frames_without_text_change: 0,
            needs_text_rebuild: true,
        }
    }
}

impl ConsoleCorpus {
    fn prompt_bounds(&self) -> UiRect {
        UiRect::new([0.0, -0.70], [2.72, 0.18])
    }

    fn cursor_world(&self) -> [f32; 2] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_width = width / height;
        [
            self.input_events.mouse.x / width * half_width * 2.0 - half_width,
            1.0 - self.input_events.mouse.y / height * 2.0,
        ]
    }

    fn append_line(&mut self, line: impl Into<String>) {
        self.interaction.append_line(line);
        self.needs_text_rebuild = true;
    }

    fn transcript_wrap_width(&self) -> u32 {
        // The transcript begins at -1.25 and may occupy 2.42 world units.
        // Orthographic 2D maps two world units to the window height in pixels.
        (self.window_size[1].max(1.0) * 1.21) as u32
    }

    fn wrapped_transcript_lines(&self) -> Vec<String> {
        let Some(font) = self.font.as_ref() else {
            return self.interaction.transcript().to_vec();
        };
        let max_width = self.transcript_wrap_width().max(1);
        self.interaction
            .transcript()
            .iter()
            .flat_map(|line| wrap_console_line(font, line, 26.0, max_width))
            .collect()
    }

    fn max_scroll_offset(&self) -> usize {
        self.wrapped_transcript_lines()
            .len()
            .saturating_sub(TRANSCRIPT_CAPACITY)
    }

    fn scroll_transcript(&mut self, direction: i32) {
        self.interaction.scroll(direction, self.max_scroll_offset());
        self.needs_text_rebuild = true;
    }

    fn visible_transcript_range(&self, line_count: usize) -> std::ops::Range<usize> {
        self.interaction
            .visible_range(line_count, TRANSCRIPT_CAPACITY)
    }

    fn submit_prompt(&mut self) {
        let Some(command) = self.interaction.take_submission() else {
            return;
        };
        match command.to_ascii_lowercase().as_str() {
            "help" => self.append_line(
                "local: HELP | CLEAR; Tosumu: STATUS | CHECK | DESCRIBE <key> | WAL STATUS",
            ),
            "clear" => {
                self.interaction
                    .replace_transcript("transcript cleared; local fixture remains active");
            }
            _ => {
                if let Some(session) = self.tosumu.as_ref() {
                    for line in session.execute(&command) {
                        self.append_line(line);
                    }
                } else {
                    self.append_line(
                        "[tosumu unavailable] build tosumu-cli or set TOSUMU_CLI_BIN; no command was interpreted locally",
                    );
                }
            }
        }
        self.needs_text_rebuild = true;
    }

    fn recall_previous_command(&mut self) {
        self.interaction.recall_previous_command();
        self.needs_text_rebuild = true;
    }

    fn recall_next_command(&mut self) {
        self.interaction.recall_next_command();
        self.needs_text_rebuild = true;
    }

    fn upload_line(
        &mut self,
        renderer: &mut WgpuBackend,
        slot: u64,
        text: &str,
        pixel_size: f32,
        origin: [f32; 2],
        tint: [u8; 3],
    ) -> PlatformResult<()> {
        let font = self
            .font
            .as_ref()
            .ok_or_else(|| "console font was not initialized".to_owned())?;
        let bitmap = font.rasterize_text(text, pixel_size);
        if bitmap.width == 0 || bitmap.height == 0 {
            return Ok(());
        }
        // Fixed slots keep a long session bounded: changed text replaces its
        // source texture instead of accumulating one GPU resource per redraw.
        let (texture, material) = text_resource_handles(slot);
        renderer.try_upload_texture(
            texture,
            &Texture::rgba8(
                bitmap.width,
                bitmap.height,
                alpha_to_rgba8(&bitmap.alpha, tint),
            ),
        )?;
        renderer.upload_material(
            material,
            &Material::new("console-font-line", Color::rgb(1.0, 1.0, 1.0)).with_texture(texture),
        )?;

        let pixel_scale = 1.0 / self.window_size[1].max(1.0);
        self.text_draws.push(TextDraw {
            material,
            center: [
                origin[0] + (bitmap.left + bitmap.width as f32 * 0.5) * pixel_scale * 2.0,
                origin[1]
                    - (bitmap.baseline + bitmap.top + bitmap.height as f32 * 0.5)
                        * pixel_scale
                        * 2.0,
            ],
            size: [
                bitmap.width as f32 * pixel_scale * 2.0,
                bitmap.height as f32 * pixel_scale * 2.0,
            ],
        });
        Ok(())
    }

    fn rebuild_text_draws(&mut self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        self.text_draws.clear();
        self.upload_line(
            renderer,
            HEADER_TITLE_SLOT,
            "TOKIMU TERMINAL  /  TOSUMU TQL FIXTURE",
            30.0,
            [-1.25, 0.76],
            ACCENT_TINT,
        )?;
        self.upload_line(
            renderer,
            HEADER_METADATA_SLOT,
            "DEPARTURE MONO | TQL JSON ACTIVE | RATATUI CELL EVIDENCE READY",
            17.0,
            [-1.25, 0.60],
            MUTED_TINT,
        )?;
        let transcript_lines = self.wrapped_transcript_lines();
        let visible_range = self.visible_transcript_range(transcript_lines.len());
        let visible_lines: Vec<_> = transcript_lines[visible_range].to_vec();
        for (index, line) in visible_lines.iter().enumerate() {
            let tint = if line.starts_with('>') {
                ACCENT_TINT
            } else {
                TEXT_TINT
            };
            self.upload_line(
                renderer,
                TRANSCRIPT_SLOT_BASE + index as u64,
                line,
                26.0,
                [
                    -1.25,
                    TRANSCRIPT_START_Y - index as f32 * TRANSCRIPT_LINE_HEIGHT,
                ],
                tint,
            )?;
        }
        let scroll_status = if self.interaction.scroll_offset() == 0 {
            "TRANSCRIPT LIVE  |  WHEEL REVIEWS HISTORY  |  UP/DOWN RECALL COMMANDS"
        } else {
            "TRANSCRIPT REVIEW  |  WHEEL DOWN RETURNS TO LIVE OUTPUT"
        };
        self.upload_line(
            renderer,
            SCROLL_STATUS_SLOT,
            scroll_status,
            14.0,
            [-1.25, -0.54],
            MUTED_TINT,
        )?;
        let prompt = if self.interaction.input().value().is_empty() {
            "> _".to_owned()
        } else {
            format!("> {}_", self.interaction.input().value())
        };
        self.upload_line(
            renderer,
            PROMPT_SLOT,
            &prompt,
            28.0,
            [-1.20, -0.68],
            ACCENT_TINT,
        )?;
        self.upload_line(
            renderer,
            PROMPT_HELP_SLOT,
            if self.interaction.focused() {
                "PROMPT FOCUSED  |  ENTER SUBMITS  |  HOME/END MOVES  |  ESC CLEARS"
            } else {
                "CLICK THE PROMPT TO FOCUS"
            },
            17.0,
            [-1.20, -0.88],
            MUTED_TINT,
        )?;
        self.needs_text_rebuild = false;
        Ok(())
    }

    fn process_input(&mut self, event: PlatformInputEvent) {
        if let Some(input_event) = event.as_input_event() {
            self.input_events.apply_event(input_event);
        }
        match event {
            PlatformInputEvent::TextInput(text) if self.interaction.focused() => {
                self.interaction.insert_text(&text);
                self.needs_text_rebuild = true;
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
            } => {
                self.interaction
                    .set_focused(self.prompt_bounds().contains(self.cursor_world()));
                self.needs_text_rebuild = true;
            }
            PlatformInputEvent::MouseWheel { delta_y, .. } if delta_y.abs() >= 1.0 => {
                // Native wheel-up conventionally reveals older transcript rows;
                // wheel-down moves back toward the live tail.
                self.scroll_transcript(if delta_y > 0.0 { 1 } else { -1 });
            }
            PlatformInputEvent::KeyboardInput { key, pressed: true }
                if self.interaction.focused() =>
            {
                match key {
                    KeyCode::ArrowUp => self.recall_previous_command(),
                    KeyCode::ArrowDown => self.recall_next_command(),
                    KeyCode::ArrowLeft => self.interaction.edit(UiTextInputOperation::MoveLeft),
                    KeyCode::ArrowRight => self.interaction.edit(UiTextInputOperation::MoveRight),
                    KeyCode::Home => self.interaction.edit(UiTextInputOperation::MoveToStart),
                    KeyCode::End => self.interaction.edit(UiTextInputOperation::MoveToEnd),
                    KeyCode::Backspace => {
                        self.interaction.edit(UiTextInputOperation::DeleteBackward)
                    }
                    KeyCode::Delete => self.interaction.edit(UiTextInputOperation::DeleteForward),
                    KeyCode::Enter => self.submit_prompt(),
                    KeyCode::Escape => self.interaction.clear_prompt(),
                    _ => return,
                }
                self.needs_text_rebuild = true;
            }
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
                self.needs_text_rebuild = true;
            }
            _ => {}
        }
    }
}

impl PlatformEventHandler for ConsoleCorpus {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        window.set_ime_allowed(true);
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.font = Some(
            UiFontRasterizer::from_bytes(UiFontSource::from_native_default()?.bytes)
                .map_err(|error| error.to_string())?,
        );
        match TosumuSession::open_fixture() {
            Ok(session) => {
                self.tosumu = Some(session);
                self.append_line(
                    "[tosumu] disposable fixture ready; TQL executes through the JSON CLI boundary",
                );
            }
            Err(error) => self.append_line(format!("[tosumu unavailable] {error}")),
        }
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        renderer.upload_material(
            BACKDROP,
            &Material::new("console-backdrop", Color::rgb(0.008, 0.014, 0.018)),
        )?;
        renderer.upload_material(
            CONSOLE,
            &Material::new("console-surface", Color::rgb(0.022, 0.055, 0.060)),
        )?;
        renderer.upload_material(
            PROMPT,
            &Material::new("console-prompt", Color::rgb(0.025, 0.105, 0.105)),
        )?;
        renderer.upload_material(
            HEADER,
            &Material::new("console-header", Color::rgb(0.030, 0.135, 0.135)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "tokimu-console-command-window",
            PipelineKind::Texture2d,
        ))?;
        self.rebuild_text_draws(&mut renderer)?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        self.process_input(event);
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(mut renderer) = self.renderer.take() else {
            return Ok(FrameOutcome::Continue);
        };
        let text_changed_this_frame = self.needs_text_rebuild;
        if text_changed_this_frame {
            self.rebuild_text_draws(&mut renderer)?;
        }
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.008, 0.014, 0.018),
        })]);
        for (center, size, material) in [
            ([0.0, 0.0], [2.82, 1.78], CONSOLE),
            ([0.0, 0.72], [2.82, 0.24], HEADER),
            (
                self.prompt_bounds().center,
                self.prompt_bounds().size,
                PROMPT,
            ),
        ] {
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD,
                material,
                pipeline: self.pipeline,
                instance: Instance2d::new(center, size, 0.0),
                camera: Some(CAMERA),
                viewport: None,
            })]);
        }
        let commands: Vec<_> = self
            .text_draws
            .iter()
            .map(|draw| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: QUAD,
                    material: draw.material,
                    pipeline: self.pipeline,
                    instance: Instance2d::new(draw.center, draw.size, 0.0),
                    camera: Some(CAMERA),
                    viewport: None,
                })
            })
            .collect();
        renderer.submit(&commands);
        let stats = renderer.present()?;
        self.frame_index += 1;
        let should_report_warm_frame = record_warm_frame(
            &mut self.warm_frames_without_text_change,
            text_changed_this_frame,
            WARM_FRAME_INTERVAL,
        );
        if should_report_warm_frame {
            let resource_churn = unchanged_frame_has_resource_churn(&stats.frame);
            println!(
                "tokimu-console-command-window warm frame {}: unchanged_frames={}, resource_churn={}, draws={}, submits={}, texture_allocations={}, texture_replacements={}, texture_writes={}, binding_allocations={}, pipeline_creations={}, mesh_uploads={}, mesh_replacements={}",
                self.frame_index,
                self.warm_frames_without_text_change,
                resource_churn,
                stats.frame.draw_calls,
                stats.frame.submit_calls,
                stats.frame.texture_allocations,
                stats.frame.texture_replacements,
                stats.frame.texture_writes,
                stats.frame.binding_allocations,
                stats.frame.pipeline_creations,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,
            );
        }
        self.renderer = Some(renderer);
        Ok(FrameOutcome::Continue)
    }
}

// A frame becomes warm only after presentation content has remained unchanged
// for a full interval. This prevents a command, resize, or prompt edit from
// being mislabeled as steady-state renderer evidence.
fn record_warm_frame(
    unchanged_frames: &mut u64,
    content_changed_this_frame: bool,
    interval: u64,
) -> bool {
    if content_changed_this_frame {
        *unchanged_frames = 0;
        return false;
    }

    *unchanged_frames += 1;
    *unchanged_frames >= interval && unchanged_frames.is_multiple_of(interval)
}

fn wrap_console_line(
    font: &UiFontRasterizer,
    text: &str,
    pixel_size: f32,
    max_width: u32,
) -> Vec<String> {
    let mut wrapped = Vec::new();
    for source_line in text.split('\n') {
        let words: Vec<_> = source_line.split_whitespace().collect();
        if words.is_empty() {
            wrapped.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in words {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if font.rasterize_text(&candidate, pixel_size).width <= max_width {
                current = candidate;
                continue;
            }

            if !current.is_empty() {
                wrapped.push(std::mem::take(&mut current));
            }

            // Diagnostics may contain an address or hash with no natural break.
            // Split only those oversized tokens and retain every character.
            for character in word.chars() {
                let mut next = current.clone();
                next.push(character);
                if !current.is_empty() && font.rasterize_text(&next, pixel_size).width > max_width {
                    wrapped.push(std::mem::take(&mut current));
                }
                current.push(character);
            }
        }
        if !current.is_empty() {
            wrapped.push(current);
        }
    }
    wrapped
}

fn text_resource_handles(slot: u64) -> (TextureHandle, MaterialHandle) {
    let handle = TEXT_RESOURCE_BASE + slot;
    (TextureHandle(handle), MaterialHandle(handle))
}

// Camera updates and draw submission are expected every frame. This helper only
// classifies allocations or uploads that an unchanged console must not trigger.
fn unchanged_frame_has_resource_churn(stats: &RenderFrameStats) -> bool {
    stats.binding_allocations > 0
        || stats.pipeline_creations > 0
        || stats.pipeline_replacements > 0
        || stats.derived_material_cache_misses > 0
        || stats.mesh_uploads > 0
        || stats.mesh_replacements > 0
        || stats.texture_allocations > 0
        || stats.texture_replacements > 0
        || stats.texture_writes > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_rasterizer() -> UiFontRasterizer {
        let source = UiFontSource::from_native_default().expect("checked-in default font");
        UiFontRasterizer::from_bytes(source.bytes).expect("valid default font")
    }

    #[test]
    fn text_resources_are_drawn_from_a_fixed_bounded_slot_set() {
        let slots = [
            HEADER_TITLE_SLOT,
            HEADER_METADATA_SLOT,
            TRANSCRIPT_SLOT_BASE,
            TRANSCRIPT_SLOT_BASE + TRANSCRIPT_CAPACITY as u64 - 1,
            SCROLL_STATUS_SLOT,
            PROMPT_SLOT,
            PROMPT_HELP_SLOT,
        ];

        let handles: Vec<_> = slots.into_iter().map(text_resource_handles).collect();
        assert!(handles.windows(2).all(|pair| pair[0].0 .0 < pair[1].0 .0));
        assert_eq!(
            handles.last().unwrap().0 .0,
            TEXT_RESOURCE_BASE + PROMPT_HELP_SLOT
        );
    }

    #[test]
    fn measured_wrapping_keeps_every_line_inside_the_viewport() {
        let font = default_rasterizer();
        let max_width = 220;
        let wrapped = wrap_console_line(
            &font,
            "[tosumu] a deliberately long diagnostic wraps without escaping the viewport",
            26.0,
            max_width,
        );

        assert!(wrapped.len() > 1);
        assert!(wrapped
            .iter()
            .all(|line| font.rasterize_text(line, 26.0).width <= max_width));
    }

    #[test]
    fn measured_wrapping_does_not_drop_oversized_tokens() {
        let font = default_rasterizer();
        let source = "hash=0123456789abcdef0123456789abcdef punctuation!?";
        let wrapped = wrap_console_line(&font, source, 26.0, 150);
        let source_without_spacing: String =
            source.chars().filter(|c| !c.is_whitespace()).collect();
        let wrapped_without_spacing: String = wrapped
            .iter()
            .flat_map(|line| line.chars())
            .filter(|c| !c.is_whitespace())
            .collect();

        assert_eq!(wrapped_without_spacing, source_without_spacing);
        assert!(wrapped
            .iter()
            .all(|line| font.rasterize_text(line, 26.0).width <= 150));
    }

    #[test]
    fn unchanged_frames_classify_only_resource_churn() {
        let mut stats = RenderFrameStats {
            draw_calls: 18,
            submit_calls: 4,
            uniform_buffer_writes: 1,
            material_resolutions: 18,
            pipeline_switches: 1,
            ..RenderFrameStats::default()
        };
        assert!(!unchanged_frame_has_resource_churn(&stats));

        stats.texture_writes = 1;
        assert!(unchanged_frame_has_resource_churn(&stats));
        stats.texture_writes = 0;
        stats.binding_allocations = 1;
        assert!(unchanged_frame_has_resource_churn(&stats));
    }

    #[test]
    fn warm_frame_reporting_waits_for_a_full_unchanged_interval() {
        let mut unchanged_frames = 0;
        for _ in 0..WARM_FRAME_INTERVAL - 1 {
            assert!(!record_warm_frame(
                &mut unchanged_frames,
                false,
                WARM_FRAME_INTERVAL,
            ));
        }
        assert_eq!(unchanged_frames, WARM_FRAME_INTERVAL - 1);
        assert!(record_warm_frame(
            &mut unchanged_frames,
            false,
            WARM_FRAME_INTERVAL,
        ));
        assert_eq!(unchanged_frames, WARM_FRAME_INTERVAL);
    }

    #[test]
    fn text_changes_reset_warm_frame_evidence() {
        let mut unchanged_frames = WARM_FRAME_INTERVAL - 1;
        assert!(!record_warm_frame(
            &mut unchanged_frames,
            true,
            WARM_FRAME_INTERVAL,
        ));
        assert_eq!(unchanged_frames, 0);
        assert!(!record_warm_frame(
            &mut unchanged_frames,
            false,
            WARM_FRAME_INTERVAL,
        ));
        assert_eq!(unchanged_frames, 1);
    }
}
