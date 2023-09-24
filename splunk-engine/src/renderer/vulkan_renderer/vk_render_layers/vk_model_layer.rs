use ash::{self, vk};

use gpu_allocator::vulkan::Allocation;

use crate::renderer::vulkan_renderer::sp_vulkan::{
    splunk_vk_context::SpVkContext,
    splunk_vk_render_pass::SpVkRenderPass
};

use super::sp_vk_render_layer::SpVkRenderLayer;

pub struct VkModelLayer
{
    vert_buffer_size: usize,
    index_buffer_size: usize,
    renderpass: SpVkRenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    desc_set_layout: vk::DescriptorSetLayout,
    desc_set_pool: vk::DescriptorPool,
    desc_sets: Vec<vk::DescriptorSet>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    storage_buffer: vk::Buffer,
    storage_alloc: Allocation,
}

impl VkModelLayer
{
    pub fn new()
    {
        todo!()
    }

    fn create_desc_sets()
    {
        /// get uniform buffers for transforms from the renderer
        todo!()
    }

    fn create_pipeline(
            vk_ctx: &SpVkContext, 
            renderpass: &SpVkRenderPass,
            desc_sets: &Vec<vk::DescriptorSet>, 
            custom_extent: Option<vk::Extent2D>
        ) -> (vk::Pipeline, vk::PipelineLayout)
    {

        todo!()
    }

    fn begin_renderpass()
    {
        todo!()
    }

    fn end_renderpass()
    {
        todo!()
    }
}

impl SpVkRenderLayer for VkModelLayer
{
    fn draw_frame(&self, spvk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: &u32)
    {
        // begin renderpass
        // draw
        // end renderpass
        todo!()
    }
}