use crate::{DrawType, Engine, FramePacket, Material, Mesh, Vertex};
use anyhow::Result;
use winit::window::Window;
use erupt::{
    cstr,
    extensions::{ext_debug_utils, khr_swapchain},
    utils::{allocator, surface},
    vk1_0 as vk, DeviceLoader, EntryLoader, InstanceLoader,
};
use std::ffi::CString;

/// Windowed mode Winit engine backend
pub struct WinitBackend;

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

        let create_info = vk::InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&instance_layers);

        let mut instance = InstanceLoader::new(&entry, &create_info, None)?;


        todo!()
    }

    // TODO: camera position should be driven by something external
    // Winit keypresses used to move camera.
    pub fn next_frame(&mut self, packet: &FramePacket) -> Result<()> {
        todo!()
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
