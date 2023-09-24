use crate::renderer::renderer_utils;

use ash::{self, vk};
use gpu_allocator::vulkan::Allocation;

use super::sp_vulkan::{
    splunk_vk_loader::SpVkLoader, 
    splunk_vk_context::SpVkContext
};
use super::vk_render_layers::sp_vk_render_layer::SpVkRenderLayer;

use winit::window::Window;
use std::ffi::CString;

pub struct VulkanRenderer
{
    pub loader: SpVkLoader,
    pub ctx: SpVkContext,
    pub layers: Vec<Box<dyn SpVkRenderLayer>>,
    pub transform_buffers: Vec<vk::Buffer>, // Uniform buffers
    pub transform_allocs: Vec<Allocation>
}

impl VulkanRenderer
{
    pub fn new(window: &Window, app_name: CString, app_version: u32) -> Self
    {
        let loader = SpVkLoader::new(window, app_name, app_version);

        let inner_size = window.inner_size();
        let ctx = SpVkContext::new(&loader, inner_size.width, inner_size.height);

        let layers: Vec<Box<dyn SpVkRenderLayer>> = vec![];

        // Self{ loader, ctx, layers }
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
