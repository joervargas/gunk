use ash::{self, vk};

use crate::renderer::vulkan_renderer::sp_vulkan::{splunk_vk_context::SpVkContext, splunk_vk_buffer::SpVkBuffer, splunk_vk_img::SpVkImage};

pub trait SpVkLayerDraw
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: &u32);

    fn destroy(&mut self, vk_ctx: &mut SpVkContext);
}

pub trait SpVk3dLayerUpdate
{
    fn update(&self, vk_ctx: &SpVkContext, transform_uniform: &SpVkBuffer, depth_img: &SpVkImage, current_img: u32);
}

pub trait VkDrawLayer3d: SpVkLayerDraw + SpVk3dLayerUpdate{}
impl<T: SpVkLayerDraw + SpVk3dLayerUpdate> VkDrawLayer3d for T{}

pub trait SpVk2dLayerUpdate
{
    fn update(&self, vk_ctx: &SpVkContext, current_img: u32);
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

impl SpVkLayerDraw for Vk3dLayerList
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: &u32)
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
}

impl SpVk3dLayerUpdate for Vk3dLayerList
{
    fn update(&self, vk_ctx: &SpVkContext, transform_uniform: &SpVkBuffer, depth_img: &SpVkImage, current_img: u32)
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

impl SpVkLayerDraw for Vk2dLayerList
{
    fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: &u32)
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
}

impl SpVk2dLayerUpdate for Vk2dLayerList
{
    fn update(&self, vk_ctx: &SpVkContext, current_img: u32)
    {
        for layer in self.list.iter()
        {
            layer.update(vk_ctx, current_img);
        }
    }
}