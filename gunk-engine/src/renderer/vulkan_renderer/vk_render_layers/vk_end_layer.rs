use ash::{self, vk};

use crate::renderer::vulkan_renderer::sp_vulkan::{
    splunk_vk_context::{SpVkContext, sp_create_vk_color_depth_framebuffers, sp_create_vk_color_only_framebuffers, sp_destroy_vk_framebuffers}, 
    splunk_vk_render_pass::{
        SpVkRenderPass, ERenderPassBit, SpVkRenderPassInfo, sp_create_vk_renderpass, sp_destroy_vk_renderpass
    }, 
    splunk_vk_img::SpVkImage
};

use super::sp_vk_render_layer::SpVkLayerDraw;

pub struct VkEndLayer
{
    renderpass:     SpVkRenderPass,
    framebuffers:   Vec<vk::Framebuffer>
}

impl VkEndLayer
{
    pub fn new(instance: &ash::Instance, vk_ctx: &SpVkContext, depth_img: Option<&SpVkImage>) -> Self
    {
        let renderpass_info = SpVkRenderPassInfo{
            b_use_color: true,
            b_clear_color: false,
            b_use_depth: depth_img.is_some(),
            b_clear_depth: false,
            color_format: vk_ctx.swapchain.format,
            flags: ERenderPassBit::LAST,
            samples: vk::SampleCountFlags::TYPE_1
        };
        let renderpass = sp_create_vk_renderpass(instance, vk_ctx, renderpass_info);

        let framebuffers = if depth_img.is_some()
        {
            sp_create_vk_color_depth_framebuffers(vk_ctx, &renderpass, &depth_img.unwrap().view)
        } else {
            sp_create_vk_color_only_framebuffers(vk_ctx, &renderpass)
        };

        Self{ renderpass, framebuffers }
    }
}

impl SpVkLayerDraw for VkEndLayer
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: usize)
    {
        self.begin_renderpass(vk_ctx, cmd_buffer, &self.renderpass, vk::Pipeline::null(), self.framebuffers[current_image]);
        self.end_renderpass(vk_ctx, cmd_buffer);
    }

    fn destroy(&mut self, vk_ctx: &mut SpVkContext) 
    {
        unsafe{
            for framebuffer in self.framebuffers.iter()
            {
                vk_ctx.device.destroy_framebuffer(*framebuffer, None);
            }
        }
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