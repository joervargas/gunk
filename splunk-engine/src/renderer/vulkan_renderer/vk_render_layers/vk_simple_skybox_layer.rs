use std::ffi::CString;

use ash::vk;
use gpu_allocator::MemoryLocation;
use nalgebra_glm as glm;
use memoffset;

use crate::renderer::{
    vulkan_renderer::sp_vulkan::{
        splunk_vk_render_pass::{
            SpVkRenderPass, SpVkRenderPassInfo, ERenderPassBit, 
            sp_create_vk_renderpass, sp_destroy_vk_renderpass
        }, 
        splunk_vk_descriptor::{
            SpVkDescriptor, sp_create_vk_desc_pool, 
            get_vk_desc_set_layout_binding, get_vk_buffer_write_desc_set, 
            get_vk_image_write_desc_set, sp_destroy_vk_descriptor
        }, 
        splunk_vk_buffer::{map_vk_allocation_data, sp_create_vk_array_buffer, sp_create_vk_buffer, sp_destroy_vk_buffer, SpVkBuffer},
        splunk_vk_img::{
            SpVkImage, sp_create_vk_image, create_vk_sampler, sp_destroy_vk_img
        }, 
        splunk_vk_context::{SpVkContext, sp_create_vk_color_depth_framebuffers, sp_destroy_vk_framebuffers}, 
        vk_utils::{
            create_vk_pipeline_layout, create_vk_pipeline_info_vertex_input, 
            create_vk_pipeline_info_assembly, create_vk_pipeline_info_color_blend_attachment, 
            create_vk_pipeline_info_viewport, create_vk_pipeline_info_rasterization, 
            create_vk_pipeline_info_multisample, create_vk_pipeline_info_color_blend, 
            create_vk_pipeline_info_depth_stencil, create_vk_pipeline_info_dynamic_states, 
            create_vk_pipeline_info_tessellation
        }, 
        vk_shader_utils::SpVkShaderModule
    },
    renderer_utils::to_shader_path
};

use crate::{ vk_check, log_info, log_err };

use super::sp_vk_render_layer::{ SpVkLayerDraw, SpVk3dLayerUpdate };

#[derive(Clone)]
pub struct SkyBoxVertex
{
    pub pos:        [f32; 3],
}

impl SkyBoxVertex
{
    pub fn new(pos: glm::Vec3) -> Self
    {
        Self
        {
            pos: [pos.x, pos.y, pos.z],
        }
    }

    fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1]
    {
        [
            vk::VertexInputBindingDescription{
                binding: 0,
                stride: std::mem::size_of::<Self>() as u32,
                input_rate: vk::VertexInputRate::VERTEX
            }
        ]
    }

    fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 1]
    {
        [
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: memoffset::offset_of!(Self, pos) as u32
            }
        ]
    }
}

const SKYBOX_VERTICES_DATA: [SkyBoxVertex; 8] =
[
    SkyBoxVertex{ pos: [-1.0, -1.0,  1.0] },
    SkyBoxVertex{ pos: [ 1.0, -1.0,  1.0] },
    SkyBoxVertex{ pos: [ 1.0,  1.0,  1.0] },
    SkyBoxVertex{ pos: [-1.0,  1.0,  1.0] },

    SkyBoxVertex{ pos: [-1.0, -1.0, -1.0] },
    SkyBoxVertex{ pos: [ 1.0, -1.0, -1.0] },
    SkyBoxVertex{ pos: [ 1.0,  1.0, -1.0] },
    SkyBoxVertex{ pos: [-1.0,  1.0, -1.0] },
];

const SKYBOX_INDICES_DATA: [u32; 36] = [
    // front
    0, 1, 2, 2, 3, 0,
    // right
    1, 5, 6, 6, 2, 1,
    // back
    7, 6, 5, 5, 4, 7,
    // left
    4, 0, 3, 3, 7, 4,
    // bottom
    4, 5, 1, 1, 0, 4,
    // top
    3, 2, 6, 6, 7, 3
];

fn get_z_up_matrix() -> glm::Mat4
{
    let s = 90.0_f32.to_radians().sin();
    let c = 90.0_f32.to_radians().cos();
    glm::Mat4::new(
        1.0, 0.0, 0.0,  0.0,
        0.0,  c,   s,    0.0,
        0.0, -s,   c,   0.0,
        0.0, 0.0, 0.0, 0.0
    )
}

pub struct VkSkyBoxLayer
{
    renderpass:         SpVkRenderPass,
    framebuffers:       Vec<vk::Framebuffer>,
    descriptor:         SpVkDescriptor,
    pipeline_layout:    vk::PipelineLayout,
    pipeline:           vk::Pipeline,
    triangle_verts:     Option<SpVkBuffer>,
    triangle_indices:   Option<SpVkBuffer>,
    texture:            Option<SpVkImage>,
    sampler:            vk::Sampler,
    model_space:        glm::Mat4,
    model_space_buffer: Option<SpVkBuffer>,
}

impl VkSkyBoxLayer
{
    pub fn new(
            instance: &ash::Instance,
            vk_ctx: &mut SpVkContext,
            camera_uniforms: &Vec<SpVkBuffer>,
            depth_img: &SpVkImage,
            texture_file: &std::path::Path
        ) -> Self
    {
        let texture = sp_create_vk_image(vk_ctx, texture_file.to_str().unwrap());
        let sampler = create_vk_sampler(&vk_ctx.device);
        
        let renderpass_info = SpVkRenderPassInfo{
            b_use_color: true,
            b_clear_color: false,
            b_use_depth: true,
            b_clear_depth: false,
            color_format: vk::Format::B8G8R8A8_UNORM,
            flags: ERenderPassBit::NONE,
            samples: vk::SampleCountFlags::TYPE_1
        };
        let renderpass = sp_create_vk_renderpass(instance, vk_ctx, renderpass_info);
        
        let model_space = get_z_up_matrix();
        let model_space_buffer = Some(sp_create_vk_buffer(
            vk_ctx, "Simple_SkyBox_model_space", 
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu, 
            std::mem::size_of::<glm::Mat4>() as vk::DeviceSize
        ));

        let descriptor = Self::create_desc_sets(vk_ctx, camera_uniforms, &texture, &sampler, model_space_buffer.as_ref().unwrap());

        let framebuffers = sp_create_vk_color_depth_framebuffers(vk_ctx, &renderpass, &depth_img.view);

        let pipeline_layout = create_vk_pipeline_layout(&vk_ctx.device, &descriptor.layouts, &Vec::new());

        let mut shader_modules: Vec<SpVkShaderModule> = vec![
            SpVkShaderModule::new(&vk_ctx.device, to_shader_path("Simple3dLayer.vert").as_path()),
            SpVkShaderModule::new(&vk_ctx.device, to_shader_path("Simple3dLayer.frag").as_path())
        ];

        let pipeline = Self::create_pipeline(
            vk_ctx,
            &mut shader_modules, 
            &renderpass, 
            &pipeline_layout, None
        );

        for shader in shader_modules.iter_mut()
        {
            shader.destroy(&vk_ctx.device);
        }

        let triangle_verts = sp_create_vk_array_buffer::<SkyBoxVertex>(vk_ctx, "Sky Verts", vk::BufferUsageFlags::VERTEX_BUFFER, &SKYBOX_VERTICES_DATA.to_vec());
        let triangle_indices = sp_create_vk_array_buffer::<u32>(vk_ctx, "Sky Indices", vk::BufferUsageFlags::INDEX_BUFFER, &SKYBOX_INDICES_DATA.to_vec());

        Self
        {
            renderpass,
            framebuffers,
            descriptor,
            pipeline_layout,
            pipeline,
            triangle_verts: Some(triangle_verts),
            triangle_indices: Some(triangle_indices),
            texture: Some(texture),
            sampler,
            model_space,
            model_space_buffer
        }
    }

    fn create_desc_sets(
            vk_ctx: &SpVkContext,
            camera_uniforms: &Vec<SpVkBuffer>,
            texture: &SpVkImage,
            sampler: &vk::Sampler,
            model_space_buffer: &SpVkBuffer
        ) -> SpVkDescriptor
    {
        let pool = sp_create_vk_desc_pool(vk_ctx, 2, 0, 1);

        let bindings: Vec<vk::DescriptorSetLayoutBinding> = vec![
            get_vk_desc_set_layout_binding(0, vk::DescriptorType::UNIFORM_BUFFER, 1, vk::ShaderStageFlags::VERTEX),
            get_vk_desc_set_layout_binding(1, vk::DescriptorType::UNIFORM_BUFFER, 1, vk::ShaderStageFlags::VERTEX),
            get_vk_desc_set_layout_binding(2, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 1, vk::ShaderStageFlags::FRAGMENT)
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo
        {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr()
        };
        let layout = unsafe{
            vk_check!(vk_ctx.device.create_descriptor_set_layout(&layout_info, None)).unwrap()
        };

        let layouts: Vec<vk::DescriptorSetLayout> = vec![layout; vk_ctx.frame_sync.get_num_frames_in_flight()];
        let alloc_info = vk::DescriptorSetAllocateInfo
        {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            descriptor_pool: pool,
            descriptor_set_count: layouts.len() as u32,
            p_set_layouts: layouts.as_ptr()
        };

        let sets = unsafe {
            vk_check!(vk_ctx.device.allocate_descriptor_sets(&alloc_info)).unwrap()
        };

        for i in 0..vk_ctx.frame_sync.get_num_frames_in_flight()
        {
            let buffer_info1 = vk::DescriptorBufferInfo{ buffer: camera_uniforms[i].handle, offset: 0, range: camera_uniforms[i].size };
            let buffer_info2 = vk::DescriptorBufferInfo{ buffer: model_space_buffer.handle, offset: 0, range: model_space_buffer.size };
            let image_info1 = vk::DescriptorImageInfo{ sampler: *sampler, image_view: texture.view, image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL};

            let desc_writes: Vec<vk::WriteDescriptorSet> = vec![
                get_vk_buffer_write_desc_set(&sets[i], &[buffer_info1], 0, vk::DescriptorType::UNIFORM_BUFFER),
                get_vk_buffer_write_desc_set(&sets[i], &[buffer_info2], 1, vk::DescriptorType::UNIFORM_BUFFER),
                get_vk_image_write_desc_set(&sets[i], &[image_info1], 2) 
            ];

            unsafe {
                vk_ctx.device.update_descriptor_sets(desc_writes.as_slice(), &[])
            }
        }   

        SpVkDescriptor
        { 
            layouts, 
            pool, 
            sets 
        }
    }

    fn create_pipeline(
            vk_ctx: &SpVkContext,
            shader_modules: &mut Vec<SpVkShaderModule>,
            renderpass: &SpVkRenderPass,
            layout: &vk::PipelineLayout,
            custom_extent: Option<vk::Extent2D>
        ) -> vk::Pipeline    
    {
        log_info!("creating VkSimple2dLayer pipeline... ");

        let mut shader_stage_infos: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();
        let entry_point = CString::new("main").unwrap();
        for shader in shader_modules.iter()
        {
            shader_stage_infos.push(shader.get_vk_pipeline_info_shader_stage(&entry_point));
        }

        let mut vertex_input_info = create_vk_pipeline_info_vertex_input();
        vertex_input_info.vertex_binding_description_count = SkyBoxVertex::get_binding_descriptions().len() as u32;
        vertex_input_info.p_vertex_binding_descriptions = SkyBoxVertex::get_binding_descriptions().as_ptr();
        vertex_input_info.vertex_attribute_description_count = SkyBoxVertex::get_attribute_descriptions().len() as u32;
        vertex_input_info.p_vertex_attribute_descriptions = SkyBoxVertex::get_attribute_descriptions().as_ptr();

        let assembly_info = create_vk_pipeline_info_assembly(vk::PrimitiveTopology::TRIANGLE_LIST, vk::FALSE);

        let mut custom_width = 0;
        let mut custom_height = 0;
        if custom_extent.is_some()
        {
            custom_width = custom_extent.as_ref().unwrap().width;
            custom_height = custom_extent.as_ref().unwrap().height;
        }
        let viewports: Vec<vk::Viewport> = vec![
            vk::Viewport
            {
                x: 0.0,
                y: 0.0,
                width: if custom_width > 0 { custom_width as f32 } else { vk_ctx.swapchain.extent.width as f32 },
                height: if custom_height > 0 { custom_height as f32 } else { vk_ctx.swapchain.extent.height as f32 },
                min_depth: 0.0,
                max_depth: 1.0
            }
        ];
        let scissors: Vec<vk::Rect2D> = vec![
            vk::Rect2D
            {
                offset: vk::Offset2D{ x: 0, y: 0 },
                extent: 
                    vk::Extent2D
                    { 
                        width: if custom_width > 0 { custom_width } else { vk_ctx.swapchain.extent.width },
                        height: if custom_height > 0 { custom_height } else { vk_ctx.swapchain.extent.height }
                    }
            }
        ];
        let viewport_info = create_vk_pipeline_info_viewport(viewports, scissors);        

        let rasterizer_info = create_vk_pipeline_info_rasterization(vk::PolygonMode::FILL, vk::CullModeFlags::BACK, vk::FrontFace::COUNTER_CLOCKWISE, 1.0);
        let multisampling_info = create_vk_pipeline_info_multisample(vk::SampleCountFlags::TYPE_1, vk::FALSE, 1.0);
        
        let color_attachments: Vec<vk::PipelineColorBlendAttachmentState> = vec![
            create_vk_pipeline_info_color_blend_attachment(true)
        ];
        let color_blending_info = create_vk_pipeline_info_color_blend(&color_attachments);
        
        let depth_stencil_info = create_vk_pipeline_info_depth_stencil();

        let dynamic_states: Vec<vk::DynamicState> = vec![
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR
        ];
        let dynamic_info = create_vk_pipeline_info_dynamic_states(&dynamic_states);

        let tessellation_info = create_vk_pipeline_info_tessellation(0);

        let create_info = vk::GraphicsPipelineCreateInfo
        {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stage_infos.len() as u32,
            p_stages: shader_stage_infos.as_ptr(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &assembly_info,
            p_viewport_state: &viewport_info,
            p_rasterization_state: &rasterizer_info,
            p_multisample_state: &multisampling_info,
            p_color_blend_state: &color_blending_info,
            p_depth_stencil_state: &depth_stencil_info,
            p_dynamic_state: &dynamic_info,
            p_tessellation_state: &tessellation_info,
            layout: *layout,
            render_pass: renderpass.handle,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1
        };

        let pipeline = unsafe {
            vk_ctx.device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None).map_err(|e| { log_err!("{}", e.1); }).unwrap()[0]
        };
        log_info!("VkSimple2dLayer pipeline created.");

        pipeline
    }

    fn draw(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer)
    {
        unsafe{
            vk_ctx.device.cmd_bind_vertex_buffers(*cmd_buffer, 0, &[self.triangle_verts.as_ref().unwrap().handle], &[0 as vk::DeviceSize]);
            vk_ctx.device.cmd_bind_index_buffer(*cmd_buffer, self.triangle_indices.as_ref().unwrap().handle, 0, vk::IndexType::UINT32);

            let desc_set = [self.descriptor.sets[vk_ctx.frame_sync.get_current_frame_index()]];
            vk_ctx.device.cmd_bind_descriptor_sets(*cmd_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &desc_set, &[]);

            // vk_ctx.device.cmd_draw(*cmd_buffer, VERTICES_DATA.len() as u32, 1, 0, 0);
            vk_ctx.device.cmd_draw_indexed(*cmd_buffer, 0 as u32, 1, 0, 0, 0);
        }
    }

}

impl SpVkLayerDraw for VkSkyBoxLayer
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: usize)
    {
        self.begin_renderpass(vk_ctx, cmd_buffer, &self.renderpass, self.pipeline, self.framebuffers[current_image]);
        self.draw(vk_ctx, cmd_buffer);
        self.end_renderpass(vk_ctx, cmd_buffer);
    }

    fn destroy(&mut self, vk_ctx: &mut SpVkContext) 
    {
        sp_destroy_vk_buffer(vk_ctx, self.triangle_verts.take().unwrap());
        sp_destroy_vk_buffer(vk_ctx, self.triangle_indices.take().unwrap());
        sp_destroy_vk_img(vk_ctx, self.texture.take().unwrap());
        unsafe { vk_ctx.device.destroy_sampler(self.sampler, None); }

        sp_destroy_vk_descriptor(vk_ctx, &self.descriptor);
        
        self.cleanup_framebuffers(&vk_ctx.device);
        sp_destroy_vk_renderpass(vk_ctx, &self.renderpass);
        unsafe {
            vk_ctx.device.destroy_pipeline_layout(self.pipeline_layout, None);
            vk_ctx.device.destroy_pipeline(self.pipeline, None);
        }
    }

    fn cleanup_framebuffers(&mut self, device: &ash::Device)
    {
        sp_destroy_vk_framebuffers(device, &mut self.framebuffers);   
    }

    fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext, depth_img: Option<&SpVkImage>)
    {
        self.framebuffers = sp_create_vk_color_depth_framebuffers(vk_ctx, &self.renderpass, &depth_img.unwrap().view);
    }

}

impl SpVk3dLayerUpdate for VkSkyBoxLayer
{
    fn update(&mut self, _vk_ctx: &SpVkContext, _transform_uniform: &SpVkBuffer, _delta_time: f32)
    {
        // update
        map_vk_allocation_data::<glm::Mat4>(&self.model_space_buffer.as_ref().unwrap().allocation, &[self.model_space], 1);
    }
}