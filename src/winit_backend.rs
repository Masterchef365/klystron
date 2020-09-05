use crate::{Engine, FramePacket, DrawType, Material, Mesh, Vertex};
use winit::window::Window;
use anyhow::Result;

pub struct WinitBackend;

impl WinitBackend {
    pub fn new(window: &Window) -> Result<Self> { todo!() }
    // Might need an amendment; camera position should be driven by something external 
    // Winit keypresses used to move camera.
    pub fn next_frame(&mut self, packet: &FramePacket) -> Result<()> { todo!() }
}

impl Engine for WinitBackend {
    fn add_material(&mut self, vertex: &[u8], fragment: &[u8], draw_type: DrawType) -> Result<Material> { todo!() }
    fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<Mesh> { todo!() }
    fn remove_material(&mut self, material: Material) { todo!() }
    fn remove_mesh(&mut self, mesh: Mesh) { todo!() }
}
