use ash::{ vk, Device };
use gpu_allocator::vulkan::Allocator;

use crate::{log_err, log_info, vk_check};

use super::vk_utils::*;
use super::splunk_vk_loader::SpVkLoader;
use super::splunk_vk_img::create_vk_image_view;
use super::splunk_vk_render_pass::SpVkRenderPass;
use super::vk_shader_utils::{ 
    SpVkShaderModule, 
    sp_create_shader_module, sp_destroy_shader_module, 
    sp_get_vk_shader_create_info, get_vk_shader_stage_from_filename
};

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


pub fn sp_create_graphics_pipeline(
        vk_ctx: &SpVkContext, 
        renderpass: &SpVkRenderPass, 
        layout: &vk::PipelineLayout, 
        shader_files: &Vec<&str>,
        topology: vk::PrimitiveTopology,
        // b_dynamic_scissor: bool,
        b_use_blending: bool,
        custom_size: Option<vk::Extent2D>,
        num_patch_control_points: u32
    ) -> vk::Pipeline
{
    let mut shader_modules: Vec<SpVkShaderModule> = Vec::new();
    let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();

    for shader_file in shader_files.iter()
    {
        let file_path = std::path::Path::new(shader_file);
        let shader_module = sp_create_shader_module(&vk_ctx.device, &file_path);
        
        let stage = get_vk_shader_stage_from_filename(&file_path);

        let shader_stage_ci = sp_get_vk_shader_create_info(&shader_module, stage, "main");

        shader_modules.push(shader_module);
        shader_stages.push(shader_stage_ci);
    }

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        ..Default::default()
    };

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        topology: topology,
        primitive_restart_enable: vk::FALSE,
        ..Default::default()
    };

    let size: vk::Extent2D;
    if custom_size.is_some()
    {
        size = custom_size.clone().unwrap();
    } else {
        size = vk_ctx.swapchain.extent.clone();
    }
    let viewport = vk::Viewport
    {
        x: 0.0,
        y: 0.0,
        width: size.width as f32,
        height: size.height as f32,
        min_depth: 0.0,
        max_depth: 1.0
    };

    let scissor = vk::Rect2D
    {
        offset: vk::Offset2D{ x: 0, y: 0 },
        extent: size
    };

    let viewport_state_info = vk::PipelineViewportStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        viewport_count: 1,
        p_viewports: &viewport,
        scissor_count: 1,
        p_scissors: &scissor,
        ..Default::default()
    };

    let rasterizer_state_info = vk::PipelineRasterizationStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::CLOCKWISE,
        line_width: 1.0,
        ..Default::default()
    };

    let multi_sample_state_info = vk::PipelineMultisampleStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 1.0,
        ..Default::default()
    };

    let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState
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
            vk::ColorComponentFlags::A,
    };

    let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        logic_op_enable: vk::FALSE,
        logic_op: vk::LogicOp::COPY,
        attachment_count: 1,
        p_attachments: &color_blend_attachment_state,
        blend_constants: [ 0.0, 0.0, 0.0, 0.0 ],
        ..Default::default()
    };

    let depth_stencil_state_info = vk::PipelineDepthStencilStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        depth_test_enable: if renderpass.info.b_use_depth { vk::TRUE } else { vk::FALSE },
        depth_write_enable: if renderpass.info.b_use_depth { vk::TRUE } else { vk::FALSE },
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: vk::FALSE,
        min_depth_bounds: 0.0,
        max_depth_bounds: 1.0,
        ..Default::default()
    };

    let dynamic_states = vec![ vk::DynamicState::SCISSOR ];
    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr()
    };

    let tessellation_state_info = vk::PipelineTessellationStateCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineTessellationStateCreateFlags::empty(),
        patch_control_points: num_patch_control_points
    };

    let create_info = vk::GraphicsPipelineCreateInfo
    {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        stage_count: shader_stages.len() as u32,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_state,
        p_input_assembly_state: &input_assembly_state,
        p_tessellation_state: &tessellation_state_info,
        p_viewport_state: &viewport_state_info,
        p_rasterization_state: &rasterizer_state_info,
        p_multisample_state: &multi_sample_state_info,
        p_depth_stencil_state: &depth_stencil_state_info,
        p_color_blend_state: &color_blend_state_info,
        p_dynamic_state: &dynamic_state_info,
        layout: *layout,
        render_pass: renderpass.handle,
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: -1,
        ..Default::default()
    };

    let pipeline = unsafe 
    {
        vk_ctx.device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None).map_err(|e| { log_err!(e.1); } ).unwrap()[0]
    };

    for module in shader_modules.iter_mut()
    {
        sp_destroy_shader_module(&vk_ctx.device,  module);
    }

    pipeline
}