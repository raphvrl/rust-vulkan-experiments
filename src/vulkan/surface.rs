use anyhow::Result;
use ash::vk;
use ash_window;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::VulkanWindow;
use crate::vulkan::{VulkanInstance, VulkanPhysicalDevice};

pub struct VulkanSurface {
    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
}

impl VulkanSurface {
    pub fn new(instance: &VulkanInstance, vulkan_window: &VulkanWindow) -> Result<Self> {
        let window = vulkan_window.window();

        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance.instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )?
        };

        let surface_loader = ash::khr::surface::Instance::new(&instance.entry, &instance.instance);

        println!("Surface created successfully");

        Ok(Self {
            surface,
            surface_loader,
        })
    }

    pub fn get_capabilities(
        &self,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR> {
        let capabilities = unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(
                    physical_device.physical_device,
                    self.surface,
                )?
        };
        Ok(capabilities)
    }

    pub fn get_formats(
        &self,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>> {
        let formats = unsafe {
            self.surface_loader.get_physical_device_surface_formats(
                physical_device.physical_device,
                self.surface,
            )?
        };
        Ok(formats)
    }

    pub fn get_present_modes(
        &self,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>> {
        let present_modes = unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(
                    physical_device.physical_device,
                    self.surface,
                )?
        };
        Ok(present_modes)
    }

    pub fn check_surface_support(
        &self,
        physical_device: &VulkanPhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool> {
        let support = unsafe {
            self.surface_loader.get_physical_device_surface_support(
                physical_device.physical_device,
                queue_family_index,
                self.surface,
            )?
        };
        Ok(support)
    }
}

impl Drop for VulkanSurface {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
        println!("Surface destroyed");
    }
}
