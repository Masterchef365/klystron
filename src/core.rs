use crate::allocated_buffer::AllocatedBuffer;
use crate::frame_sync::FrameSync;
use crate::handle::HandleMap;
use crate::material::Material;
use crate::swapchain_images::{SwapChainImage, SwapchainImages};
use crate::vertex::Vertex;
use anyhow::Result;
use erupt::{
    utils::{
        self,
        allocator::{self, Allocator},
    },
    vk1_0 as vk, vk1_1, DeviceLoader, InstanceLoader,
};
use std::sync::Arc;

pub struct VkPrelude {
    pub queue: vk::Queue,
    pub queue_family_index: u32,
    pub device: DeviceLoader,
    pub physical_device: vk::PhysicalDevice,
    pub instance: InstanceLoader,
    pub entry: utils::loading::DefaultEntryLoader,
}

pub(crate) const FRAMES_IN_FLIGHT: usize = 2;
pub(crate) const COLOR_FORMAT: vk::Format = vk::Format::B8G8R8A8_SRGB;
pub(crate) const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

pub type CameraUbo = [f32; 32];

pub struct Mesh {
    pub indices: AllocatedBuffer<u16>,
    pub vertices: AllocatedBuffer<Vertex>,
    pub n_indices: u32,
}

pub struct Core {
    pub allocator: Allocator,
    pub materials: HandleMap<Material>,
    pub meshes: HandleMap<Mesh>,
    pub render_pass: vk::RenderPass,
    pub frame_sync: FrameSync,
    pub swapchain_images: Option<SwapchainImages>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub camera_ubos: Vec<AllocatedBuffer<CameraUbo>>,
    pub prelude: Arc<VkPrelude>,
}

impl Core {
    pub fn new(prelude: Arc<VkPrelude>, vr: bool) -> Result<Self> {
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
            prelude
                .device
                .create_descriptor_set_layout(&descriptor_set_layout_ci, None, None)
        }
        .result()?;

        // Create descriptor pool
        let pool_sizes = [vk::DescriptorPoolSizeBuilder::new()
            ._type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(FRAMES_IN_FLIGHT as u32)];
        let create_info = vk::DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(&pool_sizes)
            .max_sets(FRAMES_IN_FLIGHT as u32);
        let descriptor_pool = unsafe {
            prelude
                .device
                .create_descriptor_pool(&create_info, None, None)
        }
        .result()?;

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
        let frame_sync = FrameSync::new(prelude.clone(), FRAMES_IN_FLIGHT)?;

        // Render pass
        let color_attachment = vk::AttachmentDescriptionBuilder::new()
            .format(COLOR_FORMAT)
            .samples(vk::SampleCountFlagBits::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(if vr {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            } else {
                vk::ImageLayout::PRESENT_SRC_KHR
            });

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

        let views = if vr { 2 } else { 1 };
        let view_mask = [!(!0 << views)];
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

    pub fn add_material(
        &mut self,
        vertex: &[u8],
        fragment: &[u8],
        draw_type: crate::DrawType,
    ) -> Result<crate::Material> {
        let material = Material::new(
            self.prelude.clone(),
            vertex,
            fragment,
            draw_type,
            self.render_pass,
            self.descriptor_set_layout,
        )?;
        Ok(crate::Material(self.materials.insert(material)))
    }

    pub fn remove_material(&mut self, material: crate::Material) -> Result<()> {
        // Figure out how not to wait?
        unsafe {
            self.prelude.device.device_wait_idle().result()?;
        }
        self.materials.remove(&material.0);
        Ok(())
    }

    pub fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<crate::Mesh> {
        let n_indices = indices.len() as u32;

        //TODO: Use staging buffers as well!
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let vertex_buffer = AllocatedBuffer::new(
            vertices.len(),
            create_info,
            &mut self.allocator,
            &self.prelude.device,
        )?;
        vertex_buffer.map(&self.prelude.device, vertices)?;

        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let index_buffer = AllocatedBuffer::new(
            indices.len(),
            create_info,
            &mut self.allocator,
            &self.prelude.device,
        )?;
        index_buffer.map(&self.prelude.device, indices)?;

        let mesh = Mesh {
            indices: index_buffer,
            vertices: vertex_buffer,
            n_indices,
        };

        Ok(crate::Mesh(self.meshes.insert(mesh)))
    }

    pub fn remove_mesh(&mut self, id: crate::Mesh) -> Result<()> {
        // Figure out how not to wait?
        unsafe {
            self.prelude.device.device_wait_idle().result()?;
        }
        if let Some(mut mesh) = self.meshes.remove(&id.0) {
            mesh.vertices
                .free(&self.prelude.device, &mut self.allocator)?;
            mesh.indices
                .free(&self.prelude.device, &mut self.allocator)?;
        }
        Ok(())
    }

    pub fn write_command_buffers(
        &self,
        frame_idx: usize,
        packet: &crate::FramePacket,
        image: &SwapChainImage,
    ) -> Result<vk::CommandBuffer> {
        // Reset and write command buffers for this frame
        let command_buffer = self.command_buffers[frame_idx];
        let descriptor_set = self.descriptor_sets[frame_idx];
        unsafe {
            self.prelude
                .device
                .reset_command_buffer(command_buffer, None)
                .result()?;

            let begin_info = vk::CommandBufferBeginInfoBuilder::new();
            self.prelude
                .device
                .begin_command_buffer(command_buffer, &begin_info)
                .result()?;

            // Set render pass
            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let begin_info = vk::RenderPassBeginInfoBuilder::new()
                .framebuffer(image.framebuffer)
                .render_pass(self.render_pass)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: image.extent,
                })
                .clear_values(&clear_values);

            self.prelude.device.cmd_begin_render_pass(
                command_buffer,
                &begin_info,
                vk::SubpassContents::INLINE,
            );

            let viewports = [vk::ViewportBuilder::new()
                .x(0.0)
                .y(0.0)
                .width(image.extent.width as f32)
                .height(image.extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)];

            let scissors = [vk::Rect2DBuilder::new()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(image.extent)];

            for (material_id, material) in self.materials.iter() {
                self.prelude.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    material.pipeline,
                );

                self.prelude
                    .device
                    .cmd_set_viewport(command_buffer, 0, &viewports);

                self.prelude
                    .device
                    .cmd_set_scissor(command_buffer, 0, &scissors);

                self.prelude.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    material.pipeline_layout,
                    0,
                    &[descriptor_set],
                    &[],
                );

                for object in packet
                    .objects
                    .iter()
                    .filter(|o| o.material.0 == *material_id)
                {
                    let mesh = match self.meshes.get(&object.mesh.0) {
                        Some(m) => m,
                        None => {
                            log::error!("Object references a mesh that no exists");
                            continue;
                        }
                    };
                    self.prelude.device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[mesh.vertices.buffer],
                        &[0],
                    );

                    self.prelude.device.cmd_bind_index_buffer(
                        command_buffer,
                        mesh.indices.buffer,
                        0,
                        vk::IndexType::UINT16,
                    );

                    let descriptor_sets = [self.descriptor_sets[frame_idx]];
                    self.prelude.device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        material.pipeline_layout,
                        0,
                        &descriptor_sets,
                        &[],
                    );

                    // TODO: ADD ANIM
                    self.prelude.device.cmd_push_constants(
                        command_buffer,
                        material.pipeline_layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        std::mem::size_of::<[f32; 16]>() as u32,
                        object.transform.data.as_ptr() as _,
                    );

                    self.prelude.device.cmd_draw_indexed(
                        command_buffer,
                        mesh.n_indices,
                        1,
                        0,
                        0,
                        0,
                    );
                }
            }

            self.prelude.device.cmd_end_render_pass(command_buffer);

            self.prelude
                .device
                .end_command_buffer(command_buffer)
                .result()?;
        }

        Ok(command_buffer)
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            self.prelude.device.device_wait_idle().unwrap();
            for (_, mesh) in self.meshes.iter_mut() {
                mesh.indices
                    .free(&self.prelude.device, &mut self.allocator)
                    .unwrap();
                mesh.vertices
                    .free(&self.prelude.device, &mut self.allocator)
                    .unwrap();
            }
            for ubo in &mut self.camera_ubos {
                ubo.free(&self.prelude.device, &mut self.allocator).unwrap();
            }
            self.prelude
                .device
                .destroy_render_pass(Some(self.render_pass), None);
            self.prelude
                .device
                .destroy_descriptor_set_layout(Some(self.descriptor_set_layout), None);
            self.prelude
                .device
                .destroy_descriptor_pool(Some(self.descriptor_pool), None);
            self.prelude
                .device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            self.prelude
                .device
                .destroy_command_pool(Some(self.command_pool), None);
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
