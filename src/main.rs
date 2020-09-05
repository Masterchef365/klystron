use anyhow::Result;
use klystron::{
    DrawType, Engine, FramePacket, Material, Mesh, Object, OpenXr, OpenXrBackend, Vertex,
    WinitBackend,
};
use nalgebra::{Matrix4, Point3, UnitQuaternion};
use std::fs;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

trait App {
    fn new(engine: &dyn Engine) -> Result<Self>;
    fn next_frame(&mut self) -> FramePacket;
    fn name() -> &'static str;
}

struct MyApp {
    material: Material,
    mesh: Mesh,
    time: f32,
}

impl MyApp {
    pub fn new(engine: &mut Engine) -> Result<Self> {
        let material = engine.add_material(
            &fs::read("./shaders/unlit.vert.spv")?,
            &fs::read("./shaders/unlit.frag.spv")?,
            DrawType::Vertices,
        )?;

        let mut vertices = [
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

        let mesh = engine.add_mesh(vertices, indices)?;

        Ok(Self {
            mesh,
            material,
            time: 0.0,
        })
    }

    pub fn next_frame(&mut self, engine: &mut Engine) -> FramePacket {
        let transform = Matrix4::from_euler_angles(self.time, 0.0, self.time);
        let object = Object {
            material: self.material,
            mesh: self.mesh,
            transform,
        };
        self.time += 1.0;
        FramePacket {
            objects: vec![object],
            time: self.time,
            camera_origin: Point3::origin(),
            camera_rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
        }
    }
}

fn main() -> Result<()> {
    let vr = std::env::args().skip(1).next().is_some();
    if vr {
        vr_backend::<MyApp>()
    } else {
        windowed_backend::<MyApp>()
    }
}

fn windowed_backend<A: App>() -> Result<()> {
    let eventloop = EventLoop::new()?;
    let window = Window::new(A::name())?;
    let engine = WinitBackend::new(&window);

    let app = A::new(&engine);

    eventloop.run(move |control_flow| {
        let packet = app.next_frame(&mut engine)?;
        if !engine.next_frame(&window, &packet)? {
            *control_flow = ControlFlow::Exit;
        }
    });
    Ok(())
}

fn vr_backend<A: App>() -> Result<()> {
    let openxr = OpenXr::new();
    let engine = OpenXrBackend::new(&openxr);

    let app = A::new(&engine);

    loop {
        let packet = app.next_frame(&mut engine)?;
        if !engine.next_frame(&openxr, &packet)? {
            break Ok(());
        }
    }
}
