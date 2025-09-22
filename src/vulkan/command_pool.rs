use anyhow::Result;
use ash::{Device, vk};
use std::sync::Arc;

use crate::vulkan::{QueueFamilyIndices, VulkanDevice, VulkanRenderPass};

pub struct VulkanCommandPool {
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub device: Arc<Device>,
}

impl VulkanCommandPool {
    pub fn new(
        device: &VulkanDevice,
        queue_family_indices: QueueFamilyIndices,
        buffer_count: usize,
    ) -> Result<Self> {
        let graphics_family = queue_family_indices.graphics_family.unwrap();

        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_family);

        let command_pool = unsafe {
            device
                .device
                .create_command_pool(&pool_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create command pool: {}", e))?
        };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(buffer_count as u32);

        let command_buffers = unsafe {
            device
                .device
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| anyhow::anyhow!("Failed to allocate command buffers: {}", e))?
        };

        Ok(Self {
            command_pool,
            command_buffers,
            device: device.device.clone(),
        })
    }

    pub fn get_command_buffer(&self, index: usize) -> &vk::CommandBuffer {
        &self.command_buffers[index]
    }

    pub fn buffer_count(&self) -> usize {
        self.command_buffers.len()
    }

    pub fn begin_command_buffer(&self, index: usize) -> Result<()> {
        let command_buffer = self.get_command_buffer(index);

        let begin_info =
            vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::empty());

        unsafe {
            self.device
                .begin_command_buffer(*command_buffer, &begin_info)
                .map_err(|e| anyhow::anyhow!("Failed to begin command buffer: {}", e))?
        };

        Ok(())
    }

    pub fn end_command_buffer(&self, index: usize) -> Result<()> {
        let command_buffer = self.get_command_buffer(index);

        unsafe {
            self.device
                .end_command_buffer(*command_buffer)
                .map_err(|e| anyhow::anyhow!("Failed to end command buffer: {}", e))?
        };

        Ok(())
    }

    pub fn reset_command_buffer(&self, index: usize) -> Result<()> {
        let command_buffer = self.get_command_buffer(index);

        unsafe {
            self.device
                .reset_command_buffer(*command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| anyhow::anyhow!("Failed to reset command buffer: {}", e))?
        };

        Ok(())
    }

    pub fn begin_render_pass(
        &self,
        command_buffer_index: usize,
        render_pass: &VulkanRenderPass,
        framebuffer: vk::Framebuffer,
        extent: &vk::Extent2D,
        clear_color: [f32; 4],
    ) {
        let command_buffer = self.get_command_buffer(command_buffer_index);

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        }];

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(render_pass.render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: *extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn end_render_pass(&self, command_buffer_index: usize) {
        let command_buffer = self.get_command_buffer(command_buffer_index);

        unsafe {
            self.device.cmd_end_render_pass(*command_buffer);
        }
    }

    pub fn draw(&self, command_buffer_index: usize, vertex_count: u32, instance_count: u32) {
        let command_buffer = self.get_command_buffer(command_buffer_index);

        unsafe {
            self.device
                .cmd_draw(*command_buffer, vertex_count, instance_count, 0, 0);
        }
    }
}

impl Drop for VulkanCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
        }
        println!("Command pool destroyed");
    }
}
