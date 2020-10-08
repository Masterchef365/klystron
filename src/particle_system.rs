use crate::core::VkPrelude;
use crate::mesh::Mesh;
use crate::vertex::Vertex;
use anyhow::Result;
use erupt::{
    utils,
    utils::allocator::{self, Allocation, Allocator},
    vk1_0 as vk,
};
use std::ffi::CString;
use std::sync::Arc;

// TODO: This leaks GPU memory!!
pub struct ParticleSet {
    pub mesh: Mesh,
    pub particles: Allocation<vk::Buffer>,
    pub descriptor_set: vk::DescriptorSet,
    pub n_particles: usize,
}

pub struct ParticleSystem {
    pub forces_pipeline: vk::Pipeline,
    pub motion_pipeline: vk::Pipeline,
    prelude: Arc<VkPrelude>,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub charge: f32,
}

unsafe impl bytemuck::Zeroable for Particle {}
unsafe impl bytemuck::Pod for Particle {}

impl ParticleSystem {
    pub fn new(
        prelude: Arc<VkPrelude>,
        forces_shader: &[u8],
        motion_shader: &[u8],
        pipeline_layout: vk::PipelineLayout,
    ) -> Result<Self> {
        // Create forces_shader modules
        let forces_shader_decoded = utils::decode_spv(forces_shader)?;
        let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&forces_shader_decoded);
        let forces_shader_module = unsafe {
            prelude
                .device
                .create_shader_module(&create_info, None, None)
        }
        .result()?;

        let motion_shader_decoded = utils::decode_spv(motion_shader)?;
        let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&motion_shader_decoded);
        let motion_shader_module = unsafe {
            prelude
                .device
                .create_shader_module(&create_info, None, None)
        }
        .result()?;

        // Create pipelines
        let entry_point = CString::new("main")?;

        let forces_stage = vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::COMPUTE)
            .module(forces_shader_module)
            .name(&entry_point)
            .build();

        let motion_stage = vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::COMPUTE)
            .module(motion_shader_module)
            .name(&entry_point)
            .build();

        let pipeline_create_infos = [
            vk::ComputePipelineCreateInfoBuilder::new()
                .stage(forces_stage)
                .layout(pipeline_layout),
            vk::ComputePipelineCreateInfoBuilder::new()
                .stage(motion_stage)
                .layout(pipeline_layout),
        ];

        let mut pipelines = unsafe {
            prelude
                .device
                .create_compute_pipelines(None, &pipeline_create_infos, None)
        }
        .result()?.into_iter();

        let forces_pipeline = pipelines.next().unwrap();
        let motion_pipeline = pipelines.next().unwrap();

        unsafe {
            prelude
                .device
                .destroy_shader_module(Some(forces_shader_module), None);
            prelude
                .device
                .destroy_shader_module(Some(motion_shader_module), None);
        }

        Ok(Self { forces_pipeline, motion_pipeline, prelude })
    }
}

impl Drop for ParticleSystem {
    fn drop(&mut self) {
        unsafe {
            self.prelude
                .device
                .destroy_pipeline(Some(self.motion_pipeline), None);
            self.prelude
                .device
                .destroy_pipeline(Some(self.forces_pipeline), None);
        }
    }
}

impl ParticleSet {
    pub fn new(
        prelude: Arc<VkPrelude>,
        allocator: &mut Allocator,
        particles: &[Particle],
        particle_descriptor_set_layout: vk::DescriptorSetLayout,
        descriptor_pool: vk::DescriptorPool,
    ) -> Result<Self> {
        // Allocate SSBO
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(std::mem::size_of_val(particles) as u64);
        let buffer = unsafe { prelude.device.create_buffer(&create_info, None, None) }.result()?;
        let particle_buffer = allocator
            .allocate(
                &prelude.device,
                buffer,
                allocator::MemoryTypeFinder::upload(),
            )
            .result()?;
        let mut map = particle_buffer.map(&prelude.device, ..).result()?;
        map.import(bytemuck::cast_slice(particles));
        map.unmap(&prelude.device).result()?;

        let indices = (0..particles.len() as u16).collect::<Vec<_>>();
        let vertices = vec![Vertex::default(); particles.len()];

        // Allocate mesh
        let mesh = Mesh::new(&prelude.device, allocator, &vertices, &indices)?;

        // Allocate a new descriptor set from the pool
        let layouts = [particle_descriptor_set_layout];
        let create_info = vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_set =
            unsafe { prelude.device.allocate_descriptor_sets(&create_info) }.result()?[0];

        // Update it with the buffers in this structure
        let vertex_buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(*mesh.vertices.object())
            .offset(0)
            .range(vk::WHOLE_SIZE)];

        let particle_buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(*particle_buffer.object())
            .offset(0)
            .range(vk::WHOLE_SIZE)];

        let writes = [
            vk::WriteDescriptorSetBuilder::new()
                .buffer_info(&vertex_buffer_infos)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0),
            vk::WriteDescriptorSetBuilder::new()
                .buffer_info(&particle_buffer_infos)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .dst_set(descriptor_set)
                .dst_binding(1)
                .dst_array_element(0),
        ];

        unsafe {
            prelude.device.update_descriptor_sets(&writes, &[]);
        }

        Ok(Self {
            mesh,
            n_particles: particles.len(),
            descriptor_set,
            particles: particle_buffer,
        })
    }
}

/*
impl Drop for ParticleSet {
    fn drop(&mut self) {
        unsafe {
            self.prelude.device.destroy_desc
        }
    }
}
*/
