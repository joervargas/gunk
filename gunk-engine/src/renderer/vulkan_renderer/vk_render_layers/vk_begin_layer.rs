use ash::{self, vk};

use crate::renderer::vulkan_renderer::sp_vulkan::{
    splunk_vk_context::{SpVkContext, sp_create_vk_color_depth_framebuffers, sp_create_vk_color_only_framebuffers, sp_destroy_vk_framebuffers},
    splunk_vk_img::SpVkImage,
    splunk_vk_render_pass::{ SpVkRenderPass, SpVkRenderPassInfo, ERenderPassBit, sp_create_vk_renderpass, sp_destroy_vk_renderpass }
};

use super::sp_vk_render_layer::SpVkLayerDraw;

pub struct VkBeginLayer
{
    pub renderpass: SpVkRenderPass,
    pub framebuffers: Vec<vk::Framebuffer>
}

impl VkBeginLayer
{
    pub fn new(
            instance: &ash::Instance,
            vk_ctx: &mut SpVkContext,
            depth_img: Option<&SpVkImage>
        ) -> Self
    {
        let renderpass_info = SpVkRenderPassInfo{
            b_use_color: true,
            b_clear_color: true,
            b_use_depth: depth_img.is_some(),
            b_clear_depth: depth_img.is_some(),
            color_format: vk_ctx.swapchain.format,
            flags: ERenderPassBit::FIRST,
            samples: vk::SampleCountFlags::TYPE_1,
        };
        let renderpass = sp_create_vk_renderpass(instance, vk_ctx, renderpass_info);

        let framebuffers = if depth_img.is_some()
        {
            sp_create_vk_color_depth_framebuffers(vk_ctx, &renderpass, &depth_img.unwrap().view)
        } else {
            sp_create_vk_color_only_framebuffers(vk_ctx, &renderpass)
        };

        Self
        { 
            renderpass,
            framebuffers
        }
    }
}

impl SpVkLayerDraw for VkBeginLayer
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: usize)
    {
        self.begin_renderpass(vk_ctx, cmd_buffer, &self.renderpass , vk::Pipeline::null(), self.framebuffers[current_image]);
        self.end_renderpass(vk_ctx, cmd_buffer);
    }

    fn destroy(&mut self, vk_ctx: &mut SpVkContext)
    {
        self.cleanup_framebuffers(&vk_ctx.device);
        sp_destroy_vk_renderpass(&vk_ctx, &self.renderpass);
    }

    fn cleanup_framebuffers(&mut self, device: &ash::Device)
    {
        sp_destroy_vk_framebuffers(device, &mut self.framebuffers);
    }

    fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext, depth_img: Option<&SpVkImage>)
    {
        self.framebuffers = if depth_img.is_some()
        {
            sp_create_vk_color_depth_framebuffers(vk_ctx, &self.renderpass, &depth_img.unwrap().view)
        } else {
            sp_create_vk_color_only_framebuffers(vk_ctx, &self.renderpass)
        };
    }
}