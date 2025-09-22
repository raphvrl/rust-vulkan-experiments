use anyhow::Result;
use ash::{Device, vk};
use std::sync::Arc;

use crate::vulkan::VulkanDevice;

pub struct VulkanSyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub device: Arc<Device>,
    pub max_frames_in_flight: usize,
}

impl VulkanSyncObjects {
    pub fn new(device: &VulkanDevice, max_frames_in_flight: usize) -> Result<Self> {
        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        let mut in_flight_fences = Vec::new();

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        for i in 0..max_frames_in_flight {
            let image_available_semaphore = unsafe {
                device
                    .device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to create image available semaphore {}: {}", i, e)
                    })?
            };

            let render_finished_semaphore = unsafe {
                device
                    .device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to create render finished semaphore {}: {}", i, e)
                    })?
            };

            let in_flight_fence = unsafe {
                device
                    .device
                    .create_fence(&fence_info, None)
                    .map_err(|e| anyhow::anyhow!("Failed to create in flight fence {}: {}", i, e))?
            };

            image_available_semaphores.push(image_available_semaphore);
            render_finished_semaphores.push(render_finished_semaphore);
            in_flight_fences.push(in_flight_fence);
        }

        println!(
            "Created sync objects for {} frames in flight",
            max_frames_in_flight
        );

        Ok(Self {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            device: device.device.clone(),
            max_frames_in_flight,
        })
    }

    pub fn wait_for_fence(&self, frame_index: usize) -> Result<()> {
        let fence = self.in_flight_fences[frame_index];

        unsafe {
            self.device
                .wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX)
                .map_err(|e| anyhow::anyhow!("Failed to wait for fence {}: {}", frame_index, e))?;
        }

        Ok(())
    }

    pub fn reset_fence(&self, frame_index: usize) -> Result<()> {
        let fence = self.in_flight_fences[frame_index];

        unsafe {
            self.device
                .reset_fences(std::slice::from_ref(&fence))
                .map_err(|e| anyhow::anyhow!("Failed to reset fence {}: {}", frame_index, e))?;
        }

        Ok(())
    }

    pub fn get_frame_sync_objects(&self, frame_index: usize) -> FrameSyncObjects {
        FrameSyncObjects {
            image_available_semaphore: self.image_available_semaphores[frame_index],
            render_finished_semaphore: self.render_finished_semaphores[frame_index],
            in_flight_fence: self.in_flight_fences[frame_index],
        }
    }
}

#[derive(Copy, Clone)]
pub struct FrameSyncObjects {
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

impl Drop for VulkanSyncObjects {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();

            for &semaphore in &self.image_available_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }

            for &semaphore in &self.render_finished_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }

            for &fence in &self.in_flight_fences {
                self.device.destroy_fence(fence, None);
            }
        }
        println!("Sync objects destroyed");
    }
}
