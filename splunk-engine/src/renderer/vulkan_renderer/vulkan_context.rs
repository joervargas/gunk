use ash::{ vk, Device };
use gpu_allocator::vulkan::Allocator;

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
    pub fn new(loader: &VulkanLoader) -> Self
    {
        let physical_device = find_suitable_vk_physical_device(&loader.instance, &loader.surface);
        
        let mut queues = VulkanQueues::new();
        queues.query_indices(&loader.instance, &physical_device);

        let queue_index_list = queues.get_index_list();
        let device = create_vk_device(&loader.instance, &physical_device, &queue_index_list);
        queues.query_queues(&device);

        let allocator = create_vk_allocator(&loader.instance, &physical_device, &device);

        todo!()
        // Self
        // {
        //     device,
        //     physical_device,
        //     allocator,
        //     queues,
        //     swapchain:
        // }
    }


    pub fn destroy(&self)
    {
        // drop(self.allocator);
        unsafe
        {
            self.device.destroy_device(None);
        }
    }

}

