//! Klystron rendering engine. Duncan's personal rendering engine project. Made primarily to render
//! simple, unlit scenes with dynamically placed objects. VR capable through the OpenXR
//! interface, and hopefully easily modifiable.
extern crate openxr as xr;
mod core;
mod extensions;
mod frame_sync;
mod hardware_query;
mod material;
pub mod runtime;
mod swapchain_images;
mod vertex;
mod vr;
mod windowed;
mod command_buffer;
use anyhow::Result;
use nalgebra::Matrix4;
pub use vertex::Vertex;
pub use vr::{xr_prelude::XrPrelude, OpenXrBackend};
pub use windowed::{Camera, WinitBackend};
use genmap::Handle;

/// All information necessary to define a frame of video (besides camera, which is passed in a
/// special camera for windowed mode and implicitly in OpenXR)
pub struct FramePacket {
    /// The entire scene's worth of objects
    pub objects: Vec<Object>,
}

/// A single object in the scene
pub struct Object {
    /// How to draw this object
    pub material: Material,
    /// Vertex and Index data for the object
    pub mesh: Mesh,
    /// Transformation applied to each vertex of this Object
    pub transform: Matrix4<f32>,
    // /// An additional time uniform passed to the vertex and fragment shaders
    // pub anim: f32,
}

/// Handle for a Material (Draw commands)
#[derive(Copy, Clone)]
pub struct Material(pub(crate) Handle);

/// Handle for a Mesh (Draw content)
#[derive(Copy, Clone)]
pub struct Mesh(pub(crate) Handle);

/// Material rasterization method
pub enum DrawType {
    /// Lines in between each pair of indices
    Lines,
    /// Pointcloud
    Points,
    /// Normal triangular rendering
    Triangles,
}

/// Traits all engines must implement; next_frame() not included because all engines have different
/// per-frame requirements.
pub trait Engine {
    /// Add a material, given SPIR-V bytecode
    fn add_material(
        &mut self,
        vertex: &[u8],
        fragment: &[u8],
        draw_type: DrawType,
    ) -> Result<Material>;
    /// Add a mesh, given vertices and indices
    fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<Mesh>;
    /// Remove the given material
    fn remove_material(&mut self, material: Material) -> Result<()>;
    /// Remove the given mesh
    fn remove_mesh(&mut self, mesh: Mesh) -> Result<()>;
    /// Update the animation value
    fn update_time_value(&self, data: f32) -> Result<()>;
}

pub(crate) const ENGINE_NAME: &'static str = "Klystron";
pub(crate) fn engine_version() -> u32 {
    erupt::vk1_0::make_version(1, 0, 0)
}
