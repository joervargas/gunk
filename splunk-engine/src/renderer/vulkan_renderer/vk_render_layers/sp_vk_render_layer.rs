use ash::{self, vk};

use crate::renderer::vulkan_renderer::sp_vulkan::splunk_vk_context::SpVkContext;

pub trait SpVkRenderLayer
{
    fn draw_frame(&self, spvk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: &u32);
}

