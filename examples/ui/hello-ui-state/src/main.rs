use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{
    layout_bitmap_text, window_to_world, UiActionId, UiActivationKey, UiButtonId, UiButtonSpec,
    UiCard, UiCardRole, UiControlRole, UiDrawer, UiEvent, UiInteractionState, UiRect, UiRegion,
    UiRegionKind, UiSurfaceCommand, UiSurfaceRole,
    UiTextAlign, UiTextRole, UiTextSpec, UiTheme,
    UiWorkspaceLayout,
};

const REGION_MESH: MeshHandle = MeshHandle(1);
const GLYPH_MESH: MeshHandle = MeshHandle(2);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SURFACE_MATERIAL: MaterialHandle = MaterialHandle(2);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(3);
const CARD_MATERIAL: MaterialHandle = MaterialHandle(4);
const ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(5);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(6);
const TEXT_MATERIAL: MaterialHandle = MaterialHandle(7);

const PREVIOUS_ASSET: UiActionId = UiActionId(1);
const TOGGLE_PINNED: UiActionId = UiActionId(2);
const NEXT_ASSET: UiActionId = UiActionId(3);

#[derive(Clone, Copy)]
struct AssetProfile {
    name: &'static str,
    summary: &'static str,
    role: UiCardRole,
    accent: UiSurfaceRole,
}

const ASSETS: [AssetProfile; 3] = [
    AssetProfile {
        name: "Sketch",
        summary: "UNSAVED",
        role: UiCardRole::Browser,
        accent: UiSurfaceRole::Card,
    },
    AssetProfile {
        name: "Mesh",
        summary: "REVIEW",
        role: UiCardRole::Editor,
        accent: UiSurfaceRole::Selected,
    },
    AssetProfile {
        name: "Telemetry",
        summary: "LIVE",
        role: UiCardRole::Preview,
        accent: UiSurfaceRole::Accent,
    },
];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI State".into(),
            width: 1360,
            height: 840,
        },
        HelloUiStateApp::new(),
    )
}

struct HelloUiStateApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    selected_asset: usize,
    hovered_asset: Option<usize>,
    hovered_button: Option<usize>,
    focused_button: UiButtonId,
    pinned: bool,
    revision: u64,
    cursor_position: [f32; 2],
    cached_window_size: [f32; 2],
    cached_selected_asset: usize,
    cached_hovered_asset: Option<usize>,
    cached_hovered_button: Option<usize>,
    cached_pinned: bool,
    cached_revision: u64,
    cached_surfaces: Vec<UiSurfaceCommand>,
    cached_text: Vec<RenderCommand>,
}

impl Default for HelloUiStateApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
            selected_asset: 0,
            hovered_asset: None,
            hovered_button: None,
            focused_button: UiButtonId(1),
            pinned: false,
            revision: 0,
            cursor_position: [0.0, 0.0],
            cached_window_size: [0.0, 0.0],
            cached_selected_asset: usize::MAX,
            cached_hovered_asset: None,
            cached_hovered_button: None,
            cached_pinned: false,
            cached_revision: u64::MAX,
            cached_surfaces: Vec::new(),
            cached_text: Vec::new(),
        }
    }
}

impl HelloUiStateApp {
    fn new() -> Self {
        Self::default()
    }

    fn layout(&self) -> UiWorkspaceLayout {
        UiWorkspaceLayout::new(
            self.window_size,
            [
                UiButtonSpec::new(UiButtonId(0), "PREV").with_action(PREVIOUS_ASSET),
                UiButtonSpec::new(UiButtonId(1), "PIN").with_action(TOGGLE_PINNED),
                UiButtonSpec::new(UiButtonId(2), "NEXT").with_action(NEXT_ASSET),
            ],
            [
                ui_tools::UiCardSpec::new(ASSETS[0].role, ASSETS[0].name, ASSETS[0].summary),
                ui_tools::UiCardSpec::new(ASSETS[1].role, ASSETS[1].name, ASSETS[1].summary),
                ui_tools::UiCardSpec::new(ASSETS[2].role, ASSETS[2].name, ASSETS[2].summary),
            ],
        )
    }

    fn cursor_world(&self) -> [f32; 2] {
        window_to_world(self.window_size, self.cursor_position)
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let profile = ASSETS[self.selected_asset];
            window.set_title(&format!(
                "Tokimu Hello UI State | selected={} | pinned={} | revision={} | hover={} | asset={}",
                self.selected_asset,
                if self.pinned { "on" } else { "off" },
                self.revision,
                self.hovered_asset
                    .map(|index| index.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                profile.name,
            ));
        }
    }

    fn select_asset(&mut self, index: usize) {
        if self.selected_asset != index {
            self.selected_asset = index;
            self.revision = self.revision.saturating_add(1);
        }
    }

    fn select_previous(&mut self) {
        let count = ASSETS.len();
        self.select_asset((self.selected_asset + count - 1) % count);
    }

    fn select_next(&mut self) {
        let count = ASSETS.len();
        self.select_asset((self.selected_asset + 1) % count);
    }

    fn toggle_pin(&mut self) {
        self.pinned = !self.pinned;
        self.revision = self.revision.saturating_add(1);
    }

    fn hovered_button_index(&self, layout: &UiWorkspaceLayout) -> Option<usize> {
        layout.buttons.iter().position(|button| button.contains(self.cursor_world()))
    }

    fn hovered_asset_index(&self, layout: &UiWorkspaceLayout) -> Option<usize> {
        layout
            .cards
            .iter()
            .position(|card| card.region.contains(self.cursor_world()))
    }

    fn material_for_role(role: UiSurfaceRole) -> MaterialHandle {
        match role {
            UiSurfaceRole::Background => BACKDROP_MATERIAL,
            UiSurfaceRole::Region => SURFACE_MATERIAL,
            UiSurfaceRole::Panel => PANEL_MATERIAL,
            UiSurfaceRole::Card => CARD_MATERIAL,
            UiSurfaceRole::Toolbar => PANEL_MATERIAL,
            UiSurfaceRole::Raised => PANEL_MATERIAL,
            UiSurfaceRole::Selected => ACTIVE_MATERIAL,
            UiSurfaceRole::Accent => ACTIVE_MATERIAL,
            UiSurfaceRole::Overlay => MUTED_MATERIAL,
        }
    }

    fn draw_surface_command(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        command: &UiSurfaceCommand,
    ) {
        let rect = command.rect;
        if matches!(
            command.style.elevation,
            ui_tools::UiElevation::Raised | ui_tools::UiElevation::Floating
        ) {
            let shadow_rect = UiRect::new(
                [rect.center[0] + 0.012, rect.center[1] - 0.012],
                [rect.size[0], rect.size[1]],
            );
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: REGION_MESH,
                material: MUTED_MATERIAL,
                pipeline,
                instance: Instance2d::new(shadow_rect.center, shadow_rect.size, 0.0),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            })]);
        }

        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: REGION_MESH,
            material: Self::material_for_role(command.style.role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);

        if let Some(border_role) = command.style.border_role {
            let border = command
                .style
                .border_width
                .min(rect.size[0] * 0.22)
                .min(rect.size[1] * 0.22);
            if border > 0.0 {
                let border_rect = UiRect::new(
                    [rect.center[0], rect.center[1] + rect.size[1] * 0.5 - border * 0.5],
                    [rect.size[0], border],
                );
                renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: REGION_MESH,
                    material: Self::material_for_role(border_role),
                    pipeline,
                    instance: Instance2d::new(border_rect.center, border_rect.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                })]);
            }
        }
    }

    fn rebuild_cache(&mut self) {
        if self.cached_window_size == self.window_size
            && self.cached_selected_asset == self.selected_asset
            && self.cached_hovered_asset == self.hovered_asset
            && self.cached_hovered_button == self.hovered_button
            && self.cached_pinned == self.pinned
            && self.cached_revision == self.revision
        {
            return;
        }

        let layout = self.layout();
        self.cached_surfaces.clear();
        self.cached_text.clear();

        let mut text = Vec::new();
        {
            let mut drawer = UiDrawer::new(&mut self.cached_surfaces, &mut text, &self.theme);

            let mut workspace = layout.workspace;
            workspace.role = UiSurfaceRole::Background;
            drawer.surface(&workspace);

            let mut header = layout.header;
            header.role = if self.pinned {
                UiSurfaceRole::Selected
            } else {
                UiSurfaceRole::Raised
            };
            drawer.surface(&header);

            let mut toolbar = layout.toolbar;
            toolbar.role = if self.hovered_button.is_some() {
                UiSurfaceRole::Accent
            } else {
                UiSurfaceRole::Toolbar
            };
            drawer.surface(&toolbar);

            let mut sidebar = layout.sidebar;
            // Keep the region's elevation stable; selection is shown by the
            // accent strip rather than changing the whole sidebar surface.
            sidebar.role = UiSurfaceRole::Panel;
            drawer.surface(&sidebar);

            let mut canvas = layout.canvas;
            // The work area is always a panel. Asset selection should not
            // make its shadow appear or disappear.
            canvas.role = UiSurfaceRole::Panel;
            drawer.surface(&canvas);

            let mut inspector = layout.inspector;
            inspector.role = if self.pinned {
                UiSurfaceRole::Accent
            } else {
                UiSurfaceRole::Panel
            };
            drawer.surface(&inspector);

            let mut status_bar = layout.status_bar;
            status_bar.role = if self.pinned {
                UiSurfaceRole::Selected
            } else {
                UiSurfaceRole::Overlay
            };
            drawer.surface(&status_bar);

            let mut card_grid = layout.card_grid;
            card_grid.role = if self.hovered_asset.is_some() {
                UiSurfaceRole::Accent
            } else {
                UiSurfaceRole::Region
            };
            drawer.surface(&card_grid);

            let selected_profile = ASSETS[self.selected_asset];
            let focus_strip = UiRegion::new(
                UiRegionKind::Panel,
                selected_profile.accent,
                UiRect::new(
                    [sidebar.rect.center[0], sidebar.rect.center[1] + sidebar.rect.size[1] * 0.28],
                    [sidebar.rect.size[0] * 0.70, 0.06],
                ),
            );
            drawer.surface(&focus_strip);

            let indicator = UiRegion::new(
                UiRegionKind::Panel,
                if self.pinned {
                    UiSurfaceRole::Selected
                } else {
                    selected_profile.accent
                },
                UiRect::new(
                    [status_bar.rect.center[0], status_bar.rect.center[1]],
                    [status_bar.rect.size[0] * 0.46, 0.06],
                ),
            );
            drawer.surface(&indicator);

            for (index, button) in layout.buttons.iter().enumerate() {
                let state = if button.id == self.focused_button {
                    UiInteractionState::Focused
                } else if Some(index) == self.hovered_button {
                    UiInteractionState::Hovered
                } else if index == 1 && self.pinned {
                    UiInteractionState::Selected
                } else {
                    UiInteractionState::Idle
                };
                let role = match index {
                    0 => UiControlRole::Secondary,
                    1 => UiControlRole::Accent,
                    _ => UiControlRole::Primary,
                };
                drawer.button(button, state, role);
            }

            for (index, card) in layout.cards.iter().enumerate() {
                let mut asset_card = *card;
                asset_card.role = if index == self.selected_asset {
                    UiCardRole::Selected
                } else if Some(index) == self.hovered_asset {
                    UiCardRole::Preview
                } else {
                    UiCardRole::Browser
                };
                drawer.card(&asset_card);
            }

            let inspector_profile = ASSETS[self.selected_asset];
            let inspector_card = UiCard::new(
                if self.pinned {
                    UiCardRole::Status
                } else {
                    UiCardRole::Inspector
                },
                inspector_profile.name,
                inspector_profile.summary,
                UiRegion::card(layout.inspector.rect),
            );
            drawer.card(&inspector_card);
        }

        for (text_value, rect, role) in [
            ("STATE", layout.header.rect, UiTextRole::Heading),
            ("SELECTION", layout.sidebar.rect, UiTextRole::Caption),
            ("STATUS", layout.status_bar.rect, UiTextRole::Caption),
        ] {
            let spec = UiTextSpec::new(text_value, rect, role)
                .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
            text.push(ui_tools::UiTextCommand {
                style: self.theme.text(role),
                spec,
            });
        }

        for command in &mut text {
            if command.spec.rect.size[1] < 0.16 {
                command.style.height = command
                    .style
                    .height
                    .min((command.spec.rect.size[1] * 0.78).max(0.020));
            }
        }

        self.cached_window_size = self.window_size;
        self.cached_selected_asset = self.selected_asset;
        self.cached_hovered_asset = self.hovered_asset;
        self.cached_hovered_button = self.hovered_button;
        self.cached_pinned = self.pinned;
        self.cached_revision = self.revision;

        self.cached_text.extend(text.into_iter().flat_map(|command| {
            layout_bitmap_text(&command.spec, command.style.height)
                .into_iter()
                .map(|quad| {
                    RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: GLYPH_MESH,
                        material: TEXT_MATERIAL,
                        pipeline: self.pipeline,
                        instance: Instance2d::new(quad.center, quad.size, 0.0),
                        camera: Some(CAMERA_HANDLE),
                        viewport: None,
                    })
                })
        }));
    }
}

impl PlatformEventHandler for HelloUiStateApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-state-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            SURFACE_MATERIAL,
            &Material::new("ui-state-surface", Color::rgb(0.18, 0.20, 0.25)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-state-panel", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            CARD_MATERIAL,
            &Material::new("ui-state-card", Color::rgb(0.22, 0.24, 0.30)),
        )?;
        renderer.upload_material(
            ACTIVE_MATERIAL,
            &Material::new("ui-state-active", Color::rgb(0.34, 0.56, 0.86)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-state-muted", Color::rgb(0.10, 0.12, 0.14)),
        )?;
        renderer.upload_material(
            TEXT_MATERIAL,
            &Material::new("ui-state-text", Color::rgb(0.90, 0.93, 0.98)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-state-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.hovered_button = None;
        self.hovered_asset = None;
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.cursor_position = [x, y];
            let layout = self.layout();
            self.hovered_button = self.hovered_button_index(&layout);
            self.hovered_asset = self.hovered_asset_index(&layout);
            self.update_window_title();
        }

        if let PlatformInputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        } = event
        {
            let layout = self.layout();
            if let Some(event) = layout.event_at(self.cursor_world(), true) {
                if let Some(button) = layout.button_at(self.cursor_world()) {
                    self.focused_button = button;
                }
                match event {
                    UiEvent::Activated(PREVIOUS_ASSET) => self.select_previous(),
                    UiEvent::Activated(TOGGLE_PINNED) => self.toggle_pin(),
                    UiEvent::Activated(NEXT_ASSET) => self.select_next(),
                    UiEvent::Activated(_) => {}
                }
            } else if let Some(index) = self.hovered_asset_index(&layout) {
                self.select_asset(index);
            } else if layout.status_bar.contains(self.cursor_world()) {
                self.toggle_pin();
            }
            self.update_window_title();
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if pressed {
                let layout = self.layout();
                let activation_key = match key {
                    KeyCode::Space => Some(UiActivationKey::Space),
                    _ => None,
                };
                if let Some(activation_key) = activation_key {
                    if let Some(UiEvent::Activated(action)) =
                        layout.focused_event(self.focused_button, activation_key, true)
                    {
                        match action {
                            PREVIOUS_ASSET => self.select_previous(),
                            TOGGLE_PINNED => self.toggle_pin(),
                            NEXT_ASSET => self.select_next(),
                            _ => {}
                        }
                    }
                } else {
                    match key {
                        KeyCode::ArrowLeft => self.select_previous(),
                        KeyCode::ArrowRight => self.select_next(),
                        _ => {}
                    }
                }
                self.update_window_title();
            }
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }

        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.rebuild_cache();

        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_camera(
            CAMERA_HANDLE,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.06, 0.08),
        })]);

        for command in &self.cached_surfaces {
            Self::draw_surface_command(renderer, self.pipeline, command);
        }
        for command in &self.cached_text {
            renderer.submit(&[*command]);
        }

        let _ = renderer.present()?;
        self.update_window_title();
        Ok(FrameOutcome::Continue)
    }
}
