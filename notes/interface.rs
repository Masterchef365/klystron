//! Goals of this interface:
//! 1. Allow experimentation. 
//!     * Do not shoehorn users into input abstractions
//! 2. VR and windowed compatible
pub struct FramePacket {
    pub objects: Vec<Object>,
    pub time: f32,
    pub camera_origin: Point3<f32>,
    pub camera_rotation: UnitQuaternion<f32>,
}

struct Object {
   material: Material, 
   mesh: Mesh, 
   transform: Matrix4<f32>,
}

struct Material;
struct Mesh;

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

pub struct WinitBackend;

impl WinitBackend {
    pub fn new(window: &Window) -> Result<Self>;
    pub fn next_frame(&mut self, packet: &FramePacket) -> Result<()>;
}

pub struct OpenXrBackend;

impl OpenXrBackend {
    pub fn new(openxr: &OpenXr) -> Result<Self>;
    pub fn next_frame(&mut self, openxr: &OpenXr, packet: &FramePacket) -> Result<()>;
}
