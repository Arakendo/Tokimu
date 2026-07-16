use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{
    window_to_world, UiButtonId, UiButtonSpec, UiCard, UiCardRole, UiControlRole, UiDrawer,
    UiInteractionState, UiRect, UiRegion, UiRegionKind, UiSurfaceCommand, UiSurfaceRole, UiTheme,
    UiWorkspaceLayout,
};

const REGION_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SURFACE_MATERIAL: MaterialHandle = MaterialHandle(2);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(3);
const CARD_MATERIAL: MaterialHandle = MaterialHandle(4);
const ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(5);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(6);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InspectorField {
    Name,
    Visible,
    Locked,
    Material,
    Position,
    Scale,
    Rotation,
}

impl InspectorField {
    const ALL: [Self; 7] = [
        Self::Name,
        Self::Visible,
        Self::Locked,
        Self::Material,
        Self::Position,
        Self::Scale,
        Self::Rotation,
    ];

    fn next(self) -> Self {
        let index = Self::ALL.iter().position(|field| *field == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    fn prev(self) -> Self {
        let index = Self::ALL.iter().position(|field| *field == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Copy)]
struct InspectableObject {
    name: &'static str,
    kind: &'static str,
    role: UiCardRole,
}

const OBJECTS: [InspectableObject; 3] = [
    InspectableObject {
        name: "Camera Rig",
        kind: "Camera",
        role: UiCardRole::Editor,
    },
    InspectableObject {
        name: "Mesh Shell",
        kind: "Mesh",
        role: UiCardRole::Browser,
    },
    InspectableObject {
        name: "Lighting Probe",
        kind: "Light",
        role: UiCardRole::Preview,
    },
];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Inspector".into(),
            width: 1360,
            height: 840,
        },
        HelloUiInspectorApp::new(),
    )
}

struct HelloUiInspectorApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    selected_object: usize,
    selected_field: InspectorField,
    hovered_object: Option<usize>,
    hovered_field: Option<InspectorField>,
    dirty: bool,
    cursor_position: [f32; 2],
    cached_window_size: [f32; 2],
    cached_selected_object: usize,
    cached_selected_field: InspectorField,
    cached_hovered_object: Option<usize>,
    cached_hovered_field: Option<InspectorField>,
    cached_dirty: bool,
    cached_surfaces: Vec<UiSurfaceCommand>,
}

impl Default for HelloUiInspectorApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
            selected_object: 0,
            selected_field: InspectorField::Name,
            hovered_object: None,
            hovered_field: None,
            dirty: false,
            cursor_position: [0.0, 0.0],
            cached_window_size: [0.0, 0.0],
            cached_selected_object: usize::MAX,
            cached_selected_field: InspectorField::Name,
            cached_hovered_object: None,
            cached_hovered_field: None,
            cached_dirty: false,
            cached_surfaces: Vec::new(),
        }
    }
}

impl HelloUiInspectorApp {
    fn new() -> Self {
        Self::default()
    }

    fn layout(&self) -> UiWorkspaceLayout {
        UiWorkspaceLayout::new(
            self.window_size,
            [
                UiButtonSpec::new(UiButtonId(0), "PREV"),
                UiButtonSpec::new(UiButtonId(1), "FIELD"),
                UiButtonSpec::new(UiButtonId(2), "NEXT"),
            ],
            [
                ui_tools::UiCardSpec::new(OBJECTS[0].role, OBJECTS[0].name, OBJECTS[0].kind),
                ui_tools::UiCardSpec::new(OBJECTS[1].role, OBJECTS[1].name, OBJECTS[1].kind),
                ui_tools::UiCardSpec::new(OBJECTS[2].role, OBJECTS[2].name, OBJECTS[2].kind),
            ],
        )
    }

    fn cursor_world(&self) -> [f32; 2] {
        window_to_world(self.window_size, self.cursor_position)
    }

    fn update_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let object = OBJECTS[self.selected_object];
            window.set_title(&format!(
                "Tokimu Hello UI Inspector | object={} | field={:?} | dirty={} | hover_object={} | hover_field={:?}",
                object.name,
                self.selected_field,
                if self.dirty { "yes" } else { "no" },
                self.hovered_object
                    .map(|index| OBJECTS[index].name)
                    .unwrap_or("none"),
                self.hovered_field,
            ));
        }
    }

    fn select_object(&mut self, index: usize) {
        if self.selected_object != index {
            self.selected_object = index;
            self.dirty = true;
        }
    }

    fn select_prev_object(&mut self) {
        let count = OBJECTS.len();
        self.select_object((self.selected_object + count - 1) % count);
    }

    fn select_next_object(&mut self) {
        let count = OBJECTS.len();
        self.select_object((self.selected_object + 1) % count);
    }

    fn select_prev_field(&mut self) {
        self.selected_field = self.selected_field.prev();
        self.dirty = true;
    }

    fn select_next_field(&mut self) {
        self.selected_field = self.selected_field.next();
        self.dirty = true;
    }

    fn toggle_dirty(&mut self) {
        self.dirty = !self.dirty;
    }

    fn field_rects(&self, layout: &UiWorkspaceLayout) -> [UiRect; 7] {
        let area = layout.inspector.rect;
        let top = area.center[1] + area.size[1] * 0.30;
        let step = area.size[1] * 0.10;
        let width = area.size[0] * 0.82;
        [
            UiRect::new([area.center[0], top], [width, 0.10]),
            UiRect::new([area.center[0], top - step], [width, 0.10]),
            UiRect::new([area.center[0], top - step * 2.0], [width, 0.10]),
            UiRect::new([area.center[0], top - step * 3.0], [width, 0.10]),
            UiRect::new([area.center[0], top - step * 4.0], [width, 0.10]),
            UiRect::new([area.center[0], top - step * 5.0], [width, 0.10]),
            UiRect::new([area.center[0], top - step * 6.0], [width, 0.10]),
        ]
    }

    fn hovered_field_at_point(&self, layout: &UiWorkspaceLayout, point: [f32; 2]) -> Option<InspectorField> {
        let rects = self.field_rects(layout);
        InspectorField::ALL
            .iter()
            .enumerate()
            .find_map(|(index, field)| rects[index].contains(point).then_some(*field))
    }

    fn hovered_object_at_point(&self, layout: &UiWorkspaceLayout, point: [f32; 2]) -> Option<usize> {
        layout
            .cards
            .iter()
            .enumerate()
            .find_map(|(index, card)| card.region.contains(point).then_some(index))
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
            && self.cached_selected_object == self.selected_object
            && self.cached_selected_field == self.selected_field
            && self.cached_hovered_object == self.hovered_object
            && self.cached_hovered_field == self.hovered_field
            && self.cached_dirty == self.dirty
        {
            return;
        }

        let layout = self.layout();
        self.cached_surfaces.clear();
        let mut text = Vec::new();
        {
            let mut drawer = UiDrawer::new(&mut self.cached_surfaces, &mut text, &self.theme);

            let mut workspace = layout.workspace;
            workspace.role = UiSurfaceRole::Background;
            drawer.surface(&workspace);

            let mut header = layout.header;
            header.role = if self.dirty {
                UiSurfaceRole::Selected
            } else {
                UiSurfaceRole::Raised
            };
            drawer.surface(&header);

            let mut toolbar = layout.toolbar;
            toolbar.role = if self.hovered_object.is_some() {
                UiSurfaceRole::Accent
            } else {
                UiSurfaceRole::Toolbar
            };
            drawer.surface(&toolbar);

            let mut sidebar = layout.sidebar;
            sidebar.role = if self.selected_object == 0 {
                UiSurfaceRole::Selected
            } else {
                UiSurfaceRole::Region
            };
            drawer.surface(&sidebar);

            let mut canvas = layout.canvas;
            canvas.role = if self.selected_object == 1 {
                UiSurfaceRole::Panel
            } else {
                UiSurfaceRole::Region
            };
            drawer.surface(&canvas);

            let mut inspector = layout.inspector;
            inspector.role = if self.hovered_field.is_some() {
                UiSurfaceRole::Accent
            } else {
                UiSurfaceRole::Panel
            };
            drawer.surface(&inspector);

            let mut status_bar = layout.status_bar;
            status_bar.role = if self.dirty {
                UiSurfaceRole::Selected
            } else {
                UiSurfaceRole::Overlay
            };
            drawer.surface(&status_bar);

            let mut card_grid = layout.card_grid;
            card_grid.role = if self.hovered_object.is_some() {
                UiSurfaceRole::Accent
            } else {
                UiSurfaceRole::Region
            };
            drawer.surface(&card_grid);

            for (index, button) in layout.buttons.iter().enumerate() {
                let state = if index == 1 && self.dirty {
                    UiInteractionState::Selected
                } else if Some(index) == self.hovered_object.map(|_| index) {
                    UiInteractionState::Hovered
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
                let mut object_card = *card;
                object_card.role = if index == self.selected_object {
                    UiCardRole::Inspector
                } else if Some(index) == self.hovered_object {
                    UiCardRole::Preview
                } else {
                    OBJECTS[index].role
                };
                drawer.card(&object_card);
            }

            let selected_object = OBJECTS[self.selected_object];
            let inspector_card = UiCard::new(
                if self.dirty {
                    UiCardRole::Status
                } else {
                    UiCardRole::Inspector
                },
                selected_object.name,
                selected_object.kind,
                UiRegion::card(layout.inspector.rect),
            );
            drawer.card(&inspector_card);
        }

        let field_rects = self.field_rects(&layout);
        for (index, field) in InspectorField::ALL.iter().enumerate() {
            let active = *field == self.selected_field;
            let hovered = Some(*field) == self.hovered_field;
            let field_region = UiRegion::new(
                UiRegionKind::Panel,
                if active {
                    UiSurfaceRole::Selected
                } else if hovered {
                    UiSurfaceRole::Accent
                } else {
                    UiSurfaceRole::Region
                },
                field_rects[index],
            );
            self.cached_surfaces.push(UiSurfaceCommand {
                rect: field_region.rect,
                style: self.theme.surface(field_region.role),
            });
        }

        self.cached_window_size = self.window_size;
        self.cached_selected_object = self.selected_object;
        self.cached_selected_field = self.selected_field;
        self.cached_hovered_object = self.hovered_object;
        self.cached_hovered_field = self.hovered_field;
        self.cached_dirty = self.dirty;
    }
}

impl PlatformEventHandler for HelloUiInspectorApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-inspector-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            SURFACE_MATERIAL,
            &Material::new("ui-inspector-surface", Color::rgb(0.18, 0.20, 0.25)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-inspector-panel", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            CARD_MATERIAL,
            &Material::new("ui-inspector-card", Color::rgb(0.22, 0.24, 0.30)),
        )?;
        renderer.upload_material(
            ACTIVE_MATERIAL,
            &Material::new("ui-inspector-active", Color::rgb(0.34, 0.56, 0.86)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-inspector-muted", Color::rgb(0.10, 0.12, 0.14)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-inspector-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.hovered_object = None;
        self.hovered_field = None;
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.cursor_position = [x, y];
            let layout = self.layout();
            self.hovered_object = self.hovered_object_at_point(&layout, self.cursor_world());
            self.hovered_field = self.hovered_field_at_point(&layout, self.cursor_world());
            self.update_title();
        }

        if let PlatformInputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        } = event
        {
            let layout = self.layout();
            if let Some(index) = self.hovered_object_at_point(&layout, self.cursor_world()) {
                self.select_object(index);
            } else if let Some(field) = self.hovered_field_at_point(&layout, self.cursor_world()) {
                self.selected_field = field;
                self.dirty = true;
            } else if layout.status_bar.contains(self.cursor_world()) {
                self.toggle_dirty();
            }
            self.update_title();
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if pressed {
                match key {
                    KeyCode::ArrowLeft => self.select_prev_object(),
                    KeyCode::ArrowRight => self.select_next_object(),
                    KeyCode::ArrowUp => self.select_prev_field(),
                    KeyCode::ArrowDown => self.select_next_field(),
                    KeyCode::Space => self.toggle_dirty(),
                    _ => {}
                }
                self.update_title();
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

        let _ = renderer.present()?;
        self.update_title();
        Ok(FrameOutcome::Continue)
    }
}