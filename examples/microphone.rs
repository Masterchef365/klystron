use anyhow::Result;
use klystron::{Camera, DrawType, Engine, FramePacket, Object, Vertex, WinitBackend};
use nalgebra::{Matrix4, Point3, Vector3};
use std::collections::VecDeque;
use std::fs;
use std::time::Duration;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const NAME: &str = "Test app";

fn main() -> Result<()> {
    // 2D engine setup boilerplate
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title(NAME).build(&event_loop)?;
    let mut engine = WinitBackend::new(&window, NAME)?;

    let unlit_vert = fs::read("./examples/shaders/unlit.vert.spv")?;
    let unlit_frag = fs::read("./examples/shaders/unlit.frag.spv")?;
    let tri_mat = engine.add_material(&unlit_vert, &unlit_frag, DrawType::Lines)?;

    let mut theta = 0.0f32;
    const SAMPLES: usize = 50;
    let mut update_sinewave = move || {
        theta += 0.05;
        theta.cos()
    };
    let mut rt_data = std::iter::repeat(0.)
        .take(SAMPLES)
        .collect::<VecDeque<f32>>();

    let indices = std::iter::successors(Some((0u16, true)), |&(v, b)| {
        Some((if b { v + 1 } else { v }, !b))
    })
    .map(|(v, _)| v)
    .take(SAMPLES * 2 - 1)
    .collect::<Vec<_>>();

    let mut vertices = std::iter::repeat(Vertex {
        pos: [0., 0., 0.],
        color: [0., 0., 0.],
    })
    .take(SAMPLES)
    .collect::<Vec<_>>();

    let grid_mesh = engine.add_mesh(&vertices, &indices, true)?;

    // Main loop
    let target_frame_time = Duration::from_micros(1_000_000 / 60);
    let mut time = 0.;
    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            *control_flow = ControlFlow::Poll;
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => (),
        },
        Event::MainEventsCleared => {
            let frame_start_time = std::time::Instant::now();

            rt_data.pop_back();
            rt_data.push_front(update_sinewave());

            for (idx, (vert, sample)) in vertices.iter_mut().zip(rt_data.iter()).enumerate() {
                let x = (idx as f32 * 2. / SAMPLES as f32) - 1.;
                *vert = Vertex {
                    pos: [x, *sample, 0.],
                    color: [1., x.abs(), *sample],
                };
            }

            engine
                .update_verts(grid_mesh, &vertices)
                .expect("Failed to update verts");

            let grid = Object {
                material: tri_mat,
                mesh: grid_mesh,
                transform: Matrix4::identity(),
            };

            engine.update_time_value(time).unwrap();
            time += 0.01;
            let packet = FramePacket {
                objects: vec![grid],
            };

            engine.next_frame(&packet, &Ortho2DCam).unwrap();
            let frame_end_time = std::time::Instant::now();
            let frame_duration = frame_end_time - frame_start_time;
            if frame_duration < target_frame_time {
                std::thread::sleep(target_frame_time - frame_duration);
            }
        }
        _ => (),
    })
}

struct Ortho2DCam;

impl Camera for Ortho2DCam {
    fn matrix(&self, width: u32, height: u32) -> Matrix4<f32> {
        Matrix4::identity()
    }
}
