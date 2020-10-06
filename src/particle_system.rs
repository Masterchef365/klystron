use crate::core::VkPrelude;
use crate::mesh::Mesh;
use crate::vertex::Vertex;
use anyhow::Result;
use erupt::{
    utils::allocator::{self, Allocation, Allocator},
    vk1_0 as vk, DeviceLoader,
};
use std::ffi::CString;
use std::sync::Arc;

pub struct ParticleSet {
    pub mesh: Mesh,
    pub particles: Allocation<vk::Buffer>,
}

pub struct ParticleSystem {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
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

impl ParticleSet {
    pub fn new(
        device: &DeviceLoader,
        allocator: &mut Allocator,
        particles: &[Particle],
    ) -> Result<Self> {
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(std::mem::size_of_val(particles) as u64);
        let buffer = unsafe { device.create_buffer(&create_info, None, None) }.result()?;
        let particle_buffer = allocator
            .allocate(device, buffer, allocator::MemoryTypeFinder::upload())
            .result()?;
        let mut map = particle_buffer.map(device, ..).result()?;
        map.import(bytemuck::cast_slice(particles));
        map.unmap(device).result()?;

        let indices = (0..particles.len() as u16).collect::<Vec<_>>();
        let vertices = vec![Vertex::default(); particles.len()];
        let mesh = Mesh::new(device, allocator, &vertices, &indices)?;

        Ok(Self {
            mesh,
            particles: particle_buffer,
        })
    }
}
