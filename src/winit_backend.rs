use crate::core::{Core, VkPrelude};
use crate::hardware_query::HardwareSelection;
use crate::{DrawType, Engine, FramePacket, Material, Mesh, Vertex};
use anyhow::Result;
use erupt::{
    cstr,
    extensions::{khr_surface, ext_debug_utils, khr_swapchain},
    utils::{allocator, surface},
    vk1_0 as vk, DeviceLoader, EntryLoader, InstanceLoader,
};
use std::ffi::CString;
use std::sync::Arc;
use winit::window::Window;

/// Windowed mode Winit engine backend
pub struct WinitBackend {
    //swapchain: Swapchain,
    surface: khr_surface::SurfaceKHR,
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

        let core = Core::new(prelude.clone())?;

        Ok(Self {
            surface,
            prelude,
            core,
        })
    }

    // TODO: camera position should be driven by something external
    // Winit keypresses used to move camera.
    pub fn next_frame(&mut self, packet: &FramePacket) -> Result<()> {
        todo!("Next frame")
    }
}

impl Engine for WinitBackend {
    fn add_material(
        &mut self,
        vertex: &[u8],
        fragment: &[u8],
        draw_type: DrawType,
    ) -> Result<Material> {
        todo!()
    }
    fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<Mesh> {
        todo!()
    }
    fn remove_material(&mut self, material: Material) {
        todo!()
    }
    fn remove_mesh(&mut self, mesh: Mesh) {
        todo!()
    }
}

impl Drop for WinitBackend {
    fn drop(&mut self) {
        unsafe {
            self.prelude.instance.destroy_surface_khr(Some(self.surface), None);
        }
    }
}
