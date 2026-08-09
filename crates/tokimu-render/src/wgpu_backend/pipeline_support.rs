use crate::{BlendMode, ColorWriteMask, CullMode, DepthTest, PipelineKind, PipelineRenderState};

use super::{GpuVertex, DEPTH_FORMAT};

pub(super) fn create_solid_color_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    instance_bind_group_layout: &wgpu::BindGroupLayout,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    render_state: PipelineRenderState,
) -> wgpu::RenderPipeline {
    create_custom_pipeline(
        device,
        surface_format,
        depth_format,
        material_bind_group_layout,
        instance_bind_group_layout,
        camera_bind_group_layout,
        "tokimu-solid-color-pipeline",
        "tokimu-solid-color-shader",
        PipelineKind::SolidColor2d.default_shader_source().unwrap(),
        "vs_main",
        "fs_main",
        render_state,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_custom_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    instance_bind_group_layout: &wgpu::BindGroupLayout,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    pipeline_label: &str,
    shader_label: &str,
    shader_source: &str,
    vertex_entry_point: &str,
    fragment_entry_point: &str,
    render_state: PipelineRenderState,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_label),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(pipeline_label),
        bind_group_layouts: &[
            material_bind_group_layout,
            instance_bind_group_layout,
            camera_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(pipeline_label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some(vertex_entry_point),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: std::mem::size_of::<[f32; 3]>() as u64,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: (std::mem::size_of::<[f32; 3]>() * 2) as u64,
                        shader_location: 2,
                    },
                ],
            }],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: match render_state.cull_mode {
                CullMode::None => None,
                CullMode::Front => Some(wgpu::Face::Front),
                CullMode::Back => Some(wgpu::Face::Back),
            },
            ..wgpu::PrimitiveState::default()
        },
        depth_stencil: depth_stencil_state(depth_format, render_state),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(fragment_entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: match render_state.blend {
                    BlendMode::Opaque => None,
                    BlendMode::AlphaBlend => Some(wgpu::BlendState::ALPHA_BLENDING),
                    BlendMode::Additive => Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                },
                write_mask: color_write_mask(render_state.color_write),
            })],
        }),
        multiview: None,
        cache: None,
    })
}

fn depth_stencil_state(
    depth_format: wgpu::TextureFormat,
    render_state: PipelineRenderState,
) -> Option<wgpu::DepthStencilState> {
    match render_state.depth_test {
        DepthTest::Disabled => None,
        DepthTest::LessEqual => Some(wgpu::DepthStencilState {
            format: depth_format,
            depth_write_enabled: render_state.depth_write,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
    }
}

fn color_write_mask(mask: ColorWriteMask) -> wgpu::ColorWrites {
    let mut writes = wgpu::ColorWrites::empty();
    if mask.red {
        writes |= wgpu::ColorWrites::RED;
    }
    if mask.green {
        writes |= wgpu::ColorWrites::GREEN;
    }
    if mask.blue {
        writes |= wgpu::ColorWrites::BLUE;
    }
    if mask.alpha {
        writes |= wgpu::ColorWrites::ALPHA;
    }
    writes
}

pub(super) fn create_depth_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tokimu-depth-texture"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    (texture, view)
}

pub(super) fn create_instance_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    uniform_bind_group_layout(device, "tokimu-instance-bind-group-layout")
}

pub(super) fn create_camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    uniform_bind_group_layout(device, "tokimu-camera-bind-group-layout")
}

fn uniform_bind_group_layout(device: &wgpu::Device, label: &str) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

pub(super) fn create_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("tokimu-material-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}
