use anyhow::Result;
use klystron::{
    DrawType, Engine, FramePacket, Material, Mesh, Object, XrPrelude, OpenXrBackend, Vertex,
    WinitBackend,
};
use nalgebra::{Matrix4, Point3, UnitQuaternion};
use std::fs;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

trait App: Sized {
    const NAME: &'static str;
    fn new(engine: &mut dyn Engine) -> Result<Self>;
    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket>;
}

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
        let transform = Matrix4::from_euler_angles(self.time, 0.0, self.time);
        let object = Object {
            material: self.material,
            mesh: self.mesh,
            transform,
            anim: self.time,
        };
        self.time += 1.0;
        Ok(FramePacket {
            objects: vec![object],
            stage_origin: Point3::origin(),
            stage_rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
        })
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let vr = std::env::args().skip(1).next().is_some();
    if vr {
        vr_backend::<MyApp>()
    } else {
        windowed_backend::<MyApp>()
    }
}

fn windowed_backend<A: App + 'static>() -> Result<()> {
    let eventloop = EventLoop::new();
    let window = WindowBuilder::new().with_title(A::NAME).build(&eventloop)?;
    let mut engine = WinitBackend::new(&window, A::NAME)?;

    let mut app = A::new(&mut engine)?;

    eventloop.run(move |_event, _, _control_flow| {
        let packet = app.next_frame(&mut engine).unwrap();
        engine.next_frame(&packet).unwrap();
    });
}

fn vr_backend<A: App>() -> Result<()> {
    let (mut engine, openxr) = OpenXrBackend::new(A::NAME)?;

    let mut app = A::new(&mut engine)?;

    loop {
        let packet = app.next_frame(&mut engine)?;
        if !engine.next_frame(&openxr, &packet)? {
            break Ok(());
        }
    }
}
