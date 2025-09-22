use anyhow::Result;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct VulkanWindow {
    window: Window,
    running: bool,
}

impl VulkanWindow {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let window_attributes = Window::default_attributes()
            .with_title("Vulkan Experiments")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            .with_resizable(true);

        let window = event_loop
            .create_window(window_attributes)
            .map_err(|e| anyhow::anyhow!("Failed to create window: {}", e))?;

        Ok(Self {
            window,
            running: true,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn on_render(&mut self) {
        self.window.request_redraw();
    }

    pub fn get_required_extensions() -> Vec<*const i8> {
        let mut extensions = vec![ash::khr::surface::NAME.as_ptr()];

        #[cfg(target_os = "windows")]
        extensions.push(ash::khr::win32_surface::NAME.as_ptr());

        #[cfg(target_os = "linux")]
        {
            extensions.push(ash::khr::wayland_surface::NAME.as_ptr());
            extensions.push(ash::khr::xlib_surface::NAME.as_ptr());
        }

        #[cfg(target_os = "macos")]
        extensions.push(ash::ext::metal_surface::NAME.as_ptr());

        extensions
    }
}
