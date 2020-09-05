extern crate openxr as xr;
mod vertex;
mod openxr_caddy;
mod openxr_backend;
mod winit_backend;
pub use openxr_caddy::OpenXr;
use nalgebra::{Matrix4, Point3, UnitQuaternion};
pub use vertex::Vertex;
use anyhow::Result;
pub use openxr_backend::OpenXrBackend;
pub use winit_backend::WinitBackend;

/// All information necessary to define a frame of video (besides camera, which is passed in in
/// winit and implicit in OpenXR)
pub struct FramePacket {
    pub objects: Vec<Object>,
    pub time: f32,
    pub camera_origin: Point3<f32>,
    pub camera_rotation: UnitQuaternion<f32>,
}

/// A single object in the scene
pub struct Object {
   pub material: Material, 
   pub mesh: Mesh, 
   pub transform: Matrix4<f32>,
}

#[derive(Copy, Clone)]
pub struct Material;
#[derive(Copy, Clone)]
pub struct Mesh;

pub enum DrawType {
    Lines,
    Points,
    Triangles,
}

pub trait Engine {
    fn add_material(&mut self, vertex: &[u8], fragment: &[u8], draw_type: DrawType) -> Result<Material>;
    fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<Mesh>;
    fn remove_material(&mut self, material: Material);
    fn remove_mesh(&mut self, mesh: Mesh);
}
