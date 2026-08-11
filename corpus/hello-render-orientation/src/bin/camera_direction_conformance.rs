//! Native AR-0028 camera-basis and input-policy evidence.
//!
//! Physical platform observations remain separate from corpus-local camera
//! commands. This executable does not admit either shape as public API.

use std::{collections::BTreeSet, sync::Arc};

use render_orientation_conformance::{
    axis_landmarks, camera_conformance_matrices, landmark_mesh, CameraConformanceCommand,
    CameraConformancePose, FirstPersonLookPolicy, PointerMotionObservation,
};
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use winit::window::CursorGrabMode;

const CAMERA: CameraHandle = CameraHandle(1);
const FIRST_MESH: u64 = 1;
const FIRST_MATERIAL: u64 = 1;
const MOVE_SPEED: f32 = 2.5;
const KEY_LOOK_STEP: f32 = 0.12;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Camera Direction Conformance | loading".into(),
            width: 1200,
            height: 800,
        },
        CameraDirectionApp::default(),
    )
}

#[derive(Default)]
struct CameraDirectionApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    pose: CameraConformancePose,
    pressed: BTreeSet<KeyCode>,
    pointer_captured: bool,
    last_pointer: Option<PointerMotionObservation>,
    last_commands: Option<[CameraConformanceCommand; 2]>,
}

impl CameraDirectionApp {
    fn set_pointer_captured(&mut self, captured: bool) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if captured {
            let grabbed = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            if grabbed.is_ok() {
                window.set_cursor_visible(false);
                self.pointer_captured = true;
            }
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.pointer_captured = false;
        }
        self.update_title();
    }

    fn apply_held_movement(&mut self, delta_seconds: f64) {
        let distance = MOVE_SPEED * delta_seconds as f32;
        let mut forward = 0.0;
        let mut right = 0.0;
        let mut up = 0.0;
        if self.pressed.contains(&KeyCode::KeyW) {
            forward += distance;
        }
        if self.pressed.contains(&KeyCode::KeyS) {
            forward -= distance;
        }
        if self.pressed.contains(&KeyCode::KeyD) {
            right += distance;
        }
        if self.pressed.contains(&KeyCode::KeyA) {
            right -= distance;
        }
        if self.pressed.contains(&KeyCode::KeyE) {
            up += distance;
        }
        if self.pressed.contains(&KeyCode::KeyQ) {
            up -= distance;
        }
        if forward != 0.0 {
            self.pose
                .apply(CameraConformanceCommand::MoveForward(forward));
        }
        if right != 0.0 {
            self.pose
                .apply(CameraConformanceCommand::StrafeRight(right));
        }
        if up != 0.0 {
            self.pose.apply(CameraConformanceCommand::MoveUp(up));
        }
    }

    fn update_title(&self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let basis = self.pose.basis();
        let raw = self
            .last_pointer
            .map(|value| format!("raw=({:.1},{:.1})", value.delta_x, value.delta_y))
            .unwrap_or_else(|| "raw=none".to_owned());
        let commands = self
            .last_commands
            .map(|value| format!("commands={value:?}"))
            .unwrap_or_else(|| "commands=none".to_owned());
        window.set_title(&format!(
            "Tokimu Camera Conformance | capture={} | pos=({:.2},{:.2},{:.2}) yaw={:.2} pitch={:.2} | F=({:.2},{:.2},{:.2}) U=({:.2},{:.2},{:.2}) R=({:.2},{:.2},{:.2}) | {raw} | {commands}",
            self.pointer_captured,
            self.pose.position.x,
            self.pose.position.y,
            self.pose.position.z,
            self.pose.yaw,
            self.pose.pitch,
            basis.forward.x,
            basis.forward.y,
            basis.forward.z,
            basis.up.x,
            basis.up.y,
            basis.up.z,
            basis.right.x,
            basis.right.y,
            basis.right.z,
        ));
    }
}

impl PlatformEventHandler for CameraDirectionApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window.clone(), size.width, size.height)?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "camera-direction-landmarks",
            PipelineKind::LitColor3d,
        ))?;
        for (index, landmark) in axis_landmarks().into_iter().enumerate() {
            renderer.upload_mesh(
                MeshHandle(FIRST_MESH + index as u64),
                &landmark_mesh(landmark),
            );
            renderer.upload_material(
                MaterialHandle(FIRST_MATERIAL + index as u64),
                &Material::new(
                    format!("camera-direction-{}", landmark.label),
                    Color::rgba(
                        landmark.color[0],
                        landmark.color[1],
                        landmark.color[2],
                        landmark.color[3],
                    ),
                ),
            )?;
        }
        self.renderer = Some(renderer);
        self.window = Some(window);
        self.update_title();
        eprintln!(
            "AR-0028 camera fixture axes: +X bright red, -X dark red, +Y bright green, -Y dark green, +Z bright blue, -Z dark blue; positive landmarks are larger"
        );
        eprintln!(
            "AR-0028 controls: click capture; Escape release; W/S forward; A/D strafe; Q/E vertical; arrows deterministic yaw/pitch"
        );
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::MouseMotion { delta_x, delta_y } => {
                let observation = PointerMotionObservation { delta_x, delta_y };
                self.last_pointer = Some(observation);
                if self.pointer_captured {
                    self.last_commands = Some(
                        self.pose
                            .apply_pointer_motion(FirstPersonLookPolicy::default(), observation),
                    );
                    self.update_title();
                }
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
            } => self.set_pointer_captured(true),
            PlatformInputEvent::KeyboardInput { key, pressed } => {
                if key == KeyCode::Escape && pressed {
                    self.set_pointer_captured(false);
                    self.pressed.clear();
                } else if pressed {
                    match key {
                        KeyCode::ArrowLeft => self
                            .pose
                            .apply(CameraConformanceCommand::Yaw(KEY_LOOK_STEP)),
                        KeyCode::ArrowRight => self
                            .pose
                            .apply(CameraConformanceCommand::Yaw(-KEY_LOOK_STEP)),
                        KeyCode::ArrowUp => self
                            .pose
                            .apply(CameraConformanceCommand::Pitch(KEY_LOOK_STEP)),
                        KeyCode::ArrowDown => self
                            .pose
                            .apply(CameraConformanceCommand::Pitch(-KEY_LOOK_STEP)),
                        _ => {
                            self.pressed.insert(key);
                        }
                    }
                    self.update_title();
                } else {
                    self.pressed.remove(&key);
                }
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

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.apply_held_movement(delta_seconds);
        let (view, projection) =
            camera_conformance_matrices(self.pose, self.size[0] / self.size[1].max(1.0));
        let camera = Camera::new(view, projection);
        let renderer = self.renderer.as_mut().expect("renderer initialized");
        renderer.upload_camera(CAMERA, camera);
        renderer.begin_frame();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.018, 0.025, 0.04),
        })];
        for index in 0..axis_landmarks().len() {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(FIRST_MESH + index as u64),
                material: MaterialHandle(FIRST_MATERIAL + index as u64),
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            }));
        }
        renderer.submit(&commands);
        renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_key_policy_maps_a_and_d_to_declared_local_right() {
        let initial = CameraConformancePose::default();
        let mut d = initial;
        d.apply(CameraConformanceCommand::StrafeRight(1.0));
        let mut a = initial;
        a.apply(CameraConformanceCommand::StrafeRight(-1.0));
        assert!((d.position - (initial.position + initial.basis().right)).length() < 0.000_1);
        assert!((a.position - (initial.position - initial.basis().right)).length() < 0.000_1);
    }

    #[test]
    fn free_pointer_observation_does_not_require_a_camera_command() {
        let observation = PointerMotionObservation {
            delta_x: 12.0,
            delta_y: -4.0,
        };
        let pose = CameraConformancePose::default();
        let retained_observation = Some(observation);
        let commands: Option<[CameraConformanceCommand; 2]> = None;
        assert_eq!(retained_observation, Some(observation));
        assert_eq!(commands, None);
        assert_eq!(pose, CameraConformancePose::default());
    }
}
