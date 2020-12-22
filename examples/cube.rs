use anyhow::Result;
use klystron::{
    runtime_3d::{launch, App},
    DrawType, Engine, FramePacket, Matrix4, Object, Portal, Vertex, UNLIT_FRAG, UNLIT_VERT,
};
use nalgebra::Vector3;

struct MyApp {
    cube: Object,
    portals: [Portal; 2],
    time: f32,
}

impl App for MyApp {
    const NAME: &'static str = "MyApp";

    type Args = ();

    fn new(engine: &mut dyn Engine, _args: Self::Args) -> Result<Self> {
        // Cube
        let material = engine.add_material(UNLIT_VERT, UNLIT_FRAG, DrawType::Triangles)?;

        let (vertices, indices) = rainbow_cube();
        let mesh = engine.add_mesh(&vertices, &indices)?;

        let cube = Object {
            material,
            mesh,
            transform: Matrix4::identity(),
        };

        // Portals
        let (vertices, indices) = quad([233. / 255., 147. / 255., 20. / 255.]);
        let mesh = engine.add_mesh(&vertices, &indices)?;

        let orange = Portal {
            mesh,
            affine: Matrix4::new_translation(&Vector3::new(0., 4., 2.)),
        };

        let (vertices, indices) = quad([20. / 255., 154. / 255., 233. / 255.]);
        let mesh = engine.add_mesh(&vertices, &indices)?;

        let blue = Portal {
            mesh,
            affine: Matrix4::new_translation(&Vector3::new(0., 8., -2.)),
        };

        Ok(Self {
            cube,
            portals: [orange, blue],
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        self.cube.transform = Matrix4::from_euler_angles(0.0, self.time, 0.0);
        engine.update_time_value(self.time)?;
        self.time += 0.01;
        Ok(FramePacket {
            objects: vec![self.cube],
            portals: self.portals,
        })
    }
}

fn main() -> Result<()> {
    let vr = std::env::args().skip(1).next().is_some();
    launch::<MyApp>(vr, ())
}

fn rainbow_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    (vertices, indices)
}

fn quad(color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::new([-2.0, -2.0, 0.0], color),
        Vertex::new([-2.0, 2.0, 0.0], color),
        Vertex::new([2.0, -2.0, 0.0], color),
        Vertex::new([2.0, 2.0, 0.0], color),
    ];

    let indices = vec![
        2, 1, 0, 
        3, 1, 2,
        0, 1, 2, 
        2, 1, 3
    ];

    (vertices, indices)
}
