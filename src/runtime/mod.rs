//! The runtime for the klystron engine. 
//!
//! A simple runtime providing only a first-person camera in VR mode and an Arcball camera in
//! windowed mode. Abstracts over platform-specific features for quick prototyping.

mod mouse_camera;
use mouse_camera::MouseCamera;
use anyhow::Result;
use log::info;
use openxr as xr;
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
use crate::{Engine, FramePacket, WinitBackend, Camera, OpenXrBackend};

/// An app that can be run on the runtime
pub trait App: Sized {
    const NAME: &'static str;
    /// Arguments passed into the structure on creation
    type Args;
    /// Create a new instance of the app, populating the engine with meshes and materials
    fn new(engine: &mut dyn Engine, args: Self::Args) -> Result<Self>;
    /// Update the app's state and render the next frame
    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket>;
}

/// Launch an `App`. Runs in OpenXR when `vr` is set.
///
/// Example:
/// ```rust
/// struct MyApp {}
/// impl App for MyApp {}
///
/// fn main() {
///     launch::<MyApp>(false, ());
/// }
/// ```
pub fn launch<A: App + 'static>(vr: bool, args: A::Args) -> Result<()> {
    if vr {
        vr_backend::<A>(args)
    } else {
        windowed_backend::<A>(args)
    }
}

/// Launch an `App` using `winit` as a surface and input mechanism for windowed mode
pub fn windowed_backend<A: App + 'static>(args: A::Args) -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(A::NAME)
        .build(&event_loop)?;
    let mut engine = WinitBackend::new(&window, A::NAME)?;

    let mut app = A::new(&mut engine, args)?;

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

/// Launch an `App` using OpenXR as a surface and input mechanism for VR
fn vr_backend<A: App>(args: A::Args) -> Result<()> {
    // Handle interrupts gracefully
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("setting Ctrl-C handler");

    let (mut engine, openxr) = OpenXrBackend::new(A::NAME)?;
    let mut app = A::new(&mut engine, args)?;

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