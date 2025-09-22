use anyhow::Result;
use ash::{Device, vk};
use std::sync::Arc;

use crate::vulkan::{VulkanDevice, VulkanSwapchain};

pub struct VulkanRenderPass {
    pub render_pass: vk::RenderPass,
    pub device: Arc<Device>,
}

impl VulkanRenderPass {
    pub fn new(device: &VulkanDevice, swapchain: &VulkanSwapchain) -> Result<Self> {
        let color_attachment = vk::AttachmentDescription::default()
            .format(swapchain.format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_create_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        let render_pass = unsafe {
            device
                .device
                .create_render_pass(&render_pass_create_info, None)?
        };

        Ok(Self {
            render_pass,
            device: device.device.clone(),
        })
    }
}

impl Drop for VulkanRenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
