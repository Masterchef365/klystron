//! Klystron rendering engine. Duncan's personal rendering engine project. Made primarily to render
//! simple, unlit scenes with dynamically placed objects. VR capable through the OpenXR
//! interface, and hopefully easily modifiable.
extern crate openxr as xr;
mod core;
mod extensions;
mod frame_sync;
mod handle;
mod hardware_query;
mod material;
mod swapchain_images;
mod vertex;
mod vr;
mod windowed;
mod particle_system;
pub mod runtime;
use anyhow::Result;
use nalgebra::Matrix4;
pub use particle_system::Particle;
pub use vertex::Vertex;
pub use vr::{xr_prelude::XrPrelude, OpenXrBackend};
pub use windowed::{Camera, WinitBackend};

/// All information necessary to define a frame of video (besides camera, which is passed in a
/// special camera for windowed mode and implicitly in OpenXR)
pub struct FramePacket {
    /// The entire scene's worth of objects
    pub objects: Vec<Object>,
    /// Particle systems drawn and simulated this frame
    pub particle_systems: Vec<ParticleSystem>,
}

/// Particle system
pub struct ParticleSystem {
    /// How to simulate particles
    pub compute_shader: ComputeShader,
    /// ParticleSet to draw
    pub particles: ParticleSet,
    /// How to draw particles
    pub material: Material,
}

/// A single object in the scene
pub struct Object {
    /// How to draw this object
    pub material: Material,
    /// Vertex and Index data for the object
    pub mesh: Mesh,
    /// Transformation applied to each vertex of this Object
    pub transform: Matrix4<f32>,
}

/// Handle for a Material (Draw commands)
#[derive(Copy, Clone)]
pub struct Material(pub(crate) handle::Id);

/// Handle for a Mesh (Draw content)
#[derive(Copy, Clone)]
pub struct Mesh(pub(crate) handle::Id);

/// Handle for a Compute Shader
#[derive(Copy, Clone)]
pub struct ComputeShader(pub(crate) handle::Id);

/// Handle for a set of ParticleSet
#[derive(Copy, Clone)]
pub struct ParticleSet(pub(crate) handle::Id);

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
    /// Add a compute shader
    fn add_compute_shader(&mut self, shader: &[u8]) -> Result<ComputeShader>;
    /// Add a particle system
    fn add_particles(&mut self, particles: &[Particle]) -> Result<ParticleSet>;
}

pub(crate) const ENGINE_NAME: &'static str = "Klystron";
pub(crate) fn engine_version() -> u32 {
    erupt::vk1_0::make_version(1, 0, 0)
}
