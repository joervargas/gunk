use std::ffi::CString;

use ash::{self, vk};

use crate::renderer::renderer_utils::to_shader_path;
use crate::renderer::vulkan_renderer::sp_vulkan::splunk_vk_context::{sp_create_vk_color_only_framebuffers, sp_destroy_vk_framebuffers};
use crate::renderer::vulkan_renderer::sp_vulkan::splunk_vk_img::SpVkImage;
use crate::renderer::vulkan_renderer::sp_vulkan::vk_utils::{
    create_vk_pipeline_info_vertex_input, create_vk_pipeline_info_assembly,
    create_vk_pipeline_info_dynamic_states, create_vk_pipeline_info_viewport, 
    create_vk_pipeline_info_rasterization, create_vk_pipeline_info_multisample, 
    create_vk_pipeline_info_color_blend_attachment, create_vk_pipeline_info_color_blend,
    create_vk_pipeline_info_tessellation, create_vk_pipeline_layout
};
use crate::renderer::vulkan_renderer::sp_vulkan::{
    splunk_vk_context::SpVkContext,
    splunk_vk_render_pass::SpVkRenderPass,
    splunk_vk_render_pass::{SpVkRenderPassInfo, ERenderPassBit, sp_create_vk_renderpass, sp_destroy_vk_renderpass},
    vk_shader_utils::SpVkShaderModule
};

use crate::{log_info, log_err};

use super::sp_vk_render_layer::{SpVkLayerDraw, SpVk2dLayerUpdate};

pub struct VkSimple2dLayer
{
    renderpass:         SpVkRenderPass,
    framebuffers:       Vec<vk::Framebuffer>,
    // descriptor:         SpVkDescriptor,
    pipeline_layout:    vk::PipelineLayout,
    pipeline:           vk::Pipeline,
    // texture:            Option<SpVkImage>,
    // sampler:            vk::Sampler
}

impl VkSimple2dLayer
{
    pub fn new(
            instance: &ash::Instance,
            vk_ctx: &mut SpVkContext
        ) -> Self
    {
        let renderpass_info = SpVkRenderPassInfo{
            b_use_color: true,
            b_clear_color: false,
            b_use_depth: false,
            b_clear_depth: false,
            color_format: vk::Format::B8G8R8A8_UNORM,
            flags: ERenderPassBit::NONE,
            samples: vk::SampleCountFlags::TYPE_1
        };
        let renderpass = sp_create_vk_renderpass(instance, vk_ctx, renderpass_info);
        
        let framebuffers = sp_create_vk_color_only_framebuffers(&vk_ctx, &renderpass);
        let pipeline_layout = create_vk_pipeline_layout(&vk_ctx.device, &Vec::new(), &Vec::new());

        let mut shader_modules: Vec<SpVkShaderModule> = vec![
            SpVkShaderModule::new(&vk_ctx.device, to_shader_path("Simple2dLayer.vert").as_path()),
            SpVkShaderModule::new(&vk_ctx.device, to_shader_path("Simple2dLayer.frag").as_path())
        ];

        let pipeline = Self::create_pipeline(
            vk_ctx,
            &mut shader_modules, 
            &renderpass, 
            &pipeline_layout, None
        );
        
        Self
        {
            renderpass,
            framebuffers,
            pipeline_layout,
            pipeline
        }
    }

    fn create_pipeline(
            vk_ctx: &SpVkContext,
            shader_modules: &mut Vec<SpVkShaderModule>,
            renderpass: &SpVkRenderPass,
            layout: &vk::PipelineLayout,
            _custom_extent: Option<vk::Extent2D>
        ) -> vk::Pipeline    
    {
        log_info!("creating VkSimple2dLayer pipeline... ");

        let mut shader_stage_infos: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();
        let entry_point = CString::new("main").unwrap();
        for shader in shader_modules.iter()
        {
            shader_stage_infos.push(shader.get_vk_pipeline_info_shader_stage(&entry_point));
        }

        let vertex_input_info = create_vk_pipeline_info_vertex_input();
        let assembly_info = create_vk_pipeline_info_assembly(vk::PrimitiveTopology::TRIANGLE_LIST, vk::FALSE);

        let viewports: Vec<vk::Viewport> = vec![
            vk::Viewport
            {
                x: 0.0,
                y: 0.0,
                // width: if custom_width > 0 { custom_width as f32 } else { vk_ctx.swapchain.extent.width as f32 },
                // height: if custom_height > 0 { custom_height as f32 } else { vk_ctx.swapchain.extent.height as f32 },
                width: vk_ctx.swapchain.extent.width as f32,
                height: vk_ctx.swapchain.extent.height as f32,
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
                        // width: if custom_width > 0 { custom_width } else { vk_ctx.swapchain.extent.width },
                        // height: if custom_height > 0 { custom_height } else { vk_ctx.swapchain.extent.height }
                        width: vk_ctx.swapchain.extent.width,
                        height: vk_ctx.swapchain.extent.height
                    }
            }
        ];
        let viewport_info = create_vk_pipeline_info_viewport(viewports, scissors);        

        let rasterizer_info = create_vk_pipeline_info_rasterization(vk::PolygonMode::FILL, vk::CullModeFlags::BACK, vk::FrontFace::CLOCKWISE, 1.0);
        let multisampling_info = create_vk_pipeline_info_multisample(vk::SampleCountFlags::TYPE_1, vk::FALSE, 1.0);
        
        let color_attachments: Vec<vk::PipelineColorBlendAttachmentState> = vec![
            create_vk_pipeline_info_color_blend_attachment(true)
        ];
        let color_blending_info = create_vk_pipeline_info_color_blend(&color_attachments);
        
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
            p_depth_stencil_state: std::ptr::null(),
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

        for shader in shader_modules.iter_mut()
        {
            shader.destroy(&vk_ctx.device);
        }

        pipeline
    }

    fn draw(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer)
    {
        unsafe{
            vk_ctx.device.cmd_draw(*cmd_buffer, 3, 1, 0, 0);
        }
    }

}

impl SpVkLayerDraw for VkSimple2dLayer
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: &u32)
    {
        // self.begin_renderpass(vk_ctx, cmd_buffer, *current_image as usize);
        self.begin_renderpass(vk_ctx, cmd_buffer, self.renderpass.handle, self.pipeline, self.framebuffers[*current_image as usize]);
        self.draw(vk_ctx, cmd_buffer);
        self.end_renderpass(vk_ctx, cmd_buffer);
    }

    fn destroy(&mut self, vk_ctx: &mut SpVkContext) 
    {
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

    fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext, _depth_img: Option<&SpVkImage>)
    {
        self.framebuffers = sp_create_vk_color_only_framebuffers(vk_ctx, &self.renderpass);
    }

}

impl SpVk2dLayerUpdate for VkSimple2dLayer
{
    fn update(&self, _vk_ctx: &SpVkContext, _current_img: u32)
    {
    }

}