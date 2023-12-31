use crate::renderer::{
    texture::Texture, 
    wgpu_utils::*,
    model::{ self, ModelVertex, Vertex, DrawModel},
};
use crate::resource_utils;

use super::layer::RendererLayer;

use nalgebra_glm as glm;

pub struct Batch3DInstance
{
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
}

impl Batch3DInstance
{
    pub fn to_raw(&self) -> Batch3DInstanceRaw
    {
        Batch3DInstanceRaw
        {
            model: (glm::translate(&glm::Mat4::identity(), &self.position) * glm::quat_to_mat4(&self.rotation)).into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Batch3DInstanceRaw
{
    model: [[f32; 4]; 4],
}

impl Batch3DInstanceRaw
{
    fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        use std::mem;
        wgpu::VertexBufferLayout
        {
            array_stride: mem::size_of::<Batch3DInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute
                {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute
                {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute
                {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute
                {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }
    }
}

pub struct Batch3DLayer
{
    pub render_pipeline: wgpu::RenderPipeline,
    pub shader: wgpu::ShaderModule,

    pub instances: Vec<Batch3DInstance>,
    pub instance_buffer: wgpu::Buffer,
    pub obj_model: model::Model
}

const NUM_INSTANCES_PER_ROW: u32 = 10;
// const INSTANCE_DISPLACEMENT: glm::Vec3 = glm::Vec3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

impl Batch3DLayer
{
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_info: &WgpuSurfaceInfo, scene_bind_group_layouts: &[&wgpu::BindGroupLayout]) -> Self
    {

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

        // order is important
        // first is group 0, second is group 1, ...
        let batch3d_layer_bind_group_layouts = [ scene_bind_group_layouts, &[&texture_bind_group_layout] ].concat();

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| 
        {
            (0..NUM_INSTANCES_PER_ROW).map(move |x|
            {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                
                let position = glm::Vec3::new(x, 0.0, z);
                let rotation = if position.eq(&glm::Vec3::new(0.0, 0.0, 0.0))
                {
                    glm::quat_angle_axis(0.0, &glm::Vec3::new(0.0, 0.0, 1.0))
                } 
                else
                {
                    glm::quat_angle_axis(45.0, &position.normalize())
                };
                Batch3DInstance{ position, rotation }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Batch3DInstance::to_raw).collect::<Vec<_>>();
        let instance_buffer = create_wgpu_buffer::<Batch3DInstanceRaw>(device, "Instance Buffer", wgpu::BufferUsages::VERTEX, instance_data.as_slice());

        let render_pipeline_layout = create_wgpu_pipelinelayout(&device, batch3d_layer_bind_group_layouts.as_slice(), &[]);
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/batch3d.wgsl"));
        let render_pipeline = create_wgpu_render_pipeline(
            &device, Some("batch3D pipeline"), 
            &surface_info.configuration, 
            &render_pipeline_layout, 
            &shader, 
            &[ModelVertex::desc(), Batch3DInstanceRaw::desc() ], 
            Some(Texture::get_depth_stencil_state())
        );

        // let obj_model = resource_utils::load_model("", &device, &queue, &texture_bind_group_layout).unwrap();
        let obj_model = resource_utils::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).unwrap();
        // let obj_model = model::Model::new("../../../res/cube.obj", &device, &queue, &texture_bind_group_layout).expect("Failed to create .obj");
        
        Self
        {
            render_pipeline,

            shader,

            instances,
            instance_buffer,
            obj_model
        }
    }
}

impl RendererLayer for Batch3DLayer
{
    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) 
    {}

    fn update(&mut self) 
    {
        todo!()
    }

    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, surface_view: &wgpu::TextureView, depth_texture_view: Option<&wgpu::TextureView>, camera_bind_group: &wgpu::BindGroup, light_bind_group: &wgpu::BindGroup) -> Result<(), wgpu::SurfaceError>
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
            label: Some("Batch 3D Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: depth_stencil_attachment
        };

        let mut render_pass = encoder.begin_render_pass(&renderpass_desc);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        // render_pass.draw_mesh_instanced(&self.obj_model.meshes[0], &self.obj_model.materials[self.obj_model.meshes[0].material], camera_bind_group, 0..self.instances.len() as u32);
        render_pass.draw_model_instanced(&self.obj_model, camera_bind_group, light_bind_group, 0..self.instances.len() as u32);

        Ok(())
    }
}