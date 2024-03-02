use ash::{self, vk::{self, MemoryMapFlags}};


use gpu_allocator::{vulkan::{
    Allocator,
    Allocation,
    AllocationCreateDesc, 
    AllocationScheme
}, MemoryLocation};

use tobj;

use crate::{vk_check, log_err};
use crate::renderer::vulkan_renderer::gk_vulkan::vertex_data::VertexData;

use crate::renderer::vulkan_renderer::gk_vulkan::gunk_vk_context::{
    gk_begin_single_time_vk_command_buffer, 
    gk_end_single_time_vk_command_buffer
};

use super::gunk_vk_context::GkVkContext;

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
        allocation_scheme: AllocationScheme::DedicatedBuffer(buffer)
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

/// ### GkVkBuffer\<T\> struct
/// <pre>
/// - Members
///     handle:         vk::Buffer
///     allocation:     vulkan::Allocation
///     count:          usize                   <i>// count of type T</i>
///     data_type:      PhantomData&lt;T&gt;          <i>// private zero size placeholder for type</i>
/// </pre>
pub struct GkVkBuffer
{
    pub handle:         vk::Buffer,
    pub allocation:     Allocation,
    pub size:           vk::DeviceSize,
}

/// ### fn gk_create_vk_buffer( ... ) -> GkVkBuffer
/// *Creates a instance of GkVkBuffer*
/// <pre>
/// - Params
///     vk_ctx:         <b>&mut</b> GkVkContext
///     label:          &str                    <i>// Used for debug purposes<i>
///     usage:          vk::BufferUsageFlags
///     mem_location:   MemoryLocation          <i>// CpuToGpu, GpuOnly, GpuToCpu, Unknown
///     size:           vk::DeviceSize          <i>// the size of the buffer in bytes
/// - Return
///     GkVkBuffer
/// </pre>
pub fn gk_create_vk_buffer(
        vk_ctx: &mut GkVkContext,
        label: &str,
        usage: vk::BufferUsageFlags,
        mem_location: MemoryLocation,
        size: vk::DeviceSize
    ) -> GkVkBuffer
{
    let handle: vk::Buffer;
    let allocation: Allocation;
    (handle, allocation) = create_vk_buffer(
        &vk_ctx.device, vk_ctx.allocator.as_mut().unwrap(), 
        label, size, 
        usage, mem_location
    );

    GkVkBuffer
    {
        handle,
        allocation,
        size
    }
}

/// ### fn gk_create_vk_buffers( ... ) -> Vec\<GkVkBuffer\>
/// *Creates {count} number of GkVkBuffer in Vec\<\>*
/// <pre>
/// - Params
///     vk_ctx:         <b>&mut</b> GkVkContext
///     label:          &str                    <i>// Used for debug purposes<i>
///     usage:          vk::BufferUsageFlags
///     mem_location:   MemoryLocation          <i>// CpuToGpu, GpuOnly, GpuToCpu, Unknown
///     size:           vk::DeviceSize          <i>// the size of the buffer in bytes
///     count:          usize
/// - Return
///     Vec&lt;GkVkBuffer&gt;
/// </pre>
pub fn gk_create_vk_buffers(
    vk_ctx: &mut GkVkContext,
    label: &str,
    usage: vk::BufferUsageFlags,
    mem_location: MemoryLocation,
    size: vk::DeviceSize, count: usize
) -> Vec<GkVkBuffer>
{
    let mut gk_vk_buffers: Vec<GkVkBuffer> = Vec::new();

    for _i in 0..count
    {
        let handle: vk::Buffer;
        let allocation: Allocation;
        (handle, allocation) = create_vk_buffer(
            &vk_ctx.device, vk_ctx.allocator.as_mut().unwrap(), 
            label, size, 
            usage, mem_location
        );
    
        gk_vk_buffers.push(
            GkVkBuffer
            {
                handle,
                allocation,
                size
            }
        );
    }

    gk_vk_buffers
}

/// ### fn gk_destroy_vk_buffer( ... )
/// *Consumes an instance of GkVkBuffer and frees its resources*
/// <pre>
/// - Params
///     vk_ctx:     <b>&mut</b> GkVkContext
///     buffer:     GkVkBuffer
/// </pre>
pub fn gk_destroy_vk_buffer(vk_ctx: &mut GkVkContext, buffer: GkVkBuffer)
{
    unsafe
    {
        vk_ctx.device.destroy_buffer(buffer.handle, None);
    }
    vk_check!( vk_ctx.allocator.as_mut().unwrap().free(buffer.allocation) ).unwrap();
}

/// ### fn gk_destroy_vk_buffers( ... )
/// *Traverses a Vec\<\> of GkVkBuffer and frees its resources*
/// <pre>
/// - Params
///     vk_ctx:     <b>&mut</b> GkVkContext
///     buffers:    Vec&lt;GkVkBuffer&gt;
/// </pre>
pub fn gk_destroy_vk_buffers(vk_ctx: &mut GkVkContext, buffers: &mut Vec<GkVkBuffer>)
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

/// ### fn gk_create_vk_array_buffer<T>( ... ) ###
/// *Creates a GkVkBuffer from Vec<T> data.*
/// <pre>
/// - Params
///     vk_ctx:     <b>&mut</b> GkVkContext
///     label:      &str                    <i>// Used for debug purposes.</i>
///     usage:      vk::BufferUsageFlags
///     data:       &Vec&lt;T&gt;
/// - Return
///     GkVkBuffer
/// </pre>
pub fn gk_create_vk_array_buffer<T>(
        vk_ctx: &mut GkVkContext,
        label: &str,
        usage: vk::BufferUsageFlags,
        data: &Vec<T>
    ) -> GkVkBuffer
{
    let buffer_size = std::mem::size_of::<T>() * data.len();
    let staging_label = format!("staging {}", label);
    let staging = gk_create_vk_buffer(
        vk_ctx, staging_label.as_str(), 
        usage | vk::BufferUsageFlags::TRANSFER_SRC, MemoryLocation::CpuToGpu, 
        buffer_size as vk::DeviceSize
    );

    unsafe{
        let mapped_ptr = staging.allocation.mapped_ptr().unwrap().as_ptr() as *mut T;
        mapped_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
    }

    let buffer = gk_create_vk_buffer(
        vk_ctx, label, 
        usage | vk::BufferUsageFlags::TRANSFER_DST, MemoryLocation::GpuOnly, 
        buffer_size as vk::DeviceSize
    );

    let cmd_buffer = gk_begin_single_time_vk_command_buffer(vk_ctx);
        copy_vk_buffer(&vk_ctx.device, &cmd_buffer, &staging.handle, &buffer.handle, buffer_size as vk::DeviceSize);
    gk_end_single_time_vk_command_buffer(vk_ctx, cmd_buffer);

    gk_destroy_vk_buffer(vk_ctx, staging);

    buffer
}

/// ### fn gk_create_vk_vertex_buffer_from_file\<T\>( ... ) -> GkVkBuffer
/// *Reads 3d format file and creates a vertex buffer using type T*<br>
/// *T is Type of Vertex Data*<br>
/// *Populates vert and index buffer sizes*
/// <pre>
/// - Params
///     vk_ctx:             <b>&mut</b> GkVkContext
///     label:              &str                    <i>// Used for debug purposes<i>
///     usage:              vk::BufferUsageFlags
///     file_path:          &path::Path
/// - Return
///     (GkVkBuffer, GkVkBuffer) <i>// (vertex_buffer, index_buffer)
/// </pre>
pub fn gk_create_vk_vertex_buffer_from_file(
        vk_ctx: &mut GkVkContext, 
        label: &str, 
        file_path: &std::path::Path,
    ) -> (Option<GkVkBuffer>, Option<GkVkBuffer>)
{
    let load_options = tobj::LoadOptions { 
        single_index: true, 
        triangulate: false, 
        ignore_points: true, 
        ignore_lines: true 
    };
    let (models, _materials) = match tobj::load_obj(file_path, &load_options)
    {
        Ok(data) => 
        {
            if let Ok(materials) = data.1 {
                (data.0, Some(materials))
            } else {
                (data.0, None)
            }
        },
        Err(err) => { log_err!(err.to_string()); return (None, None); }
    };

    let mut vertices: Vec<VertexData> = vec![];
    let mut indices: Vec<u32> = vec![];

    for m in models.iter()
    {
        let mesh = &m.mesh;

        if mesh.texcoords.len() == 0
        {
            log_err!(format!("Missing texture coordinates for {}", file_path.to_str().unwrap()));
            return (None, None);
        }
        
        let total_vertices_count = mesh.positions.len() / 3;
        for i in 0..total_vertices_count
        {
            let vertex = VertexData
            {
                pos: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2]
                ],
                color: [ 0.5, 0.5, 0.5 ],
                tex_coord: [
                    mesh.texcoords[ i * 2 ],
                    -mesh.texcoords[ i * 2 + 1]
                ]
            };
            vertices.push(vertex);
        }
        indices = mesh.indices.clone();
    }

    let vert_buffer_size = std::mem::size_of::<VertexData>() * vertices.len();
    let index_buffer_size = std::mem::size_of::<u32>() * indices.len();

    // let buffer_size = (vert_buffer_size + index_buffer_size) as vk::DeviceSize;
    let staging_vert_label = String::from(format!("staging vert{}", label));

    let staging_vertex = gk_create_vk_buffer(
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
    let staging_indices = gk_create_vk_buffer(
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
    let vert_buffer = gk_create_vk_buffer(
        vk_ctx,
        &vert_label, 
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
        vert_buffer_size as vk::DeviceSize
    );

    let index_label = String::from(format!("index {}", label));
    let index_buffer = gk_create_vk_buffer(
        vk_ctx,
        &index_label,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
        index_buffer_size as vk::DeviceSize
    );

    let cmd_buffer = gk_begin_single_time_vk_command_buffer(vk_ctx);
        copy_vk_buffer(&vk_ctx.device, &cmd_buffer, &staging_vertex.handle, &vert_buffer.handle, vert_buffer_size as vk::DeviceSize);
        copy_vk_buffer(&vk_ctx.device, &cmd_buffer, &staging_indices.handle, &index_buffer.handle, index_buffer_size as vk::DeviceSize);
    gk_end_single_time_vk_command_buffer(vk_ctx, cmd_buffer);

    gk_destroy_vk_buffer(vk_ctx, staging_vertex);
    gk_destroy_vk_buffer(vk_ctx, staging_indices);

    (Some(vert_buffer), Some(index_buffer))
}

pub struct GkVkVertStorageBuffer
{
    pub handle:         vk::Buffer,
    pub allocation:     Allocation,
    pub size:           vk::DeviceSize,
    pub verts_size:      usize,
    pub indices_size:   usize,
}

pub fn gk_create_vk_vertex_storage_buffer<T>(
        vk_ctx: &mut GkVkContext,
        label: &str,
        verts: &Vec<T>,
        indices: Option<&Vec<u32>>
    ) -> GkVkVertStorageBuffer
{
    let verts_size = std::mem::size_of::<T>() * verts.len();
    let mut indices_size: usize = 0;
    if indices.is_some()
    {
        indices_size = std::mem::size_of::<u32>() * indices.unwrap().len();
    }
    let buffer_size = verts_size + indices_size;

    let staging_label = format!("staging {}", label);
    let staging = gk_create_vk_buffer(
        vk_ctx, staging_label.as_str(), 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        MemoryLocation::CpuToGpu, 
        buffer_size as vk::DeviceSize
    );

    unsafe{
        // let mapped_ptr = staging.allocation.mapped_ptr().unwrap().as_ptr() as *mut T;
        let mapped_ptr = staging.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
        // mapped_ptr.copy_from_nonoverlapping(verts.as_ptr(), verts.len());
        std::ptr::copy_nonoverlapping(verts.as_ptr() as * const u8, mapped_ptr, verts_size);
        std::ptr::copy(indices.unwrap().as_ptr() as * const u8, mapped_ptr.add(verts_size), indices_size);
    }

    let buffer = gk_create_vk_buffer(
        vk_ctx, label, 
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, MemoryLocation::GpuOnly, 
        buffer_size as vk::DeviceSize
    );

    let cmd_buffer = gk_begin_single_time_vk_command_buffer(vk_ctx);
        copy_vk_buffer(&vk_ctx.device, &cmd_buffer, &staging.handle, &buffer.handle, buffer_size as vk::DeviceSize);
    gk_end_single_time_vk_command_buffer(vk_ctx, cmd_buffer);

    gk_destroy_vk_buffer(vk_ctx, staging);

    // buffer
    GkVkVertStorageBuffer { handle: buffer.handle, allocation: buffer.allocation, size: buffer.size, verts_size, indices_size }
}

pub fn gk_destroy_vk_vertex_storage_buffer(vk_ctx: &mut GkVkContext, buffer: GkVkVertStorageBuffer)
{
    unsafe
    {
        vk_ctx.device.destroy_buffer(buffer.handle, None);
    }
    vk_check!( vk_ctx.allocator.as_mut().unwrap().free(buffer.allocation) ).unwrap();
}