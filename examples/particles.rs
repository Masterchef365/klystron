use anyhow::Result;
use klystron::{
    runtime::{launch, App},
    DrawType, Engine, FramePacket, Material, Mesh, Object, Vertex, ParticleSet, Particle, ParticleSystem, ParticleSimulation,
};
use nalgebra::{Matrix4, Point3, Vector3};
use std::fs;

struct MyApp {
    triangle_mat: Material,
    point_mat: Material,
    simulation: ParticleSystem,
    mesh: Mesh,
    particles: ParticleSet,
    time: f32,
}

impl App for MyApp {
    const NAME: &'static str = "MyApp";

    type Args = ();

    fn new(engine: &mut dyn Engine, _args: Self::Args) -> Result<Self> {
        let triangle_mat = engine.add_material(
            &fs::read("./examples/shaders/unlit.vert.spv")?,
            &fs::read("./examples/shaders/unlit.frag.spv")?,
            DrawType::Triangles,
        )?;

        let (vertices, indices) = rainbow_cube();
        let mesh = engine.add_mesh(&vertices, &indices)?;

        let point_mat = engine.add_material(
            &fs::read("./examples/shaders/unlit.vert.spv")?,
            &fs::read("./examples/shaders/unlit.frag.spv")?,
            DrawType::Points,
        )?;

        let simulation = engine.add_particle_system(
            &fs::read("./examples/shaders/particle_forces.comp.spv")?,
            &fs::read("./examples/shaders/particle_motion.comp.spv")?
        )?;

        const SIDE_LEN: usize = 10;
        let mut particles = Vec::with_capacity(SIDE_LEN * SIDE_LEN * SIDE_LEN);
        let mass = 10.0;
        for x in 0..SIDE_LEN {
            let x = x as f32 / SIDE_LEN as f32;
            for y in 0..SIDE_LEN {
                let y = y as f32 / SIDE_LEN as f32;
                for z in 0..SIDE_LEN {
                    let z = z as f32 / SIDE_LEN as f32;
                    let charge = x.cos() + y.sin() + z.cos();
                    particles.push(Particle::new(
                        [x, y, z],
                        [0.0; 3],
                        mass,
                        charge,
                    ));
                }
            }
        }

        let particles = engine.add_particles(&particles)?;

        Ok(Self {
            point_mat,
            simulation,
            particles,
            mesh,
            triangle_mat,
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        let transform = Matrix4::from_euler_angles(0.0, self.time, 0.0)
            * Matrix4::new_translation(&Vector3::new(0.0, -1.0, 0.0));
        let object = Object {
            material: self.triangle_mat,
            mesh: self.mesh,
            transform,
        };
        let particle_sim = ParticleSimulation {
            particle_system: self.simulation,
            material: self.point_mat,
            particles: self.particles,
        };
        engine.update_time_value(self.time)?;
        self.time += 0.01;
        Ok(FramePacket {
            objects: vec![object],
            particle_simulations: vec![particle_sim],
        })
    }
}

fn main() -> Result<()> {
    let vr = std::env::args().skip(1).next().is_some();
    env_logger::init();
    launch::<MyApp>(vr, ())
}

fn rainbow_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::from_nalgebra(Point3::new(-1.0, -1.0, -1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, -1.0, -1.0), Point3::new(1.0, 0.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, 1.0, -1.0), Point3::new(1.0, 1.0, 0.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, 1.0, -1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, -1.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, -1.0, 1.0), Point3::new(1.0, 1.0, 0.0)),
        Vertex::from_nalgebra(Point3::new(1.0, 1.0, 1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, 1.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
    ];

    let indices = vec![
        0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6,
        4, 5, 0, 0, 5, 1,
    ];

    (vertices, indices)
}
