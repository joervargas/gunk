use crate::renderer::renderer_utils;
use crate::renderer::vulkan_renderer::sp_vulkan::splunk_vk_buffer::{sp_destroy_vk_buffer, sp_create_vk_buffer};
use crate::renderer::vulkan_renderer::vk_render_layers::vk_model_layer::VkModelLayer;

use ash::{self, vk};

use super::sp_vulkan::splunk_vk_buffer::SpVkBuffer;
use super::sp_vulkan::splunk_vk_img::{sp_create_vk_depth_img, sp_destroy_vk_img, SpVkImage};
use super::sp_vulkan::{
    splunk_vk_loader::SpVkLoader, 
    splunk_vk_context::SpVkContext
};
use super::vk_render_layers::sp_vk_render_layer::SpVkRenderLayer;

use winit::window::Window;
use std::ffi::CString;
use nalgebra_glm as glm;

pub struct VulkanRenderer
{
    pub loader: SpVkLoader,
    pub vk_ctx: SpVkContext,
    pub layers: Vec<Box<dyn SpVkRenderLayer>>,
    pub transform_uniforms: Vec<SpVkBuffer>, // Uniform buffers
    pub depth_img: SpVkImage
}

impl VulkanRenderer
{
    pub fn new(window: &Window, app_name: CString, app_version: u32) -> Self
    {
        let loader = SpVkLoader::new(window, app_name, app_version);

        let inner_size = window.inner_size();
        let mut vk_ctx = SpVkContext::new(&loader, inner_size.width, inner_size.height);

        let depth_img = sp_create_vk_depth_img(&loader.instance, &mut vk_ctx, inner_size.width, inner_size.height);

        let transform_uniforms = {
            let mut uniforms: Vec<SpVkBuffer> = Vec::new();
            for _i in 0..vk_ctx.swapchain.images.len()
            {
                let buffer = sp_create_vk_buffer(&mut vk_ctx, "transform_uniform", vk::BufferUsageFlags::UNIFORM_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu, std::mem::size_of::<glm::Mat4>() as vk::DeviceSize);
                uniforms.push(buffer);
            }
            uniforms
        };

        let layers: Vec<Box<dyn SpVkRenderLayer>> = vec![
            Box::new( VkModelLayer::new(&loader.instance, &mut vk_ctx, &transform_uniforms, &depth_img, std::path::Path::new("./assets/rubber_duck/scent.gltf"), std::path::Path::new("./assets/rubber_duck/textures/Duck_baseColor.png"))  )
        ];

        Self
        {
            loader,
            vk_ctx,
            layers,
            transform_uniforms,
            depth_img
        }
    }

}

impl renderer_utils::GfxRenderer for VulkanRenderer
{
    fn init(&self) 
    {
        todo!()    
    }

    fn destroy(mut self) 
    {
        sp_destroy_vk_img(&mut self.vk_ctx, self.depth_img);
        for buffer in self.transform_uniforms.into_iter()
        {
            sp_destroy_vk_buffer(&mut self.vk_ctx, buffer);
        }

        for layer in self.layers.iter_mut()
        {
            layer.destroy(&mut self.vk_ctx);
        }

        self.vk_ctx.destroy();
        self.loader.destroy();
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
