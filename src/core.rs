use crate::frame_sync::FrameSync;
use crate::material::Material;
use crate::swapchain_images::{SwapChainImage, SwapchainImages};
use crate::vertex::Vertex;
use crate::Sampling;
use crate::desc_set_allocator::DescriptorSetAllocator;
use anyhow::{Result, ensure};
use erupt::{vk1_0 as vk, vk1_1, DeviceLoader};
use genmap::GenMap;
use vk_core::SharedCore;
use gpu_alloc_erupt::EruptMemoryDevice;

pub(crate) const FRAMES_IN_FLIGHT: usize = 2;
pub(crate) const COLOR_FORMAT: vk::Format = vk::Format::B8G8R8A8_SRGB;
pub(crate) const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
const TEXTURE_FORMAT: vk::Format = vk::Format::R8G8B8A8_SRGB;

pub type CameraUbo = [f32; 32];

// TODO: yes, I know this is a bad way to do things.
pub struct AllocatedBuffer {
    buffer: vk::Buffer,
    memory: gpu_alloc::MemoryBlock<vk::DeviceMemory>,
}

pub struct AllocatedImage {
    image: vk::Image,
    memory: gpu_alloc::MemoryBlock<vk::DeviceMemory>,
}

pub struct Mesh {
    pub indices: AllocatedBuffer,
    pub vertices: AllocatedBuffer,
    pub n_indices: u32,
}

pub struct Texture {
    pub alloc: AllocatedImage,
    pub sampler: vk::Sampler,
    pub view: vk::ImageView,
    pub width: u32,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

// TODO: Turn the Vec<T>'s into [T; FRAMES_IN_FLIGHT]!
// Do this when you switch over to gpu-alloc

pub struct Core {
    pub materials: GenMap<Material>,
    pub meshes: GenMap<Mesh>,
    pub textures: GenMap<Texture>,
    pub render_pass: vk::RenderPass,
    pub frame_sync: FrameSync,
    pub swapchain_images: Option<SwapchainImages>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub transfer_cmd_buf: vk::CommandBuffer,
    pub desc_set_allocator: DescriptorSetAllocator,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub camera_ubos: Vec<AllocatedBuffer>,
    pub time_ubos: Vec<AllocatedBuffer>,
    pub prelude: SharedCore,
}

impl Core {
    pub fn new(prelude: SharedCore, core_meta: vk_core::CoreMeta, vr: bool) -> Result<Self> {
        // Command pool
        let create_info = vk::CommandPoolCreateInfoBuilder::new()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(core_meta.queue_family_index);
        let command_pool =
            unsafe { prelude.device.create_command_pool(&create_info, None, None) }.result()?;

        // Allocate command buffers
        let allocate_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(FRAMES_IN_FLIGHT as u32 + 1);

        let mut command_buffers =
            unsafe { prelude.device.allocate_command_buffers(&allocate_info) }.result()?;

        let transfer_cmd_buf = command_buffers.pop().unwrap();

        // Create descriptor layout
        let bindings = [
            vk::DescriptorSetLayoutBindingBuilder::new()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
            vk::DescriptorSetLayoutBindingBuilder::new()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBindingBuilder::new()
                .binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
        ];

        let descriptor_set_layout_ci =
            vk::DescriptorSetLayoutCreateInfoBuilder::new().bindings(&bindings);

        let descriptor_set_layout = unsafe {
            prelude
                .device
                .create_descriptor_set_layout(&descriptor_set_layout_ci, None, None)
        }
        .result()?;

        // Create descriptor pool
        let pool_sizes = vec![
            vk::DescriptorPoolSizeBuilder::new()
            ._type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count((FRAMES_IN_FLIGHT * 2) as u32),
            vk::DescriptorPoolSizeBuilder::new()
            ._type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(FRAMES_IN_FLIGHT as u32),
        ];

        let desc_set_allocator = DescriptorSetAllocator::new(pool_sizes, descriptor_set_layout, prelude.clone());

        // UBOs
        let ubo_create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(std::mem::size_of::<CameraUbo>() as u64);

        // Camera:
        let mut camera_ubos = Vec::new();
        for _ in 0..FRAMES_IN_FLIGHT {
            use gpu_alloc::UsageFlags as UF;
            let buffer =
                unsafe { prelude.device.create_buffer(&ubo_create_info, None, None) }.result()?;
            let requirements = unsafe { prelude.device.get_buffer_memory_requirements(buffer, None) };
            let request = gpu_alloc::Request {
                size: requirements.size,
                align_mask: requirements.alignment,
                usage: UF::DOWNLOAD | UF::UPLOAD | UF::HOST_ACCESS,
                memory_types: requirements.memory_type_bits,
            };
            let memory = unsafe { prelude.allocator()?
                .alloc(EruptMemoryDevice::wrap(&prelude.device), request)? };
            unsafe {
                prelude.device.bind_buffer_memory(buffer, *memory.memory(), memory.offset()).result()?;
            }
            camera_ubos.push(AllocatedBuffer {
                buffer,
                memory,
            });
        }

        // Animation
        let ubo_create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(std::mem::size_of::<f32>() as u64);

        let mut time_ubos = Vec::new();
        for _ in 0..FRAMES_IN_FLIGHT {
            use gpu_alloc::UsageFlags as UF;
            let buffer =
                unsafe { prelude.device.create_buffer(&ubo_create_info, None, None) }.result()?;
            let requirements = unsafe { prelude.device.get_buffer_memory_requirements(buffer, None) };
            let request = gpu_alloc::Request {
                size: requirements.size,
                align_mask: requirements.alignment,
                usage: UF::DOWNLOAD | UF::UPLOAD | UF::HOST_ACCESS,
                memory_types: requirements.memory_type_bits,
            };
            let memory = unsafe { prelude.allocator()?
                .alloc(EruptMemoryDevice::wrap(&prelude.device), request)? };
            unsafe {
                prelude.device.bind_buffer_memory(buffer, *memory.memory(), memory.offset()).result()?;
            }
            time_ubos.push(AllocatedBuffer {
                buffer,
                memory,
            });
        }

        /*
        // Bind buffers to descriptors
        for (animation_ubo, (camera_ubo, descriptor)) in time_ubos
            .iter()
            .zip(camera_ubos.iter().zip(descriptor_sets.iter()))
        {
            let camera_buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
                .buffer(camera_ubo.buffer)
                .offset(0)
                .range(std::mem::size_of::<CameraUbo>() as u64)];

            let animation_buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
                .buffer(animation_ubo.buffer)
                .offset(0)
                .range(std::mem::size_of::<f32>() as u64)];

            let writes = [
                vk::WriteDescriptorSetBuilder::new()
                    .buffer_info(&camera_buffer_infos)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .dst_set(*descriptor)
                    .dst_binding(0)
                    .dst_array_element(0),
                vk::WriteDescriptorSetBuilder::new()
                    .buffer_info(&animation_buffer_infos)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .dst_set(*descriptor)
                    .dst_binding(1)
                    .dst_array_element(0),
            ];

            unsafe {
                prelude.device.update_descriptor_sets(&writes, &[]);
            }
        }
        */

        // Frame synchronization
        let frame_sync = FrameSync::new(prelude.clone(), FRAMES_IN_FLIGHT)?;

        let render_pass = create_render_pass(&prelude.device, vr)?;

        Ok(Self {
            prelude,
            camera_ubos,
            time_ubos,
            desc_set_allocator,
            descriptor_set_layout,
            command_pool,
            frame_sync,
            command_buffers,
            transfer_cmd_buf,
            render_pass,
            swapchain_images: None,
            materials: GenMap::with_capacity(10),
            meshes: GenMap::with_capacity(10),
            textures: GenMap::with_capacity(10),
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
        self.materials.remove(material.0);
        Ok(())
    }

    pub fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<crate::Mesh> {
        let n_indices = indices.len() as u32;
        use gpu_alloc::UsageFlags as UF;

        // Vertex
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(std::mem::size_of_val(vertices) as u64);
        let buffer =
            unsafe { self.prelude.device.create_buffer(&create_info, None, None) }.result()?;
        let requirements = unsafe { self.prelude.device.get_buffer_memory_requirements(buffer, None) };
        let request = gpu_alloc::Request {
            size: requirements.size,
            align_mask: requirements.alignment,
            usage: UF::DOWNLOAD | UF::UPLOAD | UF::HOST_ACCESS,
            memory_types: requirements.memory_type_bits,
        };
        let memory = unsafe { self.prelude.allocator()?
            .alloc(EruptMemoryDevice::wrap(&self.prelude.device), request)? };
        unsafe {
            self.prelude.device.bind_buffer_memory(buffer, *memory.memory(), memory.offset()).result()?;
        }
        unsafe {
        memory.write_bytes(
                EruptMemoryDevice::wrap(&self.prelude.device),
                0,
                &bytemuck::cast_slice(vertices),
            )?;
        }
        let vertex_buffer = AllocatedBuffer {
            memory,
            buffer,
        };

        // Indices
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(std::mem::size_of_val(indices) as u64);
        let buffer =
            unsafe { self.prelude.device.create_buffer(&create_info, None, None) }.result()?;
        let requirements = unsafe { self.prelude.device.get_buffer_memory_requirements(buffer, None) };
        let request = gpu_alloc::Request {
            size: requirements.size,
            align_mask: requirements.alignment,
            usage: UF::DOWNLOAD | UF::UPLOAD | UF::HOST_ACCESS,
            memory_types: requirements.memory_type_bits,
        };
        let memory = unsafe { self.prelude.allocator()?
            .alloc(EruptMemoryDevice::wrap(&self.prelude.device), request)? };
        unsafe {
            self.prelude.device.bind_buffer_memory(buffer, *memory.memory(), memory.offset()).result()?;
        }
        unsafe {
        memory.write_bytes(
                EruptMemoryDevice::wrap(&self.prelude.device),
                0,
                &bytemuck::cast_slice(indices),
            )?;
        }
        let index_buffer = AllocatedBuffer {
            memory,
            buffer,
        };


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
        if let Some(mesh) = self.meshes.remove(id.0) {
            unsafe {
                self.prelude.allocator()?.dealloc(EruptMemoryDevice::wrap(&self.prelude.device), mesh.indices.memory);
                self.prelude.allocator()?.dealloc(EruptMemoryDevice::wrap(&self.prelude.device), mesh.vertices.memory);
                self.prelude.device.destroy_buffer(Some(mesh.indices.buffer), None);
                self.prelude.device.destroy_buffer(Some(mesh.vertices.buffer), None);
            }
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

            let handles = self.materials.iter().collect::<Vec<_>>();
            for material_id in handles {
                let material = match self.materials.get(material_id) {
                    Some(m) => m,
                    None => continue,
                };
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

                for object in packet
                    .objects
                    .iter()
                    .filter(|o| o.material.0 == material_id)
                {
                    let mesh = match self.meshes.get(object.mesh.0) {
                        Some(m) => m,
                        None => {
                            log::error!("Object references a mesh that no longer exists");
                            continue;
                        }
                    };

                    let texture = match self.textures.get(object.texture.0) {
                        Some(m) => m,
                        None => {
                            log::error!("Object references a texture that no longer exists");
                            continue;
                        }
                    };

                    self.prelude.device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        material.pipeline_layout,
                        0,
                        &[texture.descriptor_sets[frame_idx]],
                        &[],
                    );

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

    /// Upload camera matricies (Two f32 camera matrics in column-major order)
    pub fn update_camera_data(&self, frame_idx: usize, data: &[f32; 32]) -> Result<()> {
        let ubo = &self.camera_ubos[frame_idx];
        unsafe {
            ubo.memory.write_bytes(EruptMemoryDevice::wrap(&self.prelude.device), 0, bytemuck::cast_slice(&data[..]))?;
        }
        Ok(())
    }

    /// Update time value
    pub fn update_time_value(&self, time: f32) -> Result<()> {
        let frame_idx = self.frame_sync.current_frame();
        let ubo = &self.time_ubos[frame_idx];
        unsafe {
            ubo.memory.write_bytes(EruptMemoryDevice::wrap(&self.prelude.device), 0, bytemuck::cast_slice(&[time]))?;
        }
        Ok(())
    }

    /// Add a new texture
    pub fn add_texture(&mut self, data: &[u8], width: u32, sampling: Sampling) -> Result<crate::Texture> {
        ensure!(width > 0, "Width must be >0");
        ensure!(
            data.len() % width as usize == 0,
            "Image data must be a multiple of its width"
        );
        ensure!(data.len() % 4 == 0, "Image data must be RGBA");

        let height = data.len() as u32 / (width * 4);

        // Staging buffer
        let create_info = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(data.len() as _);
        let image_buffer =
            unsafe { self.prelude.device.create_buffer(&create_info, None, None) }.result()?;
        let requirements = unsafe { self.prelude.device.get_buffer_memory_requirements(image_buffer, None) };
        use gpu_alloc::UsageFlags as UF;
        let request = gpu_alloc::Request {
            size: requirements.size,
            align_mask: requirements.alignment,
            usage: UF::UPLOAD | UF::HOST_ACCESS,
            memory_types: requirements.memory_type_bits,
        };
        let memory = unsafe { self.prelude.allocator()?
            .alloc(EruptMemoryDevice::wrap(&self.prelude.device), request)? };
        unsafe {
            self.prelude.device.bind_buffer_memory(image_buffer, *memory.memory(), memory.offset()).result()?;
        }
        unsafe {
            memory.write_bytes(
                EruptMemoryDevice::wrap(&self.prelude.device),
                0,
                data,
            )?;
        }
        let image_buffer_alloc = AllocatedBuffer {
            buffer: image_buffer,
            memory,
        };

        // Create texture image
        let extent = vk::Extent3DBuilder::new()
            .width(width)
            .height(height)
            .depth(1)
            .build();
        let create_info = vk::ImageCreateInfoBuilder::new()
            .image_type(vk::ImageType::_2D)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .format(TEXTURE_FORMAT)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlagBits::_1)
            .build();
        let image =
            unsafe { self.prelude.device.create_image(&create_info, None, None) }.result()?;
        let requirements = unsafe { self.prelude.device.get_image_memory_requirements(image, None) };
        let request = gpu_alloc::Request {
            size: requirements.size,
            align_mask: requirements.alignment,
            usage: UF::FAST_DEVICE_ACCESS,
            memory_types: requirements.memory_type_bits,
        };
        let memory = unsafe { self.prelude.allocator()?
            .alloc(EruptMemoryDevice::wrap(&self.prelude.device), request)? };
        unsafe {
            self.prelude.device.bind_image_memory(image, *memory.memory(), memory.offset()).result()?;
        }
        let image_allocation = AllocatedImage {
            image,
            memory,
        };

        self.begin_transfer_cmds()?;
        // Barrier 
        let subresource_range = vk::ImageSubresourceRangeBuilder::new()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        // Copy the staging buffer into the image
        unsafe {
            // TODO: Src/DstAspectMask
            let barrier = vk::ImageMemoryBarrierBuilder::new()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .image(image)
                .subresource_range(subresource_range);
            self.prelude.device.cmd_pipeline_barrier(
                self.transfer_cmd_buf,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                None,
                &[],
                &[],
                &[barrier],
            );

            let offset = vk::Offset3DBuilder::new().x(0).y(0).z(0).build();
            let image_subresources = vk::ImageSubresourceLayersBuilder::new()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1)

                .build();
            let copy = vk::BufferImageCopyBuilder::new()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(image_subresources)
                .image_offset(offset)
                .image_extent(extent);

            self.prelude.device.cmd_copy_buffer_to_image(
                self.transfer_cmd_buf,
                image_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy],
            );

            // TODO: Src/DstAspectMask
            let barrier = vk::ImageMemoryBarrierBuilder::new()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::empty())
                .image(image)
                .subresource_range(subresource_range);
            self.prelude.device.cmd_pipeline_barrier(
                self.transfer_cmd_buf,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                None,
                &[],
                &[],
                &[barrier],
            );

        }
        self.end_transfer_cmds()?;

        unsafe {
            self.prelude.allocator()?.dealloc(EruptMemoryDevice::wrap(&self.prelude.device), image_buffer_alloc.memory);
        }

        // Create image view
        let create_info = vk::ImageViewCreateInfoBuilder::new()
            .image(image)
            .view_type(vk::ImageViewType::_2D)
            .format(TEXTURE_FORMAT)
            .subresource_range(subresource_range)
            .build();
        let image_view = unsafe { self.prelude.device.create_image_view(&create_info, None, None) }.result()?;

        let (filter, mipmode) = match sampling {
            Sampling::Nearest => (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST),
            Sampling::Linear => (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR),
        };

        // Create sampler
        let create_info = vk::SamplerCreateInfoBuilder::new()
            .mag_filter(filter)
            .min_filter(filter)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(sampling == Sampling::Linear)
            .max_anisotropy(16.)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(mipmode)
            .mip_lod_bias(0.)
            .min_lod(0.)
            .max_lod(0.)
            .build();
        let sampler = unsafe { self.prelude.device.create_sampler(&create_info, None, None) }.result()?;

        let descriptor_sets = (0..FRAMES_IN_FLIGHT).map(|_| self.desc_set_allocator.pop()).collect::<Result<Vec<_>>>()?;

        // Populate new descriptor set
        for (animation_ubo, (camera_ubo, descriptor)) in self.time_ubos
            .iter()
            .zip(self.camera_ubos.iter().zip(descriptor_sets.iter()))
        {
            let camera_buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
                .buffer(camera_ubo.buffer)
                .offset(0)
                .range(std::mem::size_of::<CameraUbo>() as u64)];

            let animation_buffer_infos = [vk::DescriptorBufferInfoBuilder::new()
                .buffer(animation_ubo.buffer)
                .offset(0)
                .range(std::mem::size_of::<f32>() as u64)];

            let image_info = [vk::DescriptorImageInfoBuilder::new()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(image_view)
                .sampler(sampler)];

            let writes = [
                vk::WriteDescriptorSetBuilder::new()
                    .buffer_info(&camera_buffer_infos)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .dst_set(*descriptor)
                    .dst_binding(0)
                    .dst_array_element(0),
                vk::WriteDescriptorSetBuilder::new()
                    .buffer_info(&animation_buffer_infos)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .dst_set(*descriptor)
                    .dst_binding(1)
                    .dst_array_element(0),
                vk::WriteDescriptorSetBuilder::new()
                    .image_info(&image_info)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .dst_set(*descriptor)
                    .dst_binding(2)
                    .dst_array_element(0),
            ];

            unsafe {
                self.prelude.device.update_descriptor_sets(&writes, &[]);
            }
        }

        let texture = Texture {
            descriptor_sets,
            alloc: image_allocation,
            view: image_view,
            sampler,
            width,
        };

        Ok(crate::Texture(self.textures.insert(texture)))
    }

    /// Remove the given mesh
    pub fn remove_texture(&mut self, _texture: crate::Texture) -> Result<()> {
        todo!()
    }

    fn begin_transfer_cmds(&mut self) -> Result<()> {
        unsafe {
            self.prelude
                .device
                .reset_command_buffer(self.transfer_cmd_buf, None)
                .result()?;
            let begin_info = vk::CommandBufferBeginInfoBuilder::new();
            self.prelude
                .device
                .begin_command_buffer(self.transfer_cmd_buf, &begin_info)
                .result()?;
            };
        Ok(())
    }

    fn end_transfer_cmds(&mut self) -> Result<()> {
        unsafe {
            self.prelude
                .device
                .end_command_buffer(self.transfer_cmd_buf)
                .result()?;
            let command_buffers = [self.transfer_cmd_buf];
            let submit_info = vk::SubmitInfoBuilder::new().command_buffers(&command_buffers);
            self.prelude
                .device
                .queue_submit(self.prelude.queue, &[submit_info], None)
                .result()?;
            self.prelude
                .device
                .queue_wait_idle(self.prelude.queue)
                .result()?;
            }
        Ok(())
    }
}

fn create_render_pass(device: &DeviceLoader, vr: bool) -> Result<vk::RenderPass> {
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

    Ok(unsafe { device.create_render_pass(&create_info, None, None) }.result()?)
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            self.prelude.device.device_wait_idle().unwrap();
            let handles = self.meshes.iter().collect::<Vec<_>>();
            for mesh in handles {
                self.remove_mesh(crate::Mesh(mesh)).unwrap();
            }
            for ubo in self.camera_ubos.drain(..) {
                self.prelude.allocator().unwrap().dealloc(EruptMemoryDevice::wrap(&self.prelude.device), ubo.memory);
                self.prelude.device.destroy_buffer(Some(ubo.buffer), None);
            }
            for ubo in self.time_ubos.drain(..) {
                self.prelude.allocator().unwrap().dealloc(EruptMemoryDevice::wrap(&self.prelude.device), ubo.memory);
                self.prelude.device.destroy_buffer(Some(ubo.buffer), None);
            }
            self.prelude
                .device
                .destroy_render_pass(Some(self.render_pass), None);
            self.prelude
                .device
                .destroy_descriptor_set_layout(Some(self.descriptor_set_layout), None);
            self.prelude
                .device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            self.prelude
                .device
                .destroy_command_pool(Some(self.command_pool), None);
            }
    }
}
