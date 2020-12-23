use anyhow::Result;
use klystron::{
    runtime_3d::{launch, App},
    DrawType, Engine, FramePacket, Matrix4, Object, Portal, Vertex, UNLIT_FRAG, UNLIT_VERT,
};
use nalgebra::Vector3;

struct MyApp {
    cube: Object,
    grid: Object,
    portals: [Portal; 2],
    time: f32,
}

impl App for MyApp {
    const NAME: &'static str = "Thinking with portals";

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

        // Grid
        let material = engine.add_material(UNLIT_VERT, UNLIT_FRAG, DrawType::Lines)?;

        let (vertices, indices) = grid(30, 1., [1.; 3]);
        let mesh = engine.add_mesh(&vertices, &indices)?;

        let grid = Object {
            material,
            mesh,
            transform: Matrix4::identity(),
        };


        // Portals
        let (vertices, indices) = quad([233. / 255., 147. / 255., 20. / 255.]);
        let mesh = engine.add_mesh(&vertices, &indices)?;

        let orange = Portal {
            mesh,
            affine: Matrix4::new_translation(&Vector3::new(9., 2., 9.)) * Matrix4::from_euler_angles(0.0, std::f32::consts::FRAC_PI_2, 0.),
        };

        let (vertices, indices) = quad([20. / 255., 154. / 255., 233. / 255.]);
        let mesh = engine.add_mesh(&vertices, &indices)?;

        let blue = Portal {
            mesh,
            affine: Matrix4::identity(),
        };

        Ok(Self {
            cube,
            grid,
            portals: [orange, blue],
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        self.cube.transform = Matrix4::new_translation(&Vector3::new(7., 2., 9.));// * Matrix4::from_euler_angles(0.0, self.time, 0.0);
        //self.portals[1].affine = Matrix4::new_translation(&Vector3::new( self.time.cos(), 2., -2.));
        engine.update_time_value(self.time)?;
        self.time += 0.004;
        Ok(FramePacket {
            objects: vec![self.cube, self.grid],
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

fn grid(size: i32, scale: f32, color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut index = 0;
    let mut push_line = |a, b| {
        vertices.push(Vertex { pos: a, color });
        vertices.push(Vertex { pos: b, color });
        indices.push(index);
        index += 1;
        indices.push(index);
        index += 1;
    };

    let l = size as f32 * scale;
    for i in -size..=size {
        let f = i as f32 * scale;
        push_line([l, 0.0, f], [-l, 0.0, f]);
        push_line([f, 0.0, l], [f, 0.0, -l]);
    }

    (vertices, indices)
}