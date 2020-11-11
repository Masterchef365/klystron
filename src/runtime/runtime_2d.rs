use anyhow::Result;
use crate::{
    Camera, Engine, FramePacket, Object, WinitBackend,
};
use std::time::Duration;
use nalgebra::{Matrix4, Vector4};
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// TODO: Optional event-driven option? Like `event()` -> Result<bool> where true means call `frame()` again

/// A 2D, windowed application
pub trait App2D: Sized {
    /// Window title
    const TITLE: &'static str;
    /// Type of args to be passed to Self::new()
    type Args;
    /// Create a new instance of this app
    fn new(engine: &mut WinitBackend, args: Self::Args) -> Result<Self>;
    /// Handle a winit window event
    fn event(&mut self, event: &WindowEvent, engine: &mut WinitBackend) -> Result<()>;
    /// Rendering logic
    fn frame(&self) -> FramePacket;
}

/// Run a 2D app given these args
pub fn run_app<App: App2D + 'static>(args: App::Args) -> Result<()> {
    // 2D engine setup boilerplate
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title(App::TITLE).build(&event_loop)?;
    let mut engine = WinitBackend::new(&window, App::TITLE)?;

    let mut app = App::new(&mut engine, args)?;

    // Main loop
    let target_frame_time = Duration::from_micros(1_000_000 / 60);
    let mut time = 0.;
    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            *control_flow = ControlFlow::Poll;
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => app.event(&event, &mut engine).unwrap(),
        },
        Event::MainEventsCleared => {
            let frame_start_time = std::time::Instant::now();

            engine.update_time_value(time).unwrap();
            time += 0.01;

            let packet = app.frame();

            engine.next_frame(&packet, &Dummy2DCam).unwrap();
            let frame_end_time = std::time::Instant::now();
            let frame_duration = frame_end_time - frame_start_time;
            if frame_duration < target_frame_time {
                std::thread::sleep(target_frame_time - frame_duration);
            }
        }
        _ => (),
    })
}

/// Simple orthographics 2D Camera, scales such that aspect ratio is preserved (x is always -1.0 to 1.0)
pub struct Dummy2DCam;

impl Camera for Dummy2DCam {
    fn matrix(&self, width: u32, height: u32) -> Matrix4<f32> {
        let aspect = width as f32 / height as f32;
        Matrix4::from_diagonal(&Vector4::new(1., aspect, 0., 1.))
    }
}


