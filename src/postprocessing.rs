use crate::core::VkPrelude;
use anyhow::Result;
use erupt::{utils, vk1_0 as vk};
use std::ffi::CString;
use std::sync::Arc;

/// Represents a shader pipeline for color rendering
pub struct PostProcessing {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    prelude: Arc<VkPrelude>,
}

impl PostProcessing {
    pub fn new(
        prelude: Arc<VkPrelude>,
        spirv: &[u8],
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<Self> {
        // Create shader modules
        let compute_decoded = utils::decode_spv(spirv)?;
        let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&compute_decoded);
        let compute = unsafe {
            prelude
                .device
                .create_shader_module(&create_info, None, None)
        }
        .result()?;

        let entry_point = CString::new("main")?;

        let stage = 
            vk::PipelineShaderStageCreateInfoBuilder::new()
                .stage(vk::ShaderStageFlagBits::COMPUTE)
                .module(compute)
                .name(&entry_point)
                .build()
        ;

        let descriptor_set_layouts = [descriptor_set_layout];

        let create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .push_constant_ranges(&[])
            .set_layouts(&descriptor_set_layouts);

        let pipeline_layout = unsafe {
            prelude
                .device
                .create_pipeline_layout(&create_info, None, None)
        }
        .result()?;

        let create_info = vk::ComputePipelineCreateInfoBuilder::new()
            .stage(stage)
            .layout(pipeline_layout);
        let pipeline =
            unsafe { prelude.device.create_compute_pipelines(None, &[create_info], None) }.result()?[0];

        unsafe {
            prelude.device.destroy_shader_module(Some(compute), None);
        }

        Ok(Self {
            pipeline,
            pipeline_layout,
            prelude,
        })
    }
}

impl Drop for PostProcessing {
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
