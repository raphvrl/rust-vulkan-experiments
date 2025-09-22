use anyhow::Result;
use ash::{Entry, Instance, vk};
use std::ffi::{CStr, CString};

pub struct VulkanInstance {
    pub entry: Entry,
    pub instance: Instance,
}

impl VulkanInstance {
    pub fn new(window_extensions: &[*const i8]) -> Result<Self> {
        let entry = unsafe { Entry::load()? };

        let app_name = CString::new("Vulkan Experiments")?;
        let engine_name = CString::new("No Engine")?;

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let mut extensions = Vec::from(window_extensions);

        #[cfg(debug_assertions)]
        {
            extensions.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        let layer_names = if cfg!(debug_assertions) {
            vec![CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")?.as_ptr()]
        } else {
            vec![]
        };

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layer_names);

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        Ok(Self { entry, instance })
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        };
    }
}
