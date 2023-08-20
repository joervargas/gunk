use ash::{self, vk};

use super::splunk_vk_img::find_vk_format_depth_img;

use crate::vk_check;

use super::splunk_vk_context::SpVkContext;

use bitflags::bitflags;


bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ERenderPassBit: u32
    {
        const NONE = 0x00;
        const FIRST = 0x01;
        const LAST = 0x02;
        const OFFSCREEN = 0x04;
        const OFFSCREEN_INTERNAL = 0x08;
    }
}

pub struct SpVkRenderPassInfo
{
    pub b_use_color: bool,
    pub b_clear_color: bool,
    pub color_format: vk::Format,
    pub b_use_depth: bool,
    pub b_clear_depth: bool,
    pub flags: ERenderPassBit
}

pub struct SpVkRenderPass
{
    pub info: SpVkRenderPassInfo,
    pub handle: vk::RenderPass
}

pub fn sp_create_vk_renderpass(instance: &ash::Instance, vk_ctx: &SpVkContext, info: SpVkRenderPassInfo) -> SpVkRenderPass
{
    let offscreen_internal: bool = (info.flags & ERenderPassBit::OFFSCREEN_INTERNAL) != ERenderPassBit::NONE;
    let first: bool = (info.flags & ERenderPassBit::FIRST) != ERenderPassBit::NONE;
    let last: bool = (info.flags & ERenderPassBit::LAST) != ERenderPassBit::NONE;
    
    let mut attachments: Vec<vk::AttachmentDescription> = Vec::new();

    let mut color_attachment: vk::AttachmentDescription;
    let mut color_attachment_ref: vk::AttachmentReference = Default::default();
    if info.b_use_color
    {
        color_attachment = vk::AttachmentDescription
        {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: info.color_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: if offscreen_internal { vk::AttachmentLoadOp::LOAD} else { if info.b_clear_color { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD }},
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: if first { vk::ImageLayout::UNDEFINED } else { if offscreen_internal { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL} else { vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL }},
            final_layout: if last { vk::ImageLayout::PRESENT_SRC_KHR } else { vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL }
        };

        if info.flags & ERenderPassBit::OFFSCREEN != ERenderPassBit::NONE
        {
            if info.b_clear_color
            {
                color_attachment.final_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            }
        }

        color_attachment_ref = vk::AttachmentReference
        {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        };

        attachments.push(color_attachment);
    }

    let mut depth_attachmment: vk::AttachmentDescription;
    let mut depth_attachment_ref: vk::AttachmentReference = Default::default();
    if info.b_use_depth
    {
        depth_attachmment = vk::AttachmentDescription
        {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: find_vk_format_depth_img(instance, &vk_ctx.physical_device),
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: if offscreen_internal { vk::AttachmentLoadOp::LOAD} else { if info.b_clear_depth { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD }},
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: if info.b_clear_depth { vk::ImageLayout::UNDEFINED } else { if offscreen_internal { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL } else { vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL }},
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };

        if info.flags & ERenderPassBit::OFFSCREEN != ERenderPassBit::NONE
        {
            if info.b_clear_depth
            {
                depth_attachmment.final_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            }
        }

        depth_attachment_ref = vk::AttachmentReference
        {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };

        attachments.push(depth_attachmment);
    }

    let mut dependencies: Vec<vk::SubpassDependency> = vec![
        vk::SubpassDependency
        {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty()
        },
    ];

    if info.flags & ERenderPassBit::OFFSCREEN != ERenderPassBit::NONE
    {
        dependencies.clear();
        dependencies = vec![
            vk::SubpassDependency
            {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags::SHADER_READ,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::BY_REGION
            },
            vk::SubpassDependency
            {
                src_subpass: 0,
                dst_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: vk::AccessFlags::SHADER_READ,
                dependency_flags: vk::DependencyFlags::BY_REGION
            }
        ];
    }

    let subpass = vk::SubpassDescription
    {
        flags: vk::SubpassDescriptionFlags::empty(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count: 0,
        p_input_attachments: std::ptr::null(),
        color_attachment_count: if info.b_use_color { 1 } else { 0 },
        p_color_attachments: if info.b_use_color { [color_attachment_ref].as_ptr() } else { std::ptr::null() },
        p_resolve_attachments: std::ptr::null(),
        p_depth_stencil_attachment: if info.b_use_depth { [depth_attachment_ref].as_ptr() } else { std::ptr::null() },
        preserve_attachment_count: 0,
        p_preserve_attachments: std::ptr::null()
    };

    let create_info = vk::RenderPassCreateInfo
    {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: attachments.len() as u32,
        p_attachments: attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: [subpass].as_ptr(),
        dependency_count: dependencies.len() as u32,
        p_dependencies: dependencies.as_ptr()
    };

    let handle = unsafe { vk_check!( vk_ctx.device.create_render_pass(&create_info, None) ).unwrap() };

    SpVkRenderPass
    {
        info,
        handle
    }
}