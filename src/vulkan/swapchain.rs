use anyhow::Result;
use ash::{Device, vk};
use std::sync::Arc;

use crate::vulkan::{VulkanDevice, VulkanInstance, VulkanPhysicalDevice, VulkanSurface};

pub struct VulkanSwapchain {
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub device: Arc<Device>,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
}

impl VulkanSwapchain {
    pub fn new(
        instance: &VulkanInstance,
        device: &VulkanDevice,
        physical_device: &VulkanPhysicalDevice,
        surface: &VulkanSurface,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self> {
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance.instance, &device.device);

        let surface_format = Self::choose_surface_format(surface, physical_device)?;

        let present_mode = Self::choose_present_mode(surface, physical_device)?;

        let extent = Self::choose_extent(surface, physical_device, window_width, window_height)?;

        let capabilities = surface.get_capabilities(physical_device)?;
        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        println!("Creating swapchain with {} images", image_count);
        println!(
            "Format: {:?}, Present mode: {:?}",
            surface_format, present_mode
        );
        println!("Extent: {}x{}", extent.width, extent.height);

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None)? };

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let image_views = Self::create_image_views(&device.device, &images, surface_format.format)?;

        Ok(Self {
            swapchain,
            swapchain_loader,
            device: device.device.clone(),
            images,
            image_views,
            format: surface_format,
            extent,
        })
    }

    fn choose_surface_format(
        surface: &VulkanSurface,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<vk::SurfaceFormatKHR> {
        let available_formats = surface.get_formats(physical_device)?;

        for format in &available_formats {
            if format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return Ok(*format);
            }
        }

        Ok(available_formats[0])
    }

    fn choose_present_mode(
        surface: &VulkanSurface,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<vk::PresentModeKHR> {
        let available_modes = surface.get_present_modes(physical_device)?;

        for &mode in &available_modes {
            if mode == vk::PresentModeKHR::MAILBOX {
                return Ok(mode);
            }
        }

        Ok(vk::PresentModeKHR::FIFO)
    }

    fn choose_extent(
        surface: &VulkanSurface,
        physical_device: &VulkanPhysicalDevice,
        window_width: u32,
        window_height: u32,
    ) -> Result<vk::Extent2D> {
        let capabilities = surface.get_capabilities(physical_device)?;

        if capabilities.current_extent.width != u32::MAX {
            Ok(capabilities.current_extent)
        } else {
            let width = window_width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            );
            let height = window_height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            );

            Ok(vk::Extent2D { width, height })
        }
    }

    fn create_image_views(
        device: &Device,
        images: &[vk::Image],
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>> {
        let mut image_views = Vec::new();

        for &image in images {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let image_view = unsafe { device.create_image_view(&create_info, None)? };

            image_views.push(image_view);
        }

        Ok(image_views)
    }
}

impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        for &image_view in &self.image_views {
            unsafe {
                self.device.destroy_image_view(image_view, None);
            }
        }

        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
        println!("Swapchain destroyed");
    }
}
