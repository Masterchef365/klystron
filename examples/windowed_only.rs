use klystron::{runtime_2d::{App2D, launch, event::WindowEvent}, DrawType, Vertex, Engine, WinitBackend, Object, FramePacket};
use anyhow::Result;
use std::fs;

struct MyApp {
    object: Object,
}

impl App2D for MyApp {
    const TITLE: &'static str = "2D example app";
    type Args = ();

    fn new(engine: &mut WinitBackend, _args: Self::Args) -> Result<Self> {
        let material = engine.add_material(
            &fs::read("./examples/shaders/unlit.vert.spv")?,
            &fs::read("./examples/shaders/unlit.frag.spv")?,
            DrawType::Lines,
        )?;

        let (vertices, indices) = wire_triangle();
        let mesh = engine.add_mesh(&vertices, &indices)?;

        let object = Object {
            mesh,
            transform: nalgebra::Matrix4::identity(),
            material,
        };

        Ok(Self {
            object,
        })
    }

    fn event(&mut self, event: &WindowEvent, engine: &mut WinitBackend) -> Result<()> {
        Ok(())
    }

    fn frame(&self) -> FramePacket {
        FramePacket {
            objects: vec![self.object],
        }
    }
}

fn wire_triangle() -> ([Vertex; 3], [u16; 6]) {
    let color = [0., 1., 0.];
    let vertices = [
        Vertex {
            pos: [-0.5, -0.5, 0.],
            color
        },
        Vertex {
            pos: [0.5, -0.5, 0.],
            color
        },
        Vertex {
            pos: [0.5, 0.5, 0.],
            color,
        },
    ];
    let indices = [0, 1, 1, 2, 2, 0 ];
    (vertices, indices)
}

fn main() -> Result<()> {
    launch::<MyApp>(())
}
