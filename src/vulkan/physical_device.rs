use anyhow::Result;
use ash::{Instance, vk};
use std::ffi::CStr;

use crate::vulkan::{VulkanInstance, VulkanSurface};

pub struct VulkanPhysicalDevice {
    pub physical_device: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl VulkanPhysicalDevice {
    pub fn select_best_device(vulkan_instance: &VulkanInstance) -> Result<Self> {
        let instance = &vulkan_instance.instance;

        let physical_devices = unsafe { instance.enumerate_physical_devices()? };

        if physical_devices.is_empty() {
            return Err(anyhow::anyhow!("No physical devices found"));
        }

        let mut best_device = None;
        let mut best_score = 0;

        for &physical_device in physical_devices.iter() {
            let properties = unsafe { instance.get_physical_device_properties(physical_device) };
            let features = unsafe { instance.get_physical_device_features(physical_device) };
            let memory_properties =
                unsafe { instance.get_physical_device_memory_properties(physical_device) };

            let score = Self::rate_device(&properties, &features);

            println!(
                "Device: {}, Score {}",
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_string_lossy(),
                score
            );

            if score > best_score {
                best_score = score;
                best_device = Some((physical_device, properties, features, memory_properties));
            }
        }

        if let Some((physical_device, properties, features, memory_properties)) = best_device {
            println!(
                "Selected device: {}",
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_string_lossy()
            );

            Ok(Self {
                physical_device,
                properties,
                features,
                memory_properties,
            })
        } else {
            Err(anyhow::anyhow!("No best device found"))
        }
    }

    fn rate_device(
        properties: &vk::PhysicalDeviceProperties,
        features: &vk::PhysicalDeviceFeatures,
    ) -> u32 {
        let mut score = 0;

        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        } else if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
            score += 500;
        }

        score += properties.limits.max_image_dimension2_d;

        if features.geometry_shader == vk::TRUE {
            score += 100;
        }

        if features.tessellation_shader == vk::TRUE {
            score += 50;
        }

        score
    }

    pub fn find_queue_families(
        &self,
        instance: &Instance,
        surface: &VulkanSurface,
    ) -> Result<QueueFamilyIndices> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(self.physical_device) };

        let mut graphics_family = None;
        let mut compute_family = None;
        let mut transfer_family = None;
        let mut present_family = None;

        for (index, queue_family) in queue_families.iter().enumerate() {
            let index = index as u32;

            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(index);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                compute_family = Some(index);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                transfer_family = Some(index);
            }

            if surface.check_surface_support(self, index)? {
                present_family = Some(index);
            }
        }

        Ok(QueueFamilyIndices {
            graphics_family,
            compute_family,
            transfer_family,
            present_family,
        })
    }

    pub fn check_device_extension_support(
        &self,
        instance: &Instance,
        required_extensions: &[*const i8],
    ) -> Result<bool> {
        let available_extensions =
            unsafe { instance.enumerate_device_extension_properties(self.physical_device)? };

        for &required_ext in required_extensions {
            let required_name = unsafe { CStr::from_ptr(required_ext) };
            let found = available_extensions.iter().any(|ext| {
                let ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                ext_name == required_name
            });

            if !found {
                println!("Missing extension: {}", required_name.to_string_lossy());
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub compute_family: Option<u32>,
    pub transfer_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}
