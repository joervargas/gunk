use ash::{ vk, Device };
use gpu_allocator::vulkan::Allocator;

use crate::{log_info, vk_check};

use super::splunk_vk_render_pass::SpVkRenderPass;
use super::vk_utils::*;

use super::splunk_vk_loader::SpVkLoader;
use super::splunk_vk_img::create_vk_image_view;

/// ### SpVkQueue struct
/// *Contain Vulkan queue family index and a VkQueue handle*
/// <pre>
/// - Members
///     index:      Option&lt;u32&gt;   <i>// Queue family index</i>
///     handle:     vk::Queue     <i>// VkQueue handle</i>
/// </pre>
pub struct SpVkQueue
{
    pub index:      Option<u32>,
    pub handle:     vk::Queue,
}

impl SpVkQueue
{
    /// ### SpVkQueue::new() -> SpVkQueue
    /// *Returns a new instance of SpVkQueue*<br>
    /// *Index and handle are not set*
    /// <pre>
    /// - Return
    ///     SpVkQueue
    /// </pre>
    fn new() -> Self
    {
        Self
        {
            index: None,
            handle: vk::Queue::null(),
        }
    }
}

/// ### SpVkQueues struct
/// *A convenience struct containing different "families" of SpVkQueue struct*
/// <pre>
/// - Members
///     graphics:       SpVkQueue       <i>// SpVkQueue for graphics family of instructrions.</i>
/// </pre>
pub struct SpVkQueues
{
    pub graphics:   SpVkQueue,
}

impl SpVkQueues
{
    /// ### SpVkQueues::new() -> SpVkQueues
    /// *Returns a new instance of SpVkQueues.*<br><br>
    /// *Each SpVkQueue family is not yet populated with usable values.*<br>
    /// *Must call **fn queury_indices()** and **fn queury_queues()***
    /// <pre>
    /// - Return 
    ///     SpVkQueues
    /// </pre>
    pub fn new() -> Self
    { 
        Self
        { 
            graphics: SpVkQueue::new() 
        } 
    } 

    /// ### fn SpVkQueues::queury_indices( &mut self, ... )
    /// *Queries VkQueueFamilyIndices from the physical device.*<br>
    /// *Populates the SpVkQueue families if found.*
    /// <pre>
    /// - Params
    ///     <b>&mut self</b>
    ///     instance:           &ash::instance
    ///     physical_device:    &vk::PhysicalDevice    
    /// </pre>
    pub fn query_indices(&mut self, instance: &ash::Instance, physical_device: &vk::PhysicalDevice)
    {
        let device_queue_families = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        // let mut graphics = SpVkQueue::new();
        let mut index: u32 = 0;
        for queue_family in device_queue_families.iter()
        {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                self.graphics.index = Some(index);
            }
            index += 1;
        }
    }

    /// ### fn queury_queues( &mut self, ... )
    /// *Sets the VkQueues from the loghical device (ash::Device/VkDevice) for the found VkQueueFamilyIndices*
    /// <pre>
    /// - Params
    ///     <b>&mut self</b>
    ///     device:     &ash::Device
    /// </pre>
    pub fn query_queues(&mut self, device: &ash::Device)
    {
        if self.graphics.index.is_some()
        {
            self.graphics.handle = unsafe { device.get_device_queue(self.graphics.index.clone().unwrap(), 0) };
        }
    }

    /// ### fn get_index_list( &self ) -> Vec\<u32\>
    /// *Returns a list (Vec\<u32\>) of the found VkQueueFamilyIndices*
    /// <pre>
    /// - Params
    ///     <b>&self</b>
    /// - Return
    ///     Vec&lt;u32&gt;
    /// </pre>
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

/// ### SpVkSwapchain struct
/// *Contains handle to vk::Swapchain and all related data.*</br>
/// *SpVkSwapchain is responsible for the images rendered to screen.*
/// <pre>
/// - Members
///     loader:     khr::Swapchain
///     handle:     vk::SwapchainKHR
///     images:     Vec&lt;vk::Image&gt;
///     views:      Vec&lt;vk::ImageView&gt;
///     format:     vk::Format
///     extent:     vk::Extent2D
/// </pre>
pub struct SpVkSwapchain
{
    pub loader:     ash::extensions::khr::Swapchain,
    pub handle:     vk::SwapchainKHR,
    pub images:     Vec<vk::Image>,
    pub views:      Vec<vk::ImageView>,
    pub format:     vk::Format,
    pub extent:     vk::Extent2D
}

impl SpVkSwapchain
{

    /// ### fn SpVkSWapchain::new( ... ) -> SpVkSwapchain
    /// *Creates an instance of SpVkSwapchain.*
    /// <pre>
    /// - Param
    ///     loader:             &SpVkLoader
    ///     device:             &ash::Device
    ///     physical_device:    &vk::PhysicalDevice
    ///     queue_indices:      &Vec&lt;u32&gt;
    ///     width:              u32
    ///     height:             u32
    /// - Return
    ///     SpVkSwapchain
    /// </pre>
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
            vk_check!( loader.get_swapchain_images(handle) ).unwrap()
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

    /// ### fn SpVkSwapchain::destroy( &self, ... )
    /// *Destroys an instance of SpVkSwapchain.*
    /// <pre>
    /// - Param
    ///     <b>&self</b>
    ///     device:     &ash::Device
    /// </pre>
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

/// ### SpVkCommands struct
/// *Contains vk::CommandPool and vk::CommandBuffer(s).*</br>
/// *SpVkCommands are the allocated commands set to execute on the gpu.*
/// <pre>
/// - Members
///     pool:       vk::CommandPool
///     buffers:    Vec&lt;vk::CommandBuffer&gt;
/// </pre>
pub struct  SpVkCommands
{
    pub pool:       vk::CommandPool,
    pub buffers:    Vec<vk::CommandBuffer>
}

impl SpVkCommands
{
    /// ### fn SpVkCommands::new( ... ) -> SpVkCommands
    /// *Creates an instance of SpVkCommands.*
    /// <pre>
    /// - Params
    ///     device:                 &ash::Device
    ///     queue_family_index:     u32
    ///     buffer_count:           u32
    /// - Return
    ///     SpVkCommands
    /// </pre>
    pub fn new(device: &ash::Device, queue_family_index: u32, buffer_count: u32) -> Self
    {
        let pool = create_vk_command_pool(device, queue_family_index);
        let buffers = allocate_vk_command_buffers(device, &pool, buffer_count);

        Self{ pool, buffers }
    }

    /// ### fn SpVkCommands::destroy( &self, ... )
    /// *Destroys the instance of SpVkCommands.*
    /// <pre>
    /// - Params
    ///     <b>&self</b>
    ///     device:     &ash::Device
    /// </pre>
    pub fn destroy(&self, device: &ash::Device)
    {
        unsafe
        {
            device.destroy_command_pool(self.pool, None);
        }
    }
}

/// ### SpVkContext struct
/// *Vulkan rendering Context.*<br>
/// <pre>
/// - Members
///     device:             ash::Device
///     physical_device:    vk::PhysicalDevice
///     allocator:          gpu_allocator::vulkan::Allocator
///     queues:             SpVkQueues
///     swapchain:          SpVkSwapChain
///     draw_cmds:          SpVkCommands
///     render_semaphore:   vk::Semaphore
///     wait_semaphore:     vk::Semaphore
/// </pre>
pub struct SpVkContext
{
    pub device:             Device,
    pub physical_device:    vk::PhysicalDevice,
    pub allocator:          Allocator,
    pub queues:             SpVkQueues,
    pub swapchain:          SpVkSwapchain,
    pub draw_cmds:          SpVkCommands,
    pub render_semaphore:   vk::Semaphore,
    pub wait_semaphore:     vk::Semaphore,
}

impl SpVkContext
{
    /// ### fn SpVkContext::new( ... ) -> SpVkContext
    /// *Creates an instance of SpVkContext.*
    /// <pre>
    /// - Params
    ///     loader:     &SpVkLoader
    ///     width:      u32
    ///     height:     u32
    /// </pre>
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

    /// ### fn SpVkContext::destroy( &self )
    /// *Destroys the instance of SpVkContext.*
    /// <pre>
    /// - Params
    ///     <b>&self</b>
    /// </pre>
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

/// ### fn sp_begin_single_time_vk_command_buffer( ... ) -> vk::CommandBuffer
/// *Allocates and sets up a vk::CommandBuffer for temporary use, then returns it.*
/// <pre>
/// - Params
///     vk_ctx:     &SpVkContext
/// - Return
///     vk::CommandBuffer
/// </pre>
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

/// ### fn sp_end_single_time_vk_command_buffer( ... )
/// *Submits a temporary vk::CommandBuffer to the gpu for execution, then frees it from memory.*
/// <pre>
/// - Params
///     vk_ctx:         &SpVkContext
///     cmd_buffer:     &vk::CommandBuffer
/// </pre>
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


pub fn sp_create_vk_color_depth_framebuffers(
        vk_ctx: &SpVkContext, 
        renderpass: &SpVkRenderPass, 
        depth_view: &vk::ImageView
    ) -> Vec<vk::Framebuffer>
{
    let mut framebuffers: Vec<vk::Framebuffer> = Vec::new();

    for image_view in vk_ctx.swapchain.views.iter()
    {
        let attachments = [
            *image_view,
            *depth_view
        ];

        let create_info = vk::FramebufferCreateInfo
        {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass: renderpass.handle,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: vk_ctx.swapchain.extent.width,
            height: vk_ctx.swapchain.extent.height,
            layers: 1
        };

        let framebuffer = unsafe {
            vk_check!(vk_ctx.device.create_framebuffer(&create_info, None)).unwrap()
        };

        framebuffers.push(framebuffer);
    }

    framebuffers
}