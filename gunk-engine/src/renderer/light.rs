use crate::renderer::wgpu_utils;


#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform
{
    pub position: [f32; 3],
    pub _padding: u32, // Due to uniforms requiring 16 byte (4 float) spacing
    pub color: [f32; 3],
    pub _padding2: u32 // Due to uniforms requiring 16 byte (4 float) spacing
}

pub struct SceneLight
{
    pub uniform: LightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl SceneLight
{
    pub fn new(device: &wgpu::Device, label: &str, position: [f32; 3], color: [f32; 3]) -> Self
    {
        let uniform = LightUniform{
            position,
            _padding: 0,
            color,
            _padding2: 0
        };

        let buffer = wgpu_utils::create_wgpu_buffer::<LightUniform>(&device, label, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, &[uniform]);

        let layout_entries = [
            wgpu::BindGroupLayoutEntry
            {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }
        ];
        let bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor
        {
            entries: &layout_entries,
            label: None
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_desc);

        let group_entries = [
            wgpu::BindGroupEntry
            {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },
        ];
        let bind_group_desc = wgpu::BindGroupDescriptor
        {
            layout: &bind_group_layout,
            entries: &group_entries,
            label: None,
        };

        let bind_group = device.create_bind_group(&bind_group_desc);

        Self{
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }
}