use ash::{self, vk};

use super::splunk_vk_context::SpVkContext;

use crate::vk_check;

/// ### fn get_vk_desc_set_layout_binding( ... ) -> vk::DescriptorSetLayoutBinding
/// *Returns a populated vk::DescriptorLayoutBinding struct*
/// <pre>
/// - Params
///     binding:        u32
///     desc_type:      vk::DescriptorType
///     count:          u32
///     shader_stage:   vk::ShaderStageFlags
/// - Return
///     vk::DescriptorSetLayoutBinding
/// </pre>
pub fn get_vk_desc_set_layout_binding(
        binding: u32, 
        desc_type: vk::DescriptorType, 
        count: u32,
        shader_stage: vk::ShaderStageFlags
    ) -> vk::DescriptorSetLayoutBinding
{
    vk::DescriptorSetLayoutBinding
    {
        binding,
        descriptor_type: desc_type,
        descriptor_count: count,
        stage_flags: shader_stage,
        p_immutable_samplers: std::ptr::null()
    }
}

/// ### fn get_vk_buffer_write_desc_set( ... ) -> vk::WriteDescriptorSet
/// *Returns a populated vk::WriteDescriptorSet struct*
/// <pre>
/// - Params
///     desc_set:       &vk::DescriptorSet
///     buffer_info:    &[vk::DescriptorBufferInfo]
///     binding:        u32
///     desc_type:      vk::DescriptorType
/// </pre>
pub fn get_vk_buffer_write_desc_set(
        desc_set: &vk::DescriptorSet,
        buffer_info: &[vk::DescriptorBufferInfo],
        binding: u32,
        desc_type: vk::DescriptorType
    ) -> vk::WriteDescriptorSet
{
    vk::WriteDescriptorSet
    {
        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
        p_next: std::ptr::null(),
        dst_set: *desc_set,
        dst_binding: binding,
        dst_array_element: 0,
        descriptor_count: 1,
        descriptor_type: desc_type,
        p_image_info: std::ptr::null(),
        p_buffer_info: buffer_info.as_ptr(),
        p_texel_buffer_view: std::ptr::null()
    }
}

/// ### fn get_vk_image_write_desc_set( ... ) -> vk::WriteDescriptorSet
/// *Returns a populated vk::WriteDescriptorSet struct*
/// <pre>
/// - Params
///     desc_set:       &vk::DescriptorSet
///     image_info:     &[vk::DescriptorImageInfo]
///     binding:        u32
/// - Return
///     vk::WriteDescriptorSet
/// </pre>
pub fn get_vk_image_write_desc_set(
        desc_set: &vk::DescriptorSet,
        image_info: &[vk::DescriptorImageInfo],
        binding: u32,
    ) -> vk::WriteDescriptorSet
{
    vk::WriteDescriptorSet
    {
        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
        p_next: std::ptr::null(),
        dst_set: *desc_set,
        dst_binding: binding,
        dst_array_element: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        p_image_info: image_info.as_ptr(),
        p_buffer_info: std::ptr::null(),
        p_texel_buffer_view: std::ptr::null()
    }
}

pub fn sp_create_vk_desc_pool(
    vk_ctx: &SpVkContext, 
    uniform_count: u32, 
    storage_count: u32, 
    img_sample_count: u32
) -> vk::DescriptorPool
{
    let img_count = vk_ctx.swapchain.images.len() as u32;
    let mut pool_sizes: Vec<vk::DescriptorPoolSize> = Vec::new();

    if uniform_count > 0
    {
        pool_sizes.push(
            vk::DescriptorPoolSize
            {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: img_count * uniform_count
            }
        );
    }

    if storage_count > 0
    {
        pool_sizes.push(
            vk::DescriptorPoolSize
            {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: img_count * storage_count
            }
        )
    }

    if img_sample_count > 0
    {
        pool_sizes.push(
            vk::DescriptorPoolSize
            {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: img_count * img_sample_count
            }
        )
    }

    let create_info = vk::DescriptorPoolCreateInfo
    {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: img_count,
        pool_size_count: pool_sizes.len() as u32,
        p_pool_sizes: pool_sizes.as_ptr()
    };

    unsafe{
        vk_check!(vk_ctx.device.create_descriptor_pool(&create_info, None)).unwrap()
    }
}

/// ### SpVkDescriptor struct
/// <pre>
/// - Members
///     layout:     Vec&lt;vk::DescriptorSetLayout&gt;
///     pool:       vk::DescriptorPool
///     sets:       Vec&lt;vk::DescriptorSet&gt;
/// </pre>
pub struct SpVkDescriptor
{
    pub layouts:    Vec<vk::DescriptorSetLayout>,
    pub pool:       vk::DescriptorPool,
    pub sets:       Vec<vk::DescriptorSet>
}

pub fn sp_destroy_vk_descriptor(vk_ctx: &SpVkContext, descriptor: &SpVkDescriptor)
{
    for layout in descriptor.layouts.iter()
    {
        unsafe { vk_ctx.device.destroy_descriptor_set_layout(*layout, None) }
    }
    unsafe{
        vk_ctx.device.destroy_descriptor_pool(descriptor.pool, None);
    }
}