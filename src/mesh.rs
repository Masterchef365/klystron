use anyhow::Result;
use crate::Vertex;
use erupt::{
    DeviceLoader,
    utils::allocator::{self, Allocation, Allocator},
    vk1_0 as vk,
};

// TODO: This leaks GPU memory!!

pub struct Mesh {
    pub indices: Allocation<vk::Buffer>,
    pub vertices: Allocation<vk::Buffer>,
    pub n_indices: u32,
}

impl Mesh {
    pub fn new(
        device: &DeviceLoader,
        allocator: &mut Allocator,
        vertices: &[Vertex],
        indices: &[u16],
    ) -> Result<Self> {
        let n_indices = indices.len() as u32;

        //TODO: Use staging buffers!
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(std::mem::size_of_val(vertices) as u64);
        let buffer =
            unsafe { device.create_buffer(&create_info, None, None) }.result()?;
        let vertex_buffer = allocator
            .allocate(device, buffer, allocator::MemoryTypeFinder::upload())
            .result()?;
        let mut map = vertex_buffer.map(device, ..).result()?;
        map.import(bytemuck::cast_slice(vertices));
        map.unmap(device).result()?;

        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(std::mem::size_of_val(indices) as u64);
        let buffer = unsafe { device.create_buffer(&create_info, None, None) }.result()?;
        let index_buffer = allocator
            .allocate(device, buffer, allocator::MemoryTypeFinder::upload())
            .result()?;
        let mut map = index_buffer.map(device, ..).result()?;
        map.import(bytemuck::cast_slice(indices));
        map.unmap(device).result()?;

        Ok(Self {
            indices: index_buffer,
            vertices: vertex_buffer,
            n_indices,
        })
    }
}
