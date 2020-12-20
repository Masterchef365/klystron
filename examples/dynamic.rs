use anyhow::Result;
use klystron::{
    runtime_2d::{event::WindowEvent, launch, App2D},
    DrawType, Engine, FramePacket, Matrix4, Object, Vertex, WinitBackend, UNLIT_FRAG, UNLIT_VERT, MeshType, DynamicMesh, Material,
};

struct Pattern {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    time: f32,
}

impl Pattern {
    pub fn new() -> Self {
    }

    pub fn update(&mut self) {
    }
}

struct MyApp {
    mesh: DynamicMesh,
    material: Material,
    pattern: Pattern,
    window_size: (u32, u32),
}

impl App2D for MyApp {
    const TITLE: &'static str = "2D example app";
    type Args = ();

    fn new(engine: &mut WinitBackend, _args: Self::Args) -> Result<Self> {
        let material = engine.add_material(UNLIT_VERT, UNLIT_FRAG, DrawType::Lines)?;
        
        let pattern = Pattern::new();
        let mesh = engine.add_dynamic_mesh(&pattern.vertices, &pattern.indices)?;

        Ok(Self {
            material,
            mesh,
            pattern,
            window_size: (500, 500),
        })
    }

    fn event(&mut self, event: &WindowEvent, _engine: &mut WinitBackend) -> Result<()> {
        match event {
            WindowEvent::Resized(size) => {
                self.window_size = (size.width, size.height);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let center = |v| v * 2. - 1.;
                let (x, y) = (
                    center(position.x as f32 / self.window_size.0 as f32),
                    center(position.y as f32 / self.window_size.1 as f32),
                );
                self.object.transform = Matrix4::from_euler_angles(0., 0., y.atan2(x) as f32);
            }
            _ => (),
        }
        Ok(())
    }

    fn frame(&self) -> FramePacket {
        self.pattern.update();
        let object = Object {
            mesh: MeshType::Dynamic(self.mesh),
            transform: Matrix4::identity(),
            material,
        };
        FramePacket {
            objects: vec![object],
        }
    }
}

fn wire_triangle() -> ([Vertex; 3], [u16; 6]) {
    let color = [0., 1., 0.];
    let vertices = [
        Vertex {
            pos: [-0.5, -0.25, 0.],
            color,
        },
        Vertex {
            pos: [-0.5, 0.25, 0.],
            color,
        },
        Vertex {
            pos: [0.; 3],
            color,
        },
    ];
    let indices = [0, 1, 1, 2, 2, 0];
    (vertices, indices)
}

fn main() -> Result<()> {
    launch::<MyApp>(())
}
