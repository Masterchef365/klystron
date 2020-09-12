use anyhow::Result;
use klystron::{
    Camera, DrawType, Engine, FramePacket, Material, Mesh, MouseCamera, Object, OpenXrBackend,
    Vertex, WinitBackend,
};
use log::info;
use nalgebra::Matrix4;
use openxr as xr;
use std::fs;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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
        let transform = Matrix4::from_euler_angles(0.0, /*self.time*/ 0.0, 0.0);
        let object = Object {
            material: self.material,
            mesh: self.mesh,
            transform,
            anim: self.time,
        };
        self.time += 0.01;
        Ok(FramePacket {
            objects: vec![object],
            //stage_origin: Point3::origin(),
            //stage_rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
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
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(A::NAME)
        .build(&event_loop)?;
    let mut engine = WinitBackend::new(&window, A::NAME)?;

    let mut app = A::new(&mut engine)?;

    let target_frame_time = Duration::from_micros(1_000_000 / 60);
    let mut mouse_camera = MouseCamera::new(Camera::default(), 0.001, 0.004);
    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            *control_flow = ControlFlow::Poll;
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => mouse_camera.handle_events(&event),
        },
        Event::MainEventsCleared => {
            let frame_start_time = std::time::Instant::now();
            let packet = app.next_frame(&mut engine).unwrap();
            engine.next_frame(&packet, &mouse_camera.inner).unwrap();
            let frame_end_time = std::time::Instant::now();
            let frame_duration = frame_end_time - frame_start_time;
            if frame_duration < target_frame_time {
                std::thread::sleep(target_frame_time - frame_duration);
            }
        }
        _ => (),
    })
}

fn vr_backend<A: App>() -> Result<()> {
    // Handle interrupts gracefully
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("setting Ctrl-C handler");

    let (mut engine, openxr) = OpenXrBackend::new(A::NAME)?;
    let mut app = A::new(&mut engine)?;

    let mut event_storage = xr::EventDataBuffer::new();
    let mut session_running = false;

    // TODO: STATE TRANSITIONS
    'main_loop: loop {
        if !running.load(Ordering::Relaxed) {
            info!("Requesting exit");
            let res = openxr.session.request_exit();
            if let Err(xr::sys::Result::ERROR_SESSION_NOT_RUNNING) = res {
                info!("OpenXR Exiting gracefully");
                break Ok(());
            }
            res?;
        }

        while let Some(event) = openxr.instance.poll_event(&mut event_storage).unwrap() {
            use xr::Event::*;
            match event {
                SessionStateChanged(e) => {
                    info!("OpenXR entered state {:?}", e.state());
                    match e.state() {
                        xr::SessionState::READY => {
                            openxr
                                .session
                                .begin(xr::ViewConfigurationType::PRIMARY_STEREO)
                                .unwrap();
                            session_running = true;
                        }
                        xr::SessionState::STOPPING => {
                            openxr.session.end().unwrap();
                            session_running = false;
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => {
                            info!("OpenXR Exiting");
                            break 'main_loop Ok(());
                        }
                        _ => {}
                    }
                }
                InstanceLossPending(_) => {
                    info!("OpenXR Pending instance loss");
                    break 'main_loop Ok(());
                }
                EventsLost(e) => {
                    info!("OpenXR lost {} events", e.lost_event_count());
                }
                _ => {}
            }
        }

        if !session_running {
            // Don't grind up the CPU
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }

        let packet = app.next_frame(&mut engine)?;
        engine.next_frame(&packet)?;
    }
}
