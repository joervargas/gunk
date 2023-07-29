use ash::{
    {self, vk},
    extensions::khr
};

use super::vulkan_loader::{AshVkSurface, VulkanLoader};
use crate::{ vk_check, log_info, log_err };


pub fn is_vk_physical_device_suitable(instance: &ash::Instance, surface: &AshVkSurface, physical_device: vk::PhysicalDevice) -> bool
{

    let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let device_features = unsafe { instance.get_physical_device_features(physical_device) };
    
    // GPU
    let is_discrete_gpu = device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
    let is_integrated_gpu = device_properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU;
    let is_gpu = is_discrete_gpu || is_integrated_gpu;

    // Geometry shader capable
    let geometry_shader_capabale = device_features.geometry_shader == 1;

    let graphics_family_index = find_vk_queue_family_index(instance, &physical_device, vk::QueueFlags::GRAPHICS);

    let present_supported = unsafe 
    { 
        surface.loader.get_physical_device_surface_support(
            physical_device, 
            graphics_family_index.clone().unwrap(), 
            surface.handle
        ).unwrap() 
    };

    is_gpu && geometry_shader_capabale && graphics_family_index.is_some() && present_supported
}


pub fn find_suitable_vk_physical_device(instance: &ash::Instance, surface: &AshVkSurface) -> vk::PhysicalDevice
{
    log_info!("Finding suitable VkPhysicalDevice...");

    let physical_devices = unsafe { vk_check!(instance.enumerate_physical_devices()).unwrap() };

    for &physical_device in physical_devices.iter()
    {
        if is_vk_physical_device_suitable(instance, surface, physical_device)
        {
            log_info!("VkPhysicalDevice found.");
            return physical_device;
        }
    }

    log_err!("VkPhysical Device not found!");
    panic!("Failed to find a suitable GPU (VkPhysicalDevice)");
}


pub fn find_vk_queue_family_index(instance: &ash::Instance, physical_device: &vk::PhysicalDevice, desired_queue_flag: vk::QueueFlags) -> Option<u32>
{
    let device_queue_families = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    let mut result: Option<u32> = None;
    let mut index: u32 = 0;
    for queue_family_index in device_queue_families.iter()
    {
        if queue_family_index.queue_flags.contains(desired_queue_flag)
        {
            result = Some(index);
            
        }
        index += 1;
    }

    result
}

pub fn create_vk_device(instance: &ash::Instance, physical_device: &vk::PhysicalDevice, queue_indices: &Vec<u32>) -> ash::Device
{
    log_info!("Creating VkDevice handle...");

    const QUEUE_PRIORITY: f32 = 1.0;
    
    let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = vec![];
    for queue_index in queue_indices.iter()
    {
        let queue_info = vk::DeviceQueueCreateInfo
        {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: *queue_index,
            p_queue_priorities: [QUEUE_PRIORITY].as_ptr(),
            queue_count: 1,
        };
        queue_create_infos.push(queue_info);
    }

    let extensions = vec![
        ash::extensions::khr::Swapchain::name().as_ptr()
    ];

    let create_info = vk::DeviceCreateInfo
    {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: queue_create_infos.len() as u32,
        p_queue_create_infos: queue_create_infos.as_ptr(),
        enabled_extension_count: extensions.len() as u32,
        pp_enabled_extension_names: extensions.as_ptr(),
        ..Default::default()
    };

    let device = unsafe { vk_check!( instance.create_device(*physical_device, &create_info, None)).unwrap() };

    log_info!("VkDevice created.");

    device
}

pub fn create_vk_allocator(instance: &ash::Instance, physical_device: &vk::PhysicalDevice, device: &ash::Device) -> gpu_allocator::vulkan::Allocator
{
    let alloc_desc = gpu_allocator::vulkan::AllocatorCreateDesc
    {
        instance: instance.clone(),
        device: device.clone(),
        physical_device: physical_device.clone(),
        debug_settings: Default::default(),
        buffer_device_address: true,
    };

    gpu_allocator::vulkan::Allocator::new(&alloc_desc).map_err(|e| { log_err!(e); } ).unwrap()
}

pub struct SwapchainDetails
{
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>
}

pub fn query_vk_swapchain_details(physical_device: &vk::PhysicalDevice, surface: &AshVkSurface) -> SwapchainDetails
{
    let capabilities = unsafe
    {
        surface.loader.get_physical_device_surface_capabilities(*physical_device, surface.handle).map_err( |e| { log_err!(e); } ).unwrap()
    };
    let formats = unsafe
    {
        surface.loader.get_physical_device_surface_formats(*physical_device, surface.handle).map_err( |e| { log_err!(e); } ).unwrap()
    };
    let present_modes = unsafe
    {
        surface.loader.get_physical_device_surface_present_modes(*physical_device, surface.handle).map_err( |e| { log_err!(e); } ).unwrap()    
    };

    SwapchainDetails { capabilities, formats, present_modes }
}

pub fn choose_vk_swap_surface_format(formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR
{
    todo!()
}

pub fn choose_vk_swap_present_mode(present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR
{
    todo!()
}

pub fn choose_vk_swap_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> u32
{
    todo!()
}

pub fn choose_vk_swap_extent(capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D
{
    todo!()
}

pub fn create_vk_swapchain(physical_device: &vk::PhysicalDevice, surface: &AshVkSurface,  queue_indices: &Vec<u32>, width: u32, height: u32) -> (khr::Swapchain, vk::SwapchainKHR)
{
    todo!()
}