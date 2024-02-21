use ash::{ vk, Device };
use gpu_allocator::vulkan::Allocator;

use crate::{log_info, vk_check};

use super::gunk_vk_render_pass::GkVkRenderPass;
use super::vk_utils::*;
use super::gunk_vk_img::create_vk_image_view;

use super::gunk_vk_loader::GkVkLoader;

/// ### GkVkQueue struct
/// *Contain Vulkan queue family index and a VkQueue handle*
/// <pre>
/// - Members
///     index:      Option&lt;u32&gt;   <i>// Queue family index</i>
///     handle:     vk::Queue     <i>// VkQueue handle</i>
/// </pre>
pub struct GkVkQueue
{
    pub index:      Option<u32>,
    pub handle:     vk::Queue,
}

impl GkVkQueue
{
    /// ### GkVkQueue::new() -> GkVkQueue
    /// *Returns a new instance of GkVkQueue*<br>
    /// *Index and handle are not set*
    /// <pre>
    /// - Return
    ///     GkVkQueue
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

/// ### GkVkQueues struct
/// *A convenience struct containing different "families" of GkVkQueue struct*
/// <pre>
/// - Members
///     graphics:       GkVkQueue       <i>// GkVkQueue for graphics family of instructrions.</i>
/// </pre>
pub struct GkVkQueues
{
    pub graphics:   GkVkQueue,
}

impl GkVkQueues
{
    /// ### GkVkQueues::new() -> GkVkQueues
    /// *Returns a new instance of GkVkQueues.*<br><br>
    /// *Each GkVkQueue family is not yet populated with usable values.*<br>
    /// *Must call **fn queury_indices()** and **fn queury_queues()***
    /// <pre>
    /// - Return 
    ///     GkVkQueues
    /// </pre>
    pub fn new() -> Self
    { 
        Self
        { 
            graphics: GkVkQueue::new() 
        } 
    } 

    /// ### fn GkVkQueues::queury_indices( &mut self, ... )
    /// *Queries VkQueueFamilyIndices from the physical device.*<br>
    /// *Populates the GkVkQueue families if found.*
    /// <pre>
    /// - Params
    ///     <b>&mut self</b>
    ///     instance:           &ash::instance
    ///     physical_device:    &vk::PhysicalDevice    
    /// </pre>
    pub fn query_indices(&mut self, instance: &ash::Instance, physical_device: &vk::PhysicalDevice)
    {
        let device_queue_families = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        // let mut graphics = GkVkQueue::new();
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

/// ### GkVkSwapchain struct
/// *Contains handle to vk::Swapchain and all related data.*</br>
/// *GkVkSwapchain is responsible for the images rendered to screen.*
/// <pre>
/// - Members
///     loader:     khr::Swapchain
///     handle:     vk::SwapchainKHR
///     images:     Vec&lt;vk::Image&gt;
///     views:      Vec&lt;vk::ImageView&gt;
///     format:     vk::Format
///     extent:     vk::Extent2D
/// </pre>
pub struct GkVkSwapchain
{
    pub loader:     ash::extensions::khr::Swapchain,
    pub handle:     vk::SwapchainKHR,
    pub images:     Vec<vk::Image>,
    pub views:      Vec<vk::ImageView>,
    pub format:     vk::Format,
    pub extent:     vk::Extent2D
}

impl GkVkSwapchain
{

    /// ### fn GkVkSWapchain::new( ... ) -> GkVkSwapchain
    /// *Creates an instance of GkVkSwapchain.*
    /// <pre>
    /// - Param
    ///     loader:             &GkVkLoader
    ///     device:             &ash::Device
    ///     physical_device:    &vk::PhysicalDevice
    ///     queue_indices:      &Vec&lt;u32&gt;
    ///     width:              u32
    ///     height:             u32
    /// - Return
    ///     GkVkSwapchain
    /// </pre>
    pub fn new(loader: &GkVkLoader, device: &ash::Device, physical_device: &vk::PhysicalDevice, queue_indices: &Vec<u32>,  width: u32, height: u32) -> Self
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

    /// ### fn GkVkSwapchain::destroy( &self, ... )
    /// *Destroys an instance of GkVkSwapchain.*
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

/// ### GkVkCommands struct
/// *Contains vk::CommandPool and vk::CommandBuffer(s).*</br>
/// *GkVkCommands are the allocated commands set to execute on the gpu.*
/// <pre>
/// - Members
///     pool:       vk::CommandPool
///     buffers:    Vec&lt;vk::CommandBuffer&gt;
/// </pre>
pub struct  GkVkCommands
{
    pub pool:               vk::CommandPool,
    pub buffers:            Vec<vk::CommandBuffer>,
    // current_frame_index:    usize
}

impl GkVkCommands
{
    /// ### fn GkVkCommands::new( ... ) -> GkVkCommands
    /// *Creates an instance of GkVkCommands.*
    /// <pre>
    /// - Params
    ///     device:                 &ash::Device
    ///     queue_family_index:     u32
    ///     buffer_count:           u32
    /// - Return
    ///     GkVkCommands
    /// </pre>
    pub fn new(device: &ash::Device, queue_family_index: u32, buffer_count: u32) -> Self
    {
        let pool = create_vk_command_pool(device, queue_family_index);
        let buffers = allocate_vk_command_buffers(device, &pool, buffer_count);

        // Self{ pool, buffers, current_frame_index: 0 }
        Self{ pool, buffers }
    }

    /// ### fn GkVkCommands::destroy( &self, ... )
    /// *Destroys the instance of GkVkCommands.*
    /// <pre>
    /// - Params
    ///     <b>&self</b>
    ///     device:     &ash::Device
    /// </pre>
    pub fn destroy(&self, device: &ash::Device)
    {
        unsafe{ device.destroy_command_pool(self.pool, None); }
    }

    // pub fn reset_pool(&self, device: &ash::Device)
    // {
    //     unsafe { vk_check!(device.reset_command_pool(self.pool, vk::CommandPoolResetFlags::empty())); }
    // }

    // pub fn reset_buffer(&self, device: &ash::Device)
    // {
    //     unsafe { vk_check!(device.reset_command_buffer(self.buffers[self.current_frame_index], vk::CommandBufferResetFlags::empty())).unwrap(); }
    // }

    // pub fn set_next_index(&mut self, next_index: usize)
    // {
    //     self.current_frame_index = next_index;
    // }

    // pub fn get_current_buffer(&self) -> &vk::CommandBuffer
    // {
    //     &self.buffers[self.current_frame_index]
    // }
}

pub struct GkVkFrameSync
{
    pub wait_semaphores:    Vec<vk::Semaphore>,
    pub render_semaphores:  Vec<vk::Semaphore>,
    pub in_flight_fences:   Vec<vk::Fence>,
    frames_in_flight:       usize,
    current_frame_index:    usize,
}

impl GkVkFrameSync
{
    pub fn new(device: &ash::Device, frames_in_flight: usize) -> Self
    {
        let mut wait_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut render_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut in_flight_fences: Vec<vk::Fence> = Vec::new();

        for _i in 0..frames_in_flight
        {
            wait_semaphores.push(create_vk_semaphore(device));
            render_semaphores.push(create_vk_semaphore(device));
            in_flight_fences.push(create_vk_fence(device, true));
        }

        Self
        {
            wait_semaphores,
            render_semaphores,
            in_flight_fences,
            frames_in_flight,
            current_frame_index: 0
        }
    }

    pub fn destroy(&mut self, device: &ash::Device)
    {
        for i in 0..self.frames_in_flight
        {
            unsafe
            {
                device.destroy_semaphore(self.wait_semaphores[i], None);
                device.destroy_semaphore(self.render_semaphores[i], None);
                device.destroy_fence(self.in_flight_fences[i], None);
            }
        }
        self.wait_semaphores.clear();
        self.render_semaphores.clear();
        self.in_flight_fences.clear();
    }

    pub fn get_num_frames_in_flight(&self) -> usize
    {
        self.frames_in_flight
    }

    pub fn get_current_frame_index(&self) -> usize
    {
        self.current_frame_index
    }

    pub fn set_next_frame_index(&mut self)
    {
        self.current_frame_index = (self.current_frame_index + 1) % self.frames_in_flight;
    }

    pub fn get_current_wait_semaphore(&self) -> &vk::Semaphore
    {
        &self.wait_semaphores[self.current_frame_index]
    }

    pub fn get_current_render_semaphore(&self) -> &vk::Semaphore
    {
        &self.render_semaphores[self.current_frame_index]
    }

    pub fn get_current_in_flight_fence(&self) -> &vk::Fence
    {
        &self.in_flight_fences[self.current_frame_index]
    }
}

/// ### GkVkContext struct
/// *Vulkan rendering Context.*<br>
/// <pre>
/// - Members
///     device:             ash::Device
///     physical_device:    vk::PhysicalDevice
///     allocator:          gpu_allocator::vulkan::Allocator
///     queues:             GkVkQueues
///     swapchain:          GkVkSwapChain
///     draw_cmds:          GkVkCommands
///     render_semaphore:   vk::Semaphore
///     wait_semaphore:     vk::Semaphore
/// </pre>
pub struct GkVkContext
{
    pub device:             Device,
    pub physical_device:    vk::PhysicalDevice,
    pub allocator:          Option<Allocator>,
    pub queues:             GkVkQueues,
    pub swapchain:          GkVkSwapchain,
    pub draw_cmds:          GkVkCommands,
    // pub render_semaphore:   vk::Semaphore,
    // pub wait_semaphore:     vk::Semaphore,
    pub frame_sync:         GkVkFrameSync,
}

impl GkVkContext
{
    /// ### fn GkVkContext::new( ... ) -> GkVkContext
    /// *Creates an instance of GkVkContext.*
    /// <pre>
    /// - Params
    ///     loader:     &GkVkLoader
    ///     width:      u32
    ///     height:     u32
    /// </pre>
    pub fn new(loader: &GkVkLoader, width: u32, height: u32) -> Self
    {
        log_info!("Creating VulkanContext...");

        let physical_device = find_suitable_vk_physical_device(&loader.instance, &loader.surface);

        let mut queues = GkVkQueues::new();
        queues.query_indices(&loader.instance, &physical_device);

        let queue_index_list = queues.get_index_list();
        let device = create_vk_device(&loader.instance, &physical_device, &queue_index_list);
        queues.query_queues(&device);

        let allocator = create_vk_allocator(&loader.instance, &physical_device, &device);

        let swapchain = GkVkSwapchain::new(loader, &device, &physical_device, &queue_index_list, width, height);

        let frame_sync = GkVkFrameSync::new(&device, 2);
        
        let draw_cmds = GkVkCommands::new(&device, queues.graphics.index.clone().unwrap(), frame_sync.get_num_frames_in_flight() as u32);

        log_info!("VulkanContext created");
        Self
        {
            device,
            physical_device,
            allocator: Some(allocator),
            queues,
            swapchain,
            draw_cmds,
            frame_sync
        }
    }

    /// ### fn GkVkContext::destroy( &self )
    /// *Destroys the instance of GkVkContext.*
    /// <pre>
    /// - Params
    ///     <b>&self</b>
    /// </pre>
    pub fn destroy(&mut self)
    {
        self.clean_swapchain();
        self.frame_sync.destroy(&self.device);
        self.draw_cmds.destroy(&self.device);
        drop(self.allocator.take().unwrap());
        unsafe
        {
            self.device.destroy_device(None);
        }
    }

    pub fn clean_swapchain(&mut self)
    {
        self.swapchain.destroy(&self.device);
    }

    pub fn recreate_swapchain(&mut self, loader: &GkVkLoader, width: u32, height: u32)
    {
        self.swapchain = GkVkSwapchain::new(loader, &self.device, &self.physical_device, &self.queues.get_index_list(), width, height);
    }

    // pub fn reset_draw_cmd_pool(&self)
    // {
    //     self.draw_cmds.reset_pool(&self.device);
    // }

    // pub fn reset_current_draw_cmd_buffer(&self)
    // {
    //     self.draw_cmds.reset_buffer(&self.device);
    // }

    // pub fn set_next_frame_index(&mut self) 
    // {
    //     self.frame_sync.set_next_frame_index();
    //     self.draw_cmds.set_next_index(self.frame_sync.get_current_frame_index());
    // }

}

/// ### fn gk_begin_single_time_vk_command_buffer( ... ) -> vk::CommandBuffer
/// *Allocates and sets up a vk::CommandBuffer for temporary use, then returns it.*
/// <pre>
/// - Params
///     vk_ctx:     &GkVkContext
/// - Return
///     vk::CommandBuffer
/// </pre>
pub fn gk_begin_single_time_vk_command_buffer(vk_ctx: &GkVkContext) -> vk::CommandBuffer
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

/// ### fn gk_end_single_time_vk_command_buffer( ... )
/// *Submits a temporary vk::CommandBuffer to the gpu for execution, then frees it from memory.*
/// <pre>
/// - Params
///     vk_ctx:         &GkVkContext
///     cmd_buffer:     &vk::CommandBuffer
/// </pre>
pub fn gk_end_single_time_vk_command_buffer(vk_ctx: &GkVkContext, cmd_buffer: vk::CommandBuffer)
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


pub fn gk_create_vk_color_depth_framebuffers(
        vk_ctx: &GkVkContext, 
        renderpass: &GkVkRenderPass, 
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


pub fn gk_create_vk_color_only_framebuffers(
        vk_ctx: &GkVkContext, 
        renderpass: &GkVkRenderPass
    ) -> Vec<vk::Framebuffer>
{
    let mut framebuffers: Vec<vk::Framebuffer> = Vec::new();

    for image_view in vk_ctx.swapchain.views.iter()
    {
        let attachments = [
            *image_view,
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

pub fn gk_destroy_vk_framebuffers(device: &ash::Device, framebuffers: &mut Vec<vk::Framebuffer>)
{
    unsafe{
        for framebuffer in framebuffers.iter()
        {
            device.destroy_framebuffer(*framebuffer, None);
        }
    }
    framebuffers.clear();
}