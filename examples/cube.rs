use anyhow::Result;
use klystron::{
    runtime::{launch, App},
    DrawType, Engine, FramePacket, Material, Mesh, Object, Vertex,
};
use nalgebra::Matrix4;
use std::fs;

struct MyApp {
    material: Material,
    mesh: Mesh,
    time: f32,
}

impl App for MyApp {
    const NAME: &'static str = "MyApp";

    fn new(engine: &mut dyn Engine) -> Result<Self> {
        let material = engine.add_material(
            &fs::read("./shaders/unlit.vert.spv")?,
            &fs::read("./shaders/unlit.frag.spv")?,
            DrawType::Triangles,
        )?;

        let vertices = [
            Vertex {
                pos: [-1.0, -1.0, -1.0],
                color: [0.0, 1.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0, -1.0],
                color: [1.0, 0.0, 1.0],
            },
            Vertex {
                pos: [1.0, 1.0, -1.0],
                color: [1.0, 1.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, -1.0],
                color: [0.0, 1.0, 1.0],
            },
            Vertex {
                pos: [-1.0, -1.0, 1.0],
                color: [1.0, 0.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0, 1.0],
                color: [1.0, 1.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, 1.0],
                color: [0.0, 1.0, 1.0],
            },
            Vertex {
                pos: [-1.0, 1.0, 1.0],
                color: [1.0, 0.0, 1.0],
            },
        ];

        let indices = [
            0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2,
            6, 4, 5, 0, 0, 5, 1,
        ];

        let mesh = engine.add_mesh(&vertices, &indices)?;

        Ok(Self {
            mesh,
            material,
            time: 0.0,
        })
    }

    fn next_frame(&mut self, _engine: &mut dyn Engine) -> Result<FramePacket> {
        let transform = Matrix4::from_euler_angles(0.0, 0.0, 0.0);
        let object = Object {
            material: self.material,
            mesh: self.mesh,
            transform,
            anim: self.time,
        };
        self.time += 0.01;
        Ok(FramePacket {
            objects: vec![object],
        })
    }
}

fn main() -> Result<()> {
    env_logger::init();
    launch::<MyApp>()
}
