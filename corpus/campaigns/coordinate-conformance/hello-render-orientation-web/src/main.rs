#[cfg(target_arch = "wasm32")]
use render_orientation_conformance::{
    axis_landmarks, camera_conformance_matrices, conformance_pipeline, cull_modes,
    directional_atlas_rgba8, fixture_cases, fixture_layout, landmark_mesh,
    CameraConformanceCommand, CameraConformancePose, FirstPersonLookPolicy,
    PointerMotionObservation, DIRECTIONAL_ATLAS_HEIGHT, DIRECTIONAL_ATLAS_WIDTH,
};
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, Material, MaterialHandle,
    MeshHandle, Pipeline, PipelineHandle, PipelineKind, RenderCommand, Renderer,
    Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle, ViewportRect, WgpuBackend,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsCast};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, HtmlCanvasElement};
#[cfg(target_arch = "wasm32")]
use web_sys::{KeyboardEvent, MouseEvent};

#[cfg(target_arch = "wasm32")]
const CAMERA: CameraHandle = CameraHandle(1);
#[cfg(target_arch = "wasm32")]
const MATERIAL: MaterialHandle = MaterialHandle(1);
#[cfg(target_arch = "wasm32")]
const DIRECTIONAL_ATLAS: TextureHandle = TextureHandle(1);
#[cfg(target_arch = "wasm32")]
const FIRST_MESH: u64 = 1;

#[cfg(target_arch = "wasm32")]
const CAMERA_LANDMARK_FIRST_MESH: u64 = 100;
#[cfg(target_arch = "wasm32")]
const CAMERA_LANDMARK_FIRST_MATERIAL: u64 = 100;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("hello-render-orientation-web is a browser/WASM corpus consumer");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_fixture() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window = window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("browser document is unavailable"))?;
    let canvas = document
        .get_element_by_id("orientation-canvas")
        .ok_or_else(|| JsValue::from_str("orientation-canvas is unavailable"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("orientation-canvas is not a canvas element"))?;
    set_status("initializing Tokimu WebGPU renderer");

    spawn_local(async move {
        match render_fixture(canvas).await {
            Ok(adapter) => {
                set_status(&format!("ready | WebGPU adapter: {adapter}"));
                set_document_state("ready");
            }
            Err(error) => {
                set_status(&format!("failed | {error}"));
                set_document_state("failed");
            }
        }
    });
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn render_fixture(canvas: HtmlCanvasElement) -> Result<String, String> {
    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut renderer = WgpuBackend::for_window(canvas, width, height)
        .await
        .map_err(|error| error.to_string())?;
    set_status("Tokimu WebGPU | provider-ready; uploading fixture meshes");
    let adapter = renderer.adapter_name().to_owned();

    for (index, case) in fixture_cases().into_iter().enumerate() {
        renderer.upload_mesh(MeshHandle(FIRST_MESH + index as u64), &case.mesh);
    }
    set_status("Tokimu WebGPU | meshes-ready; uploading directional atlas");
    renderer
        .create_texture_rgba8(
            DIRECTIONAL_ATLAS,
            Rgba8TextureDescriptor::new(
                DIRECTIONAL_ATLAS_WIDTH,
                DIRECTIONAL_ATLAS_HEIGHT,
                Rgba8TextureColorSpace::Srgb,
            ),
            &directional_atlas_rgba8(),
        )
        .map_err(|error| error.to_string())?;
    set_status("Tokimu WebGPU | atlas-ready; uploading material");
    renderer
        .upload_material(
            MATERIAL,
            &Material::new("orientation-fixture-material", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(DIRECTIONAL_ATLAS),
        )
        .map_err(|error| error.to_string())?;
    set_status("Tokimu WebGPU | material-ready; uploading camera");
    renderer.upload_camera(CAMERA, Camera::default());
    set_status("Tokimu WebGPU | camera-ready; registering cull pipelines");
    let pipelines = cull_modes()
        .into_iter()
        .map(|mode| renderer.register_pipeline(&conformance_pipeline(mode)))
        .collect::<Result<Vec<PipelineHandle>, _>>()
        .map_err(|error| error.to_string())?;
    set_status("Tokimu WebGPU | pipelines-ready; building draw commands");

    let mut commands = vec![RenderCommand::Clear(ClearCommand {
        color: Color::rgb(0.025, 0.035, 0.045),
    })];
    for cell in fixture_layout(width, height) {
        commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: MeshHandle(FIRST_MESH + cell.case_index as u64),
            material: MATERIAL,
            pipeline: pipelines[cell.cull_index],
            instance: cell.instance,
            camera: Some(CAMERA),
            viewport: Some(ViewportRect {
                x: cell.viewport[0],
                y: cell.viewport[1],
                width: cell.viewport[2],
                height: cell.viewport[3],
            }),
        }));
    }
    set_status("Tokimu WebGPU | commands-ready; submitting first frame");
    renderer.begin_frame();
    renderer.submit(&commands);
    set_status("Tokimu WebGPU | submitted; presenting first frame");
    renderer.present().map_err(|error| error.to_string())?;
    set_status("Tokimu WebGPU | first frame presented");
    Ok(adapter)
}

#[cfg(target_arch = "wasm32")]
struct BrowserCameraFixture {
    renderer: WgpuBackend,
    pose: CameraConformancePose,
    viewport: [f32; 2],
    pipeline: PipelineHandle,
    pointer_captured: bool,
    last_pointer: Option<PointerMotionObservation>,
    last_commands: Option<[CameraConformanceCommand; 2]>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_camera_fixture() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window = window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("browser document is unavailable"))?;
    let canvas = document
        .get_element_by_id("camera-canvas")
        .ok_or_else(|| JsValue::from_str("camera-canvas is unavailable"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("camera-canvas is not a canvas element"))?;
    set_status("initializing Tokimu browser camera fixture");

    spawn_local(async move {
        match initialize_camera_fixture(canvas.clone()).await {
            Ok((app, adapter)) => {
                if let Err(error) = install_camera_controls(app.clone(), canvas) {
                    set_status(&format!("failed | {error}"));
                    set_document_state("failed");
                    return;
                }
                app.borrow().publish_status(&adapter);
                set_document_state("ready");
            }
            Err(error) => {
                set_status(&format!("failed | {error}"));
                set_document_state("failed");
            }
        }
    });
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn initialize_camera_fixture(
    canvas: HtmlCanvasElement,
) -> Result<(Rc<RefCell<BrowserCameraFixture>>, String), String> {
    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut renderer = WgpuBackend::for_window(canvas, width, height)
        .await
        .map_err(|error| error.to_string())?;
    let adapter = renderer.adapter_name().to_owned();
    let pipeline = renderer
        .register_pipeline(&Pipeline::new(
            "browser-camera-direction-landmarks",
            PipelineKind::LitColor3d,
        ))
        .map_err(|error| error.to_string())?;
    for (index, landmark) in axis_landmarks().into_iter().enumerate() {
        renderer.upload_mesh(
            MeshHandle(CAMERA_LANDMARK_FIRST_MESH + index as u64),
            &landmark_mesh(landmark),
        );
        renderer
            .upload_material(
                MaterialHandle(CAMERA_LANDMARK_FIRST_MATERIAL + index as u64),
                &Material::new(
                    format!("browser-camera-direction-{}", landmark.label),
                    Color::rgba(
                        landmark.color[0],
                        landmark.color[1],
                        landmark.color[2],
                        landmark.color[3],
                    ),
                ),
            )
            .map_err(|error| error.to_string())?;
    }
    let mut fixture = BrowserCameraFixture {
        renderer,
        pose: CameraConformancePose::default(),
        viewport: [width as f32, height as f32],
        pipeline,
        pointer_captured: false,
        last_pointer: None,
        last_commands: None,
    };
    fixture.redraw()?;
    Ok((Rc::new(RefCell::new(fixture)), adapter))
}

#[cfg(target_arch = "wasm32")]
impl BrowserCameraFixture {
    fn apply_command(&mut self, command: CameraConformanceCommand) -> Result<(), String> {
        self.pose.apply(command);
        self.redraw()
    }

    fn redraw(&mut self) -> Result<(), String> {
        let (view, projection) =
            camera_conformance_matrices(self.pose, self.viewport[0] / self.viewport[1].max(1.0));
        let camera = Camera::new(view, projection);
        self.renderer.upload_camera(CAMERA, camera);
        self.renderer.begin_frame();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.018, 0.025, 0.04),
        })];
        for index in 0..axis_landmarks().len() {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(CAMERA_LANDMARK_FIRST_MESH + index as u64),
                material: MaterialHandle(CAMERA_LANDMARK_FIRST_MATERIAL + index as u64),
                pipeline: self.pipeline,
                instance: tokimu::Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            }));
        }
        self.renderer.submit(&commands);
        self.renderer.present().map_err(|error| error.to_string())?;
        Ok(())
    }

    fn publish_status(&self, adapter: &str) {
        let basis = self.pose.basis();
        let raw = self
            .last_pointer
            .map(|value| format!("raw=({:.1},{:.1})", value.delta_x, value.delta_y))
            .unwrap_or_else(|| "raw=none".to_owned());
        let commands = self
            .last_commands
            .map(|value| format!("commands={value:?}"))
            .unwrap_or_else(|| "commands=none".to_owned());
        set_status(&format!(
            "ready | adapter={adapter} | capture={} | pos=({:.2},{:.2},{:.2}) yaw={:.2} pitch={:.2} | F=({:.2},{:.2},{:.2}) U=({:.2},{:.2},{:.2}) R=({:.2},{:.2},{:.2}) | {raw} | {commands}",
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

#[cfg(target_arch = "wasm32")]
fn install_camera_controls(
    app: Rc<RefCell<BrowserCameraFixture>>,
    canvas: HtmlCanvasElement,
) -> Result<(), String> {
    let document = window()
        .and_then(|window| window.document())
        .ok_or("browser document is unavailable")?;

    let capture_canvas = canvas.clone();
    let click = Closure::wrap(Box::new(move |_event: MouseEvent| {
        capture_canvas.request_pointer_lock();
    }) as Box<dyn FnMut(MouseEvent)>);
    canvas
        .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())
        .map_err(|error| format!("could not register click capture: {error:?}"))?;
    click.forget();

    let capture_app = app.clone();
    let capture_document = document.clone();
    let capture_status_adapter = "browser-webgpu".to_owned();
    let pointer_lock_change = Closure::wrap(Box::new(move || {
        let mut app = capture_app.borrow_mut();
        app.pointer_captured = capture_document.pointer_lock_element().is_some();
        app.publish_status(&capture_status_adapter);
    }) as Box<dyn FnMut()>);
    document
        .add_event_listener_with_callback(
            "pointerlockchange",
            pointer_lock_change.as_ref().unchecked_ref(),
        )
        .map_err(|error| format!("could not register pointer-lock state: {error:?}"))?;
    pointer_lock_change.forget();

    let motion_app = app.clone();
    let mouse_move = Closure::wrap(Box::new(move |event: MouseEvent| {
        let mut app = motion_app.borrow_mut();
        let observation = PointerMotionObservation {
            delta_x: event.movement_x() as f32,
            delta_y: event.movement_y() as f32,
        };
        app.last_pointer = Some(observation);
        if app.pointer_captured {
            app.last_commands = Some(
                app.pose
                    .apply_pointer_motion(FirstPersonLookPolicy::default(), observation),
            );
            if let Err(error) = app.redraw() {
                set_status(&format!("failed | {error}"));
                set_document_state("failed");
                return;
            }
        }
        app.publish_status("browser-webgpu");
    }) as Box<dyn FnMut(MouseEvent)>);
    document
        .add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref())
        .map_err(|error| format!("could not register mouse motion: {error:?}"))?;
    mouse_move.forget();

    let keyboard_app = app;
    let keyboard_document = document.clone();
    let key_down = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let command = match event.key().to_ascii_lowercase().as_str() {
            "w" => Some(CameraConformanceCommand::MoveForward(0.25)),
            "s" => Some(CameraConformanceCommand::MoveForward(-0.25)),
            "d" => Some(CameraConformanceCommand::StrafeRight(0.25)),
            "a" => Some(CameraConformanceCommand::StrafeRight(-0.25)),
            "e" => Some(CameraConformanceCommand::MoveUp(0.25)),
            "q" => Some(CameraConformanceCommand::MoveUp(-0.25)),
            "arrowleft" => Some(CameraConformanceCommand::Yaw(0.12)),
            "arrowright" => Some(CameraConformanceCommand::Yaw(-0.12)),
            "arrowup" => Some(CameraConformanceCommand::Pitch(0.12)),
            "arrowdown" => Some(CameraConformanceCommand::Pitch(-0.12)),
            "escape" => {
                keyboard_document.exit_pointer_lock();
                None
            }
            _ => return,
        };
        event.prevent_default();
        let mut app = keyboard_app.borrow_mut();
        if let Some(command) = command {
            if let Err(error) = app.apply_command(command) {
                set_status(&format!("failed | {error}"));
                set_document_state("failed");
                return;
            }
        }
        app.publish_status("browser-webgpu");
    }) as Box<dyn FnMut(KeyboardEvent)>);
    document
        .add_event_listener_with_callback("keydown", key_down.as_ref().unchecked_ref())
        .map_err(|error| format!("could not register keyboard commands: {error:?}"))?;
    key_down.forget();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn set_status(message: &str) {
    if let Some(element) = window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("status"))
    {
        element.set_text_content(Some(message));
    }
}

#[cfg(target_arch = "wasm32")]
fn set_document_state(state: &str) {
    if let Some(root) = window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    {
        let _ = root.set_attribute("data-orientation-state", state);
    }
}
