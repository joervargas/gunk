pub mod vulkan_context;
pub mod vulkan_loader;
pub mod vk_macros;
pub mod vk_utils;

use self::{vulkan_loader::VulkanLoader, vulkan_context::VulkanContext};

use super::renderer_utils;


pub struct VulkanRenderer
{
    pub loader: VulkanLoader,
    pub ctx: VulkanContext,

}

impl VulkanRenderer
{
    pub fn new() -> Self
    {
        todo!()
    }

}

impl renderer_utils::GfxRenderer for VulkanRenderer
{
    fn init(&self) 
    {
        todo!()    
    }

    fn destroy(&self) 
    {
        todo!()    
    }

    fn update(&mut self) 
    {
        todo!()    
    }

    fn render(&self) 
    {
        todo!()    
    }
}