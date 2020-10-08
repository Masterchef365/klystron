use crate::core::Core;
use crate::material::Material;
use crate::mesh::Mesh;
use crate::swapchain_images::SwapChainImage;
use crate::particle_system::ParticleSystem;
use anyhow::Result;
use erupt::vk1_0 as vk;

impl Core {
    unsafe fn cmd_render_objects(
        &self,
        command_buffer: vk::CommandBuffer,
        descriptor_set: vk::DescriptorSet,
        packet: &crate::FramePacket,
        image: &SwapChainImage,
    ) {
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
                        log::error!("Object references a mesh that no longer exists");
                        continue;
                    }
                };

                self.cmd_draw_mesh(
                    command_buffer,
                    descriptor_set,
                    material,
                    mesh,
                    &object.transform,
                );
            }

            for particle_system in packet
                .particle_simulations
                .iter()
                .filter(|o| o.material.0 == *material_id)
            {
                let particle_set = match self.particle_sets.get(&particle_system.particles.0) {
                    Some(m) => m,
                    None => {
                        log::error!("Particle system references a mesh that no longer exists");
                        continue;
                    }
                };

                self.cmd_draw_mesh(
                    command_buffer,
                    descriptor_set,
                    material,
                    &particle_set.mesh,
                    &nalgebra::Matrix4::identity(),
                );
            }
        }

        self.prelude.device.cmd_end_render_pass(command_buffer);
    }

    unsafe fn cmd_draw_mesh(
        &self,
        command_buffer: vk::CommandBuffer,
        descriptor_set: vk::DescriptorSet,
        material: &Material,
        mesh: &Mesh,
        transform: &nalgebra::Matrix4<f32>,
    ) {
        self.prelude.device.cmd_bind_vertex_buffers(
            command_buffer,
            0,
            &[*mesh.vertices.object()],
            &[0],
        );

        self.prelude.device.cmd_bind_index_buffer(
            command_buffer,
            *mesh.indices.object(),
            0,
            vk::IndexType::UINT16,
        );

        let descriptor_sets = [descriptor_set];
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
            transform.data.as_ptr() as _,
        );

        self.prelude
            .device
            .cmd_draw_indexed(command_buffer, mesh.n_indices, 1, 0, 0, 0);
    }

    unsafe fn cmd_particle_exec_pipeline(
        &self,
        command_buffer: vk::CommandBuffer,
        packet: &crate::FramePacket,
        extract_pipeline: impl Fn(&ParticleSystem) -> vk::Pipeline,
    ) {
        for (&part_sys_id, particle_system) in self.particle_systems.iter() {
            self.prelude.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                extract_pipeline(particle_system),
            );
            let relevant_sims = packet
                .particle_simulations
                .iter()
                .filter(|sim| sim.particle_system.0 == part_sys_id);
            for particle_sim in relevant_sims {
                let particle_set = self
                    .particle_sets
                    .get(&particle_sim.particles.0)
                    .expect("Associated particle set deleted");
                self.prelude.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    self.particle_pipeline_layout,
                    0,
                    &[particle_set.descriptor_set],
                    &[],
                );
                let size_x = (particle_set.n_particles / 64) as u32 + 1;
                self.prelude.device.cmd_dispatch(command_buffer, size_x, 1, 1);
            }
        }
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

            self.cmd_particle_exec_pipeline(command_buffer, packet, |p| p.forces_pipeline);

            self.prelude.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                None,
                &[],
                &[],
                &[],
            );

            self.cmd_particle_exec_pipeline(command_buffer, packet, |p| p.motion_pipeline);

            self.prelude.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::VERTEX_INPUT,
                None,
                &[],
                &[],
                &[],
            );

            self.cmd_render_objects(command_buffer, descriptor_set, packet, image);

            self.prelude
                .device
                .end_command_buffer(command_buffer)
                .result()?;
        }

        Ok(command_buffer)
    }
}
