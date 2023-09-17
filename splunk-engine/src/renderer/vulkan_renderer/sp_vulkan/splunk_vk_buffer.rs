use std::ffi::c_void;

use ash::{self, vk};

use gpu_allocator::{
    MemoryLocation,
    vulkan::{
        Allocator,
        Allocation,
        AllocationCreateDesc, 
        AllocationScheme
    }, 
};

use crate::{vk_check, log_err};

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
/// *Creates a vk::Buffer and allocates memory.*
/// <pre>
/// - Params
///     device:
///     allocator:
///     label:
///     size:
///     usage:
/// - Return
///     (vk::Buffer, gpu_allocator::vulkan::Allocation)
/// </pre>
pub fn create_vk_buffer(
        device: &ash::Device, allocator: &mut Allocator,
        label: &str, size: vk::DeviceSize,
        usage: vk::BufferUsageFlags
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
        location: MemoryLocation::CpuToGpu,
        linear: true,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged
    };
    let allocation = allocator.allocate(&alloc_info).map_err( |e| { log_err!(e); } ).unwrap();

    unsafe{ vk_check!( device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) ).unwrap(); }
    
    (buffer, allocation)
}

/// ### fn map_vk_buffer_data( ... )
/// *Maps data to memory*
/// <pre>
/// - Param
///     device:         &ash::Device
///     memory:         &vk::DeviceMemory
///     offset:         &vk::DeviceSize (u64)
///     data:           &c_void
///     size:           vk::DeviceSize (u64)
/// </pre>
pub fn map_vk_buffer_data(device: &ash::Device, memory: &vk::DeviceMemory, offset: vk::DeviceSize, data: &c_void, size: vk::DeviceSize)
{   
    unsafe
    {
        let mapped_data = vk_check!(device.map_memory(*memory, offset, size, vk::MemoryMapFlags::empty())).unwrap();
            mapped_data.copy_from_nonoverlapping(data, size as usize);
        device.unmap_memory(*memory);
    }
}