use ash::{self, vk};

use super::sp_vulkan::splunk_vk_context::SpVkContext;

pub trait SpVkRenderLayer
{
    fn fill_command_buffers(spvk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: &u32);
}