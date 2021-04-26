use bytemuck::offset_of;
use erupt::vk1_0 as vk;
use nalgebra::Point3;

/// Vertex suitable for use from vertex shaders
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    pub fn new(pos: [f32; 3], color: [f32; 3]) -> Self {
        Self { pos, color }
    }
    pub fn from_nalgebra(pos: Point3<f32>, color: Point3<f32>) -> Self {
        Self {
            pos: *pos.coords.as_ref(),
            color: *color.coords.as_ref(),
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescriptionBuilder<'static> {
        vk::VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescriptionBuilder<'static>; 2]
    {
        [
            vk::VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32),
            vk::VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, color) as u32),
        ]
    }
}
