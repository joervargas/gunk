
use ash::{
    {self, vk},
    extensions::khr
};

use super::splunk_vk_loader::SpVkSurface;
use crate::{ vk_check, log_info, log_err };

use std::ffi::CString;

/// ### fn is_vk_physical_device_suitable( ... ) -> bool
/// *Determines if vk::PhysicalDevice is suitable for use.*<br>
/// *Queueries the device for features, properties and queue family indices.*
/// <pre>
/// - Params
///     instance:           &ash::Instance
///     surface:            &SpVkSurface
///     physical_device:    vk::PhysicalDevice
/// - Return
///     bool
/// </pre>
pub fn is_vk_physical_device_suitable(instance: &ash::Instance, surface: &SpVkSurface, physical_device: vk::PhysicalDevice) -> bool
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

/// ### fn find_suitable_vk_physical_device( ... ) -> vk::PhysicalDevice
/// *Queueies available devices, then picks the most suitable.*<br>
/// *Intrinsically uses "is_vk_physical_device_suitable()"*
/// <pre>
/// - Params
///     instance:       &ash::Instance
///     surface:        &SpVkSurface
/// - Return
///     vk::PhsyicalDevice
/// </pre>
pub fn find_suitable_vk_physical_device(instance: &ash::Instance, surface: &SpVkSurface) -> vk::PhysicalDevice
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

/// ### fn find_vk_queue_family_index( ... ) -> Option\<u32\>
/// *Finds a vulkan queue family index.*
/// <pre>
/// - Params
///     instance:               &ash::Instance
///     physical_device:        &vk::PhysicalDevice
///     desired_queue_flags:    &vk::QueueFlags
/// - Return
///     Option&lt;u32&gt;
/// </pre>
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

/// ### fn create_vk_device( ... ) -> ash::Device
/// *Creates an ash::Device (VkDevice) struct.*<br>
/// *Used to execute vulkan gpu functions.*
/// <pre>
/// - Params
///     instance:           &ash::Instance
///     physical_device:    &vk::PhysicalDevice
///     queue_indices:      &Vec&lt;u32&gt;
/// - Return
///     ash::Device
/// </pre>
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

    let vk_khr_shader_draw_parameters = CString::new("VK_KHR_shader_draw_parameters").unwrap();
    let extensions = vec![
        ash::extensions::khr::Swapchain::name().as_ptr(),
        vk_khr_shader_draw_parameters.as_ptr()
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

/// ### fn create_vk_allocator( ... ) -> gpu_allocator::vulkan::Allocator
/// *Creates a vulkan gpu_allocator::Allocator struct.*<br>
/// *Used to allocate gpu memory from a pool.*
/// <pre>
/// - Params
///     instance:           &ash::Instance
///     physical_device:    &vk::PhysicalDevice
///     device:             &ash::Device
/// -Return
///     gpu_allocator::vulkan::Allocator
/// </pre>
pub fn create_vk_allocator(instance: &ash::Instance, physical_device: &vk::PhysicalDevice, device: &ash::Device) -> gpu_allocator::vulkan::Allocator
{
    log_info!("Creating Memory Allocator...");

    let alloc_desc = gpu_allocator::vulkan::AllocatorCreateDesc
    {
        instance: instance.clone(),
        device: device.clone(),
        physical_device: physical_device.clone(),
        debug_settings: Default::default(),
        buffer_device_address: false, // uses VK_KHR_buffer_device_address extension
        allocation_sizes: Default::default(),
    };

    let allocator = gpu_allocator::vulkan::Allocator::new(&alloc_desc).map_err(|e| { log_err!(e); } ).unwrap();

    log_info!("Memory Allocator created");

    allocator
}

/// ### VkSwapchainDetails struct
/// *Details specific to the swapchain of this GPU*
/// <pre>
/// - Members
///     capabilities:       vk::SurfaceCapabilitiesKHR
///     formats:            Vec&lt;vk::SurfaceFormatKHR&gt;
///     present_modes:      Vec&lt;vk::PresentModeKHR&gt;
/// </pre>
pub struct VkSwapchainDetails
{
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>
}

/// ### fn query_vk_swapchain_details( ... ) -> VkSwapchainDetails
/// *Queuries the vk::PhysicalDevice and Surface for VkSwapchainDetails*
/// <pre>
/// - Params
///     physical_device:    &vk::PhysicalDevice
///     surface:            &SpVkSurface
/// - Return
///     VkSwapchainDetails
/// </pre>
pub fn query_vk_swapchain_details(physical_device: &vk::PhysicalDevice, surface: &SpVkSurface) -> VkSwapchainDetails
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

    VkSwapchainDetails { capabilities, formats, present_modes }
}

/// ### fn choose_vk_swap_surface_format( ... ) -> vk::SurfaceFormatKHR
/// *Chooses preferred vk::SurfaceFormatKHR from Vec<> of vk::SurfaceFormatKHR*
/// <pre>
/// - Params
///     formats:    Vec&lt;vk::SurfaceFormatKHR&gt;        <i>// vk::SurfaceFormat(s) to choose from</i>
/// - Return
///     vk::SurfaceFormatKHR                        <i>// Chosen vk::SurfaceFormatKHR</i>
/// </pre>
pub fn choose_vk_swap_surface_format(formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR
{
    for format in formats.iter()
    {
        if format.format == vk::Format::B8G8R8A8_UNORM && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return format.clone();
        }
    }
    formats[0].clone()
}

/// ### fn choose_vk_swap_present_mode( ... ) -> vk::PresentModeKHR
/// *Chooses a vk::PresentMode for the vk::Swapchain*<br>
/// *Choice is made by traversing a Vec<> of vk::PresentMode*
/// <pre>
/// - Params
///     present_modes:  Vec&lt;vk::PresentMode&gt;    <i>// vk::PresentMode(s) to choose from</i>
/// - Return
///     vk::PresentMode                         <i>// Chosen vk::PresentMode</i>
/// </pre>
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

/// ### fn choose_vk_swap_image_count( ... ) -> u32
/// *Chooses and returns the count of vk::Swapchain Images*<br>
/// *based on vk::SurfaceCapabilitiesKHR*
/// <pre>
/// - Params
///     capabilites:    vk::SurfaceCapabilitiesKHR
/// - Return
///     u32      <i>// The count of vk::Swapchain images</i>
/// </pre>
pub fn choose_vk_swap_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> u32
{
    let image_count = capabilities.min_image_count + 1;

    let image_count_exceeded = capabilities.max_image_count > 0 && image_count > capabilities.max_image_count;

    if image_count_exceeded { capabilities.max_image_count } else { image_count }
}

/// ### fn create_vk_swapchain( ... ) -> (khr::Swapchain, vk::SwapchainKHR)
/// *Creates a tuple of khr::Swapchain and vk::SwapchainKHR* <br>
/// *khr::Swapchain loads and destroys an instance of vk::SwapchainKHR*
/// <pre>
/// - Params
///     instance:           &ash::Instance
///     device:             &ash::Device
///     surface:            &SpVkSurface
///     queue_indices:      &Vec<u32>
///     capabilities:       vk::SurfaceCapabilitiesKHR
///     surface_format:     &vk::SurfaceFormatKHR
///     present_mode:       &vk::PresentModeKHR
///     extent:             &vk::Extent2D
/// - Returns
///     (khr::Swapchain, vk::SwapchainKHR)
/// </pre>
pub fn create_vk_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        surface: &SpVkSurface, 
        queue_indices: &Vec<u32>,
        capabilities: vk::SurfaceCapabilitiesKHR,
        surface_format: &vk::SurfaceFormatKHR,
        present_mode: &vk::PresentModeKHR,
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
        min_image_count: choose_vk_swap_image_count(capabilities),
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

/// ### fn create_vk_command_pool( ... ) -> vk::CommandPool
/// *Creates a vk::CommandPool*
/// <pre>
/// - Params
///     device:                 &ash::Device
///     queue_family_index:     u32     <i>// Queue index of the command family</i>
/// - Return
///     vk::CommandPool
/// </pre>
pub fn create_vk_command_pool(device: &ash::Device, queue_family_index: u32) -> vk::CommandPool
{
    let create_info = vk::CommandPoolCreateInfo
    {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: queue_family_index,
    };

    unsafe{
        vk_check!(device.create_command_pool(&create_info, None)).unwrap()
    }
}

/// ### fn allocate_vk_command_buffers() -> Vec\<vk::CommandBuffer\>
/// *Allocates a Vec\<vk::CommandBuffer\>*
/// <pre>
/// - Params
///     device:        &ash::Device
///     cmd_pool:      &vk::CommandPool
///     buffer_count:  u32                 <i>// Amount to allocate.</i>
/// - Return
///     Vec&lt;vk::CommandBuffer&gt;
/// </pre>
pub fn allocate_vk_command_buffers(device: &ash::Device, cmd_pool: &vk::CommandPool, buffer_count: u32 ) -> Vec<vk::CommandBuffer>
{
    let alloc_info = vk::CommandBufferAllocateInfo
    {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        command_pool: *cmd_pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: buffer_count
    };

    unsafe{
        vk_check!(device.allocate_command_buffers(&alloc_info)).unwrap()
    }
}

/// ### fn create_vk_semaphore( ... ) -> vk::Semaphore
/// *Creates a vk::Semaphore*
/// <pre>
/// - Params
///      device:     &ash::Device
/// - Return
///       vk::Semaphore
/// </pre>
pub fn create_vk_semaphore(device: &ash::Device) -> vk::Semaphore
{
    let create_info = vk::SemaphoreCreateInfo
    {
        s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SemaphoreCreateFlags::empty()
    };
    unsafe {
        vk_check!(device.create_semaphore(&create_info, None)).unwrap()
    }
}

/// ### fn create_vk_fence( ... ) -> vk::Fence
/// *Creates a vk::Fence*
/// <pre>
/// - Params
///     device:         &ash::Device
///     is_signaled:    bool
/// - Return
///       vk::Fence
/// </pre>
pub fn create_vk_fence(device: &ash::Device, is_signaled: bool) -> vk::Fence
{
    let create_info = vk::FenceCreateInfo
    {
        s_type: vk::StructureType::FENCE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: if is_signaled { vk::FenceCreateFlags::SIGNALED } else { vk::FenceCreateFlags::empty() }
    };
    unsafe{
        vk_check!(device.create_fence(&create_info, None)).unwrap()
    }
}

/// ### fn create_vk_pipeline_layout( ... ) -> vk::PipelineLayout
/// *Creates a vk::PipelineLayout*
/// <pre>
/// - Params
///     device:             &ash::Devie
///     desc_set_layouts:   &Vec&lt;vk::DescriptorSetLayout&gt;
///     push_const_ranges:  &Vec&lt;vk::PushConstantRange&gt;
/// - Return
///     vk::PipelineLayout
/// </pre>
pub fn create_vk_pipeline_layout(
        device: &ash::Device, 
        desc_set_layouts: &Vec<vk::DescriptorSetLayout>, 
        push_const_ranges: &Vec<vk::PushConstantRange>
    ) -> vk::PipelineLayout
{
    let create_info = vk::PipelineLayoutCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: desc_set_layouts.len() as u32,
        p_set_layouts: desc_set_layouts.as_ptr(),
        push_constant_range_count: push_const_ranges.len() as u32,
        p_push_constant_ranges: push_const_ranges.as_ptr()
    };

    unsafe
    {
        vk_check!(device.create_pipeline_layout(&create_info, None)).unwrap()
    }
}

/// ### fn create_vk_pipeline_info_vertex_input() -> vk::PipelineVertexInputStateCreateInfo
/// *Creates a vk::PipelineVertexInputStateCreateInfo struct*
/// <pre>
/// - Return
///     vk::PipelineVertexInputStateCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_vertex_input() -> vk::PipelineVertexInputStateCreateInfo
{
    vk::PipelineVertexInputStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        ..Default::default()
    }
}

/// ### fn create_vk_pipeline_info_assembly( ... ) -> vk::PipelineInputAssemblyStateCreateInfo
/// *Creates a vk::PipelineInputAssemblyStateCreateInfo struct*
/// <pre>
/// - Params
///     topology:               vk::PrimitiveTopology
///     b_primitive_restart:    vk::Bool32
/// - Return
///     vk::PipelineInputAssemblyStateCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_assembly(topology: vk::PrimitiveTopology, b_primitive_restart: vk::Bool32) -> vk::PipelineInputAssemblyStateCreateInfo
{
    vk::PipelineInputAssemblyStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        topology,
        primitive_restart_enable: b_primitive_restart,
        ..Default::default()
    }
}

/// ### fn create_vk_pipeline_info_viewport() -> vk::PipelineViewportStateCreateInfo
/// *Creates a vk::PipelineViewportStateCreateInfo struct*
/// <pre>
/// - Params
///     viewports:      Vec&lt;vk::Viewport&gt;
///     scissors:       Vec&lt;vk::Rect2D&gt;
/// - Return
///     vk::PipelineViewportStateCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_viewport(viewports: Vec<vk::Viewport>, scissors: Vec<vk::Rect2D>) -> vk::PipelineViewportStateCreateInfo
{
    vk::PipelineViewportStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        viewport_count: viewports.len() as u32,
        // p_viewports: viewports.as_ptr(),
        scissor_count: scissors.len() as u32,
        // p_scissors: scissors.as_ptr(),
        ..Default::default()
    }
}

/// ### fn create_vk_pipeline_info_rasterization( ... ) -> vk::PipelineRasterizationStateCreateInfo
/// *Creates a vk::PipelineRasterizationStateCreateInfo struct*
/// <pre>
/// - Params
///     polygon_mode:   vk::PolygonMode
///     cull_mode:      vk::CullModeFlags,
///     front_face:     vk::FrontFace,
///     line_width:     f32
/// - Return
///     vk::PipelineRasterizationStateCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_rasterization(
        polygon_mode: vk::PolygonMode,
        cull_mode: vk::CullModeFlags,
        front_face: vk::FrontFace,
        line_width: f32
    ) -> vk::PipelineRasterizationStateCreateInfo
{
    vk::PipelineRasterizationStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        polygon_mode,
        cull_mode,
        front_face,
        line_width,
        ..Default::default()
    }
}

/// ### fn create_vk_pipeline_info_multisample( ... ) -> vk::PipelineMultisampleStateCreateInfo
/// *Creates a vk::PipelineMultisampleCreateInfo struct*
/// <pre>
/// - Params
///     samples:            vk::SampleCountFlags
///     b_sample_shading:   vk::Bool32
///     min_sample_shading: f32
/// - Return
///     vk::PipelineMultisampleCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_multisample(
        samples: vk::SampleCountFlags, 
        b_sample_shading: vk::Bool32, min_sample_shading: f32
    ) -> vk::PipelineMultisampleStateCreateInfo
{
    vk::PipelineMultisampleStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        rasterization_samples: samples,
        sample_shading_enable: b_sample_shading,
        min_sample_shading,
        ..Default::default()
    }
}

/// ### fn create_vk_pipeline_info_color_blend_attachment( ... ) -> vk::PipelineColorBlendAttachmentState
/// *Creates a vk::PipelineColorBlendAttachmentState struct*
/// <pre>
/// - Param
///     b_use_blending:     bool
/// - Return
///     vk::PipelineColorBlendAttachmentState
/// </pre>
pub fn create_vk_pipeline_info_color_blend_attachment(b_use_blending: bool) -> vk::PipelineColorBlendAttachmentState
{
    vk::PipelineColorBlendAttachmentState
    {
        blend_enable: vk::TRUE,
        src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: if b_use_blending { vk::BlendFactor::ONE_MINUS_SRC_ALPHA } else { vk::BlendFactor::ONE },
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: 
            vk::ColorComponentFlags::R |
            vk::ColorComponentFlags::G |
            vk::ColorComponentFlags::B |
            vk::ColorComponentFlags::A
    }
}

/// ### fn create_vk_pipeline_info_color_blend( ... ) -> Vec<vk::PipelineColorBlendStateCreateInfo
/// *Creates a vk::PipelineColorBlendStateCreateInfo struct*
/// <pre>
/// - Params
///     attachments:    Vec&lt;vk::PipelineColorBlendStateCreateInfo&gt;
/// - Return
///     vk::PipelineColorBlendStateCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_color_blend(attachments: &Vec<vk::PipelineColorBlendAttachmentState>) -> vk::PipelineColorBlendStateCreateInfo
{
    vk::PipelineColorBlendStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        logic_op_enable: vk::FALSE,
        logic_op: vk::LogicOp::COPY,
        attachment_count: attachments.len() as u32,
        p_attachments: attachments.as_ptr(),
        blend_constants: [ 0.0, 0.0, 0.0, 0.0],
        ..Default::default()
    }   
}

/// ### fn create_vk_pipeline_info_depth_stencil() -> vk::PipelineDepthStencilStateCreateInfo
/// *Creates a vk::PipelineDepthStencilStateCreateInfo struct*
/// <pre>
/// - Return
///     vk::PipelineDepthStencilStateCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_depth_stencil() -> vk::PipelineDepthStencilStateCreateInfo
{
    vk::PipelineDepthStencilStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        depth_test_enable: vk::TRUE,
        depth_write_enable: vk::TRUE,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: vk::FALSE,
        min_depth_bounds: 0.0,
        max_depth_bounds: 1.0,
        ..Default::default()
    }
}

/// ### fn create_vk_pipeline_info_dynamic_states( ... ) -> vk::PipelineDynamicStateCreateInfo
/// *Creates a vk::PipelineDynamicStateCreateInfo struct*
/// <pre>
/// - Params
///     dynamic_states:     Vec&lt;vk::DynamicState&gt;
/// - Return
///     vk::PipelineDynamicStateCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_dynamic_states(dynamic_states: &Vec<vk::DynamicState>) -> vk::PipelineDynamicStateCreateInfo
{
    vk::PipelineDynamicStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr()
    }
}

/// ### fn create_vk_pipeline_info_tessellation( ... ) -> vk::PipelineTessellationStateCreateInfo
/// *Creates a vk::PipelineTessellationStateCreateInfo struct*
/// <pre>
/// - Param
///     num_patch_points:   u32
/// - Return
///     vk::PipelineTessellationStateCreateInfo
/// </pre>
pub fn create_vk_pipeline_info_tessellation(num_patch_points: u32) -> vk::PipelineTessellationStateCreateInfo
{
    vk::PipelineTessellationStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineTessellationStateCreateFlags::empty(),
        patch_control_points: num_patch_points
    }
}