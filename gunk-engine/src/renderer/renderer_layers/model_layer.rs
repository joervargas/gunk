use crate::renderer::{
    texture::{self, Texture}, 
    wgpu_utils::*,
};

use super::layer::RendererLayer;

pub struct ModelLayer
{
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub shader: wgpu::ShaderModule,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    pub texture: texture::Texture,
}

impl ModelLayer
{
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_info: &WgpuSurfaceInfo, camera_bind_layout: &[&wgpu::BindGroupLayout]) -> Self
    {
        let diffuse_bytes = include_bytes!("../../assets/textures/happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, Some("happy-tree.png")).unwrap();
        
        let bind_group_entries = [
            wgpu::BindGroupLayoutEntry
            {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture 
                    { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                count: None,
            },
            wgpu::BindGroupLayoutEntry
            {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];
        let bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor
        {
            label: Some("texture_bind_group_layout"),
            entries: &bind_group_entries
        };
        let texture_bind_group_layout = device.create_bind_group_layout(&bind_group_layout_desc);
        let bind_group_entries = [
            wgpu::BindGroupEntry
            {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            },
            wgpu::BindGroupEntry
            {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            }
        ];
        let bind_group_desc = wgpu::BindGroupDescriptor
        {
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &bind_group_entries,
        };
        let diffuse_bind_group = device.create_bind_group(&bind_group_desc);

        // order is important
        // first is group 0, second is group 1, ...
        let model_layer_bind_group_layouts = [ camera_bind_layout, &[&texture_bind_group_layout] ].concat();
        
        let render_pipeline_layout = create_wgpu_pipelinelayout(&device, model_layer_bind_group_layouts.as_slice(), &[]);
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/shader.wgsl"));
        let render_pipeline = create_wgpu_render_pipeline(
            &device, Some("model pipeline"), 
            &surface_info.configuration,
            &render_pipeline_layout, 
            &shader, 
            &[Vertex::desc()],
            Some(Texture::get_depth_stencil_state())
        );

        let vertex_buffer = create_wgpu_buffer::<Vertex>(&device, "Vertex Buffer", wgpu::BufferUsages::VERTEX, VERTICES);
        let index_buffer = create_wgpu_buffer::<u32>(&device, "Index Buffer", wgpu::BufferUsages::INDEX, INDICES);

        Self
        {
            render_pipeline,
            bind_group: diffuse_bind_group,
            shader,
            vertex_buffer,
            index_buffer,
            texture: diffuse_texture,
        }
    }
}

impl RendererLayer for ModelLayer
{
    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) 
    {}

    fn update(&mut self) 
    {
        todo!()
    }

    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, surface_view: &wgpu::TextureView, depth_texture_view: Option<&wgpu::TextureView>, camera_bind_group: &wgpu::BindGroup) -> Result<(), wgpu::SurfaceError>
    {
        let color_attachment = wgpu::RenderPassColorAttachment
        {
            view: &surface_view,
            resolve_target: None,
            ops: wgpu::Operations
            {
                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                store: true
            }
        };
        let depth_stencil_attachment = if let Some(depth_img) = depth_texture_view
        {
            Some(wgpu::RenderPassDepthStencilAttachment
            {
                view: depth_img,
                depth_ops: Some(wgpu::Operations{ load: wgpu::LoadOp::Clear(1.0), store: true }),
                stencil_ops: None,
            })
        }
        else{ None };

        let renderpass_desc = wgpu::RenderPassDescriptor
        {
            label: Some("Model Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: depth_stencil_attachment,
        };

        let mut render_pass = encoder.begin_render_pass(&renderpass_desc);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // render_pass.draw(0..VERTICES.len() as u32, 0..1);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

        Ok(())
    }
}