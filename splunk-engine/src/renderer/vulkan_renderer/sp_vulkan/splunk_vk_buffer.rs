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