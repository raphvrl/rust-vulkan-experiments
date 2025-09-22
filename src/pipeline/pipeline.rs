use anyhow::{Result, bail};
use ash::{Device, vk};
use std::ffi::CString;
use std::sync::Arc;

use crate::vulkan::VulkanDevice;

pub struct VulkanPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    device: Arc<Device>,
}

pub struct VulkanPipelineBuilder {
    device: Arc<Device>,
    render_pass: Option<vk::RenderPass>,
    extent: Option<vk::Extent2D>,

    shader_entries: Vec<(vk::ShaderModule, vk::ShaderStageFlags, CString)>,

    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    push_constant_ranges: Vec<vk::PushConstantRange>,

    vertex_input_bindings: Vec<vk::VertexInputBindingDescription>,
    vertex_input_attributes: Vec<vk::VertexInputAttributeDescription>,

    topology: vk::PrimitiveTopology,
    primitive_restart_enable: bool,

    viewport: Option<vk::Viewport>,
    scissor: Option<vk::Rect2D>,
    dynamic_states: Vec<vk::DynamicState>,

    polygon_mode: vk::PolygonMode,
    cull_mode: vk::CullModeFlags,
    front_face: vk::FrontFace,
    line_width: f32,
    depth_clamp_enable: bool,

    rasterization_samples: vk::SampleCountFlags,
    sample_shading_enable: bool,

    depth_test_enable: bool,
    depth_write_enable: bool,
    depth_compare_op: vk::CompareOp,
    depth_bounds_test_enable: bool,
    stencil_test_enable: bool,

    color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
    logic_op_enable: bool,
    logic_op: vk::LogicOp,
    blend_constants: [f32; 4],
}

impl VulkanPipelineBuilder {
    pub fn new(device: &VulkanDevice) -> Self {
        Self {
            device: device.device.clone(),
            render_pass: None,
            extent: None,
            shader_entries: Vec::new(),
            descriptor_set_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
            vertex_input_bindings: Vec::new(),
            vertex_input_attributes: Vec::new(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: false,
            viewport: None,
            scissor: None,
            dynamic_states: vec![],
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            line_width: 1.0,
            depth_clamp_enable: false,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: false,
            depth_test_enable: false,
            depth_write_enable: false,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: false,
            stencil_test_enable: false,
            color_blend_attachments: Vec::new(),
            logic_op_enable: false,
            logic_op: vk::LogicOp::COPY,
            blend_constants: [0.0; 4],
        }
    }

    pub fn set_render_pass(mut self, render_pass: vk::RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }

    pub fn set_extent(mut self, extent: vk::Extent2D) -> Self {
        self.extent = Some(extent);
        self
    }

    pub fn with_descriptor_set_layout(mut self, layout: vk::DescriptorSetLayout) -> Self {
        self.descriptor_set_layouts.push(layout);
        self
    }

    pub fn with_push_constant_range(mut self, range: vk::PushConstantRange) -> Self {
        self.push_constant_ranges.push(range);
        self
    }

    pub fn with_shader_spv(
        mut self,
        code: &[u8],
        stage: vk::ShaderStageFlags,
        entry_point: Option<&CString>,
    ) -> Result<Self> {
        let module = self.create_shader_module(code)?;
        let name_cstr = match entry_point {
            Some(c) => c.to_owned(),
            None => CString::new("main").unwrap(),
        };
        self.shader_entries.push((module, stage, name_cstr));
        Ok(self)
    }

    pub fn with_vertex_spv(self, code: &[u8]) -> Result<Self> {
        self.with_shader_spv(code, vk::ShaderStageFlags::VERTEX, None)
    }

    pub fn with_fragment_spv(self, code: &[u8]) -> Result<Self> {
        self.with_shader_spv(code, vk::ShaderStageFlags::FRAGMENT, None)
    }

    pub fn with_vertex_binding(mut self, binding: vk::VertexInputBindingDescription) -> Self {
        self.vertex_input_bindings.push(binding);
        self
    }

    pub fn with_vertex_attribute(mut self, attribute: vk::VertexInputAttributeDescription) -> Self {
        self.vertex_input_attributes.push(attribute);
        self
    }

    pub fn with_topology(mut self, topology: vk::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    pub fn with_primitive_restart(mut self, enable: bool) -> Self {
        self.primitive_restart_enable = enable;
        self
    }

    pub fn with_viewport(mut self, viewport: vk::Viewport) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub fn with_scissor(mut self, scissor: vk::Rect2D) -> Self {
        self.scissor = Some(scissor);
        self
    }

    pub fn with_dynamic_states(mut self, states: &[vk::DynamicState]) -> Self {
        self.dynamic_states.extend_from_slice(states);
        self
    }

    pub fn with_polygon_mode(mut self, mode: vk::PolygonMode) -> Self {
        self.polygon_mode = mode;
        self
    }

    pub fn with_cull_mode(mut self, mode: vk::CullModeFlags) -> Self {
        self.cull_mode = mode;
        self
    }

    pub fn with_front_face(mut self, face: vk::FrontFace) -> Self {
        self.front_face = face;
        self
    }

    pub fn with_line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    pub fn with_depth_clamp(mut self, enable: bool) -> Self {
        self.depth_clamp_enable = enable;
        self
    }

    pub fn with_multisampling(mut self, samples: vk::SampleCountFlags) -> Self {
        self.rasterization_samples = samples;
        self
    }

    pub fn with_sample_shading(mut self, enable: bool) -> Self {
        self.sample_shading_enable = enable;
        self
    }

    pub fn with_depth_test(
        mut self,
        enable: bool,
        write_enable: bool,
        compare_op: vk::CompareOp,
    ) -> Self {
        self.depth_test_enable = enable;
        self.depth_write_enable = write_enable;
        self.depth_compare_op = compare_op;
        self
    }

    pub fn with_depth_bounds_test(mut self, enable: bool) -> Self {
        self.depth_bounds_test_enable = enable;
        self
    }

    pub fn with_stencil_test(mut self, enable: bool) -> Self {
        self.stencil_test_enable = enable;
        self
    }

    pub fn with_color_blend_attachment(
        mut self,
        attachment: vk::PipelineColorBlendAttachmentState,
    ) -> Self {
        self.color_blend_attachments.push(attachment);
        self
    }

    pub fn with_alpha_blending(mut self) -> Self {
        let attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
        self.color_blend_attachments.push(attachment);
        self
    }

    pub fn with_logic_op(mut self, enable: bool, op: vk::LogicOp) -> Self {
        self.logic_op_enable = enable;
        self.logic_op = op;
        self
    }

    pub fn with_blend_constants(mut self, constants: [f32; 4]) -> Self {
        self.blend_constants = constants;
        self
    }

    pub fn build(self) -> Result<VulkanPipeline> {
        let render_pass = match self.render_pass {
            Some(rp) => rp,
            None => bail!("render_pass is required"),
        };
        let extent = match self.extent {
            Some(e) => e,
            None => bail!("extent is required"),
        };

        if self.shader_entries.is_empty() {
            bail!("at least one shader stage is required")
        }

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&self.descriptor_set_layouts)
            .push_constant_ranges(&self.push_constant_ranges);
        let layout = unsafe { self.device.create_pipeline_layout(&layout_info, None)? };

        let stage_infos: Vec<vk::PipelineShaderStageCreateInfo> = self
            .shader_entries
            .iter()
            .map(|(module, stage, name)| {
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(*stage)
                    .module(*module)
                    .name(name.as_c_str())
            })
            .collect();

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&self.vertex_input_bindings)
            .vertex_attribute_descriptions(&self.vertex_input_attributes);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(self.topology)
            .primitive_restart_enable(self.primitive_restart_enable);

        let (viewport, scissor) = (
            self.viewport.unwrap_or_else(|| {
                vk::Viewport::default()
                    .x(0.0)
                    .y(0.0)
                    .width(extent.width as f32)
                    .height(extent.height as f32)
                    .min_depth(0.0)
                    .max_depth(1.0)
            }),
            self.scissor.unwrap_or_else(|| {
                vk::Rect2D::default()
                    .offset(vk::Offset2D { x: 0, y: 0 })
                    .extent(extent)
            }),
        );
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(self.depth_clamp_enable)
            .rasterizer_discard_enable(false)
            .polygon_mode(self.polygon_mode)
            .line_width(self.line_width)
            .cull_mode(self.cull_mode)
            .front_face(self.front_face)
            .depth_bias_enable(false);

        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(self.rasterization_samples)
            .sample_shading_enable(self.sample_shading_enable);

        let depth_stencil = if self.depth_test_enable
            || self.depth_bounds_test_enable
            || self.stencil_test_enable
        {
            Some(
                vk::PipelineDepthStencilStateCreateInfo::default()
                    .depth_test_enable(self.depth_test_enable)
                    .depth_write_enable(self.depth_write_enable)
                    .depth_compare_op(self.depth_compare_op)
                    .depth_bounds_test_enable(self.depth_bounds_test_enable)
                    .stencil_test_enable(self.stencil_test_enable),
            )
        } else {
            None
        };

        let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(self.logic_op_enable)
            .logic_op(self.logic_op)
            .attachments(&self.color_blend_attachments)
            .blend_constants(self.blend_constants);

        let dynamic_state_info = if self.dynamic_states.is_empty() {
            None
        } else {
            Some(vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&self.dynamic_states))
        };

        let mut pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stage_infos)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .color_blend_state(&color_blend)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0);
        if let Some(ds) = &depth_stencil {
            pipeline_info = pipeline_info.depth_stencil_state(ds);
        }
        if let Some(ds) = &dynamic_state_info {
            pipeline_info = pipeline_info.dynamic_state(ds);
        }

        let pipeline = unsafe {
            self.device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            )
        }
        .map_err(|(_, e)| e)?[0];

        for (module, _, _) in self.shader_entries {
            unsafe { self.device.destroy_shader_module(module, None) };
        }

        Ok(VulkanPipeline {
            pipeline,
            layout,
            device: self.device,
        })
    }

    fn create_shader_module(&self, code: &[u8]) -> Result<vk::ShaderModule> {
        let words =
            unsafe { std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / 4) };
        let info = vk::ShaderModuleCreateInfo::default().code(words);
        let module = unsafe { self.device.create_shader_module(&info, None)? };
        Ok(module)
    }
}

impl VulkanPipeline {
    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
        }
    }
}

impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
