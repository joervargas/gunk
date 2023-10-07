
use ash::{self, vk::{self, MemoryMapFlags}};
use nalgebra_glm as glm;

use russimp::scene::{ PostProcess, Scene };

use gpu_allocator::{vulkan::{
    Allocator,
    Allocation,
    AllocationCreateDesc, 
    AllocationScheme
}, MemoryLocation};

use crate::{vk_check, log_err, check_err, renderer::vulkan_renderer::sp_vulkan::vertex_data::{self, VertexData}};
    
use crate::renderer::vulkan_renderer::sp_vulkan::splunk_vk_context::{
    sp_begin_single_time_vk_command_buffer, 
    sp_end_single_time_vk_command_buffer
};

use super::splunk_vk_context::SpVkContext;

/// ### fn find_vk_memory_type_index( ... ) -> u32
/// <pre>
/// - Params
///     type_filter:            u32
///     required_properties:    vk::MemoryPropertyFlags
///     avail_mem_properties:   vk::PhysicalDeviceMemoryProperties
/// - Return
///     u32
/// </pre>
pub fn find_vk_memory_type_index(type_filter: u32, required_properties: vk::MemoryPropertyFlags, avail_mem_properties: vk::PhysicalDeviceMemoryProperties) -> u32
{
    for(i, memory_type) in avail_mem_properties.memory_types.iter().enumerate()
    {
        if(type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties)
        {
            return i as u32;
        }
    }

    log_err!("Failed to find a suitable memory type!");
    panic!("Failed to find a suitable memory type!");
}

/// ### fn create_vk_buffer( ... ) -> (vk::Buffer, vulkan::Allocation)
/// *Creates a vk::Buffer and allocates memory.*<br>
/// <pre>
/// <b><i>Note:</i></b>
///     <i>Use MemoryLocation::CpuToGpu for Staging buffer. GpuOnly for destination buffers.</i>
///     <i>Use AllocationScheme::DedicatedBuffer for buffers, AllocationScheme::DedicatedImage for images.</i>
/// - Params
///     device:         &ash::Device
///     allocator:      &mut Allocator
///     label:          &str
///     size:           vk::DeviceSize
///     usage:          vk::BufferUsageFlags
///     mem_location:   gpu_allocator::MemoryLocation    <i>// GpuOnly, CpuToGpu, GpuToCpu, Unknown</i>
/// - Return
///     (vk::Buffer, gpu_allocator::vulkan::Allocation)
/// </pre>
pub fn create_vk_buffer(
        device: &ash::Device, allocator: &mut Allocator,
        label: &str, size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        mem_location: gpu_allocator::MemoryLocation,
    ) -> (vk::Buffer, Allocation)
{
    let buffer_info = vk::BufferCreateInfo
    {
        s_type: vk::StructureType::BUFFER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: std::ptr::null()
    };

    let buffer = unsafe{ vk_check!( device.create_buffer(&buffer_info, None) ).unwrap() };
    let mem_requirements = unsafe{ device.get_buffer_memory_requirements(buffer) };

    let alloc_info = AllocationCreateDesc
    {
        name: label,
        requirements: mem_requirements,
        location: mem_location,
        linear: true,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged
    };
    let allocation = allocator.allocate(&alloc_info).map_err( |e| { log_err!(e); } ).unwrap();
    unsafe{ vk_check!( device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) ).unwrap(); }

    (buffer, allocation)
}

/// ### fn map_vk_allocation_data\<T\>( ... )
/// *Maps type data to allocation*
/// <pre>
/// - Param
///     allocation:     &Allocation
///     data:           &[T]
///     count:          usize          <i>// Count of type T in data slice.</i>
/// </pre>
pub fn map_vk_allocation_data<T>(allocation: &Allocation, data: &[T], count: usize)
{
    unsafe
    {
        let mapped_ptr = allocation.mapped_ptr().unwrap().as_ptr() as *mut T;
        mapped_ptr.copy_from_nonoverlapping(data.as_ptr(), count);
    }
}

/// ### fn map_vk_buffer_data( ... )
/// *Maps byte data to a vk_buffer*
/// <pre>
/// - Params
///     device:         &ash::Device
///     allocation:     &Allocation
///     data:           *const c_void
///     size:           vk::DeviceSize      <i>// size of data in bytes</i>
/// </pre>
pub fn map_vk_buffer_data(device: &ash::Device, allocation: &Allocation, data: &[u8], size: vk::DeviceSize)
{
    unsafe
    {
        let mapped_ptr = vk_check!( device.map_memory(allocation.memory(), allocation.offset(), size, MemoryMapFlags::empty()) ).unwrap() as *mut u8;
            mapped_ptr.copy_from_nonoverlapping(data.as_ptr(), size as usize);
        device.unmap_memory(allocation.memory());
    }
}

/// ### fn copy_vk_buffer( ... )
/// *Copies the contents of a source buffer to a destination buffer.*
/// <pre>
/// - Params
///     device:         &ash::Device
///     cmd_buffer:     &vk::CommandBuffer
///     src_buffer:     &vk::Buffer         <i>// source buffer</i>
///     dst_buffer:     &vk::Buffer         <i>// destination buffer</i>
///     size:           vk::DeviceSize
/// </pre>
pub fn copy_vk_buffer(device: &ash::Device, cmd_buffer: &vk::CommandBuffer, src_buffer: &vk::Buffer, dst_buffer: &vk::Buffer, size: vk::DeviceSize)
{
    let region = vk::BufferCopy
    {
        src_offset: 0,
        dst_offset: 0,
        size
    };
    unsafe
    {
        device.cmd_copy_buffer(*cmd_buffer, *src_buffer, *dst_buffer, &[region]);
    }
}

/// ### SpVkBuffer\<T\> struct
/// <pre>
/// - Members
///     handle:         vk::Buffer
///     allocation:     vulkan::Allocation
///     count:          usize                   <i>// count of type T</i>
///     data_type:      PhantomData&lt;T&gt;          <i>// private zero size placeholder for type</i>
/// </pre>
pub struct SpVkBuffer
{
    pub handle:         vk::Buffer,
    pub allocation:     Allocation,
    pub size:           vk::DeviceSize,
}

/// ### fn sp_create_vk_buffer( ... ) -> SpVkBuffer
/// *Creates a instance of SpVkBuffer*
/// <pre>
/// - Params
///     vk_ctx:         <b>&mut</b> SpVkContext
///     label:          &str                    <i>// Used for debug purposes<i>
///     usage:          vk::BufferUsageFlags
///     mem_location:   MemoryLocation          <i>// CpuToGpu, GpuOnly, GpuToCpu, Unknown
///     size:           vk::DeviceSize          <i>// the size of the buffer in bytes
/// - Return
///     SpVkBuffer
/// </pre>
pub fn sp_create_vk_buffer(
        vk_ctx: &mut SpVkContext,
        label: &str,
        usage: vk::BufferUsageFlags,
        mem_location: MemoryLocation,
        size: vk::DeviceSize
    ) -> SpVkBuffer
{
    let handle: vk::Buffer;
    let allocation: Allocation;
    (handle, allocation) = create_vk_buffer(
        &vk_ctx.device, vk_ctx.allocator.as_mut().unwrap(), 
        label, size, 
        usage, mem_location
    );

    SpVkBuffer
    {
        handle,
        allocation,
        size
    }
}

/// ### fn sp_create_vk_buffers( ... ) -> Vec\<SpVkBuffer\>
/// *Creates {count} number of SpVkBuffer in Vec\<\>*
/// <pre>
/// - Params
///     vk_ctx:         <b>&mut</b> SpVkContext
///     label:          &str                    <i>// Used for debug purposes<i>
///     usage:          vk::BufferUsageFlags
///     mem_location:   MemoryLocation          <i>// CpuToGpu, GpuOnly, GpuToCpu, Unknown
///     size:           vk::DeviceSize          <i>// the size of the buffer in bytes
///     count:          usize
/// - Return
///     Vec&lt;SpVkBuffer&gt;
/// </pre>
pub fn sp_create_vk_buffers(
    vk_ctx: &mut SpVkContext,
    label: &str,
    usage: vk::BufferUsageFlags,
    mem_location: MemoryLocation,
    size: vk::DeviceSize, count: usize
) -> Vec<SpVkBuffer>
{
    let mut sp_vk_buffers: Vec<SpVkBuffer> = Vec::new();

    for _i in 0..count
    {
        let handle: vk::Buffer;
        let allocation: Allocation;
        (handle, allocation) = create_vk_buffer(
            &vk_ctx.device, vk_ctx.allocator.as_mut().unwrap(), 
            label, size, 
            usage, mem_location
        );
    
        sp_vk_buffers.push(
            SpVkBuffer
            {
                handle,
                allocation,
                size
            }
        );
    }

    sp_vk_buffers
}

/// ### fn sp_destroy_vk_buffer( ... )
/// *Consumes an instance of SpVkBuffer and frees its resources*
/// <pre>
/// - Params
///     vk_ctx:     <b>&mut</b> SpVkContext
///     buffer:     SpVkBuffer
/// </pre>
pub fn sp_destroy_vk_buffer(vk_ctx: &mut SpVkContext, buffer: SpVkBuffer)
{
    unsafe
    {
        vk_ctx.device.destroy_buffer(buffer.handle, None);
    }
    vk_check!( vk_ctx.allocator.as_mut().unwrap().free(buffer.allocation) ).unwrap();
}

/// ### fn sp_destroy_vk_buffers( ... )
/// *Traverses a Vec\<\> of SpVkBuffer and frees its resources*
/// <pre>
/// - Params
///     vk_ctx:     <b>&mut</b> SpVkContext
///     buffers:    Vec&lt;SpVkBuffer&gt;
/// </pre>
pub fn sp_destroy_vk_buffers(vk_ctx: &mut SpVkContext, buffers: &mut Vec<SpVkBuffer>)
{
    for _i in 0..buffers.len()
    {
        let buffer = buffers.pop().unwrap();
        unsafe
        {
            vk_ctx.device.destroy_buffer(buffer.handle, None);
        }
        vk_check!( vk_ctx.allocator.as_mut().unwrap().free(buffer.allocation) ).unwrap();
    }
}

/// ### fn sp_create_vk_vertex_buffer_from_file\<T\>( ... ) -> SpVkBuffer
/// *Reads 3d format file and creates a vertex buffer using type T*<br>
/// *T is Type of Vertex Data*<br>
/// *Populates vert and index buffer sizes*
/// <pre>
/// - Params
///     vk_ctx:             <b>&mut</b> SpVkContext
///     label:              &str                    <i>// Used for debug purposes<i>
///     usage:              vk::BufferUsageFlags
///     file_path:          &path::Path
/// - Return
///     (SpVkBuffer, SpVkBuffer) <i>// (vertex_buffer, index_buffer)
/// </pre>
pub fn sp_create_vk_vertex_buffer_from_file<T>(
        vk_ctx: &mut SpVkContext, 
        label: &str, 
        usage: vk::BufferUsageFlags, 
        file_path: &std::path::Path,
    ) -> (SpVkBuffer, SpVkBuffer)
{
    let scene_flags = vec![
        PostProcess::Triangulate
    ];
    let scene = check_err!(Scene::from_file(file_path.to_str().unwrap(), scene_flags)).unwrap();
    if scene.meshes.is_empty()
    {
        log_err!("unable to load {}", file_path.to_str().unwrap());
    }

    let mesh = scene.meshes.first().unwrap();
    let mut vertices: Vec<vertex_data::VertexData> = Vec::new();
    for (i, v) in mesh.vertices.iter().enumerate()
    {
        let t = mesh.texture_coords[0].as_ref().unwrap()[i];
        vertices.push( VertexData::new(glm::vec3(v.x, v.y, v.z), glm::vec2(t.x, 1.0 - t.y)));
    }

    let mut indices: Vec<u32> = Vec::new();
    for face in mesh.faces.iter()
    {
        for f in face.0.iter()
        {
            indices.push(*f);
        }
    }
    drop(scene);

    let vert_buffer_size = std::mem::size_of::<VertexData>() * vertices.len();
    let index_buffer_size = std::mem::size_of::<u32>() * indices.len();

    // let buffer_size = (vert_buffer_size + index_buffer_size) as vk::DeviceSize;
    let staging_vert_label = String::from(format!("staging vert{}", label));

    let staging_vertex = sp_create_vk_buffer(
        vk_ctx, 
        &staging_vert_label, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        MemoryLocation::CpuToGpu, 
        vert_buffer_size as vk::DeviceSize
    );

    unsafe
    {
        // let mapped_ptr = vk_check!( vk_ctx.device.map_memory(staging_vertex.allocation.memory(), staging_vertex.allocation.offset(), vert_buffer_size as vk::DeviceSize, MemoryMapFlags::empty()) ).unwrap() as *mut u8;
        let mapped_ptr = staging_vertex.allocation.mapped_slice().unwrap().as_ptr() as *mut u8;
            mapped_ptr.copy_from_nonoverlapping(vertices.as_slice().as_ptr() as *const u8, vert_buffer_size);
        // vk_ctx.device.unmap_memory(staging_vertex.allocation.memory());
    }

    let staging_index_label = String::from(format!("staging index{}", label));
    let staging_indices = sp_create_vk_buffer(
        vk_ctx, 
        &staging_index_label, 
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
        index_buffer_size as vk::DeviceSize
    );

    unsafe
    {
        // let mapped_ptr = vk_check!( vk_ctx.device.map_memory(staging_indices.allocation.memory(), staging_indices.allocation.offset(), index_buffer_size as vk::DeviceSize, MemoryMapFlags::empty()) ).unwrap() as *mut u8;
        let mapped_ptr = staging_indices.allocation.mapped_slice().unwrap().as_ptr() as *mut u8;
            mapped_ptr.copy_from_nonoverlapping(indices.as_slice().as_ptr() as *const u8, index_buffer_size);
        // vk_ctx.device.unmap_memory(staging_indices.allocation.memory());
    }

    let vert_label = String::from(format!("vertex {}", label));
    let vert_buffer = sp_create_vk_buffer(
        vk_ctx,
        &vert_label, 
        usage | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
        vert_buffer_size as vk::DeviceSize
    );

    let index_label = String::from(format!("index {}", label));
    let index_buffer = sp_create_vk_buffer(
        vk_ctx,
        &index_label,
        usage | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
        index_buffer_size as vk::DeviceSize
    );

    let cmd_buffer = sp_begin_single_time_vk_command_buffer(vk_ctx);
        copy_vk_buffer(&vk_ctx.device, &cmd_buffer, &staging_vertex.handle, &vert_buffer.handle, vert_buffer_size as vk::DeviceSize);
        copy_vk_buffer(&vk_ctx.device, &cmd_buffer, &staging_indices.handle, &index_buffer.handle, index_buffer_size as vk::DeviceSize);
    sp_end_single_time_vk_command_buffer(vk_ctx, cmd_buffer);

    sp_destroy_vk_buffer(vk_ctx, staging_vertex);
    sp_destroy_vk_buffer(vk_ctx, staging_indices);

    (vert_buffer, index_buffer)
}


// /// ### fn sp_create_vk_vertex_buffer_from_file\<T\>( ... ) -> SpVkBuffer
// /// *Reads 3d format file and creates a vertex buffer using type T*<br>
// /// *T is Type of Vertex Data*<br>
// /// *Populates vert and index buffer sizes*
// /// <pre>
// /// - Params
// ///     vk_ctx:             <b>&mut</b> SpVkContext
// ///     label:              &str                    <i>// Used for debug purposes<i>
// ///     usage:              vk::BufferUsageFlags
// ///     file_path:          &path::Path
// ///     vert_buffer_size:   &mut u32                <i>// populates this value
// ///     index_buffer_size:  &mut u32                <i>// populates this value
// /// - Return
// ///     SpVkBuffer
// /// </pre>
// pub fn sp_create_vk_vertex_buffer_from_file<T>(
//         vk_ctx: &mut SpVkContext, 
//         label: &str, 
//         usage: vk::BufferUsageFlags, 
//         file_path: &std::path::Path,
//         vert_buffer_size: &mut usize,
//         index_buffer_size: &mut usize,
//     ) -> SpVkBuffer
// {
//     let scene_flags = vec![
//         PostProcess::Triangulate
//     ];
//     let scene = check_err!(Scene::from_file(file_path.to_str().unwrap(), scene_flags)).unwrap();
//     if scene.meshes.is_empty()
//     {
//         log_err!("unable to load {}", file_path.to_str().unwrap());
//     }

//     let mesh = scene.meshes.first().unwrap();
//     let mut vertices: Vec<vertex_data::VertexData> = Vec::new();
//     for (i, v) in mesh.vertices.iter().enumerate()
//     {
//         let t = mesh.texture_coords[0].as_ref().unwrap()[i];
//         vertices.push( VertexData{ pos: glm::Vec3::new(v.x, v.y, v.z), tc: glm::Vec2::new(t.x, 1.0 - t.y) } );
//     }

//     let mut indices: Vec<u32> = Vec::new();
//     for face in mesh.faces.iter()
//     {
//         for f in face.0.iter()
//         {
//             indices.push(*f);
//         }
//     }
//     drop(scene);

//     *vert_buffer_size = std::mem::size_of::<VertexData>() * vertices.len();
//     *index_buffer_size = std::mem::size_of::<u32>() * indices.len();

//     let buffer_size = (*vert_buffer_size + *index_buffer_size) as vk::DeviceSize;
//     let staging_label = String::from(format!("staging {}", label));

//     let staging = sp_create_vk_buffer(
//         vk_ctx, 
//         &staging_label, 
//         vk::BufferUsageFlags::TRANSFER_SRC, 
//         MemoryLocation::CpuToGpu, 
//         buffer_size
//     );

//     unsafe
//     {
//         let mapped_ptr = vk_check!( vk_ctx.device.map_memory(staging.allocation.memory(), staging.allocation.offset(), buffer_size, MemoryMapFlags::empty()) ).unwrap() as *mut u8;
//             mapped_ptr.copy_from_nonoverlapping(vertices.as_slice().as_ptr() as *const u8, *vert_buffer_size);
//             mapped_ptr.copy_from_nonoverlapping(indices.as_slice().as_ptr() as *const u8, *index_buffer_size);
//         vk_ctx.device.unmap_memory(staging.allocation.memory());
//     }

//     let buffer = sp_create_vk_buffer(
//         vk_ctx,
//         label, 
//         usage,
//         MemoryLocation::GpuOnly,
//         buffer_size
//     );

//     let cmd_buffer = sp_begin_single_time_vk_command_buffer(vk_ctx);
//         copy_vk_buffer(&vk_ctx.device, &cmd_buffer, &staging.handle, &buffer.handle, buffer_size);
//     sp_end_single_time_vk_command_buffer(vk_ctx, cmd_buffer);

//     sp_destroy_vk_buffer(vk_ctx, staging);
    
//     buffer
// }
