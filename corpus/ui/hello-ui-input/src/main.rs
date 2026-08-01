use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use tokimu_input::{InputState, KeyCode};
use ui_tools::{
    UiActivationKey, UiButtonId, UiFocusDirection, UiInteractionState, UiNodeId, UiNodeInteraction,
    UiNodeKind, UiNodeLayout, UiNodeSpec, UiPointerEvent, UiPointerPhase, UiPointerRouter, UiRect,
    UiRegionKind, UiResolvedFocus, UiResolvedTree, UiSurfaceRole, UiTree,
};

const REGION_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SURFACE_MATERIAL: MaterialHandle = MaterialHandle(2);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(3);
const CARD_MATERIAL: MaterialHandle = MaterialHandle(4);
const ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(5);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(6);

const ROOT_NODE: UiNodeId = UiNodeId(1);
const MOUSE_NODE: UiNodeId = UiNodeId(2);
const KEYBOARD_NODE: UiNodeId = UiNodeId(3);
const CAPTURE_NODE: UiNodeId = UiNodeId(4);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Input".into(),
            width: 1200,
            height: 760,
        },
        HelloUiInputApp::new(),
    )
}

struct HelloUiInputApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    input: InputState,
    focus: UiResolvedFocus,
    pointer: UiPointerRouter,
    captured: bool,
}

impl Default for HelloUiInputApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            input: InputState::default(),
            focus: UiResolvedFocus::default(),
            pointer: UiPointerRouter::default(),
            captured: false,
        }
    }
}

impl HelloUiInputApp {
    fn new() -> Self {
        Self::default()
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

    fn draw_region(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        rect: UiRect,
        role: UiSurfaceRole,
        active: bool,
    ) {
        let style_role = if active {
            UiSurfaceRole::Selected
        } else {
            role
        };
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: REGION_MESH,
            material: Self::material_for_role(style_role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);
    }

    fn layout(&self) -> [UiRect; 3] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        let column_width = (half_width * 1.68 / 3.0).max(0.42);
        let card_y = 0.06;
        [
            UiRect::new([-column_width - 0.18, card_y], [column_width, 0.72]),
            UiRect::new([0.0, card_y], [column_width, 0.72]),
            UiRect::new([column_width + 0.18, card_y], [column_width, 0.72]),
        ]
    }

    fn focus_rects(&self) -> [UiRect; 3] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        let base_y = -0.56;
        [
            UiRect::new([-half_width + 0.42, base_y], [0.42, 0.12]),
            UiRect::new([0.0, base_y], [0.42, 0.12]),
            UiRect::new([half_width - 0.42, base_y], [0.42, 0.12]),
        ]
    }

    fn viewport(&self) -> UiRect {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        UiRect::new([0.0, 0.0], [2.0 * width / height, 2.0])
    }

    fn interaction_tree(&self) -> UiResolvedTree {
        let rects = self.focus_rects();
        let controls = [
            (MOUSE_NODE, UiButtonId(0), "mouse target", rects[0]),
            (KEYBOARD_NODE, UiButtonId(1), "keyboard target", rects[1]),
            (CAPTURE_NODE, UiButtonId(2), "capture toggle", rects[2]),
        ];
        let root = controls.into_iter().fold(
            UiNodeSpec::new(
                ROOT_NODE,
                UiNodeKind::Region(UiRegionKind::Workspace),
                UiSurfaceRole::Background,
                UiNodeLayout::Fill,
            ),
            |root, (id, button_id, label, rect)| {
                root.with_child(
                    UiNodeSpec::new(
                        id,
                        UiNodeKind::Button(button_id),
                        UiSurfaceRole::Raised,
                        UiNodeLayout::Explicit(rect),
                    )
                    .with_parent(ROOT_NODE)
                    .with_interaction(UiNodeInteraction::Activatable)
                    .with_semantic_label(label),
                )
            },
        );
        UiTree::new(root)
            .resolve(self.viewport())
            .expect("hello-ui-input uses unique, valid semantic node identities")
    }

    fn cursor_world(&self) -> [f32; 2] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        let x = (self.input.mouse.x / width) * (half_width * 2.0) - half_width;
        let y = half_height - (self.input.mouse.y / height) * (half_height * 2.0);
        [x, y]
    }

    fn route_pointer(&mut self, phase: UiPointerPhase) -> Option<UiNodeId> {
        let tree = self.interaction_tree();
        let resolution = self
            .pointer
            .route(&tree, UiPointerEvent::new(self.cursor_world(), phase));
        if matches!(phase, UiPointerPhase::Press) {
            self.focus.set_focus(&tree, resolution.target);
        }
        resolution.activated
    }

    fn update_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello UI Input | focus={:?} | hovered={:?} | mouse={} | left={} | right={} | capture={}",
                self.focus.focused(),
                self.pointer.hover(),
                if self.input.mouse.is_pressed(MouseButton::Left) { "down" } else { "up" },
                if self.input.keyboard.is_pressed(KeyCode::ArrowLeft) { "down" } else { "up" },
                if self.input.keyboard.is_pressed(KeyCode::ArrowRight) { "down" } else { "up" },
                if self.captured { "on" } else { "off" },
            ));
        }
    }

    fn draw_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let columns = self.layout();
        let focus_rects = self.focus_rects();
        let interaction_tree = self.interaction_tree();

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

        let focused = self.focus.focused();
        Self::draw_region(
            renderer,
            self.pipeline,
            columns[0],
            UiSurfaceRole::Panel,
            focused == Some(MOUSE_NODE),
        );
        Self::draw_region(
            renderer,
            self.pipeline,
            columns[1],
            UiSurfaceRole::Card,
            focused == Some(KEYBOARD_NODE),
        );
        Self::draw_region(
            renderer,
            self.pipeline,
            columns[2],
            UiSurfaceRole::Toolbar,
            focused == Some(CAPTURE_NODE),
        );

        Self::draw_region(
            renderer,
            self.pipeline,
            focus_rects[0],
            UiSurfaceRole::Region,
            self.pointer
                .interaction_state(&interaction_tree, &self.focus, MOUSE_NODE, false)
                != UiInteractionState::Idle,
        );
        Self::draw_region(
            renderer,
            self.pipeline,
            focus_rects[1],
            UiSurfaceRole::Region,
            self.pointer
                .interaction_state(&interaction_tree, &self.focus, KEYBOARD_NODE, false)
                != UiInteractionState::Idle,
        );
        Self::draw_region(
            renderer,
            self.pipeline,
            focus_rects[2],
            UiSurfaceRole::Region,
            self.pointer.interaction_state(
                &interaction_tree,
                &self.focus,
                CAPTURE_NODE,
                self.captured,
            ) != UiInteractionState::Idle,
        );

        let _ = renderer.present()?;
        self.update_title();
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloUiInputApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-input-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            SURFACE_MATERIAL,
            &Material::new("ui-input-surface", Color::rgb(0.18, 0.20, 0.25)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-input-panel", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            CARD_MATERIAL,
            &Material::new("ui-input-card", Color::rgb(0.22, 0.24, 0.30)),
        )?;
        renderer.upload_material(
            ACTIVE_MATERIAL,
            &Material::new("ui-input-active", Color::rgb(0.34, 0.56, 0.86)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-input-muted", Color::rgb(0.10, 0.12, 0.14)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-input-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.route_pointer(UiPointerPhase::Move);
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let Some(input_event) = event.as_input_event() {
            self.input.apply_event(input_event);
        }

        match event {
            PlatformInputEvent::CursorMoved { .. } => {
                self.route_pointer(UiPointerPhase::Move);
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
            } => {
                self.route_pointer(UiPointerPhase::Press);
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: false,
            } if self.route_pointer(UiPointerPhase::Release) == Some(CAPTURE_NODE) => {
                self.captured = !self.captured;
            }
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::ArrowLeft => {
                    self.focus
                        .move_focus(&self.interaction_tree(), UiFocusDirection::Backward);
                }
                KeyCode::ArrowRight => {
                    self.focus
                        .move_focus(&self.interaction_tree(), UiFocusDirection::Forward);
                }
                KeyCode::Space
                    if self
                        .focus
                        .activate(&self.interaction_tree(), UiActivationKey::Space)
                        == Some(CAPTURE_NODE) =>
                {
                    self.captured = !self.captured;
                }
                _ => {}
            },
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
                let tree = self.interaction_tree();
                self.focus.reconcile(&tree);
                self.route_pointer(UiPointerPhase::Move);
            }
            _ => {}
        }

        self.update_title();
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.draw_scene()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_and_keyboard_resolve_the_same_capture_identity() {
        let app = HelloUiInputApp::new();
        let tree = app.interaction_tree();
        let capture_center = app.focus_rects()[2].center;
        let mut pointer = UiPointerRouter::default();

        let pressed = pointer.route(
            &tree,
            UiPointerEvent::new(capture_center, UiPointerPhase::Press),
        );
        assert_eq!(pressed.target, Some(CAPTURE_NODE));
        assert_eq!(pressed.captured, Some(CAPTURE_NODE));

        let released = pointer.route(
            &tree,
            UiPointerEvent::new(capture_center, UiPointerPhase::Release),
        );
        assert_eq!(released.activated, Some(CAPTURE_NODE));

        let mut focus = UiResolvedFocus::default();
        assert_eq!(
            focus.move_focus(&tree, UiFocusDirection::Backward),
            Some(CAPTURE_NODE)
        );
        assert_eq!(
            focus.activate(&tree, UiActivationKey::Space),
            Some(CAPTURE_NODE)
        );
    }
}
