use anyhow::Result;
use klystron::{
    runtime::{launch, App},
    DrawType, Engine, FramePacket, Material, Mesh, Object, Vertex,
};
use nalgebra::{Matrix4, Point3};
use std::fs;

struct MyApp {
    material: Material,
    mesh: Mesh,
    time: f32,
}

impl App for MyApp {
    const NAME: &'static str = "MyApp";

    type Args = ();

    fn new(engine: &mut dyn Engine, _args: Self::Args) -> Result<Self> {
        let material = engine.add_material(
            &fs::read("./examples/shaders/unlit.vert.spv")?,
            &fs::read("./examples/shaders/unlit.frag.spv")?,
            DrawType::Triangles,
        )?;

        let (vertices, indices) = rainbow_cube();
        let mesh = engine.add_mesh(&vertices, &indices)?;

        Ok(Self {
            mesh,
            material,
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        let transform = Matrix4::from_euler_angles(0.0, self.time, 0.0);
        let object = Object {
            material: self.material,
            mesh: self.mesh,
            transform,
        };
        engine.update_time_value(self.time)?;
        self.time += 0.01;
        Ok(FramePacket {
            particle_systems: vec![],
            objects: vec![object],
        })
    }
}

fn main() -> Result<()> {
    let vr = std::env::args().skip(1).next().is_some();
    env_logger::init();
    launch::<MyApp>(vr, ())
}

fn rainbow_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::from_nalgebra(Point3::new(-1.0, -1.0, -1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, -1.0, -1.0), Point3::new(1.0, 0.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, 1.0, -1.0), Point3::new(1.0, 1.0, 0.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, 1.0, -1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, -1.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, -1.0, 1.0), Point3::new(1.0, 1.0, 0.0)),
        Vertex::from_nalgebra(Point3::new(1.0, 1.0, 1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, 1.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
    ];

    let indices = vec![
        0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6,
        4, 5, 0, 0, 5, 1,
    ];

    (vertices, indices)
}
