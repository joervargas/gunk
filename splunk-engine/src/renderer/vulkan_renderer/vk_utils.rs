use ash::{
    {self, vk},
    extensions::khr
};

use super::vulkan_loader::AshVkSurface;
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
    log_info!("Creating Memory Allocator...");

    let alloc_desc = gpu_allocator::vulkan::AllocatorCreateDesc
    {
        instance: instance.clone(),
        device: device.clone(),
        physical_device: physical_device.clone(),
        debug_settings: Default::default(),
        buffer_device_address: true,
    };

    let allocator = gpu_allocator::vulkan::Allocator::new(&alloc_desc).map_err(|e| { log_err!(e); } ).unwrap();

    log_info!("Memory Allocator created");

    allocator
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
    for format in formats.iter()
    {
        if format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return format.clone();
        }
    }
    formats[0].clone()
}

pub fn choose_vk_swap_present_mode(present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR
{
    for mode in present_modes.iter()
    {
        if *mode == vk::PresentModeKHR::MAILBOX
        {
            return mode.clone();
        }
    }
    vk::PresentModeKHR::FIFO
}

pub fn choose_vk_swap_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> u32
{
    let image_count = capabilities.min_image_count + 1;

    let image_count_exceeded = capabilities.max_image_count > 0 && image_count > capabilities.max_image_count;

    if image_count_exceeded { capabilities.max_image_count } else { image_count }
}

pub fn create_vk_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        surface: &AshVkSurface, 
        queue_indices: &Vec<u32>,
        capabilities: vk::SurfaceCapabilitiesKHR,
        surface_format: &vk::SurfaceFormatKHR,
        present_mode: &vk::PresentModeKHR,
        image_count: u32,
        extent: &vk::Extent2D,
    ) -> (khr::Swapchain, vk::SwapchainKHR)
{
    log_info!("Createing VkSwaphainKHR handle...");

    let create_info = vk::SwapchainCreateInfoKHR
    {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: std::ptr::null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface: surface.handle,
        min_image_count: image_count,
        image_color_space: surface_format.color_space,
        image_format: surface_format.format,
        image_extent: *extent,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
        image_sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: queue_indices.len() as u32,
        p_queue_family_indices: queue_indices.as_ptr(),
        pre_transform: capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode: *present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        image_array_layers: 1
    };

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
    let swapchain_handle = unsafe 
    {
        swapchain_loader.create_swapchain(&create_info, None).map_err( |e| { log_err!(e); } ).unwrap()
    };

    log_info!("VkSwapchainKHR created");
    
    (swapchain_loader, swapchain_handle)
}

pub fn create_vk_image_view(device: &ash::Device, image: &vk::Image, format: &vk::Format, aspect_flags: vk::ImageAspectFlags, view_type: vk::ImageViewType, layer_count: u32, mip_levels: u32) -> vk::ImageView
{
    let create_info = vk::ImageViewCreateInfo
    {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ImageViewCreateFlags::empty(),
        image: *image,
        view_type: view_type,
        format: *format,
        subresource_range: vk::ImageSubresourceRange
        {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: layer_count,
        },
        ..Default::default()
    };

    unsafe { vk_check!( device.create_image_view(&create_info, None) ).unwrap() }
}