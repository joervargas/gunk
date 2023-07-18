use std::ops::Range;

use crate::renderer::{ 
    wgpu_utils, 
    model::{Mesh, Model}, 
};


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

pub trait DrawLight<'a>
{
    fn draw_light_mesh(
        &mut self, 
        mesh: &'a Mesh, 
        camera_bind_group: &'a wgpu::BindGroup, 
        light_bind_group: &'a wgpu::BindGroup
    );

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        instances: Range<u32>
    );

    fn draw_light_model(
        &mut self, 
        mesh: &'a Model, 
        camera_bind_group: &'a wgpu::BindGroup, 
        light_bind_group: &'a wgpu::BindGroup
    );

    fn draw_light_model_instanced(
        &mut self,
        mesh: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        instances: Range<u32>
    );
}

impl<'a, 'b> DrawLight<'b> for wgpu::RenderPass<'a>
where
    'b: 'a
{
    fn draw_light_mesh(
            &mut self, 
            mesh: &'b Mesh, 
            camera_bind_group: &'b wgpu::BindGroup, 
            light_bind_group: &'b wgpu::BindGroup
        ) 
    {
        self.draw_light_mesh_instanced(mesh, camera_bind_group, light_bind_group, 0..1);
    }

    fn draw_light_mesh_instanced(
            &mut self,
            mesh: &'b Mesh,
            camera_bind_group: &'b wgpu::BindGroup,
            light_bind_group: &'b wgpu::BindGroup,
            instances: Range<u32>
        ) 
    {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(
            &mut self, 
            model: &'b Model, 
            camera_bind_group: &'b wgpu::BindGroup, 
            light_bind_group: &'b wgpu::BindGroup
        ) 
    {
        self.draw_light_model_instanced(model, camera_bind_group, light_bind_group, 0..1);
    }

    fn draw_light_model_instanced(
            &mut self,
            model: &'b Model,
            camera_bind_group: &'b wgpu::BindGroup,
            light_bind_group: &'b wgpu::BindGroup,
            instances: Range<u32>
        ) 
    {
        for mesh in &model.meshes
        {
            self.draw_light_mesh_instanced(mesh, camera_bind_group, light_bind_group, instances.clone());
        }
    }
}