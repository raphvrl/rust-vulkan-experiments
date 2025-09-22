use anyhow::Result;
use ash::{Device, vk};
use std::sync::Arc;

use crate::vulkan::{VulkanDevice, VulkanRenderPass, VulkanSwapchain};

pub struct VulkanFramebuffers {
    pub framebuffers: Vec<vk::Framebuffer>,
    pub device: Arc<Device>,
}

impl VulkanFramebuffers {
    pub fn new(
        device: &VulkanDevice,
        render_pass: &VulkanRenderPass,
        swapchain: &VulkanSwapchain,
    ) -> Result<Self> {
        let mut framebuffers = Vec::with_capacity(swapchain.images.len());

        for &image_view in swapchain.image_views.iter() {
            let attachments = [image_view];

            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass.render_pass)
                .attachments(&attachments)
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);

            let framebuffer = unsafe { device.device.create_framebuffer(&framebuffer_info, None)? };

            framebuffers.push(framebuffer);
        }

        Ok(Self {
            framebuffers,
            device: device.device.clone(),
        })
    }

    pub fn get_framebuffer(&self, index: usize) -> vk::Framebuffer {
        self.framebuffers[index]
    }
}

impl Drop for VulkanFramebuffers {
    fn drop(&mut self) {
        unsafe {
            for framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
        }
        println!("Framebuffers destroyed");
    }
}
