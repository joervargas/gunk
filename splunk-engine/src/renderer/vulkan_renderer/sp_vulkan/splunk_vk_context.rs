use ash::{ vk, Device };
use gpu_allocator::vulkan::Allocator;

use crate::{log_err, log_info, vk_check};

use super::vk_utils::*;

use super::splunk_vk_loader::SpVkLoader;
use super::splunk_vk_img::create_vk_image_view;

pub struct SpVkQueue
{
    pub index: Option<u32>,
    pub handle: vk::Queue,
}

impl SpVkQueue
{
    fn new() -> Self
    {
        Self
        {
            index: None,
            handle: vk::Queue::null(),
        }
    }
}

pub struct SpVkQueues
{
    pub graphics: SpVkQueue,
}

impl SpVkQueues
{
    pub fn new() -> Self
    { 
        Self
        { 
            graphics: SpVkQueue::new() 
        } 
    } 

    pub fn query_indices(&self, instance: &ash::Instance, physical_device: &vk::PhysicalDevice) -> Self
    {
        let device_queue_families = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        let mut graphics = SpVkQueue::new();
        let mut index: u32 = 0;
        for queue_family in device_queue_families.iter()
        {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                graphics.index = Some(index);
            }
            index += 1;
        }

        Self 
        { 
            graphics,
        }
    }

    pub fn query_queues(&mut self, device: &ash::Device)
    {
        if self.graphics.index.is_some()
        {
            self.graphics.handle = unsafe { device.get_device_queue(self.graphics.index.clone().unwrap(), 0) };
        }
    }

    pub fn get_index_list(&self) -> Vec<u32>
    {
        let mut index_list: Vec<u32> = vec![];
        if self.graphics.index.is_some()
        {
            index_list.push(self.graphics.index.clone().unwrap());
        }
        index_list
    }

}

pub struct SpVkSwapchain
{
    pub loader: ash::extensions::khr::Swapchain,
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D
}

impl SpVkSwapchain
{
    pub fn new(loader: &SpVkLoader, device: &ash::Device, physical_device: &vk::PhysicalDevice, queue_indices: &Vec<u32>,  width: u32, height: u32) -> Self
    {
        log_info!("Creating VulkanSwapchain struct...");
        let details = query_vk_swapchain_details(physical_device, &loader.surface);
    
        let format = choose_vk_swap_surface_format(details.formats);
        let present_mode = choose_vk_swap_present_mode(details.present_modes);
        let extent = vk::Extent2D{ width: width, height: height };

        let (loader, handle) = create_vk_swapchain(&loader.instance, device, &loader.surface, queue_indices, details.capabilities, &format, &present_mode, &extent);
        let images = unsafe 
        {
            loader.get_swapchain_images(handle).map_err( |e| { log_err!(e); } ).unwrap()    
        };

        let mut views: Vec<vk::ImageView> = vec![];
        for image in images.iter()
        {
            let view = create_vk_image_view(device, image, &format.format, vk::ImageAspectFlags::COLOR, vk::ImageViewType::TYPE_2D, 1, 1);
            views.push(view);
        }

        log_info!("VulkanSwapchain struct created");

        Self
        {
            loader,
            handle,
            images,
            views,
            format: format.format,
            extent
        }
    }

    pub fn destroy(&self, device: &ash::Device)
    {
        unsafe
        {
            for view in self.views.iter()
            {
                device.destroy_image_view(*view, None);
            }
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}

pub struct  SpVkCommands
{
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>
}

impl SpVkCommands
{
    pub fn new(device: &ash::Device, queue_family_index: u32, buffer_count: u32) -> Self
    {
        let pool = create_vk_command_pool(device, queue_family_index);
        let buffers = allocate_vk_command_buffers(device, &pool, buffer_count);

        Self{ pool, buffers }
    }

    pub fn destroy(&self, device: &ash::Device)
    {
        unsafe
        {
            device.destroy_command_pool(self.pool, None);
        }
    }
}

pub struct SpVkContext
{
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub allocator: Allocator,
    pub queues: SpVkQueues,
    pub swapchain: SpVkSwapchain,
    pub draw_cmds: SpVkCommands,
    pub render_semaphore: vk::Semaphore,
    pub wait_semaphore: vk::Semaphore,
}

impl SpVkContext
{
    pub fn new(loader: &SpVkLoader, width: u32, height: u32) -> Self
    {
        log_info!("Creating VulkanContext...");

        let physical_device = find_suitable_vk_physical_device(&loader.instance, &loader.surface);
        
        let mut queues = SpVkQueues::new();
        queues.query_indices(&loader.instance, &physical_device);

        let queue_index_list = queues.get_index_list();
        let device = create_vk_device(&loader.instance, &physical_device, &queue_index_list);
        queues.query_queues(&device);

        let allocator = create_vk_allocator(&loader.instance, &physical_device, &device);

        let swapchain = SpVkSwapchain::new(loader, &device, &physical_device, &queue_index_list, width, height);

        let draw_cmds = SpVkCommands::new(&device, queues.graphics.index.clone().unwrap(), swapchain.images.len() as u32);
        
        let render_semaphore = create_vk_semaphore(&device);
        let wait_semaphore = create_vk_semaphore(&device);

        log_info!("VulkanContext created");
        Self
        {
            device,
            physical_device,
            allocator,
            queues,
            swapchain,
            draw_cmds,
            render_semaphore,
            wait_semaphore
        }
    }


    pub fn destroy(&self)
    {
        // drop(self.allocator);

        self.swapchain.destroy(&self.device);
        self.draw_cmds.destroy(&self.device);
        unsafe
        {
            self.device.destroy_device(None);
        }
    }

}


pub fn sp_begin_single_time_vk_command_buffer(vk_ctx: &SpVkContext) -> vk::CommandBuffer
{
    let alloc_info = vk::CommandBufferAllocateInfo
    {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        command_pool: vk_ctx.draw_cmds.pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: 1
    };
    let cmd_buffer = unsafe{ vk_check!(vk_ctx.device.allocate_command_buffers(&alloc_info)).unwrap()[0] };

    let begin_info = vk::CommandBufferBeginInfo
    {
        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        p_next: std::ptr::null(),
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        p_inheritance_info: std::ptr::null()
    };
    unsafe{ vk_check!(vk_ctx.device.begin_command_buffer(cmd_buffer, &begin_info)).unwrap() }

    cmd_buffer
}

pub fn sp_end_single_time_vk_command_buffer(vk_ctx: &SpVkContext, cmd_buffer: vk::CommandBuffer)
{
    unsafe { vk_check!(vk_ctx.device.end_command_buffer(cmd_buffer)).unwrap(); }

    let submit_info = [vk::SubmitInfo
    {
        s_type: vk::StructureType::SUBMIT_INFO,
        p_next: std::ptr::null(),
        wait_semaphore_count: 0,
        p_wait_semaphores: std::ptr::null(),
        p_wait_dst_stage_mask: std::ptr::null(),
        command_buffer_count: 1,
        p_command_buffers: &cmd_buffer,
        signal_semaphore_count: 0,
        p_signal_semaphores: std::ptr::null()
    }];

    unsafe 
    {
        vk_check!(vk_ctx.device.queue_submit(vk_ctx.queues.graphics.handle, &submit_info, vk::Fence::null())).unwrap();
    
        vk_check!(vk_ctx.device.queue_wait_idle(vk_ctx.queues.graphics.handle)).unwrap();

        vk_ctx.device.free_command_buffers(vk_ctx.draw_cmds.pool, &[cmd_buffer]);
    }
}
