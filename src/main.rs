use anyhow::Result;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};

use rust_vulkan_experiments::VulkanWindow;
use rust_vulkan_experiments::{
    VulkanCommandPool, VulkanDevice, VulkanFramebuffers, VulkanInstance, VulkanPhysicalDevice,
    VulkanRenderPass, VulkanRenderer, VulkanSurface, VulkanSwapchain, VulkanSyncObjects,
};
use rust_vulkan_experiments::{VulkanPipeline, VulkanPipelineBuilder};

struct App {
    renderer: Option<VulkanRenderer>,
    pipeline: Option<VulkanPipeline>,
    sync_objects: Option<VulkanSyncObjects>,
    command_pool: Option<VulkanCommandPool>,
    framebuffers: Option<VulkanFramebuffers>,
    render_pass: Option<VulkanRenderPass>,
    swapchain: Option<VulkanSwapchain>,
    logical_device: Option<VulkanDevice>,
    surface: Option<VulkanSurface>,
    physical_device: Option<VulkanPhysicalDevice>,
    instance: Option<VulkanInstance>,
    window: Option<VulkanWindow>,
}

impl App {
    fn new() -> Self {
        Self {
            renderer: None,
            pipeline: None,
            sync_objects: None,
            command_pool: None,
            framebuffers: None,
            render_pass: None,
            swapchain: None,
            instance: None,
            physical_device: None,
            surface: None,
            window: None,
            logical_device: None,
        }
    }

    fn initalize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let window = VulkanWindow::new(event_loop)?;
        println!("Window created");

        let extensions = VulkanWindow::get_required_extensions();

        let vulkan_instance = VulkanInstance::new(&extensions)?;
        println!("Vulkan instance created");

        let surface = VulkanSurface::new(&vulkan_instance, &window)?;
        println!("Surface created");

        let vulkan_physical_device = VulkanPhysicalDevice::select_best_device(&vulkan_instance)?;
        println!("Physical device selected");

        let queue_families =
            vulkan_physical_device.find_queue_families(&vulkan_instance.instance, &surface)?;
        if !queue_families.is_complete() {
            return Err(anyhow::anyhow!(
                "Device doesn't support required queue families"
            ));
        }
        println!("Queue families found");

        let logical_device = VulkanDevice::new(
            &vulkan_instance,
            &vulkan_physical_device,
            queue_families.clone(),
        )?;
        println!("Logical device created");

        let swapchain = VulkanSwapchain::new(
            &vulkan_instance,
            &logical_device,
            &vulkan_physical_device,
            &surface,
            window.window().inner_size().width,
            window.window().inner_size().height,
        )?;
        println!("Swapchain created");

        let render_pass = VulkanRenderPass::new(&logical_device, &swapchain)?;
        println!("Render pass created");

        let framebuffers = VulkanFramebuffers::new(&logical_device, &render_pass, &swapchain)?;
        println!("Framebuffers created");

        let command_pool = VulkanCommandPool::new(
            &logical_device,
            queue_families.clone(),
            swapchain.images.len(),
        )?;
        println!("Command pool created");

        let sync_objects = VulkanSyncObjects::new(&logical_device, swapchain.images.len())?;
        println!("Sync objects created");

        let renderer = VulkanRenderer::new(&logical_device, &vulkan_instance);
        println!("Renderer created");

        let pipeline = VulkanPipelineBuilder::new(&logical_device)
            .set_render_pass(render_pass.render_pass)
            .set_extent(swapchain.extent)
            .with_vertex_spv(include_bytes!("../bin/triangle.vert.spv"))?
            .with_fragment_spv(include_bytes!("../bin/triangle.frag.spv"))?
            .with_topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST)
            .with_dynamic_states(&[
                ash::vk::DynamicState::VIEWPORT,
                ash::vk::DynamicState::SCISSOR,
            ])
            .with_alpha_blending()
            .build()?;

        self.window = Some(window);
        self.instance = Some(vulkan_instance);
        self.physical_device = Some(vulkan_physical_device);
        self.surface = Some(surface);
        self.logical_device = Some(logical_device);
        self.swapchain = Some(swapchain);
        self.render_pass = Some(render_pass);
        self.framebuffers = Some(framebuffers);
        self.command_pool = Some(command_pool);
        self.sync_objects = Some(sync_objects);
        self.renderer = Some(renderer);
        self.pipeline = Some(pipeline);

        Ok(())
    }

    fn render_frame(&mut self) {
        if let (
            Some(renderer),
            Some(sync_objects),
            Some(command_pool),
            Some(framebuffers),
            Some(render_pass),
            Some(swapchain),
            Some(logical_device),
            Some(pipeline),
        ) = (
            &mut self.renderer,
            &self.sync_objects,
            &self.command_pool,
            &self.framebuffers,
            &self.render_pass,
            &self.swapchain,
            &self.logical_device,
            &self.pipeline,
        ) {
            if let Err(e) = renderer.draw_frame(
                logical_device,
                swapchain,
                render_pass,
                framebuffers,
                command_pool,
                sync_objects,
                pipeline,
            ) {
                eprintln!("Failed to draw frame: {}", e);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            if let Err(e) = self.initalize(event_loop) {
                eprintln!("Failed to initialize: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(ref device) = self.logical_device {
                    let _ = device.wait_idle();
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref vulkan_window) = self.window {
            vulkan_window.window().request_redraw();
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let mut app = App::new();

    event_loop.run_app(&mut app)?;
    Ok(())
}
