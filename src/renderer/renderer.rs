use anyhow::Result;
use ash::{Device, vk};
use std::sync::Arc;

use crate::vulkan::{
    FrameSyncObjects, VulkanCommandPool, VulkanDevice, VulkanFramebuffers, VulkanInstance,
    VulkanRenderPass, VulkanSwapchain, VulkanSyncObjects,
};

use crate::pipeline::VulkanPipeline;

pub struct VulkanRenderer {
    pub device: Arc<Device>,
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub current_frame: usize,
    pub max_frames_in_flight: usize,
}

impl VulkanRenderer {
    pub fn new(logical_device: &VulkanDevice, instance: &VulkanInstance) -> Self {
        let swapchain_loader =
            ash::khr::swapchain::Device::new(&instance.instance, &logical_device.device);

        Self {
            device: logical_device.device.clone(),
            swapchain_loader,
            current_frame: 0,
            max_frames_in_flight: 3,
        }
    }

    pub fn draw_frame(
        &mut self,
        logical_device: &VulkanDevice,
        swapchain: &VulkanSwapchain,
        render_pass: &VulkanRenderPass,
        framebuffers: &VulkanFramebuffers,
        command_pool: &VulkanCommandPool,
        sync_objects: &VulkanSyncObjects,
        pipeline: &VulkanPipeline,
    ) -> Result<()> {
        sync_objects.wait_for_fence(self.current_frame)?;

        let frame_image_semaphore = sync_objects.image_available_semaphores
            [self.current_frame % sync_objects.max_frames_in_flight];

        let (image_index, _is_suboptimal) = unsafe {
            self.swapchain_loader
                .acquire_next_image(
                    swapchain.swapchain,
                    u64::MAX,
                    frame_image_semaphore,
                    vk::Fence::null(),
                )
                .map_err(|e| anyhow::anyhow!("Failed to acquire swapchain image: {}", e))?
        };

        let frame_sync = sync_objects.get_frame_sync_objects(self.current_frame);

        sync_objects.reset_fence(self.current_frame)?;

        self.record_command_buffer(
            command_pool,
            render_pass,
            framebuffers,
            swapchain,
            pipeline,
            image_index as usize,
        )?;

        self.submit_command_buffer(
            logical_device,
            command_pool,
            image_index as usize,
            &frame_sync,
        )?;

        self.present_frame(logical_device, swapchain, image_index, &frame_sync)?;

        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;

        Ok(())
    }

    fn record_command_buffer(
        &self,
        command_pool: &VulkanCommandPool,
        render_pass: &VulkanRenderPass,
        framebuffers: &VulkanFramebuffers,
        swapchain: &VulkanSwapchain,
        pipeline: &VulkanPipeline,
        image_index: usize,
    ) -> Result<()> {
        command_pool.reset_command_buffer(image_index)?;
        command_pool.begin_command_buffer(image_index)?;

        command_pool.begin_render_pass(
            image_index,
            render_pass,
            framebuffers.get_framebuffer(image_index),
            &swapchain.extent,
            [0.1, 0.1, 0.1, 1.0],
        );

        pipeline.bind(*command_pool.get_command_buffer(image_index));

        let viewport = ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = ash::vk::Rect2D {
            offset: ash::vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        };

        unsafe {
            command_pool.device.cmd_set_viewport(
                *command_pool.get_command_buffer(image_index),
                0,
                &[viewport],
            );

            command_pool.device.cmd_set_scissor(
                *command_pool.get_command_buffer(image_index),
                0,
                &[scissor],
            );
        }

        command_pool.draw(image_index, 3, 1);

        command_pool.end_render_pass(image_index);
        command_pool.end_command_buffer(image_index)?;

        Ok(())
    }

    fn submit_command_buffer(
        &self,
        logical_device: &VulkanDevice,
        command_pool: &VulkanCommandPool,
        image_index: usize,
        frame_sync: &FrameSyncObjects,
    ) -> Result<()> {
        let command_buffer = command_pool.get_command_buffer(image_index);

        let wait_semaphores = [frame_sync.image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [frame_sync.render_finished_semaphore];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&command_buffer))
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.device
                .queue_submit(
                    logical_device.graphics_queue,
                    &[submit_info],
                    frame_sync.in_flight_fence,
                )
                .map_err(|e| anyhow::anyhow!("Failed to submit command buffer: {}", e))?;
        }

        Ok(())
    }

    fn present_frame(
        &self,
        logical_device: &VulkanDevice,
        swapchain: &VulkanSwapchain,
        image_index: u32,
        frame_sync: &FrameSyncObjects,
    ) -> Result<()> {
        let wait_semaphores = [frame_sync.render_finished_semaphore];
        let swapchains = [swapchain.swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain_loader
                .queue_present(logical_device.present_queue.unwrap(), &present_info)
                .map_err(|e| anyhow::anyhow!("Failed to present swapchain image: {}", e))?;
        }

        Ok(())
    }
}
