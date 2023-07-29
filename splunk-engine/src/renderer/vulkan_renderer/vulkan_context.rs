use ash::{ vk, Device };
use gpu_allocator::vulkan::Allocator;

use crate::{log_err, log_info};

use super::vk_utils::*;

use super::vulkan_loader::VulkanLoader;

pub struct VulkanQueue
{
    pub index: Option<u32>,
    pub handle: vk::Queue,
}

impl VulkanQueue
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

pub struct VulkanQueues
{
    pub graphics: VulkanQueue,
}

impl VulkanQueues
{
    pub fn new() -> Self
    { 
        Self
        { 
            graphics: VulkanQueue::new() 
        } 
    } 

    pub fn query_indices(&self, instance: &ash::Instance, physical_device: &vk::PhysicalDevice) -> Self
    {
        let device_queue_families = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        let mut graphics = VulkanQueue::new();
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

pub struct VulkanSwapchain
{
    pub loader: ash::extensions::khr::Swapchain,
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D
}

impl VulkanSwapchain
{
    pub fn new(loader: &VulkanLoader, device: &ash::Device, physical_device: &vk::PhysicalDevice, queue_indices: &Vec<u32>,  width: u32, height: u32) -> Self
    {
        log_info!("Creating VulkanSwapchain struct...");
        let details = query_vk_swapchain_details(physical_device, &loader.surface);
    
        let format = choose_vk_swap_surface_format(details.formats);
        let present_mode = choose_vk_swap_present_mode(details.present_modes);
        let image_count = choose_vk_swap_image_count(details.capabilities);
        let extent = vk::Extent2D{ width: width, height: height };

        let (loader, handle) = create_vk_swapchain(&loader.instance, device, &loader.surface, queue_indices, details.capabilities, &format, &present_mode, image_count, &extent);
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

pub struct VulkanContext
{
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub allocator: Allocator,

    pub queues: VulkanQueues,

    pub swapchain: VulkanSwapchain,

}

impl VulkanContext
{
    pub fn new(loader: &VulkanLoader, width: u32, height: u32) -> Self
    {
        log_info!("Creating VulkanContext...");

        let physical_device = find_suitable_vk_physical_device(&loader.instance, &loader.surface);
        
        let mut queues = VulkanQueues::new();
        queues.query_indices(&loader.instance, &physical_device);

        let queue_index_list = queues.get_index_list();
        let device = create_vk_device(&loader.instance, &physical_device, &queue_index_list);
        queues.query_queues(&device);

        let allocator = create_vk_allocator(&loader.instance, &physical_device, &device);

        let swapchain = VulkanSwapchain::new(loader, &device, &physical_device, &queue_index_list, width, height);

        log_info!("VulkanContext created");
        Self
        {
            device,
            physical_device,
            allocator,
            queues,
            swapchain
        }
    }


    pub fn destroy(&self)
    {
        // drop(self.allocator);

        self.swapchain.destroy(&self.device);
        unsafe
        {
            self.device.destroy_device(None);
        }
    }

}

