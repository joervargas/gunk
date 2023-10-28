use ash::{self, vk};

use crate::renderer::vulkan_renderer::sp_vulkan::{splunk_vk_context::SpVkContext, splunk_vk_buffer::SpVkBuffer, splunk_vk_img::SpVkImage, splunk_vk_render_pass::SpVkRenderPass};

pub trait SpVkLayerDraw
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: usize);

    fn destroy(&mut self, vk_ctx: &mut SpVkContext);

    fn cleanup_framebuffers(&mut self, device: &ash::Device);

    fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext, depth_img: Option<&SpVkImage>);

    fn begin_renderpass(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, renderpass: &SpVkRenderPass, pipeline: vk::Pipeline, framebuffer: vk::Framebuffer)
    {
        let mut clear_values: Vec<vk::ClearValue> = Vec::new();
        if renderpass.info.b_clear_color
        {
            clear_values.push(
                vk::ClearValue
                {
                    color: vk::ClearColorValue{ float32: [ 0.0, 0.0, 0.0, 1.0 ] }
                }
            );
        }
        if renderpass.info.b_clear_depth
        {
            clear_values.push(
                vk::ClearValue
                {
                    depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 }
                }
            );
        }

        let screen_rect = vk::Rect2D
        {
            offset: vk::Offset2D{ x: 0, y: 0 },
            extent: vk_ctx.swapchain.extent
        };

        let render_begin_info = vk::RenderPassBeginInfo
        {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: std::ptr::null(),
            render_pass: renderpass.handle,
            framebuffer,
            render_area: screen_rect,
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr()
        };

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

        unsafe
        {
            vk_ctx.device.cmd_begin_render_pass(*cmd_buffer, &render_begin_info, vk::SubpassContents::INLINE);

            if pipeline != vk::Pipeline::null()
            {
                vk_ctx.device.cmd_bind_pipeline(*cmd_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
            }

            vk_ctx.device.cmd_set_viewport(*cmd_buffer, 0, &viewports.as_slice());
            vk_ctx.device.cmd_set_scissor(*cmd_buffer, 0, &scissors.as_slice());

        }
    }

    fn end_renderpass(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer)
    {
        unsafe{ vk_ctx.device.cmd_end_render_pass(*cmd_buffer); }
    }

}

pub trait SpVk3dLayerUpdate
{
    fn update(&self, vk_ctx: &SpVkContext, transform_uniform: &SpVkBuffer, depth_img: &SpVkImage, current_img: usize);

    // fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext, depth_img: &SpVkImage);
}

pub trait VkDrawLayer3d: SpVkLayerDraw + SpVk3dLayerUpdate{}
impl<T: SpVkLayerDraw + SpVk3dLayerUpdate> VkDrawLayer3d for T{}

pub trait SpVk2dLayerUpdate
{
    fn update(&self, vk_ctx: &SpVkContext, current_img: usize);

    // fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext);
}

pub trait VkDrawLayer2d: SpVkLayerDraw + SpVk2dLayerUpdate{}
impl<T: SpVkLayerDraw + SpVk2dLayerUpdate> VkDrawLayer2d for T{}

pub struct Vk3dLayerList
{
    list: Vec<Box<dyn VkDrawLayer3d>>
}

impl Vk3dLayerList
{
    pub fn new() -> Self
    {
        Self{ list: Vec::new() }
    }

    pub fn push(&mut self, layer: Box<dyn VkDrawLayer3d>)
    {
        self.list.push(layer);
    }

    pub fn pop(&mut self) -> Option<Box<dyn VkDrawLayer3d>>
    {
        self.list.pop()
    }
}

impl std::ops::Index<usize> for Vk3dLayerList
{
    type Output = Box<dyn VkDrawLayer3d>;

    fn index(&self, index: usize) -> &Self::Output 
    {
        &self.list[index]
    }
}

impl std::ops::IndexMut<usize> for Vk3dLayerList
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output
    {
        &mut self.list[index]
    }
}

impl SpVkLayerDraw for Vk3dLayerList
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: usize)
    {
        for layer in self.list.iter()
        {
            layer.draw_frame(vk_ctx, cmd_buffer, current_image);
        }
    }

    fn destroy(&mut self, vk_ctx: &mut SpVkContext)
    {
        for layer in self.list.iter_mut()
        {
            layer.destroy(vk_ctx);
        }
    }

    fn cleanup_framebuffers(&mut self, device: &ash::Device)
    {
        for layer in self.list.iter_mut()
        {
            layer.cleanup_framebuffers(device);
        }
    }

    fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext, depth_img: Option<&SpVkImage>)
    {
        for layer in self.list.iter_mut()
        {
            layer.recreate_framebuffers(vk_ctx, depth_img);
        }
    }
}

impl SpVk3dLayerUpdate for Vk3dLayerList
{
    fn update(&self, vk_ctx: &SpVkContext, transform_uniform: &SpVkBuffer, depth_img: &SpVkImage, current_img: usize)
    {
        for layer in self.list.iter()
        {
            layer.update(vk_ctx, transform_uniform, depth_img, current_img);
        }
    }

}


pub struct Vk2dLayerList
{
    list: Vec<Box<dyn VkDrawLayer2d>>
}

impl Vk2dLayerList
{
    pub fn new() -> Self
    {
        Self{ list: Vec::new() }
    }

    pub fn push(&mut self, layer: Box<dyn VkDrawLayer2d>)
    {
        self.list.push(layer);
    }

    pub fn pop(&mut self) -> Option<Box<dyn VkDrawLayer2d>>
    {
        self.list.pop()
    }
}

impl std::ops::Index<usize> for Vk2dLayerList
{
    type Output = Box<dyn VkDrawLayer2d>;

    fn index(&self, index: usize) -> &Self::Output 
    {
        &self.list[index]
    }
}

impl std::ops::IndexMut<usize> for Vk2dLayerList
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output
    {
        &mut self.list[index]
    }
}

impl SpVkLayerDraw for Vk2dLayerList
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: usize)
    {
        for layer in self.list.iter()
        {
            layer.draw_frame(vk_ctx, cmd_buffer, current_image);
        }
    }

    fn destroy(&mut self, vk_ctx: &mut SpVkContext)
    {
        for layer in self.list.iter_mut()
        {
            layer.destroy(vk_ctx);
        }
    }

    fn cleanup_framebuffers(&mut self, device: &ash::Device)
    {
        for layer in self.list.iter_mut()
        {
            layer.cleanup_framebuffers(device);
        }
    }

    fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext, _depth_img: Option<&SpVkImage>)
    {
        for layer in self.list.iter_mut()
        {
            layer.recreate_framebuffers(vk_ctx, None);
        }
    }
}

impl SpVk2dLayerUpdate for Vk2dLayerList
{
    fn update(&self, vk_ctx: &SpVkContext, current_img: usize)
    {
        for layer in self.list.iter()
        {
            layer.update(vk_ctx, current_img);
        }
    }

}