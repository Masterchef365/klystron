use crate::{DrawType, Engine, FramePacket, Material, Mesh, Vertex};
use anyhow::Result;
use winit::window::Window;

/// Windowed mode Winit engine backend
pub struct WinitBackend;

impl WinitBackend {
    /// Create a new engine instance.
    pub fn new(window: &Window) -> Result<Self> {
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
