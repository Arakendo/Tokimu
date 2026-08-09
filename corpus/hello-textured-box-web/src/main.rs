#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

#[cfg(target_arch = "wasm32")]
use gltf_corpus::decode_glb;
#[cfg(target_arch = "wasm32")]
use raster_image_corpus::{decode_png, prepare_renderer_texture, DecodeLimits, TextureUse};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    Camera, CameraHandle, ClearCommand, Color, CullMode, DrawMeshCommand, Instance2d, Material,
    MaterialHandle, Mesh, MeshHandle, Pipeline, PipelineHandle, PipelineKind, PipelineRenderState,
    RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle,
    WgpuBackend,
};
use tokimu::{TextureAddressMode, TextureFilter, TextureSampler};
#[cfg(target_arch = "wasm32")]
use tokimu_core::math::{Mat4, Vec3};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, HtmlCanvasElement, KeyboardEvent};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("hello-textured-box-web is a browser/WASM corpus consumer");
}
#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
const GRID_PNG: &[u8] = include_bytes!("../../assets/PNG/Dark/texture_01.png");
#[cfg(target_arch = "wasm32")]
const DARK_DOOR_PNG: &[u8] = include_bytes!("../../assets/PNG/Dark/texture_11.png");
#[cfg(target_arch = "wasm32")]
const GREEN_DOOR_PNG: &[u8] = include_bytes!("../../assets/PNG/Green/texture_11.png");
#[cfg(target_arch = "wasm32")]
const BOX_GLB: &[u8] = include_bytes!("../../../third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb");
#[cfg(target_arch = "wasm32")]
const MESH: MeshHandle = MeshHandle(1);
#[cfg(target_arch = "wasm32")]
const MATERIAL: MaterialHandle = MaterialHandle(1);
#[cfg(target_arch = "wasm32")]
const CAMERA: CameraHandle = CameraHandle(1);
const ADDRESSING_UV_SCALE: f32 = 3.25;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
struct TextureFixture {
    label: &'static str,
    handle: TextureHandle,
    bytes: &'static [u8],
}

#[cfg(target_arch = "wasm32")]
const TEXTURES: [TextureFixture; 3] = [
    TextureFixture {
        label: "grid",
        handle: TextureHandle(1),
        bytes: GRID_PNG,
    },
    TextureFixture {
        label: "door-dark",
        handle: TextureHandle(2),
        bytes: DARK_DOOR_PNG,
    },
    TextureFixture {
        label: "door-green",
        handle: TextureHandle(3),
        bytes: GREEN_DOOR_PNG,
    },
];

#[derive(Clone, Copy, Default)]
enum SamplerMode {
    #[default]
    PointClamp,
    PointRepeat,
    LinearClamp,
    LinearRepeat,
}

impl SamplerMode {
    fn next(self) -> Self {
        match self {
            Self::PointClamp => Self::PointRepeat,
            Self::PointRepeat => Self::LinearClamp,
            Self::LinearClamp => Self::LinearRepeat,
            Self::LinearRepeat => Self::PointClamp,
        }
    }

    fn sampler(self) -> TextureSampler {
        match self {
            Self::PointClamp => TextureSampler::default(),
            Self::PointRepeat => TextureSampler {
                filter: TextureFilter::Point,
                address_u: TextureAddressMode::Repeat,
                address_v: TextureAddressMode::Repeat,
            },
            Self::LinearClamp => TextureSampler {
                filter: TextureFilter::Linear,
                address_u: TextureAddressMode::Clamp,
                address_v: TextureAddressMode::Clamp,
            },
            Self::LinearRepeat => TextureSampler {
                filter: TextureFilter::Linear,
                address_u: TextureAddressMode::Repeat,
                address_v: TextureAddressMode::Repeat,
            },
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::PointClamp => "point clamp",
            Self::PointRepeat => "point repeat",
            Self::LinearClamp => "linear clamp",
            Self::LinearRepeat => "linear repeat",
        }
    }
}

#[derive(Clone, Copy, Default)]
enum UvMode {
    #[default]
    Identity,
    FlipU,
    SwapUv,
}

impl UvMode {
    fn next(self) -> Self {
        match self {
            Self::Identity => Self::FlipU,
            Self::FlipU => Self::SwapUv,
            Self::SwapUv => Self::Identity,
        }
    }

    fn apply(self, [u, v]: [f32; 2]) -> [f32; 2] {
        match self {
            Self::Identity => [u, v],
            Self::FlipU => [ADDRESSING_UV_SCALE - u, v],
            Self::SwapUv => [v, u],
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Identity => "uv identity",
            Self::FlipU => "uv flip-u",
            Self::SwapUv => "uv swap",
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserApp {
    renderer: WgpuBackend,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    pipeline: PipelineHandle,
    viewport: [f32; 2],
    texture_index: usize,
    sampler_mode: SamplerMode,
    uv_mode: UvMode,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_fixture() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let canvas = window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("textured-box-canvas"))
        .ok_or_else(|| JsValue::from_str("textured-box-canvas is unavailable"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("textured-box-canvas is not a canvas"))?;
    set_status("initializing Tokimu WebGPU renderer");
    spawn_local(async move {
        match initialize(canvas).await {
            Ok((app, adapter)) => {
                let app = Rc::new(RefCell::new(app));
                if let Err(error) = install_controls(app.clone()) {
                    set_status(&format!("failed | {error}"));
                    set_state("failed");
                } else {
                    set_status(&format!(
                        "ready | WebGPU adapter: {adapter} | {} | M texture; R sampler; X UV",
                        app.borrow().selection_label()
                    ));
                    set_state("ready");
                }
            }
            Err(error) => {
                set_status(&format!("failed | {error}"));
                set_state("failed");
            }
        }
    });
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn initialize(canvas: HtmlCanvasElement) -> Result<(BrowserApp, String), String> {
    let mut renderer = WgpuBackend::for_window(
        canvas.clone(),
        canvas.width().max(1),
        canvas.height().max(1),
    )
    .await
    .map_err(|e| e.to_string())?;
    let model = decode_glb(BOX_GLB).map_err(|e| e.to_string())?;
    let primitive = model.primitives.first().ok_or("Box has no primitive")?;
    let mut positions = Vec::with_capacity(primitive.indices.len());
    let mut normals = Vec::with_capacity(primitive.indices.len());
    for &index in &primitive.indices {
        let index = index as usize;
        positions.push(
            *primitive
                .positions
                .get(index)
                .ok_or("Box index outside positions")?,
        );
        normals.push(
            *primitive
                .normals
                .get(index)
                .ok_or("Box index outside normals")?,
        );
    }
    for fixture in TEXTURES {
        let decoded =
            decode_png(fixture.bytes, DecodeLimits::default()).map_err(|e| e.to_string())?;
        let prepared =
            prepare_renderer_texture(&decoded, TextureUse::ColorSrgb).map_err(|e| e.to_string())?;
        renderer
            .create_texture_rgba8(
                fixture.handle,
                Rgba8TextureDescriptor::new(
                    prepared.texture.width,
                    prepared.texture.height,
                    Rgba8TextureColorSpace::Srgb,
                ),
                &prepared.texture.rgba8,
            )
            .map_err(|e| e.to_string())?;
    }
    let pipeline = renderer
        .register_pipeline(
            &Pipeline::new("textured-box-web", PipelineKind::Textured3d)
                .with_render_state(PipelineRenderState {
                    cull_mode: CullMode::Back,
                    ..PipelineRenderState::depth_writing_3d()
                })
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    let adapter = renderer.adapter_name().to_owned();
    let mut app = BrowserApp {
        renderer,
        positions,
        normals,
        pipeline,
        viewport: [canvas.width().max(1) as f32, canvas.height().max(1) as f32],
        texture_index: 0,
        sampler_mode: SamplerMode::default(),
        uv_mode: UvMode::default(),
    };
    app.redraw()?;
    Ok((app, adapter))
}

#[cfg(target_arch = "wasm32")]
impl BrowserApp {
    fn redraw(&mut self) -> Result<(), String> {
        let uvs = self
            .positions
            .iter()
            .zip(&self.normals)
            .map(|(&position, &normal)| self.uv_mode.apply(planar_uv(position, normal)))
            .collect();
        let mesh = Mesh::new(self.positions.clone(), self.normals.clone())
            .with_texture_coordinates(uvs)
            .map_err(|e| e.to_string())?;
        let fixture = TEXTURES[self.texture_index];
        self.renderer.upload_mesh(MESH, &mesh);
        self.renderer
            .upload_material(
                MATERIAL,
                &Material::new(
                    format!("textured-box-{}", fixture.label),
                    Color::rgb(1.0, 1.0, 1.0),
                )
                .with_texture(fixture.handle)
                .with_texture_sampler(self.sampler_mode.sampler()),
            )
            .map_err(|e| e.to_string())?;
        let mut camera = Camera::perspective_3d(self.viewport[0], self.viewport[1]);
        camera.view = Mat4::look_at_rh(Vec3::new(2.8, 1.8, 2.8), Vec3::ZERO, Vec3::Y);
        self.renderer.upload_camera(CAMERA, camera);
        self.renderer.begin_frame();
        self.renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.03, 0.04, 0.06),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MESH,
                material: MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            }),
        ]);
        self.renderer.present().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn handle_key(&mut self, key: &str) -> Result<bool, String> {
        match key.to_ascii_lowercase().as_str() {
            "m" => self.texture_index = (self.texture_index + 1) % TEXTURES.len(),
            "r" => self.sampler_mode = self.sampler_mode.next(),
            "x" => self.uv_mode = self.uv_mode.next(),
            _ => return Ok(false),
        }
        self.redraw()?;
        let fixture = TEXTURES[self.texture_index];
        set_status(&format!(
            "ready | {} | M texture; R sampler; X UV",
            self.selection_label()
        ));
        Ok(true)
    }

    fn selection_label(&self) -> String {
        format!(
            "{} | {} | {}",
            TEXTURES[self.texture_index].label,
            self.sampler_mode.label(),
            self.uv_mode.label()
        )
    }
}

#[cfg(target_arch = "wasm32")]
fn install_controls(app: Rc<RefCell<BrowserApp>>) -> Result<(), String> {
    let window = window().ok_or("browser window is unavailable")?;
    let handler = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        match app.borrow_mut().handle_key(&event.key()) {
            Ok(true) => event.prevent_default(),
            Ok(false) => {}
            Err(error) => {
                set_status(&format!("failed | {error}"));
                set_state("failed");
            }
        }
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window
        .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())
        .map_err(|error| format!("could not register keyboard control: {error:?}"))?;
    handler.forget();
    Ok(())
}

fn planar_uv([x, y, z]: [f32; 3], normal: [f32; 3]) -> [f32; 2] {
    let uv = if normal[2].abs() > 0.5 {
        [x + 0.5, 0.5 - y]
    } else if normal[0].abs() > 0.5 {
        [z + 0.5, 0.5 - y]
    } else {
        [x + 0.5, z + 0.5]
    };
    [uv[0] * ADDRESSING_UV_SCALE, uv[1] * ADDRESSING_UV_SCALE]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_cycle_retains_the_four_declared_generic_modes() {
        let point_clamp = SamplerMode::default();
        assert_eq!(point_clamp.label(), "point clamp");
        assert_eq!(point_clamp.sampler(), TextureSampler::default());

        let point_repeat = point_clamp.next();
        assert_eq!(point_repeat.label(), "point repeat");
        assert_eq!(
            point_repeat.sampler(),
            TextureSampler {
                filter: TextureFilter::Point,
                address_u: TextureAddressMode::Repeat,
                address_v: TextureAddressMode::Repeat,
            }
        );

        let linear_clamp = point_repeat.next();
        assert_eq!(linear_clamp.label(), "linear clamp");
        assert_eq!(linear_clamp.sampler().filter, TextureFilter::Linear);
        assert_eq!(linear_clamp.sampler().address_u, TextureAddressMode::Clamp);

        let linear_repeat = linear_clamp.next();
        assert_eq!(linear_repeat.label(), "linear repeat");
        assert_eq!(
            linear_repeat.sampler().address_v,
            TextureAddressMode::Repeat
        );
        assert_eq!(linear_repeat.next().label(), "point clamp");
    }

    #[test]
    fn uv_modes_transform_the_same_corpus_coordinates_deterministically() {
        let coordinates = [0.5, 1.25];
        assert_eq!(UvMode::Identity.apply(coordinates), coordinates);
        assert_eq!(UvMode::FlipU.apply(coordinates), [2.75, 1.25]);
        assert_eq!(UvMode::SwapUv.apply(coordinates), [1.25, 0.5]);
        assert_eq!(UvMode::SwapUv.next().label(), "uv identity");
    }

    #[test]
    fn planar_mapping_intentionally_exceeds_the_unit_interval() {
        assert_eq!(planar_uv([0.5, -0.5, 0.0], [0.0, 0.0, 1.0]), [3.25, 3.25]);
        assert!(planar_uv([0.5, -0.5, 0.0], [0.0, 0.0, 1.0])
            .iter()
            .any(|coordinate| *coordinate > 1.0));
    }
}

#[cfg(target_arch = "wasm32")]
fn set_status(message: &str) {
    if let Some(e) = window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("status"))
    {
        e.set_text_content(Some(message));
    }
}

#[cfg(target_arch = "wasm32")]
fn set_state(state: &str) {
    if let Some(e) = window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = e.set_attribute("data-state", state);
    }
}
