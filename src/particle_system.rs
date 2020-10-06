use crate::core::VkPrelude;
use crate::mesh::Mesh;
use crate::vertex::Vertex;
use anyhow::Result;
use erupt::{
    utils::allocator::{self, Allocation, Allocator},
    utils,
    vk1_0 as vk, DeviceLoader,
};
use std::ffi::CString;
use std::sync::Arc;

pub struct ParticleSet {
    pub mesh: Mesh,
    pub particles: Allocation<vk::Buffer>,
    pub descriptor_set: vk::DescriptorSet,
    prelude: Arc<VkPrelude>,
}

pub struct ParticleSystem {
    pub pipeline: vk::Pipeline,
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
    pub fn new(prelude: Arc<VkPrelude>, shader: &[u8], pipeline_layout: vk::PipelineLayout) -> Result<Self> {
        // Create shader modules
        let shader_decoded = utils::decode_spv(shader)?;
        let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&shader_decoded);
        let shader_module = unsafe {
                prelude.device
                .create_shader_module(&create_info, None, None)
        }
        .result()?;

        // Create pipeline
        let entry_point = CString::new("main")?;
        let stage = vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::COMPUTE)
            .module(shader_module)
            .name(&entry_point)
            .build();
        let create_info = vk::ComputePipelineCreateInfoBuilder::new()
            .stage(stage)
            .layout(pipeline_layout);
        let pipeline =
            unsafe { prelude.device.create_compute_pipelines(None, &[create_info], None) }.result()?[0];

        Ok(Self {
            pipeline,
            prelude,
        })
    }
}

impl Drop for ParticleSystem {
    fn drop(&mut self) {
        unsafe {
            self.prelude.device.destroy_pipeline(Some(self.pipeline), None);
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
        let camera_buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(*mesh.vertices.object())
            .offset(0)
            .range(std::mem::size_of::<Particle>() as u64)];

        let animation_buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(*particle_buffer.object())
            .offset(0)
            .range(std::mem::size_of::<f32>() as u64)];

        let writes = [
            vk::WriteDescriptorSetBuilder::new()
                .buffer_info(&camera_buffer_infos)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0),
            vk::WriteDescriptorSetBuilder::new()
                .buffer_info(&animation_buffer_infos)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_set(descriptor_set)
                .dst_binding(1)
                .dst_array_element(0),
        ];

        unsafe {
            prelude.device.update_descriptor_sets(&writes, &[]);
        }

        Ok(Self {
            mesh,
            descriptor_set,
            particles: particle_buffer,
            prelude,
        })
    }
}
