use ash::{self, vk};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{ 
        Allocation, 
        Allocator, 
        AllocationCreateDesc, 
        AllocationScheme 
    }, 
};

use image::{self, EncodableLayout};


use super::gunk_vk_buffer::create_vk_buffer;
use super::gunk_vk_context::{
    GkVkContext, 
    gk_begin_single_time_vk_command_buffer, 
    gk_end_single_time_vk_command_buffer
};

use crate::renderer::{
    cubemap_utils::{
        convert_equirectangle_to_vertical_cross, 
        convert_multi_file_to_cubemap_faces, 
        convert_vertical_cross_to_cubemap_faces
    }, 
    gk_bitmap::{self, EBitMapFormat, GkBitMap},
};

use crate::{ log_err, vk_check };


use core::panic;
// use std::{ffi::c_char, os::raw::c_int};



/// ### fn create_vk_image( ... ) -> (vk::Image, vulkan::Allocation)
/// *Creates a vk::Image and an Allocation for memory)
/// <pre>
/// - Param
///     device:
///     width:
///     height:
///     format:
///     tiling:
///     usage:
///     create_flags:
///     mip_levels:
/// - Return
///     (vk::Image, vulkan::Allocation)
/// </pre>
pub fn create_vk_image(
        device: &ash::Device, allocator: &mut Allocator, label: &str, 
        width: u32, height: u32, 
        format: vk::Format, tiling: vk::ImageTiling, 
        usage: vk::ImageUsageFlags, mip_levels: u32,
        create_flags: vk::ImageCreateFlags
    ) -> (vk::Image, Allocation)
{
    let create_info = vk::ImageCreateInfo
    {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: create_flags,
        image_type: vk::ImageType::TYPE_2D,
        format,
        extent: vk::Extent3D { width, height, depth: 1 },
        mip_levels,
        array_layers: if create_flags == vk::ImageCreateFlags::CUBE_COMPATIBLE { 6 } else { 1 },
        samples: vk::SampleCountFlags::TYPE_1,
        tiling,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: std::ptr::null(),
        initial_layout: vk::ImageLayout::UNDEFINED
    };
    let img = unsafe { vk_check!( device.create_image(&create_info, None) ).unwrap() };

    let mem_requirements = unsafe{ device.get_image_memory_requirements(img) };

    let alloc_info = AllocationCreateDesc
    {
        name: label,
        requirements: mem_requirements,
        location: MemoryLocation::GpuOnly,
        linear: true,
        allocation_scheme: AllocationScheme::DedicatedImage(img)
    };
    let allocation = vk_check!(allocator.allocate(&alloc_info)).unwrap();
    unsafe { vk_check!(device.bind_image_memory(img, allocation.memory(), allocation.offset())); }

    (img, allocation)
}


/// ### fn create_vk_image_view( ... ) -> vk::ImageView
/// *Creates a vk::ImageView*
/// <pre>
/// - Params
///     device:             &ash::Device
///     image:              &vk::Image
///     format:             &vk::Format
///     aspect_flags:       vk::ImageAspectFlags
///     view_type:          vk::ImageViewType
///     layer_count:        u32
///     mip_levels:         u32
/// - Return
///     vk::ImageView
/// </pre>
pub fn create_vk_image_view(
        device: &ash::Device, image: &vk::Image, 
        format: &vk::Format, aspect_flags: vk::ImageAspectFlags, 
        view_type: vk::ImageViewType, layer_count: u32, mip_levels: u32
    ) -> vk::ImageView
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

/// ### fn create_vk_sampler( ... ) -> vk::Sampler
/// *Creates a vk::Sampler*
/// <pre>
/// - Params
///     device:     &ash::Device
/// - Return
///     vk::Sampler
/// </pre>
pub fn create_vk_sampler(device: &ash::Device) -> vk::Sampler
{
    let create_info = vk::SamplerCreateInfo
    {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        mip_lod_bias: 0.0,
        max_anisotropy: 1.0,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::ALWAYS,
        min_lod: 0.0,
        max_lod: 0.0,
        border_color: vk::BorderColor::INT_OPAQUE_BLACK,
        unnormalized_coordinates: vk::FALSE,
        ..Default::default()
    };

    unsafe{ vk_check!(device.create_sampler(&create_info, None)).unwrap() }
}


/// ### fn find_supported_vk_format( ... ) -> vk::Format
/// *Loops through supplied candidate formats. <br>
/// Determines best one based on Tiling and and format features.*
/// <pre>
/// - Params
///     instance:           &ash::Instance
///     phys_device:        &vk::PhysicalDevice
///     candidates:         &Vec&lt;vk::Format&gt;          <i>// Candidate formats to loop through.</i>
///     tiling:             vk::ImageTiling
///     features:           vk::FormatFeatureFlags
/// - Return
///     vk::Format
/// </pre>
pub fn find_supported_vk_format(instance: &ash::Instance, phys_device: &vk::PhysicalDevice, candidates: &Vec<vk::Format>, tiling: vk::ImageTiling, features: vk::FormatFeatureFlags) -> vk::Format
{
    for format in candidates.iter()
    {
        let props = unsafe { instance.get_physical_device_format_properties(*phys_device, *format) };

        if tiling == vk::ImageTiling::LINEAR && (props.linear_tiling_features & features) == features
        {
            return *format;
        }
        else if tiling == vk::ImageTiling::OPTIMAL && (props.optimal_tiling_features & features) == features
        {
            return *format;
        }
    }

    log_err!("Fn 'find_supported_vk_format()' Failed to find supported format!");
    panic!("Fn 'find_supported_vk_format()' Failed to find supported format!");
}

pub fn get_bytes_per_pixel_vk_format(fmt: vk::Format) -> u32
{
    match fmt
    {
        vk::Format::R8_SINT | 
        vk::Format::R8_UNORM => { 1 },
        vk::Format::R16_SFLOAT => { 2 },
        vk::Format::R16G16_SFLOAT |
        vk::Format::R16G16_SNORM => { 4 },
        vk::Format::R8G8B8A8_SRGB |
        vk::Format::B8G8R8A8_UNORM |
        vk::Format::R8G8B8A8_UNORM => { 4 },
        vk::Format::R16G16B16A16_SFLOAT => { 4 * std::mem::size_of::<u16>() as u32 },
        vk::Format::R32G32B32A32_SFLOAT => { 4 * std::mem::size_of::<f32>() as u32 },
        _ => { 0 }
    }
}

/// ### fn has_vk_stencil_component( ... ) -> bool
/// *Determines whether the given vk::Format has stencil capabilities.*
/// <pre>
/// - Params
///     format:     vk::Format
/// - Return
///     bool
/// </pre>
pub fn has_vk_stencil_component(format: vk::Format) -> bool
{
    format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT
}

/// ### transition_vk_image_layout( ... )
/// *Transitions a vk::Image to a new layout*
/// <pre>
/// - Params
///     device:         &ash::Device
///     cmd_buffer:     &vk::CommandBuffer
///     img:            vk::Image
///     format:         vk::Format
///     old_layout:     vk::ImageLayout         <i>// Current image layout.</i>
///     new_layout:     vk::ImageLayout         <i>// Desired image layout.</i>
///     layer_count:    u32
///     mip_levels:     u32
/// </pre>
pub fn transition_vk_image_layout( 
        device: &ash::Device, cmd_buffer: &vk::CommandBuffer, 
        img: vk::Image, format: vk::Format, 
        old_layout: vk::ImageLayout, new_layout: vk::ImageLayout, 
        layer_count: u32, mip_levels: u32
    )
{
    let mut barrier = vk::ImageMemoryBarrier
    {
        s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
        p_next: std::ptr::null(),
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::empty(),
        old_layout,
        new_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: img,
        subresource_range: vk::ImageSubresourceRange
        {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: layer_count
        },
    };

    let src_stage: vk::PipelineStageFlags;
    let dst_stage: vk::PipelineStageFlags;

    if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL ||
        (format == vk::Format::D16_UNORM) ||
        (format == vk::Format::X8_D24_UNORM_PACK32) ||
        (format == vk::Format::D32_SFLOAT) ||
        (format == vk::Format::S8_UINT) ||
        (format == vk::Format::D16_UNORM_S8_UINT) ||
        (format == vk::Format::D32_SFLOAT_S8_UINT)
    {
        barrier.subresource_range.aspect_mask = vk::ImageAspectFlags::DEPTH;
        if has_vk_stencil_component(format)
        {
            barrier.subresource_range.aspect_mask |= vk::ImageAspectFlags::STENCIL;
        }
    } else {
        barrier.subresource_range.aspect_mask |= vk::ImageAspectFlags::COLOR;
    }

    if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::empty();
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        src_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        dst_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    }
    else if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::GENERAL
    {
        barrier.src_access_mask = vk::AccessFlags::empty();
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        src_stage = vk::PipelineStageFlags::TRANSFER;
        dst_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    }
    else if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::empty();
        barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

        src_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        dst_stage = vk::PipelineStageFlags::TRANSFER;
    }
    // Convert back from read-only updatable
    else if old_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::SHADER_READ;
        barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

        src_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        dst_stage = vk::PipelineStageFlags::TRANSFER;
    }
    // Convert from updateable texture to shader read only
    else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        src_stage = vk::PipelineStageFlags::TRANSFER;
        dst_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    }
    // Convert from depth texture from undefined state to depth stencil buffer
    else if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::empty();
        barrier.dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;

        src_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        dst_stage = vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
    }
    // Wait for render pass to complete
    else if old_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::empty();
        barrier.dst_access_mask = vk::AccessFlags::empty();

        src_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        dst_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    }
    // Convert back from read-only to color attachment
    else if old_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL && new_layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::SHADER_READ;
        barrier.dst_access_mask = vk::AccessFlags::COLOR_ATTACHMENT_WRITE;

        src_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        dst_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
    }
    // Convert from updateable texture to shader read only
    else if old_layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        src_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        dst_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    }
    // Convert back from read-only to depth attachment
    else if old_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL && new_layout == vk::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::SHADER_READ;
        barrier.dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;

        src_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        dst_stage = vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
    }
    // Convert from updateable depth texture to shader read only
    else if old_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    {
        barrier.src_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        src_stage = vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
        dst_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    } else {
        src_stage = vk::PipelineStageFlags::NONE;
        dst_stage = vk::PipelineStageFlags::NONE;
    }

    unsafe {
        device.cmd_pipeline_barrier(
            *cmd_buffer, 
            src_stage, dst_stage, 
            vk::DependencyFlags::empty(), 
            &[], 
            &[], 
            &[barrier]
        );
    }
}


/// ### fn copy_vk_buffer_to_img( ... )
/// *Copies vk::Buffer contents to a vk::Image*
/// <pre>
/// - Params
///     device:         &ash::Device
///     cmd_buffer:     &vk::CommandBuffer
///     buffer:         &vk::Buffer
///     img:            &vk::Image
///     width:          u32
///     height:         u32
///     layer_count:    u32
/// </pre>
pub fn copy_vk_buffer_to_img(
        device: &ash::Device, cmd_buffer: &vk::CommandBuffer, 
        buffer: &vk::Buffer, img: &vk::Image, 
        width: u32, height: u32, layer_count: u32
    )
{
    let copy_region = vk::BufferImageCopy
    {
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image_subresource: vk::ImageSubresourceLayers
        {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count,
        },
        image_offset: vk::Offset3D{ x: 0, y: 0, z: 0 },
        image_extent: vk::Extent3D{ width, height, depth: 1 }
    };

    unsafe { 
        device.cmd_copy_buffer_to_image(
            *cmd_buffer, 
            *buffer, *img, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[copy_region]
        ); 
    }
}

/// ### fn find_vk_format_depth_img( ... ) -> vk::Format
/// *Finds a suitable format for a depth image texture*
/// <pre>
/// - Params
///     instance:       &ash::Instance
///     phys_device:    &vk::PhysicalDevice
/// - Return
///     vk::Format      <i>// A format suitable for a depth image texture.*
/// </pre>
pub fn find_vk_format_depth_img(instance: &ash::Instance, phys_device: &vk::PhysicalDevice) -> vk::Format
{
    find_supported_vk_format(
        instance, phys_device, 
        &vec![vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT ], 
        vk::ImageTiling::OPTIMAL, 
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
    )
}


/// ### struct GkVkImage
/// *A convenience struct. has the image, memory allocation, and view*
/// <pre>
/// - Members
///     handle:     vk::Image
///     alloc:      vulkan::Allocation
///     view:       vk::ImageView
/// </pre>
pub struct GkVkImage
{
    pub handle:     vk::Image,
    pub alloc:      Allocation,
    pub view:       vk::ImageView,
    pub size:       vk::DeviceSize,
}

/// ### gk_create_vk_image( ... ) -> GkVkImage
/// *Creates a generic GkVkImage from a given file_name.*
/// <pre>
/// - Params
///     vk_ctx:         &mut GkVkContext        <i>// mutable because of allocator</i>
///     file_name:      &str
/// - Return
///     GkVkImage
/// </pre>
pub fn gk_create_vk_image(vk_ctx: &mut GkVkContext, file_name: &str) -> GkVkImage
{  

    let img = image::open(std::path::Path::new(file_name)).map_err( |e| { log_err!(e); } ).unwrap();
    let pixels = img.to_rgba8().into_raw();
    
    let img_size : vk::DeviceSize = (std::mem::size_of::<u8>() as u32 * img.width() * img.height() * 4) as vk::DeviceSize;
    let staging_buffer: vk::Buffer;
    let staging_allocation: Allocation;

    let label = String::from(format!("staging_allocation: {}", file_name));
    (staging_buffer, staging_allocation) = create_vk_buffer(
        &vk_ctx.device, vk_ctx.allocator.as_mut().unwrap(), label.as_str(), 
        img_size, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        MemoryLocation::CpuToGpu,
    );

    // map_vk_allocation_data(&staging_allocation, pixels.as_slice(), pixels.len());
    unsafe
    {
        let mapped_ptr = staging_allocation.mapped_slice().unwrap().as_ptr() as *mut u8;
            mapped_ptr.copy_from_nonoverlapping(pixels.as_slice().as_ptr() as *const u8, pixels.len());
        // vk_ctx.device.unmap_memory(staging_allocation.memory());
    }

    let img_format = vk::Format::R8G8B8A8_SRGB;
    let (handle, alloc) = create_vk_image(
        &vk_ctx.device, vk_ctx.allocator.as_mut().unwrap(), file_name, 
        img.width(), img.height(), img_format, 
        vk::ImageTiling::OPTIMAL, vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED, 
        1, vk::ImageCreateFlags::empty());

    let cmd_buffer = gk_begin_single_time_vk_command_buffer(vk_ctx);

        transition_vk_image_layout(
            &vk_ctx.device, &cmd_buffer, 
            handle, img_format,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            1, 1);

        copy_vk_buffer_to_img(
            &vk_ctx.device, &cmd_buffer, 
            &staging_buffer, &handle, 
            img.width(), img.height(),
            1);

        transition_vk_image_layout(
            &vk_ctx.device, &cmd_buffer, 
            handle, img_format, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 
            1, 1);

    gk_end_single_time_vk_command_buffer(vk_ctx, cmd_buffer);

    unsafe
    {
        vk_ctx.device.destroy_buffer(staging_buffer, None);
        vk_check!( vk_ctx.allocator.as_mut().unwrap().free(staging_allocation) ).unwrap()
    }

    let view = create_vk_image_view(
        &vk_ctx.device, &handle, 
        &img_format, vk::ImageAspectFlags::COLOR, 
        vk::ImageViewType::TYPE_2D, 
        1, 1);

    GkVkImage { handle, alloc, view, size: img_size }
}

/// ### fn gk_create_vk_depth_img( ... ) -> GkVkImage
/// *Creates an GkVkImage used for depth textures.*
/// <pre>
/// - Params
///     instance:       &ash::Instance
///     vk_ctx:         &mut GkVkContext        <i>// mutable because of allocator</i>
///     width:          u32
///     height:         u32
/// </pre>
pub fn gk_create_vk_depth_img(instance: &ash::Instance, vk_ctx: &mut GkVkContext, width: u32, height: u32) -> GkVkImage
{
    let format = find_vk_format_depth_img(instance, &vk_ctx.physical_device);
    let (img, alloc) = create_vk_image(
        &vk_ctx.device, &mut vk_ctx.allocator.as_mut().unwrap(), "depth image",
        width, height, 
        format, vk::ImageTiling::OPTIMAL, 
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, 
        1, vk::ImageCreateFlags::empty());

    let view = create_vk_image_view(
        &vk_ctx.device, &img, &format, 
        vk::ImageAspectFlags::DEPTH, 
        vk::ImageViewType::TYPE_2D,
        1, 1);

    let cmd_buffer = gk_begin_single_time_vk_command_buffer(vk_ctx);
        transition_vk_image_layout(
            &vk_ctx.device, &cmd_buffer, 
            img, format, 
            vk::ImageLayout::UNDEFINED, 
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL, 
            1, 1);
    gk_end_single_time_vk_command_buffer(vk_ctx, cmd_buffer);

    let size : vk::DeviceSize = (std::mem::size_of::<u8>() as u32 * width * height) as vk::DeviceSize;

    GkVkImage { handle: img, alloc, view, size }
}

#[allow(dead_code)]
// fn float24_to_float32(w: usize, h: usize, input_data: &Vec<image::Rgb<f32>>) -> Vec<f32>
fn img_f3_to_f4(w: usize, h: usize, input_data: &Vec<f32>) -> Vec<f32>
{
    let num_pixels = w * h;
    let mut output: Vec<f32> = Vec::new();
    output.resize(num_pixels * 4, 0.0); // size of pixels and its channels
    let mut input_index = 0;
    // dbg!(output.capacity());
    let mut output_index = 0;
    for _i in 0..num_pixels
    {
        // R channel
        output[output_index] = input_data[input_index];
        output_index += 1;
        input_index +=1;
        // G channel
        output[output_index] = input_data[input_index];
        output_index += 1;
        input_index +=1;
        // B channel
        output[output_index] = input_data[input_index];
        output_index += 1;
        input_index +=1;
        // A channel
        output[output_index] = 1.0;
        output_index += 1;
    }
    // output

    // for rgb in input_data.iter()
    // {
    //     output[output_index] = rgb.0[0];
    //     // output.insert(output_index, rgb.0[0]);
    //     output_index += 1;
    //     output[output_index] = rgb.0[1];
    //     // output.insert(output_index, rgb.0[1]);
    //     output_index += 1;
    //     output[output_index] = rgb.0[2];
    //     // output.insert(output_index, rgb.0[2]);
    //     output_index += 1;

    //     output_index += 1;
    //     // output.push(rgb.0[0]);
    //     // output.push(rgb.0[1]);
    //     // output.push(rgb.0[2]);
    //     // output.push(1.0);
    // }
    output
}

pub fn gk_create_vk_cubemap_image(vk_ctx: &mut GkVkContext, file_names: Vec<std::path::PathBuf>) -> Result<GkVkImage, String>
{
    let mut correct_file_count = false;
    if file_names.len() == 1 { correct_file_count = true; }
    if file_names.len() == 6 { correct_file_count = true; }
    if !correct_file_count 
    {
        log_err!("Cubemap file_names count must be either 1 or 6");
        return  Err(String::from("Cubemap file_names count must be either 1 or 6"));
    }

    // let mut width = 0;
    // let mut height = 0;

    let cube: GkBitMap = if file_names.len() == 1
    {
        let img = image::open(std::path::Path::new(&file_names[0])).map_err( |e| { log_err!(e); } ).unwrap();
        let img = img.flipv();
        // let pixels = img_f3_to_f4(img.width() as usize, img.height() as usize, &img.to_rgb32f().into_raw());
        let pixels = img.to_rgba32f().into_raw();
    
        let bitmap_in = GkBitMap::new(img.width(), img.height(), 1, 4, EBitMapFormat::Float, pixels.as_bytes().to_vec());
        let bitmap_out = convert_equirectangle_to_vertical_cross(&bitmap_in);

        convert_vertical_cross_to_cubemap_faces(&bitmap_out)
    } else {
        let result = convert_multi_file_to_cubemap_faces(&file_names);
        match result
        {
            Ok(cube_map) => { cube_map },
            Err(e) => { return Err(e); }
        }
    };

    let img_format = match cube.format
    {
        gk_bitmap::EBitMapFormat::UByte => vk::Format::R8G8B8A8_SRGB,
        gk_bitmap::EBitMapFormat::Float => vk::Format::R32G32B32A32_SFLOAT
    };
    let bytes_per_pixel = get_bytes_per_pixel_vk_format(img_format);
    let img_size : vk::DeviceSize = (cube.width * cube.height * bytes_per_pixel) as vk::DeviceSize * 6;

    let staging_buffer: vk::Buffer;
    let staging_allocation: Allocation;

    let label = String::from(format!("staging_allocation: {}", file_names[0].to_str().unwrap()));
    (staging_buffer, staging_allocation) = create_vk_buffer(
        &vk_ctx.device, vk_ctx.allocator.as_mut().unwrap(), label.as_str(), 
        img_size, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        MemoryLocation::CpuToGpu,
    );

    unsafe {
        let mapped_ptr = staging_allocation.mapped_slice().unwrap().as_ptr() as * mut u8;
        mapped_ptr.copy_from_nonoverlapping(cube.data.as_ptr(), cube.data.len());
    }

    let (handle, alloc) = create_vk_image(
        &vk_ctx.device, vk_ctx.allocator.as_mut().unwrap(), file_names[0].to_str().unwrap(),
        cube.width, cube.height, img_format, 
        vk::ImageTiling::OPTIMAL, vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        1, vk::ImageCreateFlags::CUBE_COMPATIBLE);

    let cmd_buffer = gk_begin_single_time_vk_command_buffer(vk_ctx);

        transition_vk_image_layout(
            &vk_ctx.device, &cmd_buffer, 
            handle, img_format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            6, 1);

        copy_vk_buffer_to_img(
            &vk_ctx.device, &cmd_buffer, 
            &staging_buffer, &handle, 
            cube.width, cube.height,
            6);

        transition_vk_image_layout(
            &vk_ctx.device, &cmd_buffer, 
            handle, img_format, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            6, 1);

    gk_end_single_time_vk_command_buffer(vk_ctx, cmd_buffer);

    unsafe
    {
        vk_ctx.device.destroy_buffer(staging_buffer, None);
        vk_check!( vk_ctx.allocator.as_mut().unwrap().free(staging_allocation) ).unwrap()
    }

    let view = create_vk_image_view(
        &vk_ctx.device, &handle, 
        &img_format, vk::ImageAspectFlags::COLOR, 
        vk::ImageViewType::CUBE, 
        6, 1);

    Ok(GkVkImage { handle, alloc, view, size: img_size })
}

/// ### fn gk_destroy_vk_img( ... )
/// *Destroys the given instance of GkVkImage*
/// <pre>
/// - Param
///     vk_ctx:     &mut GkVkContext
///     img:        GkVkImage           <i>// GkVkImage to be destroyed.</i>
/// </pre>
pub fn gk_destroy_vk_img(vk_ctx: &mut GkVkContext, img: GkVkImage)
{
    unsafe
    {
        vk_ctx.device.destroy_image(img.handle, None);
        // device.free_memory(img.memory, None);
        vk_ctx.allocator.as_mut().unwrap().free(img.alloc).map_err(|e| { log_err!(e); } ).unwrap();
        vk_ctx.device.destroy_image_view(img.view, None);
    }
}

