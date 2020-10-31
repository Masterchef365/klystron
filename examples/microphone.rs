use anyhow::Result;
use klystron::{
    Camera, DrawType, Engine, FramePacket, Object, Vertex, WinitBackend,
};
use nalgebra::{Matrix4, Point3, Vector3};
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
    let tri_mat = engine.add_material(&unlit_vert, &unlit_frag, DrawType::Triangles)?;

    // Draw a colored grid for now
    let width = 8;
    let height = 8;
    let mut cells = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            cells.push([
                x as f32 / (width - 1) as f32,
                y as f32 / (height - 1) as f32,
                1.,
            ]);
        }
    }

    let (vertices, indices) = grid(20., 5., &cells, width);
    let grid_mesh = engine.add_mesh(&vertices, &indices)?;

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

fn grid(
    square_size: f32,
    spacing: f32,
    cells: &[[f32; 3]],
    width: usize,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (y, row) in cells.chunks_exact(width).enumerate() {
        let y = y as f32 * (square_size + spacing);
        for (x, cell) in row.iter().enumerate() {
            let x = x as f32 * (square_size + spacing);
            let (verts, idx) = square(Point3::new(x, y, 0.), square_size, *cell);
            vertices.extend_from_slice(&verts[..]);
            let off = vertices.len();
            indices.extend(idx.iter().map(|i| i + off as u16));
        }
    }
    (vertices, indices)
}

fn square(pos: Point3<f32>, size: f32, color: [f32; 3]) -> ([Vertex; 4], [u16; 6]) {
    let vertices = [
        Vertex {
            pos: *(pos + Vector3::new(0., 0., 0.)).coords.as_ref(),
            color,
        },
        Vertex {
            pos: *(pos + Vector3::new(size, 0., 0.)).coords.as_ref(),
            color,
        },
        Vertex {
            pos: *(pos + Vector3::new(0., size, 0.)).coords.as_ref(),
            color,
        },
        Vertex {
            pos: *(pos + Vector3::new(size, size, 0.)).coords.as_ref(),
            color,
        },
    ];
    let indices = [2, 1, 0, 1, 2, 3];
    (vertices, indices)
}

struct Ortho2DCam;

impl Camera for Ortho2DCam {
    fn matrix(&self, width: u32, height: u32) -> Matrix4<f32> {
        Matrix4::new_orthographic(0., width as _, 0., height as _, -1., 1.)
    }
}
