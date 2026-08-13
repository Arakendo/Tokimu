#[cfg(target_arch = "wasm32")]
use hello_render_resource_identity::{
    observe_failure_boundary_fixture, ApplicationMeshRegistry, ExplicitLifecycleLedger,
    FailureObservationCategory, GenerationalMeshRegistry, LogicalMesh,
};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, Instance2d, Material,
    MaterialHandle, Mesh, MeshHandle, Pipeline, PipelineKind, RenderCommand, Renderer, WgpuBackend,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("hello-render-resource-identity-web is a browser/WASM corpus consumer");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

/// Runs the same B/D/E identity pressure in browser WASM, then proves that the
/// browser WGPU provider retains the existing same-handle replacement mechanic.
/// The returned record is fixture evidence, not a public renderer contract.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run_fixture(canvas: HtmlCanvasElement) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    let mut application = ApplicationMeshRegistry::default();
    let application_resource = LogicalMesh::Dynamic(7);
    let application_handle = application.create(application_resource);
    application
        .replace(application_handle, application_resource)
        .map_err(js_debug)?;
    let application_mismatch = application
        .replace(application_handle, LogicalMesh::Dynamic(8))
        .expect_err("different logical identity must be rejected");

    let mut generational = GenerationalMeshRegistry::default();
    let old_generation = generational.create(LogicalMesh::Dynamic(11));
    generational.retire(old_generation).map_err(js_debug)?;
    let new_generation = generational.create(LogicalMesh::Dynamic(12));
    let stale_generation = generational
        .resolve(old_generation)
        .expect_err("retired generation must remain stale after slot reuse");

    let mut explicit = ExplicitLifecycleLedger::default();
    let explicit_handle = MeshHandle(44);
    explicit
        .create(explicit_handle, LogicalMesh::Dynamic(44))
        .map_err(js_debug)?;
    let duplicate_create = explicit
        .create(explicit_handle, LogicalMesh::Dynamic(45))
        .expect_err("duplicate live create must be rejected");
    explicit
        .replace(explicit_handle, LogicalMesh::Dynamic(44))
        .map_err(js_debug)?;
    explicit.retire(explicit_handle).map_err(js_debug)?;
    let missing_after_retire = explicit
        .resolve(explicit_handle)
        .expect_err("retired explicit identity must not resolve");

    let unresolved = observe_failure_boundary_fixture()
        .retained()
        .find(|record| record.category == FailureObservationCategory::ResourceUnresolved)
        .ok_or_else(|| JsValue::from_str("resource-unresolved observation is absent"))?;

    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut renderer = WgpuBackend::for_window(canvas, width, height)
        .await
        .map_err(js_debug)?;
    let backend = renderer.backend_api();
    let device = renderer.device_kind();
    let adapter = renderer.adapter_name().to_owned();
    let mesh = MeshHandle(1);
    renderer.upload_mesh(mesh, &Mesh::triangle());
    renderer.upload_mesh(mesh, &Mesh::diamond());
    renderer
        .upload_material(
            MaterialHandle(1),
            &Material::new("resource-identity-replacement", Color::rgb(0.1, 0.9, 0.7)),
        )
        .map_err(js_debug)?;
    renderer.upload_camera(CameraHandle(1), Camera::default());
    let pipeline = renderer
        .register_pipeline(&Pipeline::new(
            "resource-identity-browser",
            PipelineKind::LitColor3d,
        ))
        .map_err(js_debug)?;
    renderer.begin_frame();
    renderer.submit(&[
        RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.01, 0.015, 0.02),
        }),
        RenderCommand::DrawMesh(DrawMeshCommand {
            mesh,
            material: MaterialHandle(1),
            pipeline,
            instance: Instance2d::identity(),
            camera: Some(CameraHandle(1)),
            viewport: None,
        }),
    ]);
    let stats = renderer.present().map_err(js_debug)?;

    Ok(format!(
        "status=presented; alternatives=B,D,E; application_handle={}; application_mismatch={application_mismatch:?}; application_counts={:?}; generational_old={}:{}; generational_new={}:{}; stale={stale_generation:?}; generational_counts={:?}; explicit_handle={}; duplicate={duplicate_create:?}; missing_after_retire={missing_after_retire:?}; explicit_counts={:?}; observation=category:{:?},resource:{:?},caller:{}; provider_same_handle_uploads={}; provider_same_handle_replacements={}; draws={}; backend={backend}; device={device}; adapter={adapter}; canvas={}x{}; host=DOM-after-provider-return",
        application_handle.0,
        application.counts(),
        old_generation.slot,
        old_generation.generation,
        new_generation.slot,
        new_generation.generation,
        generational.counts(),
        explicit_handle.0,
        explicit.counts(),
        unresolved.category,
        unresolved.resource,
        unresolved.caller,
        stats.lifetime.mesh_uploads,
        stats.lifetime.mesh_replacements,
        stats.frame.draw_calls,
        width,
        height,
    ))
}

#[cfg(target_arch = "wasm32")]
fn js_debug(error: impl std::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}
