use crate::renderer::renderer_utils::{self, to_asset_path};
use crate::{log_err, vk_check, log_info};

use crate::renderer::vulkan_renderer::sp_vulkan::splunk_vk_buffer::map_vk_allocation_data;
use crate::renderer::vulkan_renderer::vk_render_layers::sp_vk_render_layer::SpVk3dLayerUpdate;

use ash::vk::BufferUsageFlags;
use ash::{self, vk};

use super::sp_vk_camera::{SpCamera, SpCameraUniformData, CamView, CamProjection};
use super::sp_vulkan::splunk_vk_buffer::{SpVkBuffer, sp_create_vk_buffers, sp_destroy_vk_buffers};
use super::sp_vulkan::splunk_vk_img::{sp_create_vk_depth_img, sp_destroy_vk_img, SpVkImage};
use super::sp_vulkan::{
    splunk_vk_loader::SpVkLoader, 
    splunk_vk_context::SpVkContext
};
use super::vk_render_layers::sp_vk_render_layer::{Vk2dLayerList, Vk3dLayerList, SpVk2dLayerUpdate};
use super::vk_render_layers::{
    sp_vk_render_layer::SpVkLayerDraw,
    vk_begin_layer::VkBeginLayer,
    vk_end_layer::VkEndLayer,
    vk_simple2d_layer::VkSimple2dLayer
};

use winit::window::Window;
use nalgebra_glm as glm;
use gpu_allocator::MemoryLocation;

use std::ffi::CString;

pub struct VulkanRenderer
{
    pub loader:             SpVkLoader,
    pub vk_ctx:             SpVkContext,
    pub transform_uniforms: Vec<SpVkBuffer>, // Uniform buffers
    pub camera:             SpCamera,
    pub depth_img:          Option<SpVkImage>,
    vk_begin_layer:         VkBeginLayer,
    vk_end_layer:           VkEndLayer,
    pub layers3d:           Vk3dLayerList,
    pub layers2d:           Vk2dLayerList,
    has_resized:            bool,
}

impl VulkanRenderer
{
    pub fn new(window: &Window, app_name: CString, app_version: u32) -> Self
    {
        let loader = SpVkLoader::new(window, app_name, app_version);

        let inner_size = window.inner_size();
        let mut vk_ctx = SpVkContext::new(&loader, inner_size.width, inner_size.height);

        let depth_img = sp_create_vk_depth_img(&loader.instance, &mut vk_ctx, inner_size.width, inner_size.height);

        let swapchain_image_count = vk_ctx.swapchain.images.len();
        let transform_uniforms = sp_create_vk_buffers(
            &mut vk_ctx,
            "transform uniform",
            BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
            std::mem::size_of::<SpCameraUniformData>() as vk::DeviceSize,
            swapchain_image_count
        );

        let view = CamView
        {
            pos: glm::Vec3::new(0.0, 1.0, -1.0),
            front: glm::Vec3::new(0.0, 0.0, 0.0),
            up: glm::Vec3::new(0.0, 1.0, 0.0)
        };
        let projection = CamProjection
        {
            aspect: vk_ctx.swapchain.extent.width as f32 / vk_ctx.swapchain.extent.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100.0
        };
        let camera = SpCamera{ view, projection };

        let vk_begin_layer = VkBeginLayer::new(&loader.instance, &mut vk_ctx, Some(&depth_img));
        let vk_end_layer = VkEndLayer::new(&loader.instance, &mut vk_ctx, Some(&depth_img));

        let layers3d = Vk3dLayerList::new();
        // layers3d.push(Box::new( VkModelLayer::new(&loader.instance, &mut vk_ctx, &transform_uniforms, &depth_img, &to_asset_path("rubber_duck/scene.gltf").as_path(), &to_asset_path("rubber_duck/textures/Duck_baseColor.png").as_path())) );

        let mut layers2d = Vk2dLayerList::new();
        layers2d.push( Box::new(VkSimple2dLayer::new(&loader.instance, &mut vk_ctx, &to_asset_path("textures/statue.jpg"))) );

        Self
        {
            loader,
            vk_ctx,
            transform_uniforms,
            camera,
            depth_img: Some(depth_img),
            vk_begin_layer,
            vk_end_layer,
            layers3d,
            layers2d,
            has_resized: false
        }
    }

    pub fn cleanup_swapchain(&mut self)
    {
        log_info!("Cleaning VkSwapchain and VkFramebuffers...");
        self.vk_begin_layer.cleanup_framebuffers(&self.vk_ctx.device);
        self.layers2d.cleanup_framebuffers(&self.vk_ctx.device);
        self.layers3d.cleanup_framebuffers(&self.vk_ctx.device);
        self.vk_end_layer.cleanup_framebuffers(&self.vk_ctx.device);

        self.vk_ctx.clean_swapchain();
        
        log_info!("VkSwapchain and VkFramebuffers cleaned.");
    }

    pub fn recreate_swapchain(&mut self, window: &Window)
    {
        log_info!("Recreating VkSwapchain and Framebuffers...");

        unsafe { vk_check!(self.vk_ctx.device.device_wait_idle()).unwrap(); }

        self.cleanup_swapchain();

        let inner_size = window.inner_size();
        self.vk_ctx.recreate_swapchain(&self.loader, inner_size.width, inner_size.height);

        self.vk_begin_layer.recreate_framebuffers(&self.vk_ctx, self.depth_img.as_ref());
        self.layers3d.recreate_framebuffers(&self.vk_ctx, self.depth_img.as_ref());
        self.layers2d.recreate_framebuffers(&self.vk_ctx, None);
        self.vk_end_layer.recreate_framebuffers(&self.vk_ctx, self.depth_img.as_ref());

        log_info!("VkSwapchain and VkFramebuffers recreated.");
    }

}

impl renderer_utils::GfxRenderer for VulkanRenderer
{
    fn init(&self) 
    {
         
    }

    fn destroy(&mut self) 
    {
        sp_destroy_vk_img(&mut self.vk_ctx, self.depth_img.take().unwrap());
        sp_destroy_vk_buffers(&mut self.vk_ctx, &mut self.transform_uniforms);

        self.vk_begin_layer.destroy(&mut self.vk_ctx);
        self.layers3d.destroy(&mut self.vk_ctx);
        self.layers2d.destroy(&mut self.vk_ctx);
        self.vk_end_layer.destroy(&mut self.vk_ctx);

        self.vk_ctx.destroy();
        self.loader.destroy(); 
    }

    fn update(&mut self, window: &Window, current_img: usize) 
    {
        let _inner_size = window.inner_size();

        let m = glm::rotate(&glm::Mat4::identity(), glm::pi::<f32>(), &glm::vec3(0.0, 1.0, 0.0)).as_slice()[..].try_into().unwrap();
        let v = self.camera.view.get_matrix().as_slice()[..].try_into().unwrap();
        let p = self.camera.projection.get_matrix().as_slice()[..].try_into().unwrap();

        let camera_uniform_data = SpCameraUniformData{ model: m, view: v, proj: p };
        // let data = &[camera_uniform_data].as_slice()[..].try_into().unwrap();
        // map_vk_buffer_data(&self.vk_ctx.device, &self.transform_uniforms[current_img as usize].allocation, data, std::mem::size_of::<SpCameraUniformData>() as vk::DeviceSize);

        map_vk_allocation_data::<SpCameraUniformData>(&self.transform_uniforms[current_img as usize].allocation, &[camera_uniform_data], 1);

        self.layers3d.update(&self.vk_ctx, &self.transform_uniforms[current_img as usize], self.depth_img.as_ref().unwrap(), current_img);
        self.layers2d.update(&self.vk_ctx, current_img);
    }

    fn draw_frame(&mut self, _window: &Window, current_img: usize) 
    {
        let draw_buffer = *self.vk_ctx.draw_cmds.get_current_buffer();

        let draw_cmd_begin_info = vk::CommandBufferBeginInfo
        {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: std::ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
            p_inheritance_info: std::ptr::null()
        };

        unsafe
        {
            vk_check!(self.vk_ctx.device.begin_command_buffer(draw_buffer, &draw_cmd_begin_info));

            self.vk_begin_layer.draw_frame(&self.vk_ctx, &draw_buffer, current_img);
            self.layers3d.draw_frame(&self.vk_ctx, &draw_buffer, current_img);
            self.layers2d.draw_frame(&self.vk_ctx, &draw_buffer, current_img);
            self.vk_end_layer.draw_frame(&self.vk_ctx, &draw_buffer, current_img);

            vk_check!(self.vk_ctx.device.end_command_buffer(draw_buffer));
        }
    }

    fn render(&mut self, window: &Window) 
    {
        unsafe { vk_check!(self.vk_ctx.device.wait_for_fences(&[*self.vk_ctx.frame_sync.get_current_in_flight_fence()], true, std::u64::MAX)).unwrap(); }

        let (current_img_idx, _is_sub_optimal) = unsafe {
            self.vk_ctx.swapchain.loader.acquire_next_image(self.vk_ctx.swapchain.handle, std::u64::MAX, *self.vk_ctx.frame_sync.get_current_wait_semaphore(), vk::Fence::null()).map_err(
                |vk_result| 
                { 
                    match vk_result
                    {
                        vk::Result::ERROR_OUT_OF_DATE_KHR => { self.recreate_swapchain(window); }
                        _ => { log_err!(vk_result); }
                    }
                }  
            ).unwrap()
        };

        unsafe { vk_check!(self.vk_ctx.device.reset_fences( &[*self.vk_ctx.frame_sync.get_current_in_flight_fence()] )).unwrap(); }

        // self.vk_ctx.reset_draw_cmd_pool();
        self.vk_ctx.reset_current_draw_cmd_buffer();

        let current_img = current_img_idx as usize;
        self.update(window, current_img as usize);
        self.draw_frame(window, current_img as usize);

        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo
        {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: self.vk_ctx.frame_sync.get_current_wait_semaphore(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: self.vk_ctx.draw_cmds.get_current_buffer(),
            signal_semaphore_count: 1,
            p_signal_semaphores: self.vk_ctx.frame_sync.get_current_render_semaphore()
        };

        unsafe { vk_check!(self.vk_ctx.device.queue_submit(self.vk_ctx.queues.graphics.handle, &[submit_info], *self.vk_ctx.frame_sync.get_current_in_flight_fence())); }

        let present_info = vk::PresentInfoKHR
        {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: std::ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: self.vk_ctx.frame_sync.get_current_render_semaphore(),
            swapchain_count: 1,
            p_swapchains: &self.vk_ctx.swapchain.handle,
            p_image_indices: &current_img_idx,
            ..Default::default()
        };

        unsafe
        {
            let is_sub_optimal = self.vk_ctx.swapchain.loader.queue_present(self.vk_ctx.queues.graphics.handle, &present_info).map_err(
                |vk_result|
                {
                    match vk_result
                    {
                        vk::Result::ERROR_OUT_OF_DATE_KHR => { self.recreate_swapchain(window); }
                        _ => { log_err!(vk_result); }
                    }
                }
            ).unwrap();
            if is_sub_optimal || self.has_resized
            {
                self.has_resized = false;
                self.recreate_swapchain(window);
            }

            vk_check!(self.vk_ctx.device.device_wait_idle());
        }

        self.vk_ctx.set_next_frame_index();
    }

    fn resized(&mut self)
    {
        self.has_resized = true;
    }

    fn wait_idle(&self)
    {
        unsafe { vk_check!(self.vk_ctx.device.device_wait_idle()); }
    }
}
