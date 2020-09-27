mod camera;
use crate::core::{Core, VkPrelude};
use crate::hardware_query::HardwareSelection;
use crate::swapchain_images::SwapchainImages;
use crate::{DrawType, Engine, FramePacket, Material, Mesh, Vertex};
use anyhow::Result;
pub use camera::Camera;
use erupt::{
    extensions::{khr_surface, khr_swapchain},
    utils::surface,
    vk1_0 as vk, DeviceLoader, EntryLoader, InstanceLoader,
};
use std::ffi::CString;
use std::sync::Arc;
use winit::window::Window;

/// Windowed mode Winit engine backend
pub struct WinitBackend {
    swapchain: Option<khr_swapchain::SwapchainKHR>,
    image_available_semaphores: Vec<vk::Semaphore>,
    surface: khr_surface::SurfaceKHR,
    hardware: HardwareSelection,
    prelude: Arc<VkPrelude>,
    core: Core,
}

impl WinitBackend {
    /// Create a new engine instance.
    pub fn new(window: &Window, application_name: &str) -> Result<Self> {
        // Entry
        let entry = EntryLoader::new()?;

        // Instance
        let application_name = CString::new(application_name)?;
        let engine_name = CString::new(crate::ENGINE_NAME)?;
        let app_info = vk::ApplicationInfoBuilder::new()
            .application_name(&application_name)
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(crate::engine_version())
            .api_version(vk::make_version(1, 0, 0));

        // Instance and device layers and extensions
        let mut instance_layers = Vec::new();
        let mut instance_extensions = surface::enumerate_required_extensions(window).result()?;
        let mut device_layers = Vec::new();
        let mut device_extensions = vec![khr_swapchain::KHR_SWAPCHAIN_EXTENSION_NAME];

        crate::extensions::extensions_and_layers(
            &mut instance_layers,
            &mut instance_extensions,
            &mut device_layers,
            &mut device_extensions,
        );

        // Instance creation
        let create_info = vk::InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&instance_layers);

        let mut instance = InstanceLoader::new(&entry, &create_info, None)?;

        // Surface
        let surface = unsafe { surface::create_surface(&mut instance, window, None) }.result()?;

        // Hardware selection
        let hardware = HardwareSelection::query(&instance, surface, &device_extensions)?;

        // Create logical device and queues
        let create_info = [vk::DeviceQueueCreateInfoBuilder::new()
            .queue_family_index(hardware.queue_family)
            .queue_priorities(&[1.0])];

        let physical_device_features = vk::PhysicalDeviceFeaturesBuilder::new();
        let create_info = vk::DeviceCreateInfoBuilder::new()
            .queue_create_infos(&create_info)
            .enabled_features(&physical_device_features)
            .enabled_extension_names(&device_extensions)
            .enabled_layer_names(&device_layers);

        let device = DeviceLoader::new(&instance, hardware.physical_device, &create_info, None)?;
        let queue = unsafe { device.get_device_queue(hardware.queue_family, 0, None) };

        let prelude = Arc::new(VkPrelude {
            queue,
            queue_family_index: hardware.queue_family,
            physical_device: hardware.physical_device,
            device,
            instance,
            entry,
        });

        let core = Core::new(prelude.clone(), false)?;

        let image_available_semaphores = (0..crate::core::FRAMES_IN_FLIGHT)
            .map(|_| {
                let create_info = vk::SemaphoreCreateInfoBuilder::new();
                unsafe {
                    prelude
                        .device
                        .create_semaphore(&create_info, None, None)
                        .result()
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            swapchain: None,
            image_available_semaphores,
            hardware,
            surface,
            prelude,
            core,
        })
    }

    // TODO: camera position should be driven by something external
    // Winit keypresses used to move camera.
    pub fn next_frame(&mut self, packet: &FramePacket, camera: &camera::Camera) -> Result<()> {
        if self.swapchain.is_none() {
            self.create_swapchain()?;
        }
        let swapchain = self.swapchain.unwrap();

        let (frame_idx, frame) = self.core.frame_sync.next_frame()?;

        let image_available = self.image_available_semaphores[frame_idx];
        let image_index = unsafe {
            self.prelude.device.acquire_next_image_khr(
                swapchain,
                u64::MAX,
                Some(image_available),
                None,
                None,
            )
        };

        // Early return and invalidate swapchain
        let image_index = if image_index.raw == vk::Result::ERROR_OUT_OF_DATE_KHR {
            self.free_swapchain()?;
            return Ok(());
        } else {
            image_index.unwrap()
        };

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

        // Upload camera matrix and time
        let mut data = [0.0; 32];
        data.iter_mut()
            .zip(
                camera
                    .matrix(image.extent.width, image.extent.height)
                    .as_slice()
                    .iter(),
            )
            .for_each(|(o, i)| *o = *i);
        self.core.update_camera_data(frame_idx, &data)?;

        // Submit to the queue
        let command_buffers = [command_buffer];
        let wait_semaphores = [image_available];
        let signal_semaphores = [frame.render_finished];
        let submit_info = vk::SubmitInfoBuilder::new()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
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
        let swapchains = [swapchain];
        let image_indices = [image_index];
        let present_info = khr_swapchain::PresentInfoKHRBuilder::new()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let queue_result = unsafe {
            self.prelude
                .device
                .queue_present_khr(self.prelude.queue, &present_info)
        };

        if queue_result.raw == vk::Result::ERROR_OUT_OF_DATE_KHR {
            self.free_swapchain()?;
            return Ok(());
        } else {
            queue_result.result()?;
        };

        Ok(())
    }

    fn free_swapchain(&mut self) -> Result<()> {
        if let Some(mut images) = self.core.swapchain_images.take() {
            images.free(&mut self.core.allocator)?;
        }

        unsafe {
            self.prelude
                .device
                .destroy_swapchain_khr(self.swapchain.take(), None);
        }

        Ok(())
    }

    fn create_swapchain(&mut self) -> Result<()> {
        let surface_caps = unsafe {
            self.prelude
                .instance
                .get_physical_device_surface_capabilities_khr(
                    self.prelude.physical_device,
                    self.surface,
                    None,
                )
        }
        .result()?;

        let mut image_count = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
            image_count = surface_caps.max_image_count;
        }

        // Build the actual swapchain
        let create_info = khr_swapchain::SwapchainCreateInfoKHRBuilder::new()
            .surface(self.surface)
            .min_image_count(image_count)
            .image_format(crate::core::COLOR_FORMAT)
            .image_color_space(self.hardware.format.color_space)
            .image_extent(surface_caps.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_caps.current_transform)
            .composite_alpha(khr_surface::CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
            .present_mode(self.hardware.present_mode)
            .clipped(true)
            .old_swapchain(khr_swapchain::SwapchainKHR::null());

        let swapchain = unsafe {
            self.prelude
                .device
                .create_swapchain_khr(&create_info, None, None)
        }
        .result()?;
        let swapchain_images = unsafe {
            self.prelude
                .device
                .get_swapchain_images_khr(swapchain, None)
        }
        .result()?;

        self.swapchain = Some(swapchain);

        // TODO: Coagulate these two into one object?
        self.swapchain = Some(swapchain);

        self.core.swapchain_images = Some(SwapchainImages::new(
            self.prelude.clone(),
            &mut self.core.allocator,
            surface_caps.current_extent,
            self.core.render_pass,
            swapchain_images,
            false,
        )?);

        Ok(())
    }
}

// TODO: This is stupid.
impl Engine for WinitBackend {
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

impl Drop for WinitBackend {
    fn drop(&mut self) {
        unsafe {
            for semaphore in self.image_available_semaphores.drain(..) {
                self.prelude.device.destroy_semaphore(Some(semaphore), None);
            }
            self.free_swapchain().unwrap();
            self.prelude
                .instance
                .destroy_surface_khr(Some(self.surface), None);
        }
    }
}
