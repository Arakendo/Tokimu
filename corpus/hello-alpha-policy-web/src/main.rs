#[cfg(target_arch = "wasm32")]
use hello_alpha_policy::{
    blend_shader_source, fixtures, interaction_manifest_fingerprint, FixtureId,
    BLEND_BACKGROUND_DEPTH, BLEND_FAR_DEPTH, BLEND_FAR_OFFSET, BLEND_NEAR_DEPTH, BLEND_NEAR_OFFSET,
    BLEND_PANELS, BLEND_PANEL_SCALE, BLEND_REFERENCE_DEPTH, BLEND_REFERENCE_TRANSLATION,
    INTERACTION_BACKGROUND_DEPTH, INTERACTION_BLEND_LEFT_DEPTH, INTERACTION_BLEND_RIGHT_DEPTH,
    INTERACTION_FOREGROUND_DEPTH, INTERACTION_PANELS, INTERACTION_PANEL_SCALE, INTERIOR_THRESHOLD,
    VIEWPORT, VISUAL_BACKGROUND_DEPTH, VISUAL_DEPTH_SCALE, VISUAL_DEPTH_TRANSLATION,
    VISUAL_FOREGROUND_DEPTH, VISUAL_PROFILE_SCALE, VISUAL_PROFILE_TRANSLATIONS,
};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color, CullMode,
    CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand, Instance2d, Material,
    MaterialHandle, Mesh, MeshHandle, Pipeline, PipelineHandle, PipelineKind, PipelineRenderState,
    RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle,
    WgpuBackend,
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
const MIXED_TEXTURE: TextureHandle = TextureHandle(1);
#[cfg(target_arch = "wasm32")]
const BINARY_TEXTURE: TextureHandle = TextureHandle(2);
#[cfg(target_arch = "wasm32")]
const MIXED_MATERIAL: MaterialHandle = MaterialHandle(1);
#[cfg(target_arch = "wasm32")]
const BINARY_MATERIAL: MaterialHandle = MaterialHandle(2);
#[cfg(target_arch = "wasm32")]
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(3);
#[cfg(target_arch = "wasm32")]
const GRADIENT_TEXTURE: TextureHandle = TextureHandle(3);
#[cfg(target_arch = "wasm32")]
const RED_BLEND_MATERIAL: MaterialHandle = MaterialHandle(4);
#[cfg(target_arch = "wasm32")]
const GREEN_BLEND_MATERIAL: MaterialHandle = MaterialHandle(5);
#[cfg(target_arch = "wasm32")]
const GRADIENT_BLEND_MATERIAL: MaterialHandle = MaterialHandle(6);
#[cfg(target_arch = "wasm32")]
const FAR_BLEND_MESH: MeshHandle = MeshHandle(6);
#[cfg(target_arch = "wasm32")]
const NEAR_BLEND_MESH: MeshHandle = MeshHandle(7);
#[cfg(target_arch = "wasm32")]
const BACKGROUND_BLEND_MESH: MeshHandle = MeshHandle(8);
#[cfg(target_arch = "wasm32")]
const REFERENCE_BLEND_MESH: MeshHandle = MeshHandle(9);
#[cfg(target_arch = "wasm32")]
const GRADIENT_BLEND_MESH: MeshHandle = MeshHandle(10);
#[cfg(target_arch = "wasm32")]
const INTERACTION_BACKGROUND_MESH: MeshHandle = MeshHandle(11);
#[cfg(target_arch = "wasm32")]
const INTERACTION_CUTOUT_MESH: MeshHandle = MeshHandle(12);
#[cfg(target_arch = "wasm32")]
const INTERACTION_BLEND_MESH: MeshHandle = MeshHandle(13);
#[cfg(target_arch = "wasm32")]
const INTERACTION_SLOPED_BLEND_MESH: MeshHandle = MeshHandle(14);

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("hello-alpha-policy-web is a browser/WASM corpus consumer");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_fixture(threshold_variant: String) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let canvas = window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("alpha-policy-canvas"))
        .ok_or_else(|| JsValue::from_str("alpha-policy-canvas is unavailable"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("alpha-policy-canvas is not a canvas element"))?;
    let threshold =
        selected_threshold(&threshold_variant).map_err(|error| JsValue::from_str(&error))?;
    set_status(&format!(
        "initializing Tokimu WebGPU provider | threshold={threshold:.7}"
    ));
    spawn_local(async move {
        match render_fixture(canvas, threshold).await {
            Ok(observation) => {
                set_status(&format!("ready | {observation}"));
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
#[wasm_bindgen]
pub fn start_blend_fixture() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let canvas = window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("alpha-policy-canvas"))
        .ok_or_else(|| JsValue::from_str("alpha-policy-canvas is unavailable"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("alpha-policy-canvas is not a canvas element"))?;
    set_status("initializing Tokimu WebGPU provider | blend comparison");
    spawn_local(async move {
        match render_blend_fixture(canvas).await {
            Ok(observation) => {
                set_status(&format!("ready | {observation}"));
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
#[wasm_bindgen]
pub fn start_interaction_fixture() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let canvas = window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("alpha-policy-canvas"))
        .ok_or_else(|| JsValue::from_str("alpha-policy-canvas is unavailable"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("alpha-policy-canvas is not a canvas element"))?;
    set_status("initializing Tokimu WebGPU provider | Slice 4 interaction comparison");
    spawn_local(async move {
        match render_interaction_fixture(canvas).await {
            Ok(observation) => {
                set_status(&format!("ready | {observation}"));
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
async fn render_fixture(canvas: HtmlCanvasElement, threshold: f32) -> Result<String, String> {
    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut renderer = WgpuBackend::for_window(canvas, width, height)
        .await
        .map_err(|error| error.to_string())?;
    set_status("Tokimu WebGPU | provider-ready; uploading exact RGBA8 fixtures");

    upload_fixture_texture(&mut renderer, MIXED_TEXTURE, FixtureId::MixedAlpha)?;
    upload_fixture_texture(&mut renderer, BINARY_TEXTURE, FixtureId::BinaryMask)?;
    renderer
        .upload_material(
            MIXED_MATERIAL,
            &Material::new("alpha-study-mixed", Color::rgba(1.0, 1.0, 1.0, 1.0))
                .with_texture(MIXED_TEXTURE),
        )
        .map_err(|error| error.to_string())?;
    renderer
        .upload_material(
            BINARY_MATERIAL,
            &Material::new("alpha-study-binary", Color::rgba(1.0, 1.0, 1.0, 1.0))
                .with_texture(BINARY_TEXTURE),
        )
        .map_err(|error| error.to_string())?;
    renderer
        .upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("alpha-study-background", Color::rgb(0.15, 0.55, 0.95)),
        )
        .map_err(|error| error.to_string())?;

    let mesh_handles = [
        MeshHandle(1),
        MeshHandle(2),
        MeshHandle(3),
        MeshHandle(4),
        MeshHandle(5),
    ];
    let depths = [
        VISUAL_FOREGROUND_DEPTH,
        VISUAL_FOREGROUND_DEPTH,
        VISUAL_FOREGROUND_DEPTH,
        VISUAL_BACKGROUND_DEPTH,
        VISUAL_FOREGROUND_DEPTH,
    ];
    for (handle, depth) in mesh_handles.into_iter().zip(depths) {
        renderer.upload_mesh(handle, &quad_at_depth(depth));
    }
    renderer.upload_camera(
        CAMERA,
        Camera::orthographic_2d_with_height(width as f32, height as f32, 2.0),
    );

    let state = PipelineRenderState {
        blend: BlendMode::Opaque,
        depth_test: DepthTest::LessEqual,
        depth_write: true,
        cull_mode: CullMode::None,
        color_write: Default::default(),
    };
    let opaque = renderer
        .register_pipeline(
            &Pipeline::new("alpha-study-opaque", PipelineKind::Textured3d)
                .with_render_state(state)
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let below = renderer
        .register_pipeline(&Pipeline::textured_3d_cutout(
            "alpha-study-cutout-below",
            CategoricalCutout::new(
                CutoutThreshold::new(threshold).map_err(|error| error.to_string())?,
                CutoutComparison::DiscardBelow,
            ),
        ))
        .map_err(|error| error.to_string())?;
    let at_or_below = renderer
        .register_pipeline(&Pipeline::textured_3d_cutout(
            "alpha-study-cutout-at-or-below",
            CategoricalCutout::new(
                CutoutThreshold::new(threshold).map_err(|error| error.to_string())?,
                CutoutComparison::DiscardAtOrBelow,
            ),
        ))
        .map_err(|error| error.to_string())?;

    set_status("Tokimu WebGPU | resources-ready; submitting fixed comparison");
    renderer.begin_frame();
    renderer.submit(&[
        RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.02, 0.025),
        }),
        draw(
            mesh_handles[0],
            MIXED_MATERIAL,
            opaque,
            VISUAL_PROFILE_TRANSLATIONS[0],
            VISUAL_PROFILE_SCALE,
        ),
        draw(
            mesh_handles[1],
            MIXED_MATERIAL,
            below,
            VISUAL_PROFILE_TRANSLATIONS[1],
            VISUAL_PROFILE_SCALE,
        ),
        draw(
            mesh_handles[2],
            MIXED_MATERIAL,
            at_or_below,
            VISUAL_PROFILE_TRANSLATIONS[2],
            VISUAL_PROFILE_SCALE,
        ),
        draw(
            mesh_handles[3],
            BACKGROUND_MATERIAL,
            opaque,
            VISUAL_DEPTH_TRANSLATION,
            VISUAL_DEPTH_SCALE,
        ),
        draw(
            mesh_handles[4],
            BINARY_MATERIAL,
            below,
            VISUAL_DEPTH_TRANSLATION,
            VISUAL_DEPTH_SCALE,
        ),
    ]);
    set_status("Tokimu WebGPU | submitted; presenting first frame");
    renderer.present().map_err(|error| error.to_string())?;
    Ok(format!(
        "first frame presented | backend={} | device={} | adapter={} | viewport={}x{} | threshold={:.7}",
        renderer.backend_api(),
        renderer.device_kind(),
        renderer.adapter_name(),
        width,
        height,
        threshold,
    ))
}

#[cfg(target_arch = "wasm32")]
async fn render_blend_fixture(canvas: HtmlCanvasElement) -> Result<String, String> {
    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut renderer = WgpuBackend::for_window(canvas, width, height)
        .await
        .map_err(|error| error.to_string())?;
    require_invalid_pipeline_state_rejection()?;
    set_status("Tokimu WebGPU | provider-ready; uploading fixed blend fixtures");

    upload_fixture_texture(&mut renderer, MIXED_TEXTURE, FixtureId::MixedAlpha)?;
    upload_fixture_texture(
        &mut renderer,
        GRADIENT_TEXTURE,
        FixtureId::ContinuousGradient,
    )?;
    for (handle, label, color) in [
        (
            RED_BLEND_MATERIAL,
            "alpha-study-red-blend",
            Color::rgba(1.0, 0.2, 0.2, 1.0),
        ),
        (
            GREEN_BLEND_MATERIAL,
            "alpha-study-green-blend",
            Color::rgba(0.2, 1.0, 0.3, 1.0),
        ),
    ] {
        renderer
            .upload_material(
                handle,
                &Material::new(label, color).with_texture(MIXED_TEXTURE),
            )
            .map_err(|error| error.to_string())?;
    }
    renderer
        .upload_material(
            GRADIENT_BLEND_MATERIAL,
            &Material::new(
                "alpha-study-continuous-gradient-blend",
                Color::rgb(1.0, 1.0, 1.0),
            )
            .with_texture(GRADIENT_TEXTURE),
        )
        .map_err(|error| error.to_string())?;
    renderer
        .upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("alpha-study-blue-blend-backing", Color::rgb(0.1, 0.3, 0.95)),
        )
        .map_err(|error| error.to_string())?;

    for (handle, depth) in [
        (FAR_BLEND_MESH, BLEND_FAR_DEPTH),
        (NEAR_BLEND_MESH, BLEND_NEAR_DEPTH),
        (BACKGROUND_BLEND_MESH, BLEND_BACKGROUND_DEPTH),
        (REFERENCE_BLEND_MESH, BLEND_REFERENCE_DEPTH),
        (GRADIENT_BLEND_MESH, BLEND_REFERENCE_DEPTH),
    ] {
        renderer.upload_mesh(handle, &quad_at_depth(depth));
    }
    renderer.upload_camera(
        CAMERA,
        Camera::orthographic_2d_with_height(width as f32, height as f32, 2.0),
    );

    let alpha_no_depth_state = PipelineRenderState {
        blend: BlendMode::AlphaBlend,
        depth_test: DepthTest::LessEqual,
        depth_write: false,
        cull_mode: CullMode::None,
        color_write: Default::default(),
    };
    let alpha_depth_state = PipelineRenderState {
        depth_write: true,
        ..alpha_no_depth_state
    };
    let opaque_state = PipelineRenderState {
        blend: BlendMode::Opaque,
        depth_test: DepthTest::LessEqual,
        depth_write: true,
        cull_mode: CullMode::None,
        color_write: Default::default(),
    };
    let alpha_no_depth = renderer
        .register_pipeline(
            &Pipeline::custom_wgsl("alpha-study-web-blend-no-depth", blend_shader_source())
                .with_render_state(alpha_no_depth_state)
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let alpha_depth = renderer
        .register_pipeline(
            &Pipeline::custom_wgsl("alpha-study-web-blend-depth", blend_shader_source())
                .with_render_state(alpha_depth_state)
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let opaque = renderer
        .register_pipeline(
            &Pipeline::new("alpha-study-web-opaque-backing", PipelineKind::Textured3d)
                .with_render_state(opaque_state)
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let solid_reference = renderer
        .register_pipeline(&Pipeline::new(
            "alpha-study-web-solid-reference",
            PipelineKind::SolidColor2d,
        ))
        .map_err(|error| error.to_string())?;

    set_status("Tokimu WebGPU | resources-ready; submitting fixed blend comparison");
    renderer.begin_frame();
    let commands = [
        RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.02, 0.025),
        }),
        draw(
            REFERENCE_BLEND_MESH,
            BACKGROUND_MATERIAL,
            solid_reference,
            BLEND_REFERENCE_TRANSLATION,
            BLEND_PANEL_SCALE,
        ),
        draw(
            GRADIENT_BLEND_MESH,
            GRADIENT_BLEND_MATERIAL,
            alpha_no_depth,
            BLEND_REFERENCE_TRANSLATION,
            BLEND_PANEL_SCALE,
        ),
        draw(
            FAR_BLEND_MESH,
            RED_BLEND_MATERIAL,
            alpha_no_depth,
            at(BLEND_PANELS[0], BLEND_FAR_OFFSET),
            BLEND_PANEL_SCALE,
        ),
        draw(
            NEAR_BLEND_MESH,
            GREEN_BLEND_MATERIAL,
            alpha_no_depth,
            at(BLEND_PANELS[0], BLEND_NEAR_OFFSET),
            BLEND_PANEL_SCALE,
        ),
        draw(
            NEAR_BLEND_MESH,
            GREEN_BLEND_MATERIAL,
            alpha_no_depth,
            at(BLEND_PANELS[1], BLEND_NEAR_OFFSET),
            BLEND_PANEL_SCALE,
        ),
        draw(
            FAR_BLEND_MESH,
            RED_BLEND_MATERIAL,
            alpha_no_depth,
            at(BLEND_PANELS[1], BLEND_FAR_OFFSET),
            BLEND_PANEL_SCALE,
        ),
        draw(
            BACKGROUND_BLEND_MESH,
            BACKGROUND_MATERIAL,
            opaque,
            BLEND_PANELS[2],
            BLEND_PANEL_SCALE,
        ),
        draw(
            NEAR_BLEND_MESH,
            GREEN_BLEND_MATERIAL,
            alpha_no_depth,
            at(BLEND_PANELS[2], BLEND_NEAR_OFFSET),
            BLEND_PANEL_SCALE,
        ),
        draw(
            FAR_BLEND_MESH,
            RED_BLEND_MATERIAL,
            alpha_no_depth,
            at(BLEND_PANELS[2], BLEND_FAR_OFFSET),
            BLEND_PANEL_SCALE,
        ),
        draw(
            BACKGROUND_BLEND_MESH,
            BACKGROUND_MATERIAL,
            opaque,
            BLEND_PANELS[3],
            BLEND_PANEL_SCALE,
        ),
        draw(
            NEAR_BLEND_MESH,
            GREEN_BLEND_MATERIAL,
            alpha_depth,
            at(BLEND_PANELS[3], BLEND_NEAR_OFFSET),
            BLEND_PANEL_SCALE,
        ),
        draw(
            FAR_BLEND_MESH,
            RED_BLEND_MATERIAL,
            alpha_depth,
            at(BLEND_PANELS[3], BLEND_FAR_OFFSET),
            BLEND_PANEL_SCALE,
        ),
    ];
    renderer.submit(&commands);
    set_status("Tokimu WebGPU | submitted; presenting blend frame");
    let first = renderer.present().map_err(|error| error.to_string())?;
    renderer.begin_frame();
    renderer.submit(&commands);
    let warm = renderer.present().map_err(|error| error.to_string())?;
    renderer.poll_diagnostics();
    let diagnostic = renderer
        .drain_diagnostics()
        .into_iter()
        .next()
        .map(|record| record.message)
        .unwrap_or_else(|| "none".to_owned());
    Ok(format!(
        "blend first + warm frame presented | first={} draws/{} materials/{} pipelines/{} binding allocations/{} mesh uploads | warm={} draws/{} materials/{} pipelines/{} binding allocations/{} mesh uploads | diagnostic={diagnostic} | backend={} | device={} | adapter={} | viewport={}x{}",
        first.frame.draw_calls,
        first.frame.material_resolutions,
        first.frame.pipeline_switches,
        first.frame.binding_allocations,
        first.frame.mesh_uploads,
        warm.frame.draw_calls,
        warm.frame.material_resolutions,
        warm.frame.pipeline_switches,
        warm.frame.binding_allocations,
        warm.frame.mesh_uploads,
        renderer.backend_api(),
        renderer.device_kind(),
        renderer.adapter_name(),
        width,
        height,
    ))
}

#[cfg(target_arch = "wasm32")]
async fn render_interaction_fixture(canvas: HtmlCanvasElement) -> Result<String, String> {
    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut renderer = WgpuBackend::for_window(canvas, width, height)
        .await
        .map_err(|error| error.to_string())?;
    set_status("Tokimu WebGPU | provider-ready; uploading Slice 4 interaction fixtures");

    upload_fixture_texture(&mut renderer, MIXED_TEXTURE, FixtureId::MixedAlpha)?;
    upload_fixture_texture(&mut renderer, BINARY_TEXTURE, FixtureId::BinaryMask)?;
    for (handle, label, color, texture) in [
        (
            MIXED_MATERIAL,
            "alpha-study-interaction-mixed",
            Color::rgb(1.0, 1.0, 1.0),
            Some(MIXED_TEXTURE),
        ),
        (
            BINARY_MATERIAL,
            "alpha-study-interaction-binary",
            Color::rgb(1.0, 1.0, 1.0),
            Some(BINARY_TEXTURE),
        ),
        (
            BACKGROUND_MATERIAL,
            "alpha-study-interaction-backing",
            Color::rgb(0.1, 0.3, 0.95),
            None,
        ),
    ] {
        let material = Material::new(label, color);
        let material = match texture {
            Some(texture) => material.with_texture(texture),
            None => material,
        };
        renderer
            .upload_material(handle, &material)
            .map_err(|error| error.to_string())?;
    }
    for (handle, mesh) in [
        (
            INTERACTION_BACKGROUND_MESH,
            quad_at_depth(INTERACTION_BACKGROUND_DEPTH),
        ),
        (
            INTERACTION_CUTOUT_MESH,
            quad_at_depth(INTERACTION_FOREGROUND_DEPTH),
        ),
        (
            INTERACTION_BLEND_MESH,
            quad_at_depth(INTERACTION_FOREGROUND_DEPTH),
        ),
        (
            INTERACTION_SLOPED_BLEND_MESH,
            sloped_quad(INTERACTION_BLEND_LEFT_DEPTH, INTERACTION_BLEND_RIGHT_DEPTH),
        ),
    ] {
        renderer.upload_mesh(handle, &mesh);
    }
    renderer.upload_camera(
        CAMERA,
        Camera::orthographic_2d_with_height(width as f32, height as f32, 2.0),
    );

    let opaque_state = PipelineRenderState {
        blend: BlendMode::Opaque,
        depth_test: DepthTest::LessEqual,
        depth_write: true,
        cull_mode: CullMode::None,
        color_write: Default::default(),
    };
    let blend_no_depth_state = PipelineRenderState {
        blend: BlendMode::AlphaBlend,
        depth_test: DepthTest::LessEqual,
        depth_write: false,
        cull_mode: CullMode::None,
        color_write: Default::default(),
    };
    let blend_depth_state = PipelineRenderState {
        depth_write: true,
        ..blend_no_depth_state
    };
    let opaque = renderer
        .register_pipeline(
            &Pipeline::new(
                "alpha-study-web-interaction-opaque",
                PipelineKind::Textured3d,
            )
            .with_render_state(opaque_state)
            .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let cutout = renderer
        .register_pipeline(&Pipeline::textured_3d_cutout(
            "alpha-study-web-interaction-cutout",
            CategoricalCutout::new(
                CutoutThreshold::new(INTERIOR_THRESHOLD).map_err(|error| error.to_string())?,
                CutoutComparison::DiscardBelow,
            ),
        ))
        .map_err(|error| error.to_string())?;
    let blend_no_depth = renderer
        .register_pipeline(
            &Pipeline::custom_wgsl(
                "alpha-study-web-interaction-blend-no-depth",
                blend_shader_source(),
            )
            .with_render_state(blend_no_depth_state)
            .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let blend_depth = renderer
        .register_pipeline(
            &Pipeline::custom_wgsl(
                "alpha-study-web-interaction-blend-depth",
                blend_shader_source(),
            )
            .with_render_state(blend_depth_state)
            .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

    set_status("Tokimu WebGPU | resources-ready; submitting Slice 4 interaction comparison");
    renderer.begin_frame();
    renderer.submit(&[
        RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.02, 0.025),
        }),
        draw(
            INTERACTION_BACKGROUND_MESH,
            BACKGROUND_MATERIAL,
            opaque,
            INTERACTION_PANELS[0],
            INTERACTION_PANEL_SCALE,
        ),
        draw(
            INTERACTION_CUTOUT_MESH,
            BINARY_MATERIAL,
            cutout,
            INTERACTION_PANELS[0],
            INTERACTION_PANEL_SCALE,
        ),
        draw(
            INTERACTION_BACKGROUND_MESH,
            BACKGROUND_MATERIAL,
            opaque,
            INTERACTION_PANELS[1],
            INTERACTION_PANEL_SCALE,
        ),
        draw(
            INTERACTION_BLEND_MESH,
            MIXED_MATERIAL,
            blend_no_depth,
            INTERACTION_PANELS[1],
            INTERACTION_PANEL_SCALE,
        ),
        draw(
            INTERACTION_BACKGROUND_MESH,
            BACKGROUND_MATERIAL,
            opaque,
            INTERACTION_PANELS[2],
            INTERACTION_PANEL_SCALE,
        ),
        draw(
            INTERACTION_SLOPED_BLEND_MESH,
            MIXED_MATERIAL,
            blend_depth,
            INTERACTION_PANELS[2],
            INTERACTION_PANEL_SCALE,
        ),
        draw(
            INTERACTION_CUTOUT_MESH,
            BINARY_MATERIAL,
            cutout,
            INTERACTION_PANELS[2],
            INTERACTION_PANEL_SCALE,
        ),
    ]);
    set_status("Tokimu WebGPU | submitted; presenting Slice 4 interaction frame");
    let frame = renderer.present().map_err(|error| error.to_string())?;
    renderer.poll_diagnostics();
    let diagnostic = renderer
        .drain_diagnostics()
        .into_iter()
        .next()
        .map(|record| record.message)
        .unwrap_or_else(|| "none".to_owned());
    Ok(format!(
        "interaction first frame presented | {} draws/{} materials/{} pipelines | manifest={} | diagnostic={diagnostic} | backend={} | device={} | adapter={} | viewport={}x{}",
        frame.frame.draw_calls,
        frame.frame.material_resolutions,
        frame.frame.pipeline_switches,
        interaction_manifest_fingerprint().map_err(|error| error.to_string())?,
        renderer.backend_api(),
        renderer.device_kind(),
        renderer.adapter_name(),
        width,
        height,
    ))
}

#[cfg(target_arch = "wasm32")]
fn selected_threshold(variant: &str) -> Result<f32, String> {
    match variant {
        "0" => Ok(0.0),
        "interior" => Ok(INTERIOR_THRESHOLD),
        "1" => Ok(1.0),
        _ => Err("expected browser threshold query 0, interior, or 1".to_owned()),
    }
}

#[cfg(target_arch = "wasm32")]
fn upload_fixture_texture(
    renderer: &mut WgpuBackend,
    handle: TextureHandle,
    id: FixtureId,
) -> Result<(), String> {
    let fixture = fixtures()
        .into_iter()
        .find(|fixture| fixture.id() == id)
        .ok_or_else(|| "shared alpha fixture missing".to_owned())?;
    renderer
        .create_texture_rgba8(
            handle,
            Rgba8TextureDescriptor::new(
                fixture.width(),
                fixture.height(),
                Rgba8TextureColorSpace::Srgb,
            ),
            fixture.rgba8(),
        )
        .map_err(|error| error.to_string())
}

#[cfg(target_arch = "wasm32")]
fn require_invalid_pipeline_state_rejection() -> Result<(), String> {
    let invalid_state = PipelineRenderState {
        depth_test: DepthTest::Disabled,
        depth_write: true,
        ..PipelineRenderState::painter_ordered_2d()
    };
    if Pipeline::new(
        "alpha-study-web-invalid-depth-state",
        PipelineKind::Textured3d,
    )
    .with_render_state(invalid_state)
    .is_ok()
    {
        return Err("invalid blend depth state unexpectedly passed pipeline validation".to_owned());
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn quad_at_depth(depth: f32) -> Mesh {
    let mut mesh = Mesh::quad()
        .with_texture_coordinates(vec![
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ])
        .expect("fixed UV count matches the quad");
    for position in &mut mesh.positions {
        position[2] = depth;
    }
    mesh
}

#[cfg(target_arch = "wasm32")]
fn sloped_quad(left_depth: f32, right_depth: f32) -> Mesh {
    let mut mesh = quad_at_depth(left_depth);
    for position in &mut mesh.positions {
        position[2] = if position[0] < 0.0 {
            left_depth
        } else {
            right_depth
        };
    }
    mesh
}

#[cfg(target_arch = "wasm32")]
fn at(panel: [f32; 2], offset: [f32; 2]) -> [f32; 2] {
    [panel[0] + offset[0], panel[1] + offset[1]]
}

#[cfg(target_arch = "wasm32")]
fn draw(
    mesh: MeshHandle,
    material: MaterialHandle,
    pipeline: PipelineHandle,
    translation: [f32; 2],
    scale: [f32; 2],
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh,
        material,
        pipeline,
        instance: Instance2d::new(translation, scale, 0.0),
        camera: Some(CAMERA),
        viewport: None,
    })
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
        let _ = root.set_attribute("data-alpha-policy-state", state);
    }
}
