use crate::openxr_caddy::{load_openxr, OpenXr};
use crate::{DrawType, Engine, FramePacket, Material, Mesh, Vertex};
use anyhow::Result;

pub struct OpenXrBackend {
/*
    frame_wait: Option<xr::FrameWaiter>,
    frame_stream: Option<xr::FrameStream<xr::Vulkan>>,
    stage: Option<xr::Space>,
    swapchain: Option<Swapchain>,
*/
}

impl OpenXrBackend {
    pub fn new(application_name: &str) -> Result<(Self, OpenXr)> {
        let entry = load_openxr()?;

        let mut enabled_extensions = xr::ExtensionSet::default();
        enabled_extensions.khr_vulkan_enable = true;
        let instance = entry.create_instance(
            &xr::ApplicationInfo {
                application_name,
                application_version: 0, // TODO: Populate these?
                engine_name: "Klystron",
                engine_version: 0,
            },
            &enabled_extensions,
            &[],
        )?;
        let instance_props = instance.properties()?;

        let system = instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .unwrap();

        /*
        OpenXr {
            instance,
            session,
            system,
        }
        */
        todo!()
    }
    /// Returns false when the loop should break
    pub fn next_frame(&mut self, openxr: &OpenXr, packet: &FramePacket) -> Result<bool> {
        todo!()
    }
}

impl Engine for OpenXrBackend {
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
