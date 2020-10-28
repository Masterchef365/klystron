use crate::core::Core;
use crate::material::Material;
use crate::swapchain_images::SwapChainImage;
use crate::Object;
use anyhow::Result;
use erupt::vk1_0 as vk;

impl Core {
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

            self.object_set_draw_cmds(command_buffer, image, descriptor_set, &packet.objects);

            self.prelude.device.cmd_end_render_pass(command_buffer);

            self.prelude
                .device
                .end_command_buffer(command_buffer)
                .result()?;
        }

        Ok(command_buffer)
    }

    fn object_set_draw_cmds(
        &self,
        command_buffer: vk::CommandBuffer,
        image: &SwapChainImage,
        descriptor_set: vk::DescriptorSet,
        objects: &[Object],
    ) {
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

            unsafe {
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
            }

            for object in objects.iter().filter(|o| o.material.0 == material_id) {
                self.object_draw_cmds(command_buffer, object, material, descriptor_set);
            }
        }
    }

    fn object_draw_cmds(
        &self,
        command_buffer: vk::CommandBuffer,
        object: &Object,
        material: &Material,
        descriptor_set: vk::DescriptorSet,
    ) {
        let mesh = match self.meshes.get(object.mesh.0) {
            Some(m) => m,
            None => {
                log::error!("Object references a mesh that no exists");
                return;
            }
        };
        unsafe {
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
                object.transform.data.as_ptr() as _,
            );

            self.prelude
                .device
                .cmd_draw_indexed(command_buffer, mesh.n_indices, 1, 0, 0, 0);
        }
    }
}
