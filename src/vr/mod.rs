pub mod xr_prelude;
use crate::core::{Core, VkPrelude};
use crate::swapchain_images::SwapchainImages;
use crate::{DrawType, Engine, FramePacket, Material, Mesh, Vertex};
use anyhow::{bail, Result};
use erupt::{vk1_0 as vk, DeviceLoader, EntryLoader, InstanceLoader};
use log::info;
use nalgebra::{Matrix4, Unit, Vector3};
use std::ffi::CString;
use std::sync::Arc;
use xr_prelude::{load_openxr, XrPrelude};

/// VR Capable OpenXR engine backend
pub struct OpenXrBackend {
    frame_wait: xr::FrameWaiter,
    frame_stream: xr::FrameStream<xr::Vulkan>,
    stage: xr::Space,
    swapchain: Option<xr::Swapchain<xr::Vulkan>>,
    openxr: Arc<XrPrelude>,
    prelude: Arc<VkPrelude>,
    core: Core,
}

impl OpenXrBackend {
    /// Create a new engine instance. Returns the OpenXr caddy for use with input handling.
    pub fn new(application_name: &str) -> Result<(Self, Arc<XrPrelude>)> {
        // Load OpenXR runtime
        let xr_entry = load_openxr()?;

        let mut enabled_extensions = xr::ExtensionSet::default();
        enabled_extensions.khr_vulkan_enable = true;
        let xr_instance = xr_entry.create_instance(
            &xr::ApplicationInfo {
                application_name,
                application_version: 0,
                engine_name: crate::ENGINE_NAME,
                engine_version: 0,
            },
            &enabled_extensions,
            &[],
        )?;
        let instance_props = xr_instance.properties()?;

        info!(
            "Loaded OpenXR runtime: {} {}",
            instance_props.runtime_name, instance_props.runtime_version
        );

        let system = xr_instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .unwrap();

        // Load Vulkan
        let vk_entry = EntryLoader::new()?;

        // Check to see if OpenXR and Vulkan are compatible
        let vk_version = unsafe { vk_entry.enumerate_instance_version(None).result()? };

        let vk_version = xr::Version::new(
            vk::version_major(vk_version) as u16,
            vk::version_minor(vk_version) as u16,
            vk::version_patch(vk_version),
        );

        info!("Loaded Vulkan version {}", vk_version);
        let reqs = xr_instance
            .graphics_requirements::<xr::Vulkan>(system)
            .unwrap();
        if reqs.min_api_version_supported > vk_version {
            bail!(
                "OpenXR runtime requires Vulkan version > {}",
                reqs.min_api_version_supported
            );
        }

        // Vulkan instance extensions required by OpenXR
        let vk_instance_exts = xr_instance
            .vulkan_instance_extensions(system)
            .unwrap()
            .split(' ')
            .map(|x| std::ffi::CString::new(x).unwrap())
            .collect::<Vec<_>>();

        let mut vk_instance_ext_ptrs = vk_instance_exts
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<_>>();

        let mut vk_instance_layers_ptrs = Vec::new();

        // Vulkan device extensions required by OpenXR
        let vk_device_exts = xr_instance
            .vulkan_device_extensions(system)
            .unwrap()
            .split(' ')
            .map(|x| CString::new(x).unwrap())
            .collect::<Vec<_>>();

        let mut vk_device_ext_ptrs = vk_device_exts
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<_>>();

        let mut vk_device_layers_ptrs = Vec::new();

        crate::extensions::extensions_and_layers(
            &mut vk_instance_layers_ptrs,
            &mut vk_instance_ext_ptrs,
            &mut vk_device_layers_ptrs,
            &mut vk_device_ext_ptrs,
        );

        // Vulkan Instance
        let application_name = CString::new(application_name)?;
        let engine_name = CString::new(crate::ENGINE_NAME)?;
        let app_info = vk::ApplicationInfoBuilder::new()
            .application_name(&application_name)
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(crate::engine_version())
            .api_version(vk::make_version(1, 1, 0));

        let create_info = vk::InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_layer_names(&vk_instance_layers_ptrs)
            .enabled_extension_names(&vk_instance_ext_ptrs);

        let vk_instance = InstanceLoader::new(&vk_entry, &create_info, None)?;

        // Obtain physical vk_device, queue_family_index, and vk_device from OpenXR
        let vk_physical_device = vk::PhysicalDevice(
            xr_instance
                .vulkan_graphics_device(system, vk_instance.handle.0 as _)
                .unwrap() as _,
        );

        let queue_family_index = unsafe {
            vk_instance
                .get_physical_device_queue_family_properties(vk_physical_device, None)
                .into_iter()
                .enumerate()
                .filter_map(|(queue_family_index, info)| {
                    if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                        Some(queue_family_index as u32)
                    } else {
                        None
                    }
                })
                .next()
                .expect("Vulkan vk_device has no graphics queue")
        };

        let priorities = [1.0];
        let queues = [vk::DeviceQueueCreateInfoBuilder::new()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)];
        let mut create_info = vk::DeviceCreateInfoBuilder::new()
            .queue_create_infos(&queues)
            .enabled_layer_names(&vk_device_layers_ptrs)
            .enabled_extension_names(&vk_device_ext_ptrs)
            .build();

        let mut phys_device_features = erupt::vk1_2::PhysicalDeviceVulkan11Features {
            multiview: vk::TRUE,
            ..Default::default()
        };

        create_info.p_next = &mut phys_device_features as *mut _ as _;

        let vk_device = DeviceLoader::new(&vk_instance, vk_physical_device, &create_info, None)?;
        let queue = unsafe { vk_device.get_device_queue(queue_family_index, 0, None) };

        let (session, frame_wait, frame_stream) = unsafe {
            xr_instance.create_session::<xr::Vulkan>(
                system,
                &xr::vulkan::SessionCreateInfo {
                    instance: vk_instance.handle.0 as _,
                    physical_device: vk_physical_device.0 as _,
                    device: vk_device.handle.0 as _,
                    queue_family_index,
                    queue_index: 0,
                },
            )
        }?;

        let stage = session
            .create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)
            .unwrap();

        let prelude = Arc::new(VkPrelude {
            queue,
            queue_family_index,
            device: vk_device,
            physical_device: vk_physical_device,
            instance: vk_instance,
            entry: vk_entry,
        });

        let core = Core::new(prelude.clone(), true)?;

        let openxr = Arc::new(XrPrelude {
            instance: xr_instance,
            session,
            system,
        });

        let instance = Self {
            frame_wait,
            frame_stream,
            stage,
            swapchain: None,
            openxr: openxr.clone(),
            prelude,
            core,
        };

        Ok((instance, openxr))
    }

    /// Render a frame of video.
    /// Returns false when the loop should break
    pub fn next_frame(&mut self, packet: &FramePacket) -> Result<()> {
        // Wait for OpenXR to signal it has a frame ready
        let xr_frame_state = self.frame_wait.wait()?;
        self.frame_stream.begin()?;

        if !xr_frame_state.should_render {
            self.frame_stream.end(
                xr_frame_state.predicted_display_time,
                xr::EnvironmentBlendMode::OPAQUE,
                &[],
            )?;
            return Ok(());
        }

        if self.swapchain.is_none() {
            self.recreate_swapchain()?;
        }

        let (frame_idx, frame) = self.core.frame_sync.next_frame()?;

        let swapchain = self.swapchain.as_mut().unwrap();

        let image_index = swapchain.acquire_image()?;

        swapchain.wait_image(xr::Duration::INFINITE)?;

        //let image: crate::swapchain_images::SwapChainImage = todo!();
        let image = {
            self.core
                .swapchain_images
                .as_mut()
                .unwrap()
                .next_image(image_index, &frame)?
        };

        // Write command buffers
        let command_buffer = self.core.write_command_buffers(frame_idx, packet, &image)?;

        // Get views
        let (_, views) = self.openxr.session.locate_views(
            xr::ViewConfigurationType::PRIMARY_STEREO,
            xr_frame_state.predicted_display_time,
            &self.stage,
        )?;

        let left = matrix_from_view(&views[0]);
        let right = matrix_from_view(&views[1]);
        let both = left.iter().chain(right.iter()).copied().collect::<Vec<_>>();
        let mut data = [0.0; 32];
        data.copy_from_slice(&both);
        self.core.update_camera_data(frame_idx, &data)?;

        // Submit to the queue
        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfoBuilder::new().command_buffers(&command_buffers);
        unsafe {
            self.prelude
                .device
                .reset_fences(&[frame.in_flight_fence])
                .result()?; // TODO: Move this into the swapchain next_image
            self.prelude
                .device
                .queue_submit(
                    self.prelude.queue,
                    &[submit_info],
                    Some(frame.in_flight_fence),
                )
                .result()?;
        }

        // Present to swapchain
        swapchain.release_image()?;

        // Tell OpenXR what to present for this frame
        let rect = xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di {
                width: image.extent.width as _,
                height: image.extent.height as _,
            },
        };
        self.frame_stream.end(
            xr_frame_state.predicted_display_time,
            xr::EnvironmentBlendMode::OPAQUE,
            &[&xr::CompositionLayerProjection::new()
                .space(&self.stage)
                .views(&[
                    xr::CompositionLayerProjectionView::new()
                        .pose(views[0].pose)
                        .fov(views[0].fov)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(&swapchain)
                                .image_array_index(0)
                                .image_rect(rect),
                        ),
                    xr::CompositionLayerProjectionView::new()
                        .pose(views[1].pose)
                        .fov(views[1].fov)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(&swapchain)
                                .image_array_index(1)
                                .image_rect(rect),
                        ),
                ])],
        )?;

        return Ok(());
    }

    fn recreate_swapchain(&mut self) -> Result<()> {
        if let Some(mut images) = self.core.swapchain_images.take() {
            images.free(&mut self.core.allocator)?;
        }
        self.swapchain = None;

        let views = self
            .openxr
            .instance
            .enumerate_view_configuration_views(
                self.openxr.system,
                xr::ViewConfigurationType::PRIMARY_STEREO,
            )
            .unwrap();

        let extent = vk::Extent2D {
            width: views[0].recommended_image_rect_width,
            height: views[0].recommended_image_rect_height,
        };

        let swapchain = self
            .openxr
            .session
            .create_swapchain(&xr::SwapchainCreateInfo {
                create_flags: xr::SwapchainCreateFlags::EMPTY,
                usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT
                    | xr::SwapchainUsageFlags::SAMPLED,
                format: crate::core::COLOR_FORMAT.0 as _,
                sample_count: 1,
                width: extent.width,
                height: extent.height,
                face_count: 1,
                array_size: 2,
                mip_count: 1,
            })
            .unwrap();

        let swapchain_images = swapchain
            .enumerate_images()?
            .into_iter()
            .map(vk::Image)
            .collect::<Vec<_>>();

        // TODO: Coagulate these two into one object?
        self.swapchain = Some(swapchain);

        self.core.swapchain_images = Some(SwapchainImages::new(
            self.prelude.clone(),
            &mut self.core.allocator,
            extent,
            self.core.render_pass,
            swapchain_images,
            true,
        )?);

        Ok(())
    }
}

// TODO: This is stupid.
impl Engine for OpenXrBackend {
    fn add_material(
        &mut self,
        vertex: &[u8],
        fragment: &[u8],
        draw_type: DrawType,
    ) -> Result<Material> {
        self.core.add_material(vertex, fragment, draw_type)
    }
    fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<Mesh> {
        self.core.add_mesh(vertices, indices)
    }
    fn remove_material(&mut self, material: Material) -> Result<()> {
        self.core.remove_material(material)
    }
    fn remove_mesh(&mut self, mesh: Mesh) -> Result<()> {
        self.core.remove_mesh(mesh)
    }
    fn update_time_value(&self, data: f32) -> Result<()> {
        self.core.update_time_value(data)
    }
}

fn matrix_from_view(view: &xr::View) -> Matrix4<f32> {
    let proj = projection_from_fov(&view.fov, 0.01, 1000.0);
    let view = view_from_pose(&view.pose);
    proj * view
}

// Ported from:
// https://gitlab.freedesktop.org/monado/demos/xrgears/-/blob/master/src/main.cpp
fn view_from_pose(pose: &xr::Posef) -> Matrix4<f32> {
    let quat = pose.orientation;
    let quat = nalgebra::Quaternion::new(quat.w, quat.x, quat.y, quat.z);
    let quat = Unit::try_new(quat, 0.0).expect("Not a unit quaternion");
    let rotation = quat.to_homogeneous();

    let position = pose.position;
    let position = Vector3::new(position.x, position.y, position.z);
    let translation = Matrix4::new_translation(&position);

    let view = translation * rotation;
    let inv = view.try_inverse().expect("Matrix didn't invert");
    inv
}

fn projection_from_fov(fov: &xr::Fovf, _near: f32, _far: f32) -> Matrix4<f32> {
    let tan_left = fov.angle_left.tan();
    let tan_right = fov.angle_right.tan();

    let tan_up = fov.angle_up.tan();
    let tan_down = fov.angle_down.tan();

    let tan_width = tan_right - tan_left;
    let tan_height = tan_down - tan_up;

    let a11 = 2.0 / tan_width;
    let a22 = 2.0 / tan_height;

    let a31 = (tan_right + tan_left) / tan_width;
    let a32 = (tan_up + tan_down) / tan_height;
    //let a33 = -far / (far - near);

    //let a43 = -(far * near) / (far - near);
    Matrix4::new(
        a11, 0.0, a31, 0.0, //
        0.0, a22, a32, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0, //
    )
}
