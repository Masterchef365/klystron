use std::sync::Arc;
use anyhow::Result;
use crate::allocated_buffer::AllocatedBuffer;
use crate::frame_sync::FrameSync;
use crate::handle::HandleMap;
use crate::swapchain_images::SwapchainImages;
use crate::material::Material;
use erupt::{
    utils::{
        self,
        allocator::{self, Allocator},
    },
    vk1_0 as vk, DeviceLoader, InstanceLoader,
    vk1_1,
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
pub(crate) const COLOR_FORMAT: vk::Format = vk::Format::B8G8R8A8_SRGB;
pub(crate) const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

pub type CameraUbo = [f32; 32];
pub struct Mesh;

pub struct Core {
    pub allocator: Allocator,
    pub materials: HandleMap<Material>,
    pub meshes: HandleMap<Mesh>,
    pub render_pass: vk::RenderPass,
    pub frame_sync: FrameSync,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub camera_ubos: Vec<AllocatedBuffer<CameraUbo>>,
    pub swapchain_images: Option<SwapchainImages>,
    pub prelude: Arc<VkPrelude>,
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

        // Render pass
        let color_attachment = vk::AttachmentDescriptionBuilder::new()
            .format(COLOR_FORMAT)
            .samples(vk::SampleCountFlagBits::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_attachment = vk::AttachmentDescriptionBuilder::new()
            .format(DEPTH_FORMAT)
            .samples(vk::SampleCountFlagBits::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let attachments = [color_attachment, depth_attachment];

        let color_attachment_refs = [vk::AttachmentReferenceBuilder::new()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let depth_attachment_ref = vk::AttachmentReferenceBuilder::new()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpasses = [vk::SubpassDescriptionBuilder::new()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)];

        let dependencies = [vk::SubpassDependencyBuilder::new()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let mut create_info = vk::RenderPassCreateInfoBuilder::new()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let view_mask = [!(!0 << 2)];
        let mut multiview = vk1_1::RenderPassMultiviewCreateInfoBuilder::new()
            .view_masks(&view_mask)
            .correlation_masks(&view_mask)
            .build();

        create_info.p_next = &mut multiview as *mut _ as _;

        let render_pass =
            unsafe { prelude.device.create_render_pass(&create_info, None, None) }.result()?;

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
            render_pass,
            swapchain_images: None,
            materials: Default::default(),
            meshes: Default::default(),
        })
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            // TODO: Drop materials and meshes
            for ubo in &mut self.camera_ubos {
                ubo.free(&self.prelude.device, &mut self.allocator).unwrap();
            }
            self.frame_sync.free(&self.prelude.device);
            self.prelude.device.destroy_render_pass(Some(self.render_pass), None);
            self.prelude.device.destroy_descriptor_set_layout(Some(self.descriptor_set_layout), None);
            self.prelude.device.destroy_descriptor_pool(Some(self.descriptor_pool), None);
            self.prelude.device.free_command_buffers(self.command_pool, &self.command_buffers);
            self.prelude.device.destroy_command_pool(Some(self.command_pool), None);
        }
    }
}

impl Drop for VkPrelude {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
