use ash::{self, vk};

use crate::renderer::vulkan_renderer::gk_vulkan::{
    gunk_vk_context::{GkVkContext, gk_create_vk_color_depth_framebuffers, gk_create_vk_color_only_framebuffers, gk_destroy_vk_framebuffers},
    gunk_vk_img::GkVkImage,
    gunk_vk_render_pass::{ GkVkRenderPass, GkVkRenderPassInfo, ERenderPassBit, gk_create_vk_renderpass, gk_destroy_vk_renderpass }
};

use super::gk_vk_render_layer::GkVkLayerDraw;

pub struct VkBeginLayer
{
    pub renderpass: GkVkRenderPass,
    pub framebuffers: Vec<vk::Framebuffer>
}

impl VkBeginLayer
{
    pub fn new(
            instance: &ash::Instance,
            vk_ctx: &mut GkVkContext,
            depth_img: Option<&GkVkImage>
        ) -> Self
    {
        let renderpass_info = GkVkRenderPassInfo{
            b_use_color: true,
            b_clear_color: true,
            b_use_depth: depth_img.is_some(),
            b_clear_depth: depth_img.is_some(),
            color_format: vk_ctx.swapchain.format,
            flags: ERenderPassBit::FIRST,
            samples: vk::SampleCountFlags::TYPE_1,
        };
        let renderpass = gk_create_vk_renderpass(instance, vk_ctx, renderpass_info);

        let framebuffers = if depth_img.is_some()
        {
            gk_create_vk_color_depth_framebuffers(vk_ctx, &renderpass, &depth_img.unwrap().view)
        } else {
            gk_create_vk_color_only_framebuffers(vk_ctx, &renderpass)
        };

        Self
        { 
            renderpass,
            framebuffers
        }
    }
}

impl GkVkLayerDraw for VkBeginLayer
{
    fn draw_frame(&self, vk_ctx: &GkVkContext, cmd_buffer: &vk::CommandBuffer, current_image: usize)
    {
        self.begin_renderpass(vk_ctx, cmd_buffer, &self.renderpass , vk::Pipeline::null(), self.framebuffers[current_image]);
        self.end_renderpass(vk_ctx, cmd_buffer);
    }

    fn destroy(&mut self, vk_ctx: &mut GkVkContext)
    {
        self.cleanup_framebuffers(&vk_ctx.device);
        gk_destroy_vk_renderpass(&vk_ctx, &self.renderpass);
    }

    fn cleanup_framebuffers(&mut self, device: &ash::Device)
    {
        gk_destroy_vk_framebuffers(device, &mut self.framebuffers);
    }

    fn recreate_framebuffers(&mut self, vk_ctx: &GkVkContext, depth_img: Option<&GkVkImage>)
    {
        self.framebuffers = if depth_img.is_some()
        {
            gk_create_vk_color_depth_framebuffers(vk_ctx, &self.renderpass, &depth_img.unwrap().view)
        } else {
            gk_create_vk_color_only_framebuffers(vk_ctx, &self.renderpass)
        };
    }
}