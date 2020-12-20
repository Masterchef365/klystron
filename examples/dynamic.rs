use anyhow::Result;
use klystron::{
    runtime_2d::{event::WindowEvent, launch, App2D},
    DrawType, DynamicMesh, Engine, FramePacket, Material, Matrix4, MeshType, Object, Vertex,
    WinitBackend, UNLIT_FRAG, UNLIT_VERT,
};

#[derive(Default)]
struct Pattern {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    time: f32,
}

impl Pattern {
    pub fn update(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        let n = 1000;
        for i in 0..n {
            let i = i as f32 / n as f32;
            let x = i * 2. - 1.;
            self.indices.push(self.vertices.len() as _);
            self.vertices.push(Vertex {
                pos: [x, (x + self.time).cos() / 2., 0.],
                color: [0., 1., 0.],
            });
            self.indices.push(self.vertices.len() as _);
        }
        self.time += 0.01;
    }
}

struct MyApp {
    mesh: DynamicMesh,
    material: Material,
    pattern: Pattern,
    monotonic: u32,
}

impl App2D for MyApp {
    const TITLE: &'static str = "2D example app";
    type Args = ();

    fn new(engine: &mut WinitBackend, _args: Self::Args) -> Result<Self> {
        let material = engine.add_material(UNLIT_VERT, UNLIT_FRAG, DrawType::Lines)?;

        let mut pattern = Pattern::default();
        pattern.update();
        let mesh = engine.add_dynamic_mesh(&pattern.vertices, &pattern.indices)?;

        Ok(Self {
            material,
            mesh,
            pattern,
            monotonic: 0,
        })
    }

    fn event(&mut self, _event: &WindowEvent, _engine: &mut WinitBackend) -> Result<()> {
        Ok(())
    }

    fn frame(&mut self, engine: &mut WinitBackend) -> Result<FramePacket> {
        self.pattern.update();
        if self.monotonic & 1 == 0 {
            engine.update_mesh(self.mesh, &self.pattern.vertices, &self.pattern.indices)?;
        }
        let object = Object {
            mesh: MeshType::Dynamic(self.mesh),
            transform: Matrix4::identity(),
            material: self.material,
        };
        self.monotonic += 1;
        Ok(FramePacket {
            objects: vec![object],
        })
    }
}

fn main() -> Result<()> {
    launch::<MyApp>(())
}
