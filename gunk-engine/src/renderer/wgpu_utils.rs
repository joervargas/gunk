
use wgpu::{util::DeviceExt, VertexBufferLayout};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex
{
    pub position: [f32; 3],
    // pub color: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex
{
    pub fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout { 
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Vertex, 
            attributes: &[
                wgpu::VertexAttribute
                {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute
                {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ], 
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641] }, // E
];

pub const INDICES: &[u32] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];


pub fn create_wgpu_instance() -> wgpu::Instance
{
    let instance_descriptor = wgpu::InstanceDescriptor
    {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    };
    wgpu::Instance::new(instance_descriptor)
}

pub fn create_wgpu_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter
{
    let adapter_options = wgpu::RequestAdapterOptions
    {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(surface),
        force_fallback_adapter: false,
    };
    pollster::block_on(instance.request_adapter(&adapter_options)).unwrap()
}

pub fn request_wgpu_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue)
{
    let device_descriptor = wgpu::DeviceDescriptor
    {
        features: wgpu::Features::empty(),
        limits: if cfg!(target_arch="wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        },
        label: None,
    };
    pollster::block_on(adapter.request_device(&device_descriptor, None)).unwrap()
}

pub struct WgpuSurfaceInfo
{
    pub format: wgpu::TextureFormat,
    pub capabilities: wgpu::SurfaceCapabilities,
    pub configuration: wgpu::SurfaceConfiguration,
}

pub fn set_wgpu_surface_info(surface: &wgpu::Surface, adapter: &wgpu::Adapter, surface_width: u32, surface_height: u32) -> WgpuSurfaceInfo
{
    let surface_caps = surface.get_capabilities(adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let surface_config = wgpu::SurfaceConfiguration
    {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: surface_width,
        height: surface_height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![]
    };

    WgpuSurfaceInfo { format: surface_format, capabilities: surface_caps, configuration: surface_config }
}

pub fn create_wgpu_pipelinelayout(device: &wgpu::Device, bind_group_layouts: &[&wgpu::BindGroupLayout], push_constant_ranges: &[wgpu::PushConstantRange]) -> wgpu::PipelineLayout
{
    let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor
    {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: bind_group_layouts,
        push_constant_ranges: push_constant_ranges,
    };
    device.create_pipeline_layout(&pipeline_layout_desc)
}

pub fn create_wgpu_render_pipeline(
        device: &wgpu::Device, 
        label: Option<&str>,
        surface_config: &wgpu::SurfaceConfiguration,
        pipeline_layout: &wgpu::PipelineLayout, 
        shader_module: &wgpu::ShaderModule,
        vertex_buffer_layouts: &[VertexBufferLayout<'_>],
        depth_stencil_state: Option<wgpu::DepthStencilState>
    ) -> wgpu::RenderPipeline
{
    let frag_targets = [
        Some(wgpu::ColorTargetState 
            { 
                format: surface_config.format, 
                blend: Some(wgpu::BlendState::REPLACE), 
                write_mask: wgpu::ColorWrites::ALL 
            })
    ];
    let render_pipeline_desc = wgpu::RenderPipelineDescriptor
        {
            label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState
                {
                    module: &shader_module,
                    entry_point: "vs_main",
                    // buffers: &[Vertex::desc()],
                    buffers: vertex_buffer_layouts,
                },
            fragment: Some(wgpu::FragmentState
                {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &frag_targets,
                }),
            primitive: wgpu::PrimitiveState
                {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
            depth_stencil: depth_stencil_state,
            multisample: wgpu::MultisampleState
                {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            multiview: None,
        };
    device.create_render_pipeline(&render_pipeline_desc)
}


pub fn create_wgpu_buffer<T>(device: &wgpu::Device, label: &str, usage: wgpu::BufferUsages, data: &[T]) -> wgpu::Buffer
    where T: bytemuck::Pod,
{
    let buffer_descriptor = wgpu::util::BufferInitDescriptor
    {
        label: Some(label),
        contents: bytemuck::cast_slice(data),
        usage: usage
    };
    device.create_buffer_init(&buffer_descriptor)
}

