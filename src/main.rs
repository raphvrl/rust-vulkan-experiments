use anyhow::Result;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};

mod window;
use window::VulkanWindow;

struct App {
    vulkan_window: Option<VulkanWindow>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.vulkan_window.is_none() {
            match VulkanWindow::new(event_loop) {
                Ok(window) => self.vulkan_window = Some(window),
                Err(e) => {
                    eprintln!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref mut vulkan_window) = self.vulkan_window {
            match event {
                WindowEvent::CloseRequested => {
                    vulkan_window.stop();
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    vulkan_window.on_render();
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref vulkan_window) = self.vulkan_window {
            vulkan_window.window().request_redraw();
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let mut app = App {
        vulkan_window: None,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
