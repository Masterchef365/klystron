use crate::core::VkPrelude;
use crate::vertex::Vertex;
use crate::DrawType;
use anyhow::Result;
use erupt::{utils, vk1_0 as vk};
use std::ffi::CString;
use std::sync::Arc;

/// Represents a shader pipeline for color rendering
pub struct Material {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    prelude: Arc<VkPrelude>,
}

impl Material {
    pub fn new(
        prelude: Arc<VkPrelude>,
        vertex_src: &[u8],
        fragment_src: &[u8],
        draw_type: DrawType,
        render_pass: vk::RenderPass,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<Self> {
        // Create shader modules
        let vert_decoded = utils::decode_spv(vertex_src)?;
        let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&vert_decoded);
        let vertex = unsafe {
            prelude
                .device
                .create_shader_module(&create_info, None, None)
        }
        .result()?;

        let frag_decoded = utils::decode_spv(fragment_src)?;
        let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&frag_decoded);
        let fragment = unsafe {
            prelude
                .device
                .create_shader_module(&create_info, None, None)
        }
        .result()?;

        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let binding_descriptions = [Vertex::binding_description()];

        // Build pipeline
        let vertex_input = vk::PipelineVertexInputStateCreateInfoBuilder::new()
            .vertex_attribute_descriptions(&attribute_descriptions[..])
            .vertex_binding_descriptions(&binding_descriptions);

        let draw_type = match draw_type {
            DrawType::Triangles => vk::PrimitiveTopology::TRIANGLE_LIST,
            DrawType::Points => vk::PrimitiveTopology::POINT_LIST,
            DrawType::Lines => vk::PrimitiveTopology::LINE_LIST,
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(draw_type)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
            .viewport_count(1)
            .scissor_count(1);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfoBuilder::new().dynamic_states(&dynamic_states);

        let rasterizer = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_clamp_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlagBits::_1);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentStateBuilder::new()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false)];
        let color_blending = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let entry_point = CString::new("main")?;

        let shader_stages = [
            vk::PipelineShaderStageCreateInfoBuilder::new()
                .stage(vk::ShaderStageFlagBits::VERTEX)
                .module(vertex)
                .name(&entry_point),
            vk::PipelineShaderStageCreateInfoBuilder::new()
                .stage(vk::ShaderStageFlagBits::FRAGMENT)
                .module(fragment)
                .name(&entry_point),
        ];

        let descriptor_set_layouts = [descriptor_set_layout];

        let push_constant_ranges = [vk::PushConstantRangeBuilder::new()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<[f32; 16]>() as u32)];

        let create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .push_constant_ranges(&push_constant_ranges)
            .set_layouts(&descriptor_set_layouts);

        let pipeline_layout = unsafe {
            prelude
                .device
                .create_pipeline_layout(&create_info, None, None)
        }
        .result()?;

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfoBuilder::new()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS) // TODO: Play with this! For fun!
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let create_info = vk::GraphicsPipelineCreateInfoBuilder::new()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .depth_stencil_state(&depth_stencil_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            prelude
                .device
                .create_graphics_pipelines(None, &[create_info], None)
        }
        .result()?[0];

        unsafe {
            prelude.device.destroy_shader_module(Some(fragment), None);
            prelude.device.destroy_shader_module(Some(vertex), None);
        }

        Ok(Self {
            pipeline,
            pipeline_layout,
            prelude,
        })
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        unsafe {
            self.prelude
                .device
                .destroy_pipeline(Some(self.pipeline), None);
            self.prelude
                .device
                .destroy_pipeline_layout(Some(self.pipeline_layout), None);
        }
    }
}
