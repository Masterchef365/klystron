use crate::{Engine, FramePacket, DrawType, Material, Mesh, Vertex};
use crate::openxr_caddy::OpenXr;
use anyhow::Result;

pub struct OpenXrBackend;

impl OpenXrBackend {
    pub fn new(openxr: &OpenXr) -> Result<Self> { todo!() }
    /// Returns false when the loop should break
    pub fn next_frame(&mut self, openxr: &OpenXr, packet: &FramePacket) -> Result<bool> { todo!() }
}

impl Engine for OpenXrBackend {
    fn add_material(&mut self, vertex: &[u8], fragment: &[u8], draw_type: DrawType) -> Result<Material> { todo!() }
    fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<Mesh> { todo!() }
    fn remove_material(&mut self, material: Material) { todo!() }
    fn remove_mesh(&mut self, mesh: Mesh) { todo!() }
}
