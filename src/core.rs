use std::sync::Arc;
use crate::handle::HandleMap;
use erupt::{
    utils::{
        self,
        allocator::Allocator,
    },
    vk1_0 as vk, DeviceLoader, InstanceLoader,
};

pub struct Caddy {
    pub queue: vk::Queue,
    pub vk_device: DeviceLoader,
    pub vk_instance: InstanceLoader,
    _entry: utils::loading::DefaultEntryLoader,
}

pub struct Core {
    pub caddy: Arc<Caddy>,
    pub allocator: Allocator,
    pub materials: HandleMap<Material>,
    pub objects: HandleMap<Mesh>,
    pub frame_sync: FrameSync,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub camera_ubos: Vec<AllocatedBuffer<CameraUbo>>,
}

pub type CameraUbo = [f32; 32];
pub struct Mesh;
pub struct Material;
pub struct FrameSync;
pub struct AllocatedBuffer<T>(T);
