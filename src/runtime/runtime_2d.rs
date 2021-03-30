use super::target_time::TargetTime;
use crate::{Camera, Engine, FramePacket, WinitBackend};
use anyhow::Result;
use nalgebra::{Matrix4, Vector4};
pub use winit::event;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// TODO: Optional event-driven option? Like `event()` -> Result<bool> where true means call `frame()` again
// TODO: This should probably be renamed WindowedApp and have an optional `camera()` function, so
// that users can supply their own camera

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
    fn frame(&mut self, engine: &mut WinitBackend) -> FramePacket;
}

/// Run a 2D app given these args
pub fn launch<App: App2D + 'static>(args: App::Args) -> Result<()> {
    // 2D engine setup boilerplate
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(App::TITLE)
        .build(&event_loop)?;
    let mut engine = WinitBackend::new(&window, App::TITLE)?;

    let mut app = App::new(&mut engine, args)?;

    // Main loop
    let mut time = 0.;
    let mut target_time = TargetTime::default();
    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            *control_flow = ControlFlow::Poll;
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => app.event(&event, &mut engine).unwrap(),
        },
        Event::MainEventsCleared => {
            engine.update_time_value(time).unwrap();
            time += 0.01;
            target_time.start_frame();
            let packet = app.frame(&mut engine);
            engine
                .next_frame(&packet, &Dummy2DCam)
                .expect("Engine frame failed");
            target_time.end_frame();
        }
        _ => (),
    })
}

/// Simple orthographics 2D Camera, scales such that aspect ratio is preserved (x is always -1.0 to 1.0)
pub struct Dummy2DCam;

impl Camera for Dummy2DCam {
    fn matrix(&self, width: u32, height: u32) -> Matrix4<f32> {
        let (w, h) = if width < height {
            (1., width as f32 / height as f32)
        } else {
            (height as f32 / width as f32, 1.)
        };
        Matrix4::from_diagonal(&Vector4::new(w, h, 1., 1.))
    }
}
