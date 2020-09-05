use std::sync::Arc;
use anyhow::Result;
use crate::allocated_buffer::AllocatedBuffer;
use crate::frame_sync::FrameSync;
use crate::handle::HandleMap;
use erupt::{
    utils::{
        self,
        allocator::{self, Allocator},
    },
    vk1_0 as vk, DeviceLoader, InstanceLoader,
};

pub struct VkPrelude {
    pub queue: vk::Queue,
    pub queue_family_index: u32,
    pub device: DeviceLoader,
    pub physical_device: vk::PhysicalDevice,
    pub instance: InstanceLoader,
    pub entry: utils::loading::DefaultEntryLoader,
}

const FRAMES_IN_FLIGHT: usize = 2;

pub type CameraUbo = [f32; 32];
pub struct Mesh;
pub struct Material;

pub struct Core {
    pub prelude: Arc<VkPrelude>,
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

impl Core {
    pub fn new(prelude: Arc<VkPrelude>) -> Result<Self> {
        // Command pool
        let create_info = vk::CommandPoolCreateInfoBuilder::new()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(prelude.queue_family_index);
        let command_pool =
            unsafe { prelude.device.create_command_pool(&create_info, None, None) }.result()?;

        // Allocate command buffers
        let allocate_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(FRAMES_IN_FLIGHT as u32);

        let command_buffers =
            unsafe { prelude.device.allocate_command_buffers(&allocate_info) }.result()?;

        // Device memory allocator
        let mut allocator = Allocator::new(
            &prelude.instance,
            prelude.physical_device,
            allocator::AllocatorCreateInfo::default(),
        )
        .result()?;

        // Create descriptor layout
        let bindings = [vk::DescriptorSetLayoutBindingBuilder::new()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];

        let descriptor_set_layout_ci =
            vk::DescriptorSetLayoutCreateInfoBuilder::new().bindings(&bindings);

        let descriptor_set_layout = unsafe {
            prelude.device.create_descriptor_set_layout(&descriptor_set_layout_ci, None, None)
        }
        .result()?;

        // Create descriptor pool
        let pool_sizes = [vk::DescriptorPoolSizeBuilder::new()
            ._type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(FRAMES_IN_FLIGHT as u32)];
        let create_info = vk::DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(&pool_sizes)
            .max_sets(FRAMES_IN_FLIGHT as u32);
        let descriptor_pool =
            unsafe { prelude.device.create_descriptor_pool(&create_info, None, None) }.result()?;

        // Create descriptor sets
        let layouts = vec![descriptor_set_layout; FRAMES_IN_FLIGHT];
        let create_info = vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets =
            unsafe { prelude.device.allocate_descriptor_sets(&create_info) }.result()?;

        // Camera's UBOs
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let camera_ubos = (0..FRAMES_IN_FLIGHT)
            .map(|_| AllocatedBuffer::new(1, create_info.clone(), &mut allocator, &prelude.device))
            .collect::<Result<Vec<_>>>()?;

        // Bind buffers to descriptors
        for (alloc, descriptor) in camera_ubos.iter().zip(descriptor_sets.iter()) {
            let buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
                .buffer(alloc.buffer)
                .offset(0)
                .range(std::mem::size_of::<CameraUbo>() as u64)];

            let writes = [vk::WriteDescriptorSetBuilder::new()
                .buffer_info(&buffer_infos)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_set(*descriptor)
                .dst_binding(0)
                .dst_array_element(0)];

            unsafe {
                prelude.device.update_descriptor_sets(&writes, &[]);
            }
        }

        // Frame synchronization
        let frame_sync = FrameSync::new(&prelude.device, FRAMES_IN_FLIGHT)?;

        Ok(Self {
            prelude,
            camera_ubos,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
            command_pool,
            frame_sync,
            allocator,
            command_buffers,
            materials: Default::default(),
            objects: Default::default(),
        })
    }
}
