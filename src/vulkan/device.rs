use anyhow::Result;
use ash::{Device, vk};
use std::collections::HashSet;
use std::sync::Arc;

use crate::vulkan::{QueueFamilyIndices, VulkanInstance, VulkanPhysicalDevice};

pub struct VulkanDevice {
    pub device: Arc<Device>,
    pub graphics_queue: vk::Queue,
    pub compute_queue: Option<vk::Queue>,
    pub transfer_queue: Option<vk::Queue>,
    pub present_queue: Option<vk::Queue>,
    pub queue_family_indices: QueueFamilyIndices,
}

impl VulkanDevice {
    pub fn new(
        instance: &VulkanInstance,
        physical_device: &VulkanPhysicalDevice,
        queue_families: QueueFamilyIndices,
    ) -> Result<Self> {
        let device_extensions = Self::get_required_device_extensions();

        if !physical_device
            .check_device_extension_support(&instance.instance, &device_extensions)?
        {
            return Err(anyhow::anyhow!(
                "Device doesn't support required extensions"
            ));
        }

        let mut unique_queue_families = HashSet::new();
        let queue_priorities = vec![1.0f32];

        if let Some(graphics_family) = queue_families.graphics_family {
            unique_queue_families.insert(graphics_family);
        }
        if let Some(compute_family) = queue_families.compute_family {
            unique_queue_families.insert(compute_family);
        }
        if let Some(transfer_family) = queue_families.transfer_family {
            unique_queue_families.insert(transfer_family);
        }
        if let Some(present_family) = queue_families.present_family {
            unique_queue_families.insert(present_family);
        }

        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
            .into_iter()
            .map(|queue_family| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(queue_family)
                    .queue_priorities(&queue_priorities)
            })
            .collect();

        let device_features = vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&device_features);

        let device = unsafe {
            instance.instance.create_device(
                physical_device.physical_device,
                &device_create_info,
                None,
            )?
        };

        let graphics_queue =
            unsafe { device.get_device_queue(queue_families.graphics_family.unwrap(), 0) };

        let compute_queue = queue_families
            .compute_family
            .map(|family| unsafe { device.get_device_queue(family, 0) });

        let transfer_queue = queue_families
            .transfer_family
            .map(|family| unsafe { device.get_device_queue(family, 0) });

        let present_queue = queue_families
            .present_family
            .map(|family| unsafe { device.get_device_queue(family, 0) });

        Ok(Self {
            device: Arc::new(device),
            graphics_queue,
            compute_queue,
            transfer_queue,
            present_queue,
            queue_family_indices: queue_families,
        })
    }

    fn get_required_device_extensions() -> Vec<*const i8> {
        vec![ash::khr::swapchain::NAME.as_ptr()]
    }

    pub fn wait_idle(&self) -> Result<()> {
        unsafe {
            self.device.device_wait_idle()?;
        }
        Ok(())
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
        println!("Logical device destroyed");
    }
}
