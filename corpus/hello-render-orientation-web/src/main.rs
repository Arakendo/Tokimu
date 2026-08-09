#[cfg(target_arch = "wasm32")]
use render_orientation_conformance::{
    conformance_pipeline, cull_modes, fixture_cases, fixture_layout,
};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, Material, MaterialHandle,
    MeshHandle, PipelineHandle, RenderCommand, Renderer, ViewportRect, WgpuBackend,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsCast};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, HtmlCanvasElement};

#[cfg(target_arch = "wasm32")]
const CAMERA: CameraHandle = CameraHandle(1);
#[cfg(target_arch = "wasm32")]
const MATERIAL: MaterialHandle = MaterialHandle(1);
#[cfg(target_arch = "wasm32")]
const FIRST_MESH: u64 = 1;

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
    set_status("Tokimu WebGPU | meshes-ready; uploading material");
    renderer
        .upload_material(
            MATERIAL,
            &Material::new("orientation-fixture-material", Color::rgb(1.0, 1.0, 1.0)),
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
