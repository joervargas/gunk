// use std::ffi::CString;

// use ash::{self, vk};

// use crate::{
//     renderer::{vulkan_renderer::gk_vulkan::{
//         gunk_vk_context::{SpVkContext, sp_create_vk_color_depth_framebuffers, sp_destroy_vk_framebuffers},
//         gunk_vk_render_pass::{SpVkRenderPass, SpVkRenderPassInfo, ERenderPassBit, sp_create_vk_renderpass, sp_destroy_vk_renderpass}, 
//         vk_utils::{
//             create_vk_pipeline_info_vertex_input, create_vk_pipeline_info_assembly, 
//             create_vk_pipeline_info_viewport, create_vk_pipeline_info_rasterization, 
//             create_vk_pipeline_info_multisample, create_vk_pipeline_info_color_blend, 
//             create_vk_pipeline_info_color_blend_attachment, create_vk_pipeline_info_depth_stencil, 
//             create_vk_pipeline_info_dynamic_states, create_vk_pipeline_info_tessellation, create_vk_pipeline_layout
//         }, 
//         vk_shader_utils::SpVkShaderModule, 
//         gunk_vk_buffer::{SpVkBuffer, sp_destroy_vk_buffer}, 
//         gunk_vk_descriptor::{
//             SpVkDescriptor, 
//             get_vk_desc_set_layout_binding, sp_create_vk_desc_pool, 
//             get_vk_buffer_write_desc_set, get_vk_image_write_desc_set, sp_destroy_vk_descriptor
//         }, 
//         gunk_vk_img::{SpVkImage, sp_create_vk_image, create_vk_sampler, sp_destroy_vk_img}
// }, renderer_utils::to_shader_path}, log_info, log_err, vk_check, log_warn};

// use super::sp_vk_render_layer::{SpVkLayerDraw, SpVk3dLayerUpdate};

// pub struct VkModelLayer
// {
//     renderpass:         SpVkRenderPass,
//     framebuffers:       Vec<vk::Framebuffer>,
//     // descriptor:         SpVkDescriptor,
//     pipeline_layout:    vk::PipelineLayout,
//     pipeline:           vk::Pipeline,
//     storage_vert:       Option<SpVkBuffer>,
//     storage_index:      Option<SpVkBuffer>,
//     texture:            Option<SpVkImage>,
//     sampler:            vk::Sampler
// }

// impl VkModelLayer
// {
//     pub fn new(
//             instance: &ash::Instance,
//             vk_ctx: &mut SpVkContext,
//             uniform_buffers: &Vec<SpVkBuffer>,
//             depth_img: &SpVkImage,
//             model_file: &std::path::Path, texture_file: &std::path::Path
//         ) -> Self
//     {
//         let texture = sp_create_vk_image(vk_ctx, texture_file.to_str().unwrap());
//         let sampler = create_vk_sampler(&vk_ctx.device);

//         // let (storage_vert, storage_index) = sp_create_vk_vertex_buffer_from_file(
//         //     vk_ctx, 
//         //     "Duck", 
//         //     // vk::BufferUsageFlags::STORAGE_BUFFER, 
//         //     model_file
//         // );
            
//         // let descriptor = Self::create_desc_sets(
//         //     &vk_ctx, 
//         //     uniform_buffers, 
//         //     &storage_vert, 
//         //     &storage_index, 
//         //     &texture, &sampler
//         // );

//         let renderpass_info = SpVkRenderPassInfo{
//             b_use_color: true,
//             b_clear_color: false,
//             b_use_depth: true,
//             b_clear_depth: false,
//             color_format: vk::Format::B8G8R8A8_UNORM,
//             flags: ERenderPassBit::NONE,
//             samples: vk::SampleCountFlags::TYPE_1
//         };

//         let renderpass = sp_create_vk_renderpass(instance, vk_ctx, renderpass_info);
//         let framebuffers = sp_create_vk_color_depth_framebuffers(vk_ctx, &renderpass, &depth_img.view);
//         let pipeline_layout = create_vk_pipeline_layout(&vk_ctx.device, &descriptor.layouts, &Vec::new());

//         let mut shader_modules: Vec<SpVkShaderModule> = vec![
//             SpVkShaderModule::new(&vk_ctx.device, to_shader_path("ModelLayer.vert").as_path()),
//             SpVkShaderModule::new(&vk_ctx.device, to_shader_path("ModelLayer.geom").as_path()),
//             SpVkShaderModule::new(&vk_ctx.device, to_shader_path("ModelLayer.frag").as_path())
//         ];

//         let pipeline = Self::create_pipeline(
//             vk_ctx,
//             &mut shader_modules, 
//             &renderpass, 
//             &pipeline_layout, None
//         );

//         Self
//         {
//             renderpass,
//             framebuffers,
//             // descriptor,
//             pipeline_layout,
//             pipeline,
//             // storage_vert: Some(storage_vert),
//             // storage_index: Some(storage_index),
//             storage_vert: None,
//             storage_index: None,
//             texture: Some(texture),
//             sampler
//         }
//     }

//     // fn create_desc_sets(
//     //         vk_ctx: &SpVkContext,
//     //         uniform_buffers: &Vec<SpVkBuffer>, 
//     //         storage_vert: &SpVkBuffer, storage_index: &SpVkBuffer,
//     //         texture: &SpVkImage,
//     //         sampler: &vk::Sampler
//     //     ) -> SpVkDescriptor
//     // {
//     //     let pool = sp_create_vk_desc_pool(vk_ctx, 1, 2, 1);

//     //     let bindings: Vec<vk::DescriptorSetLayoutBinding> = vec![
//     //         get_vk_desc_set_layout_binding(0, vk::DescriptorType::UNIFORM_BUFFER, 1, vk::ShaderStageFlags::VERTEX),
//     //         get_vk_desc_set_layout_binding(1, vk::DescriptorType::STORAGE_BUFFER, 1, vk::ShaderStageFlags::VERTEX),
//     //         get_vk_desc_set_layout_binding(2, vk::DescriptorType::STORAGE_BUFFER, 1, vk::ShaderStageFlags::VERTEX),
//     //         get_vk_desc_set_layout_binding(3, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 1, vk::ShaderStageFlags::FRAGMENT)
//     //     ];

//     //     let layout_info = vk::DescriptorSetLayoutCreateInfo
//     //     {
//     //         s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
//     //         p_next: std::ptr::null(),
//     //         flags: vk::DescriptorSetLayoutCreateFlags::empty(),
//     //         binding_count: bindings.len() as u32,
//     //         p_bindings: bindings.as_ptr()
//     //     };
//     //     let layout = unsafe{
//     //         vk_check!(vk_ctx.device.create_descriptor_set_layout(&layout_info, None)).unwrap()
//     //     };

//     //     let layouts: Vec<vk::DescriptorSetLayout> = vec![layout; vk_ctx.swapchain.images.len()];
//     //     let alloc_info = vk::DescriptorSetAllocateInfo
//     //     {
//     //         s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
//     //         p_next: std::ptr::null(),
//     //         descriptor_pool: pool,
//     //         descriptor_set_count: layouts.len() as u32,
//     //         p_set_layouts: layouts.as_ptr()
//     //     };

//     //     let sets = unsafe {
//     //         vk_check!(vk_ctx.device.allocate_descriptor_sets(&alloc_info)).unwrap()
//     //     };

//     //     for i in 0..vk_ctx.swapchain.images.len()
//     //     {
//     //         let buffer_info1 = vk::DescriptorBufferInfo{ buffer: uniform_buffers[i].handle, offset: 0, range: uniform_buffers[i].size };
//     //         let buffer_info2 = vk::DescriptorBufferInfo{ buffer: storage_vert.handle, offset: 0, range: storage_vert.size };
//     //         let buffer_info3 = vk::DescriptorBufferInfo{ buffer: storage_index.handle, offset: 0, range: storage_index.size };
//     //         let image_info1 = vk::DescriptorImageInfo{ sampler: *sampler, image_view: texture.view, image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL};

//     //         let desc_writes: Vec<vk::WriteDescriptorSet> = vec![
//     //             get_vk_buffer_write_desc_set(&sets[i], &[buffer_info1], 0, vk::DescriptorType::UNIFORM_BUFFER),
//     //             get_vk_buffer_write_desc_set(&sets[i], &[buffer_info2], 1, vk::DescriptorType::STORAGE_BUFFER),
//     //             get_vk_buffer_write_desc_set(&sets[i], &[buffer_info3], 2, vk::DescriptorType::STORAGE_BUFFER),
//     //             get_vk_image_write_desc_set(&sets[i], &[image_info1], 3) 
//     //         ];

//     //         unsafe {
//     //             vk_ctx.device.update_descriptor_sets(desc_writes.as_slice(), &[])
//     //         }
//     //     }   

//     //     SpVkDescriptor
//     //     { 
//     //         layouts, 
//     //         pool, 
//     //         sets 
//     //     }
//     // }

//     fn create_pipeline(
//             vk_ctx: &SpVkContext,
//             shader_modules: &mut Vec<SpVkShaderModule>,
//             renderpass: &SpVkRenderPass,
//             layout: &vk::PipelineLayout,
//             _custom_extent: Option<vk::Extent2D>
//         ) -> vk::Pipeline
//     {
//         log_info!("creating VkModelLayer pipeline... ");

//         let mut shader_stage_infos: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();
//         let entry_point = CString::new("main").unwrap();
//         for shader in shader_modules.iter()
//         {
//             shader_stage_infos.push(shader.get_vk_pipeline_info_shader_stage(&entry_point));
//         }

//         let vertex_input_info = create_vk_pipeline_info_vertex_input();
//         let assembly_info = create_vk_pipeline_info_assembly(vk::PrimitiveTopology::TRIANGLE_LIST, vk::FALSE);

//         // let mut custom_width: u32 = 0;
//         // let mut custom_height: u32 = 0;
//         // if custom_extent.is_some()
//         // {
//         //     custom_width = custom_extent.as_ref().unwrap().width.clone();
//         //     custom_height = custom_extent.as_ref().unwrap().height.clone();
//         // }

//         log_warn!("swapchain extent: ", vk_ctx.swapchain.extent.width, vk_ctx.swapchain.extent.height);
//         let viewports: Vec<vk::Viewport> = vec![
//             vk::Viewport
//             {
//                 x: 0.0,
//                 y: 0.0,
//                 // width: if custom_width > 0 { custom_width as f32 } else { vk_ctx.swapchain.extent.width as f32 },
//                 // height: if custom_height > 0 { custom_height as f32 } else { vk_ctx.swapchain.extent.height as f32 },
//                 width: vk_ctx.swapchain.extent.width as f32,
//                 height: vk_ctx.swapchain.extent.height as f32,
//                 min_depth: 0.0,
//                 max_depth: 1.0
//             }
//         ];
//         let scissors: Vec<vk::Rect2D> = vec![
//             vk::Rect2D
//             {
//                 offset: vk::Offset2D{ x: 0, y: 0 },
//                 extent: 
//                     vk::Extent2D
//                     { 
//                         // width: if custom_width > 0 { custom_width } else { vk_ctx.swapchain.extent.width },
//                         // height: if custom_height > 0 { custom_height } else { vk_ctx.swapchain.extent.height }
//                         width: vk_ctx.swapchain.extent.width,
//                         height: vk_ctx.swapchain.extent.height
//                     }
//             }
//         ];
//         let viewport_info = create_vk_pipeline_info_viewport(viewports, scissors);
    
//         let rasterizer_info = create_vk_pipeline_info_rasterization(vk::PolygonMode::FILL, vk::CullModeFlags::NONE, vk::FrontFace::CLOCKWISE, 1.0);
//         let multisampling_info = create_vk_pipeline_info_multisample(vk::SampleCountFlags::TYPE_1, vk::FALSE, 1.0);

//         let color_attachments: Vec<vk::PipelineColorBlendAttachmentState> = vec![
//             create_vk_pipeline_info_color_blend_attachment(true)
//         ];
//         let color_blending_info = create_vk_pipeline_info_color_blend(&color_attachments);

//         let depth_stencil_info = create_vk_pipeline_info_depth_stencil();

//         let dynamic_states: Vec<vk::DynamicState> = vec![
//             vk::DynamicState::VIEWPORT,
//             vk::DynamicState::SCISSOR
//         ];
//         let dynamic_info = create_vk_pipeline_info_dynamic_states(&dynamic_states);

//         let tessellation_info = create_vk_pipeline_info_tessellation(0);

//         let create_info = vk::GraphicsPipelineCreateInfo
//         {
//             s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
//             p_next: std::ptr::null(),
//             flags: vk::PipelineCreateFlags::empty(),
//             stage_count: shader_stage_infos.len() as u32,
//             p_stages: shader_stage_infos.as_ptr(),
//             p_vertex_input_state: &vertex_input_info,
//             p_input_assembly_state: &assembly_info,
//             p_viewport_state: &viewport_info,
//             p_rasterization_state: &rasterizer_info,
//             p_multisample_state: &multisampling_info,
//             p_color_blend_state: &color_blending_info,
//             p_depth_stencil_state: &depth_stencil_info,
//             p_dynamic_state: &dynamic_info,
//             p_tessellation_state: &tessellation_info,
//             layout: *layout,
//             render_pass: renderpass.handle,
//             subpass: 0,
//             base_pipeline_handle: vk::Pipeline::null(),
//             base_pipeline_index: -1
//         };

//         let pipeline = unsafe {
//             vk_ctx.device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None).map_err(|e| { log_err!("{}", e.1); }).unwrap()[0]
//         };
//         log_info!("VkModelLayer pipeline created.");

//         for shader in shader_modules.iter_mut()
//         {
//             shader.destroy(&vk_ctx.device);
//         }
        
//         pipeline
//     }

//     fn draw(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer)
//     {
//         unsafe{
//             vk_ctx.device.cmd_draw(*cmd_buffer, (self.storage_index.as_ref().unwrap().size / std::mem::size_of::<u32>() as vk::DeviceSize) as u32, 1, 0, 0);
//         }
//     }

// }


// impl SpVkLayerDraw for VkModelLayer
// {
//     fn draw_frame(&self, vk_ctx: &SpVkContext, cmd_buffer: &vk::CommandBuffer, current_image: usize)
//     {
//         self.begin_renderpass(vk_ctx, cmd_buffer, &self.renderpass, self.pipeline, self.framebuffers[current_image]);
//         self.draw(vk_ctx, cmd_buffer);
//         self.end_renderpass(vk_ctx, cmd_buffer);
//     }

//     fn destroy(&mut self, vk_ctx: &mut SpVkContext)
//     {
//         sp_destroy_vk_buffer(vk_ctx, self.storage_vert.take().unwrap());
//         sp_destroy_vk_buffer(vk_ctx, self.storage_index.take().unwrap());
//         sp_destroy_vk_img(vk_ctx, self.texture.take().unwrap());
//         unsafe { vk_ctx.device.destroy_sampler(self.sampler, None) }

//         sp_destroy_vk_descriptor(vk_ctx, &self.descriptor);

//         self.cleanup_framebuffers(&vk_ctx.device);
//         sp_destroy_vk_renderpass(vk_ctx, &self.renderpass);

//         unsafe {
//             vk_ctx.device.destroy_pipeline_layout(self.pipeline_layout, None);
//             vk_ctx.device.destroy_pipeline(self.pipeline, None);
//         }
//     }

//     fn cleanup_framebuffers(&mut self, device: &ash::Device)
//     {
//         sp_destroy_vk_framebuffers(device, &mut self.framebuffers);
//     }

//     fn recreate_framebuffers(&mut self, vk_ctx: &SpVkContext, depth_img: Option<&SpVkImage>)
//     {
//         self.framebuffers = sp_create_vk_color_depth_framebuffers(vk_ctx, &self.renderpass, &depth_img.unwrap().view);
//     }

// }


// impl SpVk3dLayerUpdate for VkModelLayer
// {
//     fn update(&mut self, _vk_ctx: &SpVkContext, _transform_uniform: &SpVkBuffer, _delta_time: f32)
//     {
//         todo!()
//     }

// }
