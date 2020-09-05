trait App {
    fn new(&dyn Engine) -> Result<Self>;
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

        let (vertices, indices) = hexagon();
        let mesh = engine.add_mesh(vertices, indices)?;

        Self {
            mesh,
            material,
            time: 0.0,
        }
    }

    pub fn next_frame(&mut self, &mut Engine) -> FramePacket {
        let transform = Matrix4::from_euler_angles(time, 0.0, time);
        let object = Object {
            material: self.material,
            mesh: self.mesh,
            transform,
        };
        self.time += 1.0;
        (vec![object], f32)
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
    let window = Window::new(app.name())?;
    let engine = EngineWinit::new(&window);

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
    let engine = EngineOpenXr::new(&openxr);

    let app = A::new(&engine);

    loop {
        let packet = app.next_frame(&mut engine)?;
        if !engine.next_frame(&openxr, &packet)? {
            break Ok(());
        }
    }
}
